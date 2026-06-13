//! A pure-Rust subset of GMP `mpz` (arbitrary-precision signed integers), faithful to the operations
//! libcob's `numeric.c` / `intrinsic.c` use, so the `cob_decimal` arithmetic can be a 1:1 port with
//! **zero runtime dependencies** (the project's standing rule) instead of linking libgmp.
//!
//! This is NOT a general bignum library -- it implements exactly the `mpz_*` surface the port needs,
//! with the same truncating/sign semantics GMP documents (so the ports below it are byte-faithful to
//! libcob, which is itself defined in terms of these GMP calls). Each method names its `mpz_*` analog.
//!
//! Representation: sign-magnitude. `mag` is little-endian base-2^64 limbs with no trailing zero limb
//! (so the zero value is `mag.is_empty()` and `sign == 0`).
#![forbid(unsafe_code)]

use core::cmp::Ordering;

/// An arbitrary-precision signed integer (`mpz_t`).
#[derive(Clone, Debug, Eq)]
pub struct Mpz {
    /// -1, 0, or +1. Invariant: `sign == 0` iff `mag` is empty.
    sign: i8,
    /// magnitude, little-endian 64-bit limbs, no trailing zeros.
    mag: Vec<u64>,
}

impl PartialEq for Mpz {
    fn eq(&self, other: &Self) -> bool {
        self.sign == other.sign && self.mag == other.mag
    }
}

impl Default for Mpz {
    fn default() -> Self {
        Mpz::new()
    }
}

impl Mpz {
    /// `mpz_init` / a freshly-`mpz_init2`-d value: zero.
    pub fn new() -> Self {
        Mpz { sign: 0, mag: Vec::new() }
    }

    /// `mpz_set_ull (dest, val)` (numeric.c:182, COB_EXPERIMENTAL): set to an unsigned 64-bit host
    /// integer. GnuCOBOL writes `_mp_d[0] = val & GMP_NUMB_MASK` and `_mp_size = (val != 0)` (a single
    /// limb where `GMP_LIMB_BITS >= 64`, which holds here); the magnitude is one 64-bit limb of `val`.
    pub fn set_ull(val: u64) -> Self {
        if val == 0 {
            Mpz::new()
        } else {
            Mpz { sign: 1, mag: vec![val] }
        }
    }

    /// `mpz_set_sll (dest, val)` (numeric.c:198, COB_EXPERIMENTAL): set to a signed 64-bit host integer
    /// — magnitude `|val|` in one limb, `_mp_size` carrying the sign of `val`.
    pub fn mpz_set_sll(val: i64) -> Self {
        let mag = (val as i128).unsigned_abs() as u64;
        if mag == 0 {
            Mpz::new()
        } else {
            Mpz { sign: if val < 0 { -1 } else { 1 }, mag: vec![mag] }
        }
    }

    /// `mpz_get_ull (src)` (numeric.c:216, COB_EXPERIMENTAL): the low 64-bit limb of the magnitude
    /// (`_mp_d[0]`), or 0 when the value is zero — wrapping past 64 bits exactly as the C does.
    pub fn mpz_get_ull(&self) -> u64 {
        self.mag.first().copied().unwrap_or(0)
    }

    /// `mpz_get_sll (src)` (numeric.c:236, COB_EXPERIMENTAL): reconstruct a signed 64-bit host integer
    /// from the low limb and the sign. Mirrors the C bit-for-bit: positive yields `vtmp & COB_MAX_LL`,
    /// negative yields `~((vtmp - 1) & COB_MAX_LL)`, with `COB_MAX_LL == i64::MAX`.
    pub fn mpz_get_sll(&self) -> i64 {
        if self.sign == 0 {
            return 0;
        }
        let vtmp = self.mag.first().copied().unwrap_or(0);
        if self.sign > 0 {
            (vtmp as i64) & i64::MAX
        } else {
            !(((vtmp as i64).wrapping_sub(1)) & i64::MAX)
        }
    }

    fn trim(mag: &mut Vec<u64>) {
        while mag.last() == Some(&0) {
            mag.pop();
        }
    }
    fn norm(sign: i8, mut mag: Vec<u64>) -> Self {
        Self::trim(&mut mag);
        if mag.is_empty() {
            Mpz { sign: 0, mag }
        } else {
            Mpz { sign, mag }
        }
    }

