//! A pure-Rust subset of GMP `mpf` (arbitrary-precision binary floating point), faithful to the
//! operations numeric.c builds on `mpf_t`: the `double`<->`cob_decimal` bridge. A value is
//! `sign * mantissa * 2^exp` with `mantissa` a non-negative [`Mpz`]; operations keep at most
//! `COB_MPF_PREC` significant bits (truncating low bits toward zero, as GMP's mpf does). This replaces
//! the earlier f64 proxy so `cob_decimal_set_mpf`/`get_mpf` take/return a real `Mpf`.
#![forbid(unsafe_code)]

use crate::gmp::Mpz;
use core::cmp::Ordering;

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

    /// `mpf_set_ui (x, v)`.
    pub fn set_ui(v: u64, prec: u64) -> Self {
        let mut f = Mpf { sign: if v == 0 { 0 } else { 1 }, mantissa: Mpz::from_u64(v), exp: 0, prec };
        f.normalize();
        f
    }

    /// `mpf_set_si (x, v)`.
    pub fn set_si(v: i64, prec: u64) -> Self {
        let mut f = Mpf {
            sign: v.signum() as i8,
            mantissa: Mpz::from_u64(v.unsigned_abs()),
            exp: 0,
            prec,
        };
        f.normalize();
        f
    }

    /// `mpf_set_str (x, s, 10)`: parse an exact decimal string `[-]int[.frac]` into an `Mpf` at `prec`
    /// bits. Used for the hardcoded transcendental constants (pi, e, sqrt2, log-half, log-ten).
    pub fn from_decimal_str(s: &str, prec: u64) -> Self {
        let s = s.trim();
        let (neg, body) = match s.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, s.strip_prefix('+').unwrap_or(s)),
        };
        let (int_part, frac_part) = match body.split_once('.') {
            Some((i, f)) => (i, f),
            None => (body, ""),
        };
        let digits: String = int_part.chars().chain(frac_part.chars()).collect();
        let value = Mpz::from_decimal_string(if digits.is_empty() { "0" } else { &digits });
        // value * 10^(-frac_len): build as Mpf and divide by 10^frac_len
        let mut f = Mpf::set_z(&value, prec);
        if neg {
            f.sign = -f.sign;
        }
        let frac_len = frac_part.len();
        if frac_len != 0 {
            let ten_pow = Mpf::set_z(&Mpz::ui_pow_ui(10, frac_len as u32), prec);
            f = f.div(&ten_pow);
        }
        f
    }

    /// `mpf_neg (r, x)` (in place).
    pub fn neg_assign(&mut self) {
        self.sign = -self.sign;
    }

    /// `mpf_abs (r, x)` (in place).
    pub fn abs_assign(&mut self) {
        if self.sign != 0 {
            self.sign = 1;
        }
    }

    /// Align two values to a common binary exponent, returning `(a_mantissa_signed, b_mantissa_signed, exp)`
    /// as [`Mpz`] integers scaled to `2^exp`.
    fn aligned(a: &Mpf, b: &Mpf) -> (Mpz, Mpz, i64) {
        let exp = a.exp.min(b.exp);
        let mut am = a.mantissa.mul_2exp((a.exp - exp) as u32);
        let mut bm = b.mantissa.mul_2exp((b.exp - exp) as u32);
        if a.sign < 0 {
            am.neg();
        }
        if b.sign < 0 {
            bm.neg();
        }
        (am, bm, exp)
    }

    /// `mpf_add (r, a, b)`.
    pub fn add(&self, other: &Mpf) -> Mpf {
        if self.sign == 0 {
            return other.clone();
        }
        if other.sign == 0 {
            return self.clone();
        }
        let (am, bm, exp) = Mpf::aligned(self, other);
        let sum = am.add(&bm);
        let sign = sum.sgn() as i8;
        let mut mantissa = sum;
        if sign < 0 {
            mantissa.neg();
        }
        let mut r = Mpf { sign, mantissa, exp, prec: self.prec.max(other.prec) };
        r.normalize();
        r
    }

    /// `mpf_sub (r, a, b)`.
    pub fn sub(&self, other: &Mpf) -> Mpf {
        let mut nb = other.clone();
        nb.neg_assign();
        self.add(&nb)
    }

    /// `mpf_mul_ui (r, a, v)`.
    pub fn mul_ui(&self, v: u64) -> Mpf {
        self.mul(&Mpf::set_ui(v, self.prec))
    }

    /// `mpf_div_ui (r, a, v)`.
    pub fn div_ui(&self, v: u64) -> Mpf {
        self.div(&Mpf::set_ui(v, self.prec))
    }

    /// `mpf_add_ui (r, a, v)`.
    pub fn add_ui(&self, v: u64) -> Mpf {
        self.add(&Mpf::set_ui(v, self.prec))
    }

    /// `mpf_sub_ui (r, a, v)`.
    pub fn sub_ui(&self, v: u64) -> Mpf {
        self.sub(&Mpf::set_ui(v, self.prec))
    }

    /// `mpf_ui_sub (r, v, a)`: `v - a`.
    pub fn ui_sub(v: u64, a: &Mpf) -> Mpf {
        Mpf::set_ui(v, a.prec).sub(a)
    }

    /// `mpf_ui_div (r, v, a)`: `v / a`.
    pub fn ui_div(v: u64, a: &Mpf) -> Mpf {
        Mpf::set_ui(v, a.prec).div(a)
    }

    /// `mpf_mul_2exp (r, a, n)`: `a * 2^n` (exact — only the exponent moves).
    pub fn mul_2exp(&self, n: u64) -> Mpf {
        if self.sign == 0 {
            return self.clone();
        }
        Mpf { sign: self.sign, mantissa: self.mantissa.clone(), exp: self.exp + n as i64, prec: self.prec }
    }

    /// `mpf_div_2exp (r, a, n)`: `a / 2^n`.
    pub fn div_2exp(&self, n: u64) -> Mpf {
        if self.sign == 0 {
            return self.clone();
        }
        Mpf { sign: self.sign, mantissa: self.mantissa.clone(), exp: self.exp - n as i64, prec: self.prec }
    }

    /// `mpf_cmp (a, b)`.
    pub fn cmp(&self, other: &Mpf) -> Ordering {
        match (self.sign, other.sign) {
            (0, 0) => return Ordering::Equal,
            _ if self.sign < other.sign => return Ordering::Less,
            _ if self.sign > other.sign => return Ordering::Greater,
            _ => {}
        }
        // same nonzero sign: compare magnitudes, then apply sign direction
        let (am, bm, _) = Mpf::aligned(self, other);
        let mag = am.cmpabs(&bm);
        if self.sign > 0 {
            mag
        } else {
            mag.reverse()
        }
    }

    /// `mpf_cmp_ui (a, v)`.
    pub fn cmp_ui(&self, v: u64) -> Ordering {
        self.cmp(&Mpf::set_ui(v, self.prec))
    }

    /// `mpf_cmp_si (a, v)`.
    pub fn cmp_si(&self, v: i64) -> Ordering {
        self.cmp(&Mpf::set_si(v, self.prec))
    }

    /// The base-2 exponent `e` such that the value is `d * 2^e` with `d` in `[0.5, 1)` — i.e. what
    /// `mpf_get_d_2exp` writes to its `*exp` out-param. (The `cob_mpf_*` series only read this exponent.)
    pub fn get_d_2exp_exp(&self) -> i64 {
        if self.sign == 0 {
            return 0;
        }
        self.mantissa.sizeinbase2() as i64 + self.exp
    }

    /// `mpf_get_ui (x)`: the integer part truncated toward zero, as a `u64`.
    pub fn get_ui(&self) -> u64 {
        if self.sign == 0 {
            return 0;
        }
        let v = if self.exp >= 0 {
            self.mantissa.mul_2exp(self.exp as u32)
        } else {
            self.mantissa.fdiv_q_2exp((-self.exp) as u32)
        };
        v.get_ui()
    }

    /// `mpf_floor (r, x)`: the greatest integer not exceeding `x`.
    pub fn floor(&self) -> Mpf {
        if self.sign == 0 || self.exp >= 0 {
            return self.clone();
        }
        let drop = (-self.exp) as u32;
        let truncated = self.mantissa.fdiv_q_2exp(drop); // floor of |mantissa|/2^drop
        if self.sign > 0 {
            let mut r = Mpf { sign: truncated.sgn() as i8, mantissa: truncated, exp: 0, prec: self.prec };
            r.normalize();
            r
        } else {
            // negative: floor goes more negative if any low bits were dropped
            let restored = truncated.mul_2exp(drop);
            let mut int_mag = truncated;
            if restored.cmp(&self.mantissa) != Ordering::Equal {
                int_mag = int_mag.add_ui(1);
            }
            let mut r = Mpf { sign: -(int_mag.sgn() as i8), mantissa: int_mag, exp: 0, prec: self.prec };
            r.normalize();
            r
        }
    }

    /// `mpf_sqrt (r, x)`: the square root to `prec` bits (truncated toward zero, as GMP does). `x` must be
    /// non-negative; a non-positive value yields zero.
    pub fn sqrt(&self) -> Mpf {
        if self.sign <= 0 {
            return Mpf::new(self.prec);
        }
        let mut m = self.mantissa.clone();
        let mut e = self.exp;
        if e & 1 != 0 {
            m = m.mul_2exp(1);
            e -= 1;
        }
        let mbits = m.sizeinbase2() as i64;
        // choose s so the floor-sqrt of (m << 2s) carries ~prec+4 significant bits
        let s = ((self.prec as i64 + 4) - mbits / 2).max(0);
        let q = m.mul_2exp((2 * s) as u32).isqrt();
        let mut r = Mpf { sign: 1, mantissa: q, exp: e / 2 - s, prec: self.prec };
        r.normalize();
        r
    }

    /// `mpf_eq (a, b, bits)`: whether `a` and `b` agree to the first `bits` significant binary digits — the
    /// convergence cutoff the `cob_mpf_*` series loop on.
    pub fn eq(&self, other: &Mpf, bits: u64) -> bool {
        if self.sign != other.sign {
            return false;
        }
        if self.sign == 0 {
            return true;
        }
        let diff = self.sub(other);
        if diff.sign == 0 {
            return true;
        }
        (self.get_d_2exp_exp() - diff.get_d_2exp_exp()) >= bits as i64
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Decimal rendering helper: leading `n` significant digits + base-10 exponent.
    fn dec(f: &Mpf, n: usize) -> (bool, String, i64) {
        let (neg, d, e) = f.get_str(n);
        (neg, d.iter().map(|b| (b + b'0') as char).collect(), e)
    }

    #[test]
    fn isqrt_exact_and_floor() {
        assert_eq!(Mpz::from_u64(144).isqrt().get_ui(), 12);
        assert_eq!(Mpz::from_u64(143).isqrt().get_ui(), 11);
        assert_eq!(Mpz::from_u64(145).isqrt().get_ui(), 12);
        assert_eq!(Mpz::from_u64(0).isqrt().get_ui(), 0);
        assert_eq!(Mpz::from_u128(1_000_000_000_000_000_000).isqrt().get_ui(), 1_000_000_000);
    }

    #[test]
    fn add_sub_mul_div() {
        let p = COB_MPF_PREC;
        let a = Mpf::set_ui(7, p);
        let b = Mpf::set_ui(3, p);
        assert_eq!(a.add(&b).get_ui(), 10);
        assert_eq!(a.sub(&b).get_ui(), 4);
        assert_eq!(b.sub(&a).cmp_si(-4), Ordering::Equal); // 3-7 = -4
        assert_eq!(a.mul(&b).get_ui(), 21);
        // 7/2 = 3.5 -> floor 3, and 3.5 compares > 3
        let half7 = a.div(&Mpf::set_ui(2, p));
        assert_eq!(half7.get_ui(), 3);
        assert_eq!(half7.cmp(&Mpf::set_ui(3, p)), Ordering::Greater);
    }

    #[test]
    fn sqrt_value() {
        let p = COB_MPF_PREC;
        // sqrt(2) leading digits 1414213562...
        let s2 = Mpf::set_ui(2, p).sqrt();
        let (neg, digs, e) = dec(&s2, 12);
        assert!(!neg);
        assert_eq!(e, 1); // 0.1414... * 10^1
        assert_eq!(&digs[..10], "1414213562");
        // sqrt(144) == 12 exactly
        assert_eq!(Mpf::set_ui(144, p).sqrt().get_ui(), 12);
    }

    #[test]
    fn from_decimal_str_roundtrip() {
        let p = COB_MPF_PREC;
        let x = Mpf::from_decimal_str("1.41421356237309", p);
        let (_, digs, e) = dec(&x, 12);
        assert_eq!(e, 1);
        assert_eq!(&digs[..10], "1414213562");
        let half = Mpf::from_decimal_str("-0.5", p);
        assert_eq!(half.cmp_si(0), Ordering::Less);
        assert_eq!(half.mul_ui(2).cmp_si(-1), Ordering::Equal);
    }

    #[test]
    fn floor_and_exp_helpers() {
        let p = COB_MPF_PREC;
        let x = Mpf::from_decimal_str("3.75", p);
        assert_eq!(x.floor().get_ui(), 3);
        let nx = Mpf::from_decimal_str("-3.25", p);
        assert_eq!(nx.floor().cmp_si(-4), Ordering::Equal);
        // get_d_2exp_exp: 4 = 1.0*2^2 -> d in [0.5,1) means 4 = 0.5*2^3 -> exp 3
        assert_eq!(Mpf::set_ui(4, p).get_d_2exp_exp(), 3);
        assert_eq!(Mpf::set_ui(1, p).get_d_2exp_exp(), 1);
    }
}
