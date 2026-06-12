//! Binary floating-point COBOL fields: `COMP-1` (IEEE-754 single / `f32`) and `COMP-2` (IEEE-754
//! double / `f64`), and the decimal<->float conversions GnuCOBOL performs (`GNURUST.FLOAT.1`).
//!
//! The field bytes are the native IEEE-754 encoding. The subtle part is the conversion between a
//! decimal value and the float, where libcob goes through GMP: `cob_decimal_get_mpf` divides the
//! integer significand by `10^scale` into a 2048-bit `mpf` (`COB_MPF_PREC`), then `mpf_get_d`
//! converts to the C double **truncating toward zero** (GMP `mpf_get_d` rounds toward zero, not to
//! nearest). So a decimal that is not exactly representable lands on the float of largest magnitude
//! that does not exceed it -- 1 ULP below a correctly-rounded parse for inexact values. This module
//! reproduces that exactly, using an arbitrary-precision unsigned integer for the exact comparison.
#![forbid(unsafe_code)]

/// A minimal arbitrary-precision unsigned integer (base 2^64, little-endian limbs). Only the
/// operations the float conversions need: build, shift, multiply by 10^k, compare, and decimal
/// digit extraction. Not a general bignum -- no subtraction or general division.
#[derive(Clone, Debug, PartialEq, Eq)]
struct BigU {
    /// little-endian 64-bit limbs, no trailing zero limbs (except `[0]`-free: empty == zero).
    limbs: Vec<u64>,
}

impl BigU {
    fn zero() -> Self {
        BigU { limbs: Vec::new() }
    }
    fn from_u128(v: u128) -> Self {
        let mut limbs = vec![v as u64, (v >> 64) as u64];
        Self::trim(&mut limbs);
        BigU { limbs }
    }
    fn is_zero(&self) -> bool {
        self.limbs.is_empty()
    }
    fn trim(limbs: &mut Vec<u64>) {
        while limbs.last() == Some(&0) {
            limbs.pop();
        }
    }
    /// Multiply in place by a small `u64`.
    fn mul_u64(&mut self, m: u64) {
        if m == 0 || self.is_zero() {
            self.limbs.clear();
            return;
        }
        let mut carry: u128 = 0;
        for l in self.limbs.iter_mut() {
            let cur = *l as u128 * m as u128 + carry;
            *l = cur as u64;
            carry = cur >> 64;
        }
        while carry != 0 {
            self.limbs.push(carry as u64);
            carry >>= 64;
        }
    }
    /// Multiply in place by 10^k.
    fn mul_pow10(&mut self, k: u32) {
        let mut rem = k;
        // 10^19 is the largest power of ten < 2^64.
        while rem >= 19 {
            self.mul_u64(10u64.pow(19));
            rem -= 19;
        }
        if rem > 0 {
            self.mul_u64(10u64.pow(rem));
        }
    }
    /// Shift left by `bits` (multiply by 2^bits).
    fn shl(&mut self, bits: u32) {
        if self.is_zero() {
            return;
        }
        let limb_shift = (bits / 64) as usize;
        let bit_shift = bits % 64;
        if bit_shift != 0 {
            let mut carry: u64 = 0;
            for l in self.limbs.iter_mut() {
                let new = (*l << bit_shift) | carry;
                carry = *l >> (64 - bit_shift);
                *l = new;
            }
            if carry != 0 {
                self.limbs.push(carry);
            }
        }
        if limb_shift != 0 {
            let mut v = vec![0u64; limb_shift];
            v.extend_from_slice(&self.limbs);
            self.limbs = v;
        }
    }
    /// Floor division by 2^bits (shift right, truncating toward zero).
    fn shr(&mut self, bits: u32) {
        let limb_shift = (bits / 64) as usize;
        let bit_shift = bits % 64;
        if limb_shift >= self.limbs.len() {
            self.limbs.clear();
            return;
        }
        if limb_shift != 0 {
            self.limbs.drain(0..limb_shift);
        }
        if bit_shift != 0 {
            let mut carry: u64 = 0;
            for l in self.limbs.iter_mut().rev() {
                let new = (*l >> bit_shift) | carry;
                carry = *l << (64 - bit_shift);
                *l = new;
            }
        }
        Self::trim(&mut self.limbs);
    }
    fn cmp(&self, other: &BigU) -> std::cmp::Ordering {
        use std::cmp::Ordering::*;
        if self.limbs.len() != other.limbs.len() {
            return self.limbs.len().cmp(&other.limbs.len());
        }
        for i in (0..self.limbs.len()).rev() {
            match self.limbs[i].cmp(&other.limbs[i]) {
                Equal => {}
                ord => return ord,
            }
        }
        Equal
    }
    /// Divide in place by a small `u64`, returning the remainder.
    fn divmod_u64(&mut self, d: u64) -> u64 {
        let mut rem: u128 = 0;
        for l in self.limbs.iter_mut().rev() {
            let cur = (rem << 64) | *l as u128;
            *l = (cur / d as u128) as u64;
            rem = cur % d as u128;
        }
        Self::trim(&mut self.limbs);
        rem as u64
    }
    /// The exact decimal digits, most-significant first (`[0]` for zero).
    fn to_decimal(&self) -> Vec<u8> {
        if self.is_zero() {
            return vec![0];
        }
        let mut n = self.clone();
        let mut chunks: Vec<u64> = Vec::new();
        while !n.is_zero() {
            chunks.push(n.divmod_u64(10u64.pow(18)));
        }
        let mut out = Vec::new();
        for (i, &c) in chunks.iter().rev().enumerate() {
            let s = if i == 0 { format!("{c}") } else { format!("{c:018}") };
            out.extend(s.bytes().map(|b| b - b'0'));
        }
        out
    }
}

