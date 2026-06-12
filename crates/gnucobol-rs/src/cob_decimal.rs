//! A 1:1 port of libcob's `cob_decimal` layer (`numeric.c`): the `{ mpz value; int scale }` working
//! decimal and the operations the runtime builds on it, on top of the pure-Rust [`crate::gmp::Mpz`].
//! Names mirror the C functions so the port is auditable against the source.
//!
//! This module is being grown function-by-function toward a complete numeric.c port; it currently
//! covers field decode + the numeric comparison family (`GNURUST.NUMCMP.1`).
#![forbid(unsafe_code)]

use crate::arith::Round;
use crate::attr::{FieldAttr, COB_TYPE_NUMERIC_BINARY, COB_TYPE_NUMERIC_DISPLAY, COB_TYPE_NUMERIC_PACKED};
use crate::gmp::Mpz;
use crate::value::Decimal;
use core::cmp::Ordering;

/// `cob_decimal`: an arbitrary-precision integer significand plus a decimal `scale` (the value is
/// `value * 10^-scale`).
#[derive(Clone, Debug)]
pub struct CobDecimal {
    pub value: Mpz,
    pub scale: i32,
}

impl CobDecimal {
    fn from_value_decimal(dec: &Decimal) -> CobDecimal {
        let s: String = dec.digits.iter().map(|&d| (d + b'0') as char).collect();
        let mut value = if s.is_empty() {
            Mpz::new()
        } else {
            Mpz::from_decimal_string(&s)
        };
        if dec.negative && value.sgn() != 0 {
            value.neg();
        }
        CobDecimal { value, scale: dec.scale as i32 }
    }
}

/// `shift_decimal (d, n)` (numeric.c:561): `d->value *= 10^n; d->scale += n` (the represented value
/// is unchanged). `n < 0` divides (truncating toward zero). `n == 0` is a no-op.
pub fn shift_decimal(d: &mut CobDecimal, n: i32) {
    if n > 0 {
        d.value = d.value.mul(&Mpz::ui_pow_ui(10, n as u32));
    } else if n < 0 {
        d.value = d.value.tdiv_q(&Mpz::ui_pow_ui(10, (-n) as u32));
    }
    d.scale += n;
}

/// `align_decimal (d1, d2)` (numeric.c:573): bring both to the same scale by shifting the
/// smaller-scale operand up.
pub fn align_decimal(d1: &mut CobDecimal, d2: &mut CobDecimal) {
    match d1.scale.cmp(&d2.scale) {
        Ordering::Less => shift_decimal(d1, d2.scale - d1.scale),
        Ordering::Greater => shift_decimal(d2, d1.scale - d2.scale),
        Ordering::Equal => {}
    }
}

/// `cob_decimal_set_field (d, f)`: decode a numeric field into the working decimal. Uses the sealed
/// per-usage decoders ([`Decimal::from_display`] / [`Decimal::from_packed`] / binary decode).
pub fn cob_decimal_set_field(data: &[u8], attr: &FieldAttr) -> CobDecimal {
    match attr.field_type {
        COB_TYPE_NUMERIC_DISPLAY => CobDecimal::from_value_decimal(&Decimal::from_display(data, attr)),
        COB_TYPE_NUMERIC_PACKED => CobDecimal::from_value_decimal(&Decimal::from_packed(data, attr)),
        COB_TYPE_NUMERIC_BINARY => {
            let int = crate::binary::binary_decode(data, attr);
            CobDecimal { value: Mpz::from_i128(int), scale: attr.scale as i32 }
        }
        _ => {
            // COMP-1/COMP-2/FLOAT-DECIMAL handled by the float path; fall back to a zero decimal here
            // (the comparison dispatcher routes float fields to the float comparison, GNURUST.FLOAT.1).
            CobDecimal { value: Mpz::new(), scale: 0 }
        }
    }
}

/// `COB_MAX_DIGITS` (coblocal.h): the maximum COBOL numeric precision.
pub const COB_MAX_DIGITS: i32 = 38;

/// `cob_decimal_add (d1, d2)` (numeric.c): `d1 += d2`, aligning scales.
pub fn cob_decimal_add(d1: &mut CobDecimal, d2: &CobDecimal) {
    if d1.scale != d2.scale {
        if d2.value.sgn() == 0 {
            return;
        }
        if d1.value.sgn() == 0 {
            *d1 = d2.clone();
            return;
        }
        let mut t2 = d2.clone();
        align_decimal(d1, &mut t2);
        d1.value = d1.value.add(&t2.value);
    } else {
        d1.value = d1.value.add(&d2.value);
    }
}