    // ---- set / get scalars (mpz_set_ui/si/ull/sll, mpz_get_ui/si/ull/sll) ----

    /// `mpz_set_ui`.
    pub fn set_ui(&mut self, v: u64) {
        *self = Self::from_u64(v);
    }
    /// `mpz_set_si`.
    pub fn set_si(&mut self, v: i64) {
        *self = Self::from_i64(v);
    }
    pub fn from_u64(v: u64) -> Self {
        if v == 0 {
            Mpz::new()
        } else {
            Mpz { sign: 1, mag: vec![v] }
        }
    }
    pub fn from_i64(v: i64) -> Self {
        if v == 0 {
            Mpz::new()
        } else if v > 0 {
            Mpz { sign: 1, mag: vec![v as u64] }
        } else {
            Mpz { sign: -1, mag: vec![(v as i128).unsigned_abs() as u64] }
        }
    }
    pub fn from_u128(v: u128) -> Self {
        Self::norm(if v == 0 { 0 } else { 1 }, vec![v as u64, (v >> 64) as u64])
    }
    pub fn from_i128(v: i128) -> Self {
        let neg = v < 0;
        let u = v.unsigned_abs();
        let m = Self::from_u128(u);
        if neg {
            Mpz { sign: -1, ..m }
        } else {
            m
        }
    }
    /// `mpz_get_ui`: the low 64 bits of the **absolute value** (GMP ignores the sign).
    pub fn get_ui(&self) -> u64 {
        self.mag.first().copied().unwrap_or(0)
    }
    /// `mpz_get_si`: low bits with sign, saturating like GMP's documented behavior for in-range values.
    pub fn get_si(&self) -> i64 {
        let lo = self.get_ui();
        if self.sign < 0 {
            (lo as i64).wrapping_neg()
        } else {
            lo as i64
        }
    }
    /// `mpz_fits_ulong_p` (treating ulong as u64): non-negative and a single limb.
    pub fn fits_ulong(&self) -> bool {
        self.sign >= 0 && self.mag.len() <= 1
    }

    // ---- sign / compare (mpz_sgn, mpz_cmp, mpz_cmpabs, mpz_abs, mpz_neg) ----

    /// `mpz_sgn`.
    pub fn sgn(&self) -> i32 {
        self.sign as i32
    }
    /// `mpz_neg` (in place).
    pub fn neg(&mut self) {
        self.sign = -self.sign;
    }
    /// `mpz_abs` (in place).
    pub fn abs(&mut self) {
        if self.sign != 0 {
            self.sign = 1;
        }
    }
    fn cmp_mag(a: &[u64], b: &[u64]) -> Ordering {
        if a.len() != b.len() {
            return a.len().cmp(&b.len());
        }
        for i in (0..a.len()).rev() {
            match a[i].cmp(&b[i]) {
                Ordering::Equal => {}
                o => return o,
            }
        }
        Ordering::Equal
    }
    /// `mpz_cmpabs`: compare absolute values.
    pub fn cmpabs(&self, other: &Mpz) -> Ordering {
        Self::cmp_mag(&self.mag, &other.mag)
    }
    /// `mpz_cmp`: signed compare.
    pub fn cmp(&self, other: &Mpz) -> Ordering {
        match self.sign.cmp(&other.sign) {
            Ordering::Equal => {
                if self.sign == 0 {
                    Ordering::Equal
                } else if self.sign > 0 {
                    self.cmpabs(other)
                } else {
                    self.cmpabs(other).reverse()
                }
            }
            o => o,
        }
    }

    // ---- magnitude add/sub helpers ----