/// Decompose a finite, non-zero, positive `f64` into `(m, e)` with `value = m * 2^e`, `m` a 53-bit
/// (or smaller, for subnormals) integer.
fn decompose_f64(v: f64) -> (u64, i32) {
    let bits = v.to_bits();
    let exp_field = ((bits >> 52) & 0x7FF) as i32;
    let frac = bits & 0x000F_FFFF_FFFF_FFFF;
    if exp_field == 0 {
        // subnormal: value = frac * 2^-1074
        (frac, -1074)
    } else {
        // normal: value = (frac | 2^52) * 2^(exp_field - 1075)
        (frac | 0x0010_0000_0000_0000, exp_field - 1075)
    }
}

/// Parse the decimal `umag / 10^scale` to an `f64`, round-to-nearest (Rust's correctly-rounded parse).
fn parse_decimal(umag: u128, scale: i32) -> f64 {
    // umag * 10^-scale ; format then parse handles the full exponent range correctly.
    format!("{umag}e{}", -scale).parse::<f64>().unwrap_or(0.0)
}

/// Compare `m * 2^e` (a positive f64's value) to `umag / 10^scale` exactly.
fn cmp_f64_to_decimal(m: u64, e: i32, umag: u128, scale: i32) -> std::cmp::Ordering {
    // Clear denominators: compare m*2^e*10^scale vs umag (e>=0), or m*10^scale vs umag*2^-e (e<0).
    let mut lhs = BigU::from_u128(m as u128);
    let mut rhs = BigU::from_u128(umag);
    if scale >= 0 {
        lhs.mul_pow10(scale as u32);
    } else {
        rhs.mul_pow10((-scale) as u32);
    }
    if e >= 0 {
        lhs.shl(e as u32);
    } else {
        rhs.shl((-e) as u32);
    }
    lhs.cmp(&rhs)
}

/// The next `f64` toward zero (for a positive, finite `v`).
fn next_toward_zero(v: f64) -> f64 {
    f64::from_bits(v.to_bits() - 1)
}

/// Convert the decimal `mag / 10^scale` to the `f64` GnuCOBOL stores in a `COMP-2` field: the value
/// **truncated toward zero** to the nearest representable double (libcob `mpf_get_d`). Exact values
/// are unchanged; inexact values land on the double of largest magnitude not exceeding the decimal.
pub fn decimal_to_f64_trunc(mag: i128, scale: i32) -> f64 {
    if mag == 0 {
        return 0.0;
    }
    let neg = mag < 0;
    let umag = mag.unsigned_abs();
    let f_rn = parse_decimal(umag, scale);
    let result = if f_rn == 0.0 {
        0.0
    } else if !f_rn.is_finite() {
        f_rn // overflowed the double range; keep the inf (sign applied below)
    } else {
        let (m, e) = decompose_f64(f_rn);
        match cmp_f64_to_decimal(m, e, umag, scale) {
            // f_rn > V: the round-to-nearest overshot, step toward zero to truncate.
            std::cmp::Ordering::Greater => next_toward_zero(f_rn),
            // f_rn <= V: f_rn is already the largest double not exceeding V.
            _ => f_rn,
        }
    };
    if neg {
        -result
    } else {
        result
    }
}

/// `COMP-1` single-precision: libcob stores `(float) cob_decimal_get_double (d)` -- it truncates the
/// decimal to a double toward zero (the same as `COMP-2`), then casts that double to a `float`
/// **rounding to nearest** (the C `(float)` cast). So the narrowing is round-to-nearest, NOT a second
/// truncation -- the `as f32` cast reproduces it exactly.
pub fn decimal_to_f32_trunc(mag: i128, scale: i32) -> f32 {
    decimal_to_f64_trunc(mag, scale) as f32
}