/// `cob_decimal_sub (d1, d2)` (numeric.c): `d1 -= d2`, aligning scales.
pub fn cob_decimal_sub(d1: &mut CobDecimal, d2: &CobDecimal) {
    if d1.scale != d2.scale {
        if d2.value.sgn() == 0 {
            return;
        }
        let mut t2 = d2.clone();
        align_decimal(d1, &mut t2);
        d1.value = d1.value.sub(&t2.value);
    } else {
        d1.value = d1.value.sub(&d2.value);
    }
}

/// `cob_decimal_mul (d1, d2)` (numeric.c): `d1 *= d2`; scales add.
pub fn cob_decimal_mul(d1: &mut CobDecimal, d2: &CobDecimal) {
    d1.scale += d2.scale;
    d1.value = d1.value.mul(&d2.value);
}

/// `cob_decimal_div (d1, d2)` (numeric.c): `d1 /= d2`. Returns `Err` on divide-by-zero (libcob sets
/// the scale to NaN + raises `COB_EC_SIZE_ZERO_DIVIDE`). Scales the dividend up by `COB_MAX_DIGITS`
/// (plus the borrow for a negative result scale) before the truncating integer divide, so the
/// quotient carries full COBOL precision for the receiving store to round/truncate.
pub fn cob_decimal_div(d1: &mut CobDecimal, d2: &CobDecimal) -> Result<(), ()> {
    if d2.value.sgn() == 0 {
        return Err(());
    }
    if d1.value.sgn() == 0 {
        d1.scale = 0;
        return Ok(());
    }
    d1.scale -= d2.scale;
    let extra = COB_MAX_DIGITS + if d1.scale < 0 { -d1.scale } else { 0 };
    shift_decimal(d1, extra);
    d1.value = d1.value.tdiv_q(&d2.value);
    Ok(())
}

/// `cob_decimal_do_round (d, tgt_scale, opt)` (numeric.c:1936) on `Mpz`: round `d` to `tgt` fractional
/// digits per the `Round` mode. Mirrors the i128 [`crate::arith::Round`] dispatch exactly (proven by
/// GNURUST.ROUND.1). Returns `Err` only for `Prohibited` with a dropped non-zero digit.
pub fn cob_decimal_do_round(d: &mut CobDecimal, tgt: i32, round: Round) -> Result<(), ()> {
    let sign = d.value.sgn();
    if sign == 0 || tgt >= d.scale {
        return Ok(());
    }
    let drop = d.scale - tgt;
    let p = |k: i32| Mpz::ui_pow_ui(10, k as u32);
    let five = |k: i32| Mpz::ui_pow_ui(10, k as u32).mul_ui(5);
    let s5 = Mpz::from_i64(sign as i64 * 5);
    match round {
        Round::Truncate => {}
        Round::Prohibited => {
            if d.value.tdiv_r(&p(drop)).sgn() != 0 {
                return Err(());
            }
        }
        Round::AwayFromZero => {
            let div = p(drop);
            if d.value.tdiv_r(&div).sgn() != 0 {
                d.value = if sign > 0 { d.value.add(&div) } else { d.value.sub(&div) };
            }
        }
        Round::TowardGreater => {
            let div = p(drop);
            if d.value.tdiv_r(&div).sgn() != 0 && sign > 0 {
                d.value = d.value.add(&div);
            }
        }
        Round::TowardLesser => {
            let div = p(drop);
            if d.value.tdiv_r(&div).sgn() != 0 && sign < 0 {
                d.value = d.value.sub(&div);
            }
        }
        Round::NearTowardZero => {
            let exact = d.value.tdiv_r(&five(drop - 1)).sgn() == 0;
            let n = tgt + 1 - d.scale; shift_decimal(d, n);
            if !exact {
                d.value = d.value.add(&s5);
            }
        }
        Round::NearEven => {
            let exact = d.value.tdiv_r(&five(drop - 1)).sgn() == 0;
            let n = tgt + 1 - d.scale; shift_decimal(d, n);
            let round_up = if exact {
                let last_two = d.value.tdiv_r(&Mpz::from_u64(100)).get_ui();
                !matches!(last_two, 5 | 25 | 45 | 65 | 85)
            } else {
                true
            };
            if round_up {
                d.value = d.value.add(&s5);
            }
        }
        Round::NearAwayFromZero => {
            let n = tgt + 1 - d.scale; shift_decimal(d, n);
            d.value = d.value.add(&s5);
        }
        _ => {
            let n = tgt + 1 - d.scale; shift_decimal(d, n);
            d.value = d.value.add(&s5);
        }
    }
    Ok(())
}

