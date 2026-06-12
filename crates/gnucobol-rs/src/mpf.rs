//! A pure-Rust subset of GMP `mpf` (arbitrary-precision binary floating point), faithful to the
//! operations numeric.c builds on `mpf_t`: the `double`<->`cob_decimal` bridge. A value is
//! `sign * mantissa * 2^exp` with `mantissa` a non-negative [`Mpz`]; operations keep at most
//! `COB_MPF_PREC` significant bits (truncating low bits toward zero, as GMP's mpf does). This replaces
//! the earlier f64 proxy so `cob_decimal_set_mpf`/`get_mpf` take/return a real `Mpf`.
#![forbid(unsafe_code)]

use crate::gmp::Mpz;

/// `COB_MPF_PREC` (coblocal.h:146): GnuCOBOL's mpf working precision in bits.
pub const COB_MPF_PREC: u64 = 2048;

/// An arbitrary-precision binary float (`mpf_t`): `sign * mantissa * 2^exp`.
#[derive(Clone, Debug)]
pub struct Mpf {
    /// -1, 0, or +1. `sign == 0` iff the value is zero.
    sign: i8,
    /// non-negative significand.
    mantissa: Mpz,
    /// binary exponent.
    exp: i64,
    /// working precision in bits.
    prec: u64,
}

impl Mpf {
    /// `mpf_init2 (x, prec)`: a zero value with the given precision.
    pub fn new(prec: u64) -> Self {
        Mpf { sign: 0, mantissa: Mpz::new(), exp: 0, prec }
    }

    /// `mpf_sgn`.
    pub fn sgn(&self) -> i32 {
        self.sign as i32
    }

    /// Drop low bits so the mantissa keeps at most `prec` significant bits (GMP mpf truncates toward
    /// zero on assignment/arithmetic). Re-establishes the sign==0 invariant for a zero mantissa.
    fn normalize(&mut self) {
        if self.mantissa.sgn() == 0 {
            self.sign = 0;
            self.exp = 0;
            return;
        }
        let bits = self.mantissa.sizeinbase2() as u64;
        if bits > self.prec {
            let drop = bits - self.prec;
            self.mantissa = self.mantissa.fdiv_q_2exp(drop as u32); // mantissa >= 0 -> floor == trunc
            self.exp += drop as i64;
        }
    }

    /// `mpf_set_d (x, v)`: set to the exact value of a finite `f64` (`m * 2^e`, exact — no precision
    /// loss since a double needs only 53 bits).
    pub fn set_d(v: f64, prec: u64) -> Self {
        if v == 0.0 || !v.is_finite() {
            return Mpf::new(prec);
        }
        let (m, e) = crate::float::decompose_f64(v.abs());
        let mut f = Mpf { sign: if v < 0.0 { -1 } else { 1 }, mantissa: Mpz::from_u64(m), exp: e as i64, prec };
        f.normalize();
        f
    }

    /// `mpf_set_z (x, z)`: set to the value of an integer.
    pub fn set_z(z: &Mpz, prec: u64) -> Self {
        let sign = z.sgn() as i8;
        let mut mantissa = z.clone();
        if sign < 0 {
            mantissa.neg();
        }
        let mut f = Mpf { sign, mantissa, exp: 0, prec };
        f.normalize();
        f
    }

    /// `mpf_mul (r, a, b)`.
    pub fn mul(&self, other: &Mpf) -> Mpf {
        if self.sign == 0 || other.sign == 0 {
            return Mpf::new(self.prec.max(other.prec));
        }
        let mut r = Mpf {
            sign: self.sign * other.sign,
            mantissa: self.mantissa.mul(&other.mantissa),
            exp: self.exp + other.exp,
            prec: self.prec.max(other.prec),
        };
        r.normalize();
        r
    }

