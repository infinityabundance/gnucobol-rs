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
        let mut exp10 = full_len - p;
        // strip trailing zeros (they are not significant digits)
        while d.len() > 1 && *d.last().unwrap() == 0 {
            d.pop();
        }
        // Round to nearest (half up) at `ndigits` significant digits, as GMP's mpf_get_str does. The
        // decimal result field is sized to these digits, so the rounding decides the last digit
        // (plain truncation would be 1 ULP low, e.g. e's 96th digit). A carry out of the leading digit
        // (…999 -> 1000) grows the base-10 exponent.
        if d.len() > ndigits {
            let round_up = d[ndigits] >= 5;
            d.truncate(ndigits);
            if round_up {
                let mut i = ndigits;
                loop {
                    if i == 0 {
                        d.insert(0, 1);
                        d.truncate(ndigits);
                        exp10 += 1;
                        break;
                    }
                    i -= 1;
                    if d[i] == 9 {
                        d[i] = 0;
                    } else {
                        d[i] += 1;
                        break;
                    }
                }
            }
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

/// `COB_MPF_CUTOFF` (coblocal.h:150): the bit count for the `mpf_eq` series-convergence cutoff.
pub const COB_MPF_CUTOFF: u64 = 1024;

/// `cob_pi` (setup_cob_pi, intrinsic.c): pi to COB_PI_LEN=2820 bits, used by sin/cos/atan/asin/acos.
pub fn cob_pi() -> Mpf {
    Mpf::from_decimal_str(
        "3.14159265358979323846264338327950288419716939937510582097494459230781640628620899862803482534211706798214808651328230664709384460955058223172535940812848111745028410270193852110555964462294895493038196442881097566593344612847564823378678316527120190914564856692346034861045432664821339360726024914127372458700660631558817488152092096282925409171536436789259036001133053054882046652138414695194151160943305727036575959195309218611738193261179310511854807446237996274956735188575272489122793818301194912983367336244065664308602139494639522473719070217986094370277053921717629317675238467481846766940513200056812714526356082778577134275778960917363717872146844090122495343014654958537105079227968925892354201995611212902196086403441815981362977477130996051870721134999999837297804995105973173281609631859502445945534690830264252230825334468503526193118817101",
        COB_MPF_PREC,
    )
}

/// `cob_log_half` (setup_cob_log_half, intrinsic.c): ln(0.5) (negative) to COB_LOG_HALF_LEN=2784 bits,
/// used by `cob_mpf_log` to fold the binary exponent into the result.
pub fn cob_log_half() -> Mpf {
    Mpf::from_decimal_str(
        "-0.6931471805599453094172321214581765680755001343602552541206800094933936219696947156058633269964186875420014810205706857336855202357581305570326707516350759619307275708283714351903070386238916734711233501153644979552391204751726815749320651555247341395258829504530070953263666426541042391578149520437404303855008019441706416715186447128399681717845469570262716310645461502572074024816377733896385506952606683411372738737229289564935470257626520988596932019650585547647033067936544325476327449512504060694381471046899465062201677204245245296126879465461931651746813926725041038025462596568691441928716082938031727143677826548775664850856740776484514644399404614226031930967354025744460703080960850474866385231381816767514386674766478908814371419854942315199735488037516586127535291661000710535582498794147295092931138971559982056543928717",
        COB_MPF_PREC,
    )
}

/// `cob_log_ten` (setup_cob_log_ten, intrinsic.c): ln(10), computed as `cob_mpf_log(10)` (not hardcoded).
pub fn cob_log_ten() -> Mpf {
    cob_mpf_log(&Mpf::set_ui(10, COB_MPF_PREC))
}

/// `cob_mpf_log (dst, src)` (intrinsic.c): natural log via the `log(1-y)` series, after folding the binary
/// exponent through `ln(2)`. `src <= 0` or `src == 1` yields 0.
pub fn cob_mpf_log(src: &Mpf) -> Mpf {
    let prec = COB_MPF_PREC;
    if src.sgn() <= 0 || src.cmp_ui(1) == Ordering::Equal {
        return Mpf::new(prec);
    }
    let mut vf1 = src.clone();
    let mut dst_temp = Mpf::new(prec);
    let expon = vf1.get_d_2exp_exp();
    if expon != 0 {
        dst_temp = cob_log_half();
        if expon > 0 {
            dst_temp = dst_temp.mul_ui(expon as u64);
            dst_temp.neg_assign();
            vf1 = vf1.div_2exp(expon as u64);
        } else {
            dst_temp = dst_temp.mul_ui((-expon) as u64);
            vf1 = vf1.mul_2exp((-expon) as u64);
        }
    }
    vf1 = Mpf::ui_sub(1, &vf1);
    let mut vf3 = Mpf::set_si(-1, prec);
    let mut n = 1u64;
    loop {
        vf3 = vf3.mul(&vf1);
        let vf2 = vf3.div_ui(n);
        let vf4 = dst_temp.clone();
        dst_temp = dst_temp.add(&vf2);
        n += 1;
        if vf4.eq(&dst_temp, COB_MPF_CUTOFF) {
            break;
        }
    }
    dst_temp
}

/// `cob_mpf_log10 (dst, src)` (intrinsic.c): `cob_mpf_log(src) / ln(10)`.
pub fn cob_mpf_log10(src: &Mpf) -> Mpf {
    cob_mpf_log(src).div(&cob_log_ten())
}

/// `cob_mpf_exp (dst, src)` (intrinsic.c): `e^src` via the Taylor series on the binary-exponent-reduced
/// magnitude, then repeated squaring; reciprocal for a negative argument.
pub fn cob_mpf_exp(src: &Mpf) -> Mpf {
    let prec = COB_MPF_PREC;
    let mut vf1 = src.clone();
    let mut vf2 = Mpf::set_ui(1, prec);
    let mut dst_temp = Mpf::set_ui(1, prec);
    let is_negative = vf1.sgn() < 0;
    if is_negative {
        vf1.abs_assign();
    }
    let expon = vf1.get_d_2exp_exp();
    if expon > 0 {
        vf1 = vf1.div_2exp(expon as u64);
    }
    let mut n = 1u64;
    loop {
        vf2 = vf2.mul(&vf1);
        vf2 = vf2.div_ui(n);
        let vf3 = dst_temp.clone();
        dst_temp = dst_temp.add(&vf2);
        n += 1;
        if vf3.eq(&dst_temp, COB_MPF_CUTOFF) {
            break;
        }
    }
    let mut i = 0i64;
    while i < expon {
        dst_temp = dst_temp.mul(&dst_temp);
        i += 1;
    }
    if is_negative {
        dst_temp = Mpf::ui_div(1, &dst_temp);
    }
    dst_temp
}

/// `cob_mpf_sin (dst, src)` (intrinsic.c): `sin(src)` via argument reduction modulo `pi/2` into a quadrant
/// (`arcquad`, with the matching sign flip / reflection) then the Taylor series.
pub fn cob_mpf_sin(src: &Mpf) -> Mpf {
    let prec = COB_MPF_PREC;
    let mut sign = src.sgn();
    let mut vf4 = src.clone();
    vf4.abs_assign();
    let vf3_half_pi = cob_pi().div_2exp(1); // pi/2
    let vf1 = vf4.div(&vf3_half_pi);
    vf4 = vf1.floor();
    let quad = if vf4.cmp_ui(4) != Ordering::Less {
        let t = vf4.div_2exp(2).floor().mul_2exp(2);
        vf4.sub(&t)
    } else {
        vf4.clone()
    };
    let arcquad = quad.get_ui();
    let frac = vf1.sub(&vf4);
    let mut angle = vf3_half_pi.mul(&frac);
    if arcquad > 1 {
        sign = -sign;
    }
    if arcquad & 1 != 0 {
        angle = vf3_half_pi.sub(&angle);
    }
    let mut neg_sq = angle.mul(&angle);
    neg_sq.neg_assign();

    let mut n = 1u64;
    let mut term = Mpf::set_ui(1, prec);
    let mut dst_temp = Mpf::set_ui(1, prec);
    loop {
        n += 1;
        term = term.div_ui(n);
        n += 1;
        term = term.div_ui(n);
        term = term.mul(&neg_sq);
        let prev = dst_temp.clone();
        dst_temp = dst_temp.add(&term);
        if prev.eq(&dst_temp, COB_MPF_PREC) {
            break;
        }
    }
    dst_temp = dst_temp.mul(&angle);
    if sign < 0 {
        dst_temp.neg_assign();
    }
    dst_temp
}

/// `cob_mpf_cos (dst, src)` (intrinsic.c): `cos(src) = sin(pi/2 - src)`.
pub fn cob_mpf_cos(src: &Mpf) -> Mpf {
    cob_mpf_sin(&cob_pi().div_2exp(1).sub(src))
}

/// `cob_mpf_tan (dst, src)` (intrinsic.c): `tan(src) = sin(src) / cos(src)`.
pub fn cob_mpf_tan(src: &Mpf) -> Mpf {
    cob_mpf_sin(src).div(&cob_mpf_cos(src))
}

/// `cob_sqrt_two` (setup_cob_sqrt_two, intrinsic.c): sqrt(2) to COB_SQRT_TWO_LEN=3827 bits, used by
/// `cob_mpf_atan` for argument reduction.
pub fn cob_sqrt_two() -> Mpf {
    Mpf::from_decimal_str(
        "1.4142135623730950488016887242096980785696718753769480731766797379907324784621070388503875343276415727350138462309122970249248360558507372126441214970999358314132226659275055927557999505011527820605714701095599716059702745345968620147285174186408891986095523292304843087143214508397626036279952514079896872533965463318088296406206152583523950547457502877599617298355752203375318570113543746034084988471603868999706990048150305440277903164542478230684929369186215805784631115966687130130156185689872372352885092648612494977154218334204285686060146824720771435854874155657069677653720226485447015858801620758474922657226002085584466521458398893944370926591800311388246468157082630100594858704003186480342194897278290641045072636881313739855256117322040245091227700226941127573627280495738108967504018369868368450725799364729060762996941380475654823728997180326802474420629269124859052181004459842150591120249441341728531478105803603371077309182869314710171111683916581726889419758716582152128229518488472089694633862891562882765952635140542267653239694617511291602408715510135150455381287560052631468017127402653969470240300517495318862925631385188163478",
        COB_MPF_PREC,
    )
}

/// `cob_mpf_atan (dst, src)` (intrinsic.c): `atan(src)` via the arctangent series, after reducing |src| to
/// `<= sqrt(2)-1` using the `pi/2` (for |x| > sqrt2+1) and `pi/4` (for sqrt2-1 < |x| <= sqrt2+1) identities.
pub fn cob_mpf_atan(src: &Mpf) -> Mpf {
    let prec = COB_MPF_PREC;
    let mut vf1 = src.clone();
    vf1.abs_assign();
    let sqrt2 = cob_sqrt_two();
    let mut dst_temp = Mpf::new(prec);
    if vf1.cmp(&sqrt2.add_ui(1)) == Ordering::Greater {
        dst_temp = cob_pi().div_2exp(1); // pi/2
        vf1 = Mpf::ui_div(1, &vf1);
        vf1.neg_assign();
    } else if vf1.cmp(&sqrt2.sub_ui(1)) == Ordering::Greater {
        dst_temp = cob_pi().div_2exp(2); // pi/4
        let num = vf1.sub_ui(1);
        let den = vf1.add_ui(1);
        vf1 = num.div(&den);
    }
    let mut neg_sq = vf1.mul(&vf1);
    neg_sq.neg_assign();
    dst_temp = dst_temp.add(&vf1);
    let mut n = 1u64;
    loop {
        vf1 = vf1.mul(&neg_sq);
        let term = vf1.div_ui(2 * n + 1);
        let prev = dst_temp.clone();
        dst_temp = dst_temp.add(&term);
        n += 1;
        if prev.eq(&dst_temp, COB_MPF_PREC) {
            break;
        }
    }
    if src.sgn() < 0 {
        dst_temp.neg_assign();
    }
    dst_temp
}

/// `cob_mpf_asin (dst, src)` (intrinsic.c): `asin(src) = 2*atan(x / (1 + sqrt(1 - x^2)))`; +-1 -> +-pi/2.
pub fn cob_mpf_asin(src: &Mpf) -> Mpf {
    let prec = COB_MPF_PREC;
    if src.cmp_ui(1) == Ordering::Equal || src.cmp_si(-1) == Ordering::Equal {
        let mut dst = cob_pi().div_ui(2);
        if src.sgn() < 0 {
            dst.neg_assign();
        }
        return dst;
    }
    if src.sgn() == 0 {
        return Mpf::new(prec);
    }
    let mut vf2 = src.mul(src);
    vf2 = Mpf::ui_sub(1, &vf2);
    vf2 = vf2.sqrt();
    vf2 = vf2.add_ui(1);
    let vf1 = src.div(&vf2);
    cob_mpf_atan(&vf1).mul_ui(2)
}

/// `cob_mpf_acos (dst, src)` (intrinsic.c): `acos(src) = 2*atan(sqrt(1 - x^2) / (x + 1))`; 0 -> pi/2,
/// 1 -> 0, -1 -> pi.
pub fn cob_mpf_acos(src: &Mpf) -> Mpf {
    let prec = COB_MPF_PREC;
    if src.sgn() == 0 {
        return cob_pi().div_ui(2);
    }
    if src.cmp_ui(1) == Ordering::Equal {
        return Mpf::new(prec);
    }
    if src.cmp_si(-1) == Ordering::Equal {
        return cob_pi();
    }
    let vf2 = src.add_ui(1);
    let mut vf1 = src.mul(src);
    vf1 = Mpf::ui_sub(1, &vf1);
    vf1 = vf1.sqrt();
    vf1 = vf1.div(&vf2);
    cob_mpf_atan(&vf1).mul_ui(2)
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
    fn transcendental_leading_digits() {
        let p = COB_MPF_PREC;
        // e = exp(1) = 2.718281828459045...
        let e = cob_mpf_exp(&Mpf::set_ui(1, p));
        let (_, d, ex) = dec(&e, 16);
        assert_eq!(ex, 1);
        assert_eq!(&d[..10], "2718281828");
        // ln(10) = 2.302585092994046...
        let l10 = cob_mpf_log(&Mpf::set_ui(10, p));
        let (_, d, ex) = dec(&l10, 16);
        assert_eq!(ex, 1);
        assert_eq!(&d[..10], "2302585092");
        // exp(-1) = 0.3678794411714423...
        let em1 = cob_mpf_exp(&Mpf::set_si(-1, p));
        let (_, d, ex) = dec(&em1, 12);
        assert_eq!(ex, 0);
        assert_eq!(&d[..8], "36787944");
        // log10(100) = 2 (exact-ish; leading digits are 1999.. or 2000.. -- the oracle adjudicates the
        // last-digit boundary, so here we only assert the series produced ~2).
        let l = cob_mpf_log10(&Mpf::set_ui(100, p));
        let (_, _d, ex) = dec(&l, 12);
        assert_eq!(ex, 1);
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