    fn mag_add(a: &[u64], b: &[u64]) -> Vec<u64> {
        let (long, short) = if a.len() >= b.len() { (a, b) } else { (b, a) };
        let mut out = Vec::with_capacity(long.len() + 1);
        let mut carry = 0u128;
        for i in 0..long.len() {
            let mut cur = long[i] as u128 + carry;
            if i < short.len() {
                cur += short[i] as u128;
            }
            out.push(cur as u64);
            carry = cur >> 64;
        }
        if carry != 0 {
            out.push(carry as u64);
        }
        out
    }
    /// `a - b` for `a >= b` (magnitudes).
    fn mag_sub(a: &[u64], b: &[u64]) -> Vec<u64> {
        let mut out = Vec::with_capacity(a.len());
        let mut borrow = 0i128;
        for i in 0..a.len() {
            let bi = if i < b.len() { b[i] as i128 } else { 0 };
            let mut cur = a[i] as i128 - bi - borrow;
            if cur < 0 {
                cur += 1i128 << 64;
                borrow = 1;
            } else {
                borrow = 0;
            }
            out.push(cur as u64);
        }
        Self::trim(&mut out);
        out
    }
    fn add_signed(asign: i8, amag: &[u64], bsign: i8, bmag: &[u64]) -> Mpz {
        if asign == 0 {
            return Self::norm(bsign, bmag.to_vec());
        }
        if bsign == 0 {
            return Self::norm(asign, amag.to_vec());
        }
        if asign == bsign {
            Self::norm(asign, Self::mag_add(amag, bmag))
        } else {
            match Self::cmp_mag(amag, bmag) {
                Ordering::Equal => Mpz::new(),
                Ordering::Greater => Self::norm(asign, Self::mag_sub(amag, bmag)),
                Ordering::Less => Self::norm(bsign, Self::mag_sub(bmag, amag)),
            }
        }
    }

    /// `mpz_add`.
    pub fn add(&self, other: &Mpz) -> Mpz {
        Self::add_signed(self.sign, &self.mag, other.sign, &other.mag)
    }
    /// `mpz_sub`.
    pub fn sub(&self, other: &Mpz) -> Mpz {
        Self::add_signed(self.sign, &self.mag, -other.sign, &other.mag)
    }
    /// `mpz_add_ui`.
    pub fn add_ui(&self, v: u64) -> Mpz {
        self.add(&Mpz::from_u64(v))
    }
    /// `mpz_sub_ui`.
    pub fn sub_ui(&self, v: u64) -> Mpz {
        self.sub(&Mpz::from_u64(v))
    }

    // ---- multiply (mpz_mul, mpz_mul_ui, mpz_mul_2exp, mpz_ui_pow_ui) ----

    fn mag_mul(a: &[u64], b: &[u64]) -> Vec<u64> {
        if a.is_empty() || b.is_empty() {
            return Vec::new();
        }
        let mut out = vec![0u64; a.len() + b.len()];
        for (i, &ai) in a.iter().enumerate() {
            let mut carry = 0u128;
            for (j, &bj) in b.iter().enumerate() {
                let cur = out[i + j] as u128 + ai as u128 * bj as u128 + carry;
                out[i + j] = cur as u64;
                carry = cur >> 64;
            }
            out[i + b.len()] = out[i + b.len()].wrapping_add(carry as u64);
        }
        Self::trim(&mut out);
        out
    }
    /// `mpz_mul`.
    pub fn mul(&self, other: &Mpz) -> Mpz {
        Self::norm(self.sign * other.sign, Self::mag_mul(&self.mag, &other.mag))
    }
    /// `mpz_mul_ui`.
    pub fn mul_ui(&self, v: u64) -> Mpz {
        self.mul(&Mpz::from_u64(v))
    }
    /// `mpz_mul_2exp`: `self << bits`.
    pub fn mul_2exp(&self, bits: u32) -> Mpz {
        if self.sign == 0 {
            return Mpz::new();
        }
        let limb_shift = (bits / 64) as usize;
        let bit_shift = bits % 64;
        let mut m = vec![0u64; limb_shift];
        if bit_shift == 0 {
            m.extend_from_slice(&self.mag);
        } else {
            let mut carry = 0u64;
            for &l in &self.mag {
                m.push((l << bit_shift) | carry);
                carry = l >> (64 - bit_shift);
            }
            if carry != 0 {
                m.push(carry);
            }
        }
        Self::norm(self.sign, m)
    }
    /// `mpz_ui_pow_ui`: `base^exp`.
    pub fn ui_pow_ui(base: u64, exp: u32) -> Mpz {
        let mut r = Mpz::from_u64(1);
        let b = Mpz::from_u64(base);
        for _ in 0..exp {
            r = r.mul(&b);
        }
        r
    }