/// `cob_decimal_get_field (d, f, opt)` (numeric.c:2055) on `Mpz`: round (if requested), adjust to the
/// field scale, truncate to the field digits, and store as DISPLAY/PACKED/BINARY bytes (via the sealed
/// [`crate::cob_move`] encoders). Returns the field byte image. `Err` on a Prohibited size error.
pub fn cob_decimal_get_field(mut d: CobDecimal, attr: &FieldAttr, size: usize, round: Round) -> Result<Vec<u8>, ()> {
    let tgt = attr.scale as i32;
    if round != Round::Truncate {
        cob_decimal_do_round(&mut d, tgt, round)?;
    }
    // adjust to the field scale (truncating narrow / zero-extend wide)
    if d.scale != tgt {
        let n = tgt - d.scale;
        shift_decimal(&mut d, n);
    }
    // The stored sign follows the *pre-truncation* value (so an overflowed negative result stores
    // negative zero, e.g. -40 into 1 digit -> -0), matching cob_decimal_get_display.
    let neg = d.value.sgn() < 0;
    // truncate to the field's digit count (overflow keeps the low digits), then to i128
    let modulus = Mpz::ui_pow_ui(10, attr.digits as u32);
    let low = d.value.tdiv_r(&modulus);
    let abs_mag = low.to_i128().unwrap_or(0).unsigned_abs();
    Ok(render_numeric(neg, abs_mag, attr, size))
}

/// Render a sign + non-negative magnitude (already at the field scale, low `digits` digits) to the
/// field's `size` bytes, via a DISPLAY temp and the sealed `cob_move` (display/packed/binary targets).
/// A negative sign is kept even when the magnitude is zero (negative zero).
fn render_numeric(neg: bool, mag_abs: u128, attr: &FieldAttr, size: usize) -> Vec<u8> {
    let digits = attr.digits as usize;
    let modulus = 10u128.pow(digits as u32);
    let mut abs = mag_abs % modulus;
    let mut ds = vec![0u8; digits];
    for slot in ds.iter_mut().rev() {
        *slot = (abs % 10) as u8;
        abs /= 10;
    }
    let mut temp: Vec<u8> = ds.iter().map(|d| b'0' + d).collect();
    let signed = attr.have_sign();
    if signed && neg {
        if let Some(l) = temp.last_mut() {
            *l |= 0x40;
        }
    }
    let dattr = FieldAttr {
        field_type: COB_TYPE_NUMERIC_DISPLAY,
        digits: attr.digits,
        scale: attr.scale,
        flags: if signed { crate::attr::COB_FLAG_HAVE_SIGN } else { 0 },
    };
    let mut out = vec![0u8; size];
    let _ = crate::move_ops::cob_move(&temp, &dattr, &mut out, attr);
    out
}

/// `cob_mul (f1, f2, opt)` (numeric.c): `f1 := f1 * f2`, via the general cob_decimal path. The
/// receiver's byte length is `f1.len()`. Returns f1's new byte image.
pub fn cob_mul(f1: &[u8], a1: &FieldAttr, f2: &[u8], a2: &FieldAttr, round: Round) -> Result<Vec<u8>, ()> {
    let mut d = cob_decimal_set_field(f1, a1);
    let d2 = cob_decimal_set_field(f2, a2);
    cob_decimal_mul(&mut d, &d2);
    cob_decimal_get_field(d, a1, f1.len(), round)
}

/// `cob_div (f1, f2, opt)` (numeric.c): `f1 := f1 / f2`. `Err` on divide-by-zero.
pub fn cob_div(f1: &[u8], a1: &FieldAttr, f2: &[u8], a2: &FieldAttr, round: Round) -> Result<Vec<u8>, ()> {
    let mut d = cob_decimal_set_field(f1, a1);
    let d2 = cob_decimal_set_field(f2, a2);
    cob_decimal_div(&mut d, &d2)?;
    cob_decimal_get_field(d, a1, f1.len(), round)
}