    /// `mpf_div (r, a, b)`: quotient to `prec` significant bits (binary long division, truncated).
    pub fn div(&self, other: &Mpf) -> Mpf {
        if other.sign == 0 || self.sign == 0 {
            return Mpf::new(self.prec.max(other.prec));
        }
        let prec = self.prec.max(other.prec);
        // shift the dividend left so the integer quotient carries >= prec+guard significant bits
        let abits = self.mantissa.sizeinbase2() as i64;
        let bbits = other.mantissa.sizeinbase2() as i64;
        let want = prec as i64 + 2; // a couple of guard bits, then re-truncate via normalize
        let shift = (want - (abits - bbits)).max(0) as u32;
        let num = self.mantissa.mul_2exp(shift); // mantissa << shift
        let q = num.tdiv_q(&other.mantissa);
        let mut r = Mpf {
            sign: self.sign * other.sign,
            mantissa: q,
            exp: self.exp - other.exp - shift as i64,
            prec,
        };
        r.normalize();
        r
    }

    /// `mpf_get_d (x)`: the `f64` nearest the value toward zero (GMP `mpf_get_d` truncates). Overflow to
    /// non-finite returns 0.0 (matching `cob_decimal_get_double`'s `cob_not_finite` guard).
    pub fn get_d(&self) -> f64 {
        if self.sign == 0 {
            return 0.0;
        }
        let bits = self.mantissa.sizeinbase2();
        let (m53, rexp) = if bits > 53 {
            let shift = (bits - 53) as u32;
            (self.mantissa.fdiv_q_2exp(shift).to_i128().unwrap_or(0) as u64, self.exp + shift as i64)
        } else {
            (self.mantissa.to_i128().unwrap_or(0) as u64, self.exp)
        };
        let v = scale_pow2(m53 as f64, rexp);
        if !v.is_finite() {
            return 0.0;
        }
        if self.sign < 0 {
            -v
        } else {
            v
        }
    }

    /// `mpf_get_str (buffer, &exp, 10, ndigits, x)`: the leading `ndigits` significant decimal digits
    /// (trailing zeros stripped, truncated toward zero past `ndigits`) and the base-10 exponent such
    /// that the value is `0.<digits> * 10^exp`. Returns `(neg, digits, exp10)`.
    pub fn get_str(&self, ndigits: usize) -> (bool, Vec<u8>, i64) {
        if self.sign == 0 {
            return (false, vec![], 0);
        }
        // value = mantissa * 2^exp.  exp>=0 -> integer mantissa<<exp ; exp<0 -> (mantissa*5^|exp|) * 10^-|exp|
        let (numer, p): (Mpz, i64) = if self.exp >= 0 {
            (self.mantissa.mul_2exp(self.exp as u32), 0)
        } else {
            let e = (-self.exp) as u32;
            (self.mantissa.mul(&Mpz::ui_pow_ui(5, e)), -self.exp)
        };
        let mut d: Vec<u8> = numer.to_decimal_string().bytes().map(|b| b - b'0').collect();
        let full_len = d.len() as i64;
        let exp10 = full_len - p;
        // strip trailing zeros (they are not significant digits)
        while d.len() > 1 && *d.last().unwrap() == 0 {
            d.pop();
        }
        // truncate toward zero to ndigits significant digits
        if d.len() > ndigits {
            d.truncate(ndigits);
            while d.len() > 1 && *d.last().unwrap() == 0 {
                d.pop();
            }
        }
        (self.sign < 0, d, exp10)
    }
}

/// `m * 2^e` for an exact-mantissa `m` and possibly-large `e`, without intermediate overflow for the
/// representable range (uses `ldexp`-style stepping; clamps to ±inf which the caller treats as 0).
fn scale_pow2(m: f64, e: i64) -> f64 {
    if e >= 0 {
        let mut v = m;
        let mut k = e;
        while k > 1023 {
            v *= 2f64.powi(1023);
            k -= 1023;
            if !v.is_finite() {
                return v;
            }
        }
        v * 2f64.powi(k as i32)
    } else {
        let mut v = m;
        let mut k = -e;
        while k > 1022 {
            v *= 2f64.powi(-1022);
            k -= 1022;
            if v == 0.0 {
                return 0.0;
            }
        }
        v * 2f64.powi(-(k as i32))
    }
}