    // ---- divide (truncating: mpz_tdiv_q/r/q_ui/ui, fdiv_r/q_2exp) ----

    /// Divide magnitude by a single u64, returning `(quotient_mag, remainder)`.
    fn mag_divmod_u64(a: &[u64], d: u64) -> (Vec<u64>, u64) {
        let mut q = vec![0u64; a.len()];
        let mut rem: u128 = 0;
        for i in (0..a.len()).rev() {
            let cur = (rem << 64) | a[i] as u128;
            q[i] = (cur / d as u128) as u64;
            rem = cur % d as u128;
        }
        Self::trim(&mut q);
        (q, rem as u64)
    }
    /// `mpz_tdiv_q_ui`: truncated quotient by a u64. Returns the quotient.
    pub fn tdiv_q_ui(&self, d: u64) -> Mpz {
        let (q, _) = Self::mag_divmod_u64(&self.mag, d);
        Self::norm(self.sign, q)
    }
    /// `mpz_tdiv_ui`: the absolute remainder mod `d` (GMP returns |r|).
    pub fn tdiv_ui(&self, d: u64) -> u64 {
        Self::mag_divmod_u64(&self.mag, d).1
    }
    /// `mpz_divisible_ui_p`.
    pub fn divisible_ui(&self, d: u64) -> bool {
        self.sign == 0 || Self::mag_divmod_u64(&self.mag, d).1 == 0
    }
    /// Full truncated division `self / d`, returning `(quotient, remainder)` with the remainder
    /// taking the sign of the dividend (`mpz_tdiv_q` + `mpz_tdiv_r`).
    pub fn tdiv_qr(&self, d: &Mpz) -> (Mpz, Mpz) {
        debug_assert!(d.sign != 0);
        if Self::cmp_mag(&self.mag, &d.mag) == Ordering::Less {
            return (Mpz::new(), self.clone());
        }
        let (qmag, rmag) = Self::mag_divmod(&self.mag, &d.mag);
        let q = Self::norm(self.sign * d.sign, qmag);
        let r = Self::norm(self.sign, rmag);
        (q, r)
    }
    /// `mpz_tdiv_q`.
    pub fn tdiv_q(&self, d: &Mpz) -> Mpz {
        self.tdiv_qr(d).0
    }
    /// `mpz_tdiv_r`.
    pub fn tdiv_r(&self, d: &Mpz) -> Mpz {
        self.tdiv_qr(d).1
    }
    /// `mpz_fdiv_r_2exp`: the low `bits` bits (floor remainder by 2^bits). For non-negative values
    /// this is a bit mask; libcob only calls it on non-negative magnitudes here.
    pub fn fdiv_r_2exp(&self, bits: u32) -> Mpz {
        if self.sign == 0 {
            return Mpz::new();
        }
        let limbs = (bits / 64) as usize;
        let rem_bits = bits % 64;
        let mut m: Vec<u64> = self.mag.iter().take(limbs + 1).copied().collect();
        if rem_bits != 0 && m.len() > limbs {
            if let Some(top) = m.get_mut(limbs) {
                *top &= (1u64 << rem_bits) - 1;
            }
        } else if rem_bits == 0 {
            m.truncate(limbs);
        }
        Self::norm(self.sign, m)
    }
    /// `mpz_fdiv_q_2exp`: `self >> bits` (floor, but non-negative here).
    pub fn fdiv_q_2exp(&self, bits: u32) -> Mpz {
        if self.sign == 0 {
            return Mpz::new();
        }
        let limb_shift = (bits / 64) as usize;
        let bit_shift = bits % 64;
        if limb_shift >= self.mag.len() {
            return Mpz::new();
        }
        let mut m: Vec<u64> = self.mag[limb_shift..].to_vec();
        if bit_shift != 0 {
            let mut carry = 0u64;
            for l in m.iter_mut().rev() {
                let new = (*l >> bit_shift) | carry;
                carry = *l << (64 - bit_shift);
                *l = new;
            }
        }
        Self::norm(self.sign, m)
    }