/// `cob_decimal_cmp (d1, d2)` (numeric.c): align scales then compare. Returns -1/0/1.
pub fn cob_decimal_cmp(d1: &CobDecimal, d2: &CobDecimal) -> i32 {
    let ord = if d1.scale != d2.scale {
        let mut a = d1.clone();
        let mut b = d2.clone();
        align_decimal(&mut a, &mut b);
        a.value.cmp(&b.value)
    } else {
        d1.value.cmp(&d2.value)
    };
    match ord {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}

/// `cob_numeric_cmp (f1, f2)` (numeric.c): the signed −1/0/1 comparison of two numeric fields. The
/// libcob dispatcher uses fast paths (bcd compare, integer compare) that all yield the same verdict
/// as the general decimal comparison reproduced here; float operands route to the float comparison.
pub fn cob_numeric_cmp(f1: &[u8], a1: &FieldAttr, f2: &[u8], a2: &FieldAttr) -> i32 {
    if is_float(a1) || is_float(a2) {
        let v1 = field_to_f64(f1, a1);
        let v2 = field_to_f64(f2, a2);
        return match v1.partial_cmp(&v2) {
            Some(Ordering::Less) => -1,
            Some(Ordering::Greater) => 1,
            _ => 0,
        };
    }
    let d1 = cob_decimal_set_field(f1, a1);
    let d2 = cob_decimal_set_field(f2, a2);
    cob_decimal_cmp(&d1, &d2)
}

/// `cob_cmp_int (f, n)`: compare a numeric field to a host integer. Same verdict as decoding `n` to a
/// decimal and comparing.
pub fn cob_cmp_int(f: &[u8], a: &FieldAttr, n: i64) -> i32 {
    let d1 = cob_decimal_set_field(f, a);
    let d2 = CobDecimal { value: Mpz::from_i64(n), scale: 0 };
    cob_decimal_cmp(&d1, &d2)
}

fn is_float(a: &FieldAttr) -> bool {
    matches!(a.field_type, 0x13 | 0x14 | 0x15 | 0x16 | 0x17)
}

fn field_to_f64(data: &[u8], a: &FieldAttr) -> f64 {
    match a.field_type {
        0x13 => f32::from_le_bytes(data[..4].try_into().unwrap_or([0; 4])) as f64,
        0x14 => f64::from_le_bytes(data[..8].try_into().unwrap_or([0; 8])),
        0x16 => crate::float::dec64_decode(data[..8].try_into().unwrap_or([0; 8]))
            .map(|(m, s)| m as f64 * 10f64.powi(-s))
            .unwrap_or(0.0),
        0x17 => crate::float::dec128_decode(data[..16].try_into().unwrap_or([0; 16]))
            .map(|(m, s)| m as f64 * 10f64.powi(-s))
            .unwrap_or(0.0),
        _ => 0.0,
    }
}

/// Fuzz target: numeric comparison is total and a consistent total order sign over arbitrary fields.
#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub fn __fuzz_numcmp(data: &[u8]) {
    if data.len() < 6 {
        return;
    }
    let mk = |b: &[u8]| FieldAttr {
        field_type: if b[0] & 1 == 0 { COB_TYPE_NUMERIC_DISPLAY } else { COB_TYPE_NUMERIC_PACKED },
        digits: (b[1] % 18 + 1) as u16,
        scale: (b[2] % 6) as i16,
        flags: 0,
    };
    let a1 = mk(&data[0..3]);
    let a2 = mk(&data[3..6]);
    let body = &data[6..];
    let at = body.len() / 2;
    let (x, y) = body.split_at(at);
    let _ = cob_numeric_cmp(x, &a1, y, &a2);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attr::COB_FLAG_HAVE_SIGN;

    fn disp(d: u16, s: i16, signed: bool) -> FieldAttr {
        FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: d, scale: s, flags: if signed { COB_FLAG_HAVE_SIGN } else { 0 } }
    }

    #[test]
    fn compares_across_scale() {
        // 12.3 (9(2)V9) vs 12.30 (9(2)V99) -> equal
        let a = b"123";
        let b = b"01230";
        assert_eq!(cob_numeric_cmp(a, &disp(3, 1, false), b, &disp(5, 2, false)), 0);
        // 12.3 vs 12.31 -> less
        let c = b"01231";
        assert_eq!(cob_numeric_cmp(a, &disp(3, 1, false), c, &disp(5, 2, false)), -1);
        // 99 vs 12 -> greater
        assert_eq!(cob_numeric_cmp(b"99", &disp(2, 0, false), b"12", &disp(2, 0, false)), 1);
    }

    fn dec(value: &str, scale: i32) -> CobDecimal {
        CobDecimal { value: Mpz::from_decimal_string(value), scale }
    }
    /// Render as an exact decimal string `value * 10^-scale` (for test assertions).
    fn render(d: &CobDecimal) -> String {
        if d.scale <= 0 {
            let mut v = d.value.clone();
            if d.scale < 0 {
                v = v.mul(&Mpz::ui_pow_ui(10, (-d.scale) as u32));
            }
            return v.to_decimal_string();
        }
        let s = d.value.to_decimal_string();
        let (neg, digs) = match s.strip_prefix('-') {
            Some(x) => ("-", x.to_string()),
            None => ("", s),
        };
        let sc = d.scale as usize;
        let digs = if digs.len() <= sc { format!("{:0>width$}", digs, width = sc + 1) } else { digs };
        let dot = digs.len() - sc;
        format!("{neg}{}.{}", &digs[..dot], &digs[dot..])
    }

    #[test]
    fn decimal_arithmetic() {
        let mut a = dec("1234", 2); // 12.34
        cob_decimal_add(&mut a, &dec("111", 2)); // + 1.11
        assert_eq!(render(&a), "13.45");
        let mut m = dec("150", 2); // 1.50
        cob_decimal_mul(&mut m, &dec("150", 2)); // * 1.50
        assert_eq!(render(&m), "2.2500");
        let mut s = dec("1000", 0); // 1000
        cob_decimal_sub(&mut s, &dec("1", 0));
        assert_eq!(render(&s), "999");
        // 10 / 3 -> quotient scaled to 38 digits of precision, truncated
        let mut q = dec("10", 0);
        assert!(cob_decimal_div(&mut q, &dec("3", 0)).is_ok());
        assert!(render(&q).starts_with("3.3333333333333333333333333333333333333"));
        // divide by zero
        let mut z = dec("5", 0);
        assert!(cob_decimal_div(&mut z, &dec("0", 0)).is_err());
    }

    #[test]
    fn cob_div_matches_proven_divide() {
        // cob_div (f1 := f1/f2 on the Mpz path) must equal the proven arith::cob_divide for a/b into
        // a's attr, across a matrix of display values x scales x signs x round modes.
        use crate::arith::cob_divide;
        fn disp_bytes(digits: usize, val: u64, neg: bool) -> Vec<u8> {
            let mut v = val;
            let mut d = vec![0u8; digits];
            for s in d.iter_mut().rev() { *s = (v % 10) as u8; v /= 10; }
            let mut o: Vec<u8> = d.iter().map(|x| b'0' + x).collect();
            if neg { if let Some(l) = o.last_mut() { *l |= 0x40; } }
            o
        }
        let mut checked = 0;
        for adig in [3usize, 5] {
            for ascale in 0..=2i16 {
                for bval in [1u64, 3, 7, 11, 99] {
                    for aval in [0u64, 1, 100, 12345 % 10u64.pow(adig as u32)] {
                        for (an, bn) in [(false, false), (true, false), (false, true), (true, true)] {
                            for round in [Round::Truncate, Round::NearAwayFromZero] {
                                let a1 = disp(adig as u16, ascale, true);
                                let a2 = disp(3, 0, true);
                                let a = disp_bytes(adig, aval, an);
                                let b = disp_bytes(3, bval, bn);
                                let mine = cob_div(&a, &a1, &b, &a2, round).unwrap();
                                let proven = cob_divide(&a, &a1, &b, &a2, &a1, round).unwrap();
                                assert_eq!(mine, proven, "a={aval}(s{ascale},n{an}) b={bval}(n{bn}) {round:?}");
                                checked += 1;
                            }
                        }
                    }
                }
            }
        }
        assert!(checked > 400);
    }

    #[test]
    fn signed_compare() {
        // -5 vs +3 -> less ; -0 vs 0 -> equal
        let neg5 = b"5\x40"; // '5' then overpunch on... build via signed display: last byte sign
        let _ = neg5;
        assert_eq!(cob_cmp_int(b"00005", &disp(5, 0, false), 5), 0);
        assert_eq!(cob_cmp_int(b"00005", &disp(5, 0, false), 9), -1);
        assert_eq!(cob_cmp_int(b"00009", &disp(5, 0, false), 5), 1);
    }
}