/// Convert a `COMP-2`/`COMP-1` value to the decimal a DISPLAY/packed field stores at `scale`: libcob
/// sets the decimal exactly from the double (`mpf_set_d`, exact for any double) and the store then
/// **truncates toward zero** to the field scale. So the result is `floor(|v| * 10^scale)` (low
/// `<=38` digits, which is all a COBOL numeric field can hold), with `v`'s sign. `scale >= 0`.
pub fn f64_to_decimal_trunc(v: f64, scale: i32) -> (i128, bool) {
    if v == 0.0 || !v.is_finite() {
        return (0, v.is_sign_negative());
    }
    let neg = v < 0.0;
    let (m, e) = decompose_f64(v.abs());
    let mut big = BigU::from_u128(m as u128);
    if scale > 0 {
        big.mul_pow10(scale as u32);
    } else if scale < 0 {
        // negative field scale (P-scaling to the left): divide by 10^-scale, truncating.
        let mut k = (-scale) as u32;
        while k >= 18 {
            big.divmod_u64(10u64.pow(18));
            k -= 18;
        }
        if k > 0 {
            big.divmod_u64(10u64.pow(k));
        }
    }
    if e >= 0 {
        big.shl(e as u32);
    } else {
        big.shr((-e) as u32); // floor division by 2^-e (truncate toward zero, v already abs)
    }
    // big = floor(|v| * 10^scale); keep the low 38 decimal digits (fit i128; field truncates further).
    let digits = big.to_decimal();
    let start = digits.len().saturating_sub(38);
    let mut mag: i128 = 0;
    for &d in &digits[start..] {
        mag = mag * 10 + d as i128;
    }
    (mag, neg)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bits(v: f64) -> u64 {
        v.to_bits()
    }

    #[test]
    fn f64_to_decimal_round_trips_and_truncates() {
        // 123.45 stored at scale 2 -> 12345 (the f64 is ~123.45000000000003, floor(.*100)=12345)
        assert_eq!(f64_to_decimal_trunc(123.45, 2), (12345, false));
        // 1/3 at scale 15 -> 333333333333333 (floor of 0.33333333333333331.. * 1e15)
        assert_eq!(f64_to_decimal_trunc(1.0 / 3.0, 15), (333333333333333, false));
        // integer value, scale 0
        assert_eq!(f64_to_decimal_trunc(123.0, 0), (123, false));
        // negative + truncation toward zero (not floor toward -inf): -2.7 at scale 0 -> -2
        assert_eq!(f64_to_decimal_trunc(-2.7, 0), (2, true));
        assert_eq!(f64_to_decimal_trunc(0.0, 5), (0, false));
    }

    #[test]
    fn decimal_to_f64_truncates_toward_zero() {
        // Oracle-characterized (decimal_harness, COMP-2): these land 1 ULP BELOW round-to-nearest.
        assert_eq!(bits(decimal_to_f64_trunc(12345, 2)), 0x405e_dccc_cccc_ccccu64); // 123.45 -> ...cc
        assert_eq!(bits(decimal_to_f64_trunc(1, 1)), 0x3fb9_9999_9999_9999u64); // 0.1 -> ...99
        assert_eq!(bits(decimal_to_f64_trunc(2, 1)), 0x3fc9_9999_9999_9999u64); // 0.2
        assert_eq!(bits(decimal_to_f64_trunc(100001, 3)), 0x4059_0010_624d_d2f1u64); // 100.001
        // exactly representable -> unchanged
        assert_eq!(decimal_to_f64_trunc(25, 1), 2.5);
        assert_eq!(decimal_to_f64_trunc(123, 0), 123.0);
        // sign: negative is the negation of the positive truncation
        assert_eq!(decimal_to_f64_trunc(-25, 1), -2.5);
        assert_eq!(decimal_to_f64_trunc(-12345, 2), -decimal_to_f64_trunc(12345, 2));
    }

    #[test]
    fn bigu_ops() {
        let mut a = BigU::from_u128(12345);
        a.mul_pow10(20);
        // 12345 * 10^20
        let d: String = a.to_decimal().iter().map(|&x| (x + b'0') as char).collect();
        assert_eq!(d, "1234500000000000000000000");
        let mut b = BigU::from_u128(1);
        b.shl(100);
        let d2: String = b.to_decimal().iter().map(|&x| (x + b'0') as char).collect();
        assert_eq!(d2, "1267650600228229401496703205376"); // 2^100
    }
}