    /// Schoolbook long division of magnitudes (binary), returning `(quotient, remainder)`.
    fn mag_divmod(a: &[u64], d: &[u64]) -> (Vec<u64>, Vec<u64>) {
        // bit-by-bit; d != 0. a >= d guaranteed by caller for the q!=0 case, but handle generally.
        let nbits = a.len() * 64;
        let mut q = vec![0u64; a.len()];
        let mut rem = Mpz::new();
        let dm = Mpz::norm(1, d.to_vec());
        for i in (0..nbits).rev() {
            // rem = (rem << 1) | bit_i(a)
            rem = rem.mul_2exp(1);
            let bit = (a[i / 64] >> (i % 64)) & 1;
            if bit != 0 {
                rem = rem.add(&Mpz::from_u64(1));
            }
            if Self::cmp_mag(&rem.mag, d) != Ordering::Less {
                rem = rem.sub(&dm);
                q[i / 64] |= 1u64 << (i % 64);
            }
        }
        Self::trim(&mut q);
        (q, rem.mag)
    }

    // ---- queries / misc ----

    /// `mpz_size`: number of limbs.
    pub fn size(&self) -> usize {
        self.mag.len()
    }
    /// `mpz_sizeinbase(_, 2)`: number of significant bits (1 for zero, like GMP).
    pub fn sizeinbase2(&self) -> usize {
        match self.mag.last() {
            None => 1,
            Some(&top) => (self.mag.len() - 1) * 64 + (64 - top.leading_zeros() as usize),
        }
    }
    /// `mpz_sqrt`: the floor of the integer square root, `floor(sqrt(self))`, for `self >= 0` (0 for a
    /// non-positive value). Newton iteration on integers — converges to the exact floor.
    pub fn isqrt(&self) -> Mpz {
        if self.sgn() <= 0 {
            return Mpz::new();
        }
        // initial over-estimate: 2^ceil(bits/2) >= sqrt(self)
        let mut x = Mpz::from_u64(1).mul_2exp(((self.sizeinbase2() + 1) / 2) as u32);
        loop {
            // y = floor((x + floor(self/x)) / 2)
            let y = x.add(&self.tdiv_q(&x)).fdiv_q_2exp(1);
            if y.cmp(&x) != Ordering::Less {
                return x;
            }
            x = y;
        }
    }

    /// `mpz_remove(_, _, 10)`: divide out all factors of ten, returning the count removed.
    pub fn remove_pow10(&mut self) -> u32 {
        if self.sign == 0 {
            return 0;
        }
        let mut count = 0;
        loop {
            let (q, r) = Self::mag_divmod_u64(&self.mag, 10);
            if r != 0 {
                break;
            }
            self.mag = q;
            count += 1;
        }
        Self::trim(&mut self.mag);
        if self.mag.is_empty() {
            self.sign = 0;
        }
        count
    }
    /// `mpz_com`: one's complement (`-self - 1`).
    pub fn com(&self) -> Mpz {
        self.add(&Mpz::from_u64(1)).into_neg()
    }
    fn into_neg(mut self) -> Mpz {
        self.sign = -self.sign;
        self
    }

    /// The value as `i128` if it fits (≤ 2 limbs and in range), else `None`. Used where the port has
    /// already reduced a value to a field's ≤38-digit precision (which always fits i128).
    pub fn to_i128(&self) -> Option<i128> {
        if self.mag.len() > 2 {
            return None;
        }
        let u = self.mag.first().copied().unwrap_or(0) as u128
            | ((self.mag.get(1).copied().unwrap_or(0) as u128) << 64);
        if self.sign < 0 {
            if u <= i128::MAX as u128 + 1 {
                Some((u as i128).wrapping_neg())
            } else {
                None
            }
        } else if u <= i128::MAX as u128 {
            Some(u as i128)
        } else {
            None
        }
    }

    /// `mpz_get_str(_, 10, _)`: decimal string.
    pub fn to_decimal_string(&self) -> String {
        if self.sign == 0 {
            return "0".to_string();
        }
        let mut m = self.mag.clone();
        let mut chunks: Vec<u64> = Vec::new();
        while !m.is_empty() {
            let (q, r) = Self::mag_divmod_u64(&m, 1_000_000_000_000_000_000);
            chunks.push(r);
            m = q;
        }
        let mut s = String::new();
        if self.sign < 0 {
            s.push('-');
        }
        for (i, c) in chunks.iter().rev().enumerate() {
            if i == 0 {
                s.push_str(&c.to_string());
            } else {
                s.push_str(&format!("{c:018}"));
            }
        }
        s
    }
    /// `mpz_set_str(_, _, 10)`: parse a decimal string (optional leading sign).
    pub fn from_decimal_string(s: &str) -> Mpz {
        let s = s.trim();
        let (neg, digits) = match s.strip_prefix('-') {
            Some(d) => (true, d),
            None => (false, s.strip_prefix('+').unwrap_or(s)),
        };
        let mut r = Mpz::new();
        let bytes = digits.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let end = (i + 18).min(bytes.len());
            let mut chunk: u64 = 0;
            let mut p = 10u64.pow((end - i) as u32 - 1);
            for &b in &bytes[i..end] {
                if b.is_ascii_digit() {
                    chunk += (b - b'0') as u64 * p;
                    p /= 10;
                }
            }
            let scale = Mpz::from_u64(10u64.pow((end - i) as u32));
            r = r.mul(&scale).add(&Mpz::from_u64(chunk));
            i = end;
        }
        if neg && r.sign != 0 {
            r.sign = -1;
        }
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(m: &Mpz) -> String {
        m.to_decimal_string()
    }

    #[test]
    fn basic_arith() {
        let a = Mpz::from_decimal_string("123456789012345678901234567890");
        let b = Mpz::from_decimal_string("987654321098765432109876543210");
        assert_eq!(s(&a.add(&b)), "1111111110111111111011111111100");
        assert_eq!(s(&b.sub(&a)), "864197532086419753208641975320");
        assert_eq!(
            s(&a.mul(&b)),
            "121932631137021795226185032733622923332237463801111263526900"
        );
        assert_eq!(a.cmp(&b), Ordering::Less);
        assert_eq!(a.cmpabs(&b), Ordering::Less);
    }

    #[test]
    fn signs_and_zero() {
        let a = Mpz::from_i64(-5);
        let b = Mpz::from_i64(5);
        assert_eq!(a.add(&b), Mpz::new());
        assert_eq!(a.sgn(), -1);
        assert_eq!(s(&a.mul(&b)), "-25");
        assert_eq!(a.get_ui(), 5); // |value| low bits
        assert_eq!(a.get_si(), -5);
    }

    #[test]
    fn division_truncating() {
        let a = Mpz::from_decimal_string("-1000000000000000000000007");
        let d = Mpz::from_u64(1000);
        let (q, r) = a.tdiv_qr(&d);
        assert_eq!(s(&q), "-1000000000000000000000"); // toward zero
        assert_eq!(s(&r), "-7"); // remainder takes dividend sign
        assert_eq!(a.tdiv_ui(1000), 7); // |remainder|
    }

    #[test]
    fn shifts_and_remove() {
        let mut x = Mpz::from_decimal_string("123000");
        assert_eq!(x.remove_pow10(), 3);
        assert_eq!(s(&x), "123");
        let y = Mpz::from_u64(1).mul_2exp(100);
        assert_eq!(s(&y), "1267650600228229401496703205376");
        assert_eq!(y.fdiv_q_2exp(100), Mpz::from_u64(1));
        assert_eq!(Mpz::from_u64(0b1011).fdiv_r_2exp(2), Mpz::from_u64(0b11));
        assert_eq!(Mpz::ui_pow_ui(10, 20), Mpz::from_decimal_string("100000000000000000000"));
    }

    #[test]
    fn sizeinbase_and_str_roundtrip() {
        assert_eq!(Mpz::from_u64(255).sizeinbase2(), 8);
        assert_eq!(Mpz::new().sizeinbase2(), 1);
        for v in ["0", "-1", "42", "-1000000000000000000000000000000001"] {
            assert_eq!(s(&Mpz::from_decimal_string(v)), v);
        }
    }
}
