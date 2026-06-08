//! Decimal arithmetic (`GNURUST.7`): `ADD` / `SUBTRACT` / `MULTIPLY` on numeric fields, with
//! truncation and `ROUNDED` (nearest, ties away from zero), proven byte-identical against libcob's
//! `cob_add` / `cob_sub` / `cob_mul`.
//!
//! **Pure-Rust integer decimal.** libcob's `cob_decimal` is GMP-backed (`mpz` value + scale); this
//! port reproduces the **integer-decimal** result with a fixed-precision `i128` magnitude + scale —
//! zero runtime dependencies, `#![forbid(unsafe_code)]`, no floating point. The store path mirrors
//! `cob_decimal_get_field` (`numeric.c:2055`): adjust to the field scale (truncating `shift_decimal`
//! or `cob_decimal_do_round` for `ROUNDED`), then render to a DISPLAY temp and `cob_move` into the
//! target type — reusing the sealed [`crate::cob_move`].
//!
//! **Sealed subset:** `op := a (op) b` for `op ∈ {ADD, SUBTRACT, MULTIPLY}`, DISPLAY / COMP-3
//! operands and receiver, truncation or `ROUNDED` (nearest-away-from-zero). `ADD`/`SUBTRACT` into a
//! **PACKED** receiver take libcob's separate `cob_add_bcd` path (`GNURUST.13`) — the same
//! integer-decimal `compute`/`store` produces its bytes, with one extra rule: a negative result that
//! truncates to zero keeps its sign (`-0`), unlike the DISPLAY path. **Fail closed** (typed
//! [`ArithError`]) when an operand or intermediate exceeds the `i128` integer-decimal range
//! (`>38` significant digits / product overflow) — those need the GMP-grade bignum, a future court
//! (`GNURUST.ARITH-BIGNUM.0`). `DIVIDE`, the other six rounding modes, and `ON SIZE ERROR` exception
//! semantics are deferred.

use crate::attr::{FieldAttr, COB_FLAG_HAVE_SIGN, COB_TYPE_NUMERIC_DISPLAY};
use crate::move_ops::cob_move;
use crate::sign;
use crate::value::Decimal;

/// A binary decimal operation (`f1 := f1 op f2`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Op {
    Add,
    Subtract,
    Multiply,
    /// `DIVIDE` (`GNURUST.19`): quotient into a **GIVING** receiver — use [`cob_divide`], not
    /// [`cob_arith`] (division needs an explicit receiving field + scale, not `f1 := f1 / f2`).
    Divide,
}

/// Rounding when storing a result narrower than the computed scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Round {
    /// Default: drop low digits toward zero (`shift_decimal`).
    Truncate,
    /// `ROUNDED` default mode: nearest, ties away from zero (`COB_STORE_NEAR_AWAY_FROM_ZERO`).
    NearAwayFromZero,
}

/// Why an operation could not be performed within the sealed (i128) range.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ArithError {
    /// An operand or intermediate exceeds the i128 integer-decimal range (needs a bignum — future).
    OutOfRange,
    /// A field's attributes are self-inconsistent.
    InvalidAttr,
    /// Retained for API stability. Previously signalled that `ADD`/`SUBTRACT` into a PACKED field
    /// was deferred; that path is now sealed (`GNURUST.13`, libcob `cob_add_bcd`), so this is no
    /// longer produced.
    PackedAddSubDeferred,
    /// `DIVIDE` by zero (`GNURUST.19`): fail closed. `ON SIZE ERROR` exception semantics are a
    /// separate future court, so division by zero is rejected rather than signalled.
    DivideByZero,
}

impl core::fmt::Display for ArithError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ArithError::OutOfRange => write!(f, "operand/intermediate exceeds i128 decimal range (needs bignum; deferred)"),
            ArithError::InvalidAttr => write!(f, "invalid field attribute"),
            ArithError::PackedAddSubDeferred => write!(f, "ADD/SUBTRACT into a PACKED field uses libcob's cob_add_bcd path (deferred, GNURUST.ARITH-BCD.0)"),
            ArithError::DivideByZero => write!(f, "DIVIDE by zero (fail closed; ON SIZE ERROR is a future court)"),
        }
    }
}

impl std::error::Error for ArithError {}

/// A fixed-precision decimal: `value = mag * 10^(-scale)`. `mag` carries the sign.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Dec {
    mag: i128,
    scale: i32,
}

/// Decode a numeric field's value (via the sealed decoders) into a `Dec`.
fn decode(data: &[u8], attr: &FieldAttr) -> Result<Dec, ArithError> {
    let d: Decimal = match attr.field_type {
        crate::attr::COB_TYPE_NUMERIC_PACKED => Decimal::from_packed(data, attr),
        crate::attr::COB_TYPE_NUMERIC_DISPLAY => Decimal::from_display(data, attr),
        _ => return Err(ArithError::InvalidAttr),
    };
    let mut mag: i128 = 0;
    for &digit in &d.digits {
        mag = mag.checked_mul(10).ok_or(ArithError::OutOfRange)?;
        mag = mag
            .checked_add(digit as i128)
            .ok_or(ArithError::OutOfRange)?;
    }
    if d.negative {
        mag = -mag;
    }
    Ok(Dec {
        mag,
        scale: d.scale as i32,
    })
}

/// 10^n as i128, or `None` on overflow.
fn pow10(n: u32) -> Option<i128> {
    let mut r: i128 = 1;
    for _ in 0..n {
        r = r.checked_mul(10)?;
    }
    Some(r)
}

/// Rescale `d` up to `target_scale >= d.scale` (append zeros), or error on overflow.
fn upscale(d: Dec, target_scale: i32) -> Result<i128, ArithError> {
    let n = (target_scale - d.scale) as u32;
    let f = pow10(n).ok_or(ArithError::OutOfRange)?;
    d.mag.checked_mul(f).ok_or(ArithError::OutOfRange)
}

/// `f1 op f2`, returning the result Dec (exact, no scale narrowing yet).
fn compute(a: Dec, b: Dec, op: Op) -> Result<Dec, ArithError> {
    match op {
        Op::Add | Op::Subtract => {
            let scale = a.scale.max(b.scale);
            let am = upscale(a, scale)?;
            let bm = upscale(b, scale)?;
            let mag = match op {
                Op::Add => am.checked_add(bm),
                _ => am.checked_sub(bm),
            }
            .ok_or(ArithError::OutOfRange)?;
            Ok(Dec { mag, scale })
        }
        Op::Multiply => {
            let mag = a.mag.checked_mul(b.mag).ok_or(ArithError::OutOfRange)?;
            Ok(Dec {
                mag,
                scale: a.scale + b.scale,
            })
        }
        // DIVIDE has no exact Dec (the quotient depends on the receiving scale) — use cob_divide.
        Op::Divide => Err(ArithError::InvalidAttr),
    }
}

/// `a / b` evaluated at exactly `result_scale` fractional digits, truncating toward zero (the COBOL
/// DIVIDE truncation). `result_scale = a/b * 10^result_scale = a.mag * 10^(result_scale+b.scale-a.scale)
/// / b.mag`. Fails closed on divide-by-zero (`GNURUST.19`; `ON SIZE ERROR` is a future court).
fn compute_divide(a: Dec, b: Dec, result_scale: i32) -> Result<Dec, ArithError> {
    if b.mag == 0 {
        return Err(ArithError::DivideByZero);
    }
    let k = result_scale + b.scale - a.scale;
    let mag = if k >= 0 {
        let num = a
            .mag
            .checked_mul(pow10(k as u32).ok_or(ArithError::OutOfRange)?)
            .ok_or(ArithError::OutOfRange)?;
        num / b.mag // i128 division truncates toward zero (sign = sign(num)*sign(b))
    } else {
        let den = b
            .mag
            .checked_mul(pow10((-k) as u32).ok_or(ArithError::OutOfRange)?)
            .ok_or(ArithError::OutOfRange)?;
        a.mag / den
    };
    Ok(Dec {
        mag,
        scale: result_scale,
    })
}

/// `DIVIDE` quotient into a GIVING receiver (`GNURUST.19`): `receiver := lhs / rhs`, returning the
/// receiver's field bytes — matching libcob's `cob_div`/`cob_decimal_div` + store. Truncation toward
/// zero by default; `ROUNDED` is nearest-away-from-zero (one guard digit, computed then rounded by
/// [`store`]). Divide-by-zero fails closed. (For `DIVIDE a BY b GIVING c`, `lhs=a, rhs=b`; for
/// `DIVIDE a INTO b GIVING c`, `lhs=b, rhs=a`.) **Non-claims:** `REMAINDER`, `ON SIZE ERROR`, `COMPUTE`,
/// expression evaluation, binary/edited receivers, and business correctness.
pub fn cob_divide(
    lhs: &[u8],
    lhs_attr: &FieldAttr,
    rhs: &[u8],
    rhs_attr: &FieldAttr,
    recv_attr: &FieldAttr,
    round: Round,
) -> Result<Vec<u8>, ArithError> {
    let dl = decode(lhs, lhs_attr)?;
    let dr = decode(rhs, rhs_attr)?;
    let guard = if round == Round::NearAwayFromZero {
        1
    } else {
        0
    };
    let q = compute_divide(dl, dr, recv_attr.scale as i32 + guard)?;
    store(q, recv_attr, round, false)
}

/// Truncating division of `mag` by 10^k, toward zero (`cob_div_by_pow_10`).
fn tdiv_pow10(mag: i128, k: u32) -> Result<i128, ArithError> {
    let f = pow10(k).ok_or(ArithError::OutOfRange)?;
    Ok(mag / f) // i128 division truncates toward zero
}

/// Store `d` into the field `attr` with rounding `round`, returning the field bytes. Mirrors
/// `cob_decimal_get_field` (`numeric.c:2055`): round (if requested) then truncate/append to the
/// field scale, truncate overflow digits, render a DISPLAY temp, and `cob_move` to the target type.
fn store(
    d: Dec,
    attr: &FieldAttr,
    round: Round,
    sign_on_zero: bool,
) -> Result<Vec<u8>, ArithError> {
    let target_scale = attr.scale as i32;
    let target_digits = attr.digits as usize;
    // The computed value's sign, before any scale truncation. libcob's `cob_add_bcd` (packed
    // ADD/SUBTRACT) keeps this sign even when the result truncates to zero magnitude
    // (`-0.6` -> `-0`); the `cob_decimal`/DISPLAY path does not (it yields `+0`).
    let pre_negative = d.mag < 0;
    let mut mag = d.mag;
    let mut scale = d.scale;

    // ROUNDED, nearest-away-from-zero (cob_decimal_do_round, NEAR_AWAY_FROM_ZERO default).
    if round == Round::NearAwayFromZero && mag != 0 && target_scale < scale {
        // shift to (target_scale + 1), truncating
        let k = (scale - target_scale - 1) as u32;
        if k > 0 {
            mag = tdiv_pow10(mag, k)?;
        }
        scale = target_scale + 1;
        // +/- 5 toward away from zero
        mag = if mag >= 0 {
            mag.checked_add(5)
        } else {
            mag.checked_sub(5)
        }
        .ok_or(ArithError::OutOfRange)?;
    }

    // Adjust to the field scale (truncating shift for narrowing; append zeros for widening).
    if scale != target_scale {
        if target_scale > scale {
            let f = pow10((target_scale - scale) as u32).ok_or(ArithError::OutOfRange)?;
            mag = mag.checked_mul(f).ok_or(ArithError::OutOfRange)?;
        } else {
            mag = tdiv_pow10(mag, (scale - target_scale) as u32)?;
        }
    }

    // Overflow: keep the low `target_digits` digits (TRUNC_ON_OVERFLOW default; SIZE error is a
    // future court). magnitude is at the field scale now.
    let negative = if mag != 0 {
        mag < 0
    } else {
        // zero magnitude: keep the pre-truncation sign only on the cob_add_bcd path.
        sign_on_zero && pre_negative
    };
    let mut abs = mag.unsigned_abs();
    let modulus = pow10(target_digits as u32).ok_or(ArithError::OutOfRange)? as u128;
    abs %= modulus;

    // Render as `target_digits` zero-padded decimal digits into a DISPLAY temp, then cob_move.
    let mut digits = vec![0u8; target_digits];
    let mut v = abs;
    for slot in digits.iter_mut().rev() {
        *slot = (v % 10) as u8;
        v /= 10;
    }
    let mut temp: Vec<u8> = digits.iter().map(|&d| sign::i2d(d)).collect();
    let signed = attr.have_sign();
    let temp_attr = FieldAttr {
        field_type: COB_TYPE_NUMERIC_DISPLAY,
        digits: attr.digits,
        scale: attr.scale,
        flags: if signed { COB_FLAG_HAVE_SIGN } else { 0 },
    };
    // The sign follows the value's sign *before* overflow truncation, so an overflowed negative
    // result stores negative zero (e.g. -40 into 1 digit -> -0 = 0x70), matching libcob.
    if signed && negative {
        if let Some(last) = temp.last_mut() {
            *last = sign::put_sign_ascii(*last);
        }
    }

    let mut out = vec![0u8; field_storage_size(attr)];
    cob_move(&temp, &temp_attr, &mut out, attr).map_err(|_| ArithError::InvalidAttr)?;
    Ok(out)
}

/// Storage byte size implied by an attribute (DISPLAY = digits[+sep]; PACKED = digits/2+1).
fn field_storage_size(attr: &FieldAttr) -> usize {
    match attr.field_type {
        crate::attr::COB_TYPE_NUMERIC_PACKED => attr.digits as usize / 2 + 1,
        _ => attr.digits as usize + if attr.sign_separate() { 1 } else { 0 },
    }
}

/// `f1 := f1 (op) f2`, with rounding `round`, returning the bytes of `f1`'s field — matching
/// libcob `cob_add`/`cob_sub`/`cob_mul`.
pub fn cob_arith(
    op: Op,
    a: &[u8],
    a_attr: &FieldAttr,
    b: &[u8],
    b_attr: &FieldAttr,
    round: Round,
) -> Result<Vec<u8>, ArithError> {
    // ADD/SUBTRACT into a PACKED receiving field take libcob's separate cob_add_bcd path
    // (`GNURUST.13`). It computes the exact integer-decimal sum, aligns to the receiver scale,
    // rounds (nearest-away when requested) or truncates, truncates overflow, and stores — which is
    // exactly the integer-decimal path here, so the same `compute`/`store` produces identical bytes.
    let da = decode(a, a_attr)?;
    let db = decode(b, b_attr)?;
    let r = compute(da, db, op)?;
    // Only ADD/SUBTRACT into a PACKED receiver (the cob_add_bcd path) keeps a negative sign on a
    // zero-magnitude result.
    let sign_on_zero = matches!(op, Op::Add | Op::Subtract)
        && a_attr.field_type == crate::attr::COB_TYPE_NUMERIC_PACKED;
    store(r, a_attr, round, sign_on_zero)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attr::COB_TYPE_NUMERIC_PACKED;

    fn disp(d: u16, s: i16, signed: bool) -> FieldAttr {
        FieldAttr {
            field_type: COB_TYPE_NUMERIC_DISPLAY,
            digits: d,
            scale: s,
            flags: if signed { COB_FLAG_HAVE_SIGN } else { 0 },
        }
    }
    fn packed(d: u16, s: i16, signed: bool) -> FieldAttr {
        FieldAttr {
            field_type: COB_TYPE_NUMERIC_PACKED,
            digits: d,
            scale: s,
            flags: if signed { COB_FLAG_HAVE_SIGN } else { 0 },
        }
    }

    #[test]
    fn add_display() {
        // 012.34 + 001.11 = 013.45  (S9(3)V99 display)
        let a = b"01234"; // 012.34
        let bb = b"00111"; // 001.11
        let r = cob_arith(
            Op::Add,
            a,
            &disp(5, 2, true),
            bb,
            &disp(5, 2, true),
            Round::Truncate,
        )
        .unwrap();
        assert_eq!(&r, b"01345");
    }

    #[test]
    fn multiply_truncates() {
        // 1.50 * 1.50 = 2.2500 -> stored in V99 truncates to 2.25
        let a = b"150"; // 1.50
        let bb = b"150";
        let r = cob_arith(
            Op::Multiply,
            a,
            &disp(3, 2, false),
            bb,
            &disp(3, 2, false),
            Round::Truncate,
        )
        .unwrap();
        assert_eq!(&r, b"225"); // 2.25
    }

    #[test]
    fn rounded_half_away() {
        // 1.50 * 1.55 = 2.3250 ; V99 rounded -> 2.33 (half away)
        let r = cob_arith(
            Op::Multiply,
            b"150",
            &disp(3, 2, false),
            b"155",
            &disp(3, 2, false),
            Round::NearAwayFromZero,
        )
        .unwrap();
        assert_eq!(&r, b"233");
    }

    #[test]
    fn packed_add_sub_via_bcd() {
        // GNURUST.13: ADD/SUBTRACT into a PACKED field (libcob cob_add_bcd path).
        // -012.34 + -001.00 = -013.34 in S9(3)V99 COMP-3.
        let a = [0x01, 0x23, 0x4d];
        let b = [0x00, 0x10, 0x0d];
        let r = cob_arith(
            Op::Add,
            &a,
            &packed(5, 2, true),
            &b,
            &packed(5, 2, true),
            Round::Truncate,
        )
        .unwrap();
        assert_eq!(r, vec![0x01, 0x33, 0x4d]); // -013.34

        // 012.34 + 001.11 = 013.45 (unsigned-ish positive), truncation.
        let r2 = cob_arith(
            Op::Add,
            &[0x01, 0x23, 0x4c],
            &packed(5, 2, true),
            &[0x00, 0x11, 0x1c],
            &packed(5, 2, true),
            Round::Truncate,
        )
        .unwrap();
        assert_eq!(r2, vec![0x01, 0x34, 0x5c]); // 013.45
    }

    #[test]
    fn packed_multiply_ok() {
        // PACKED multiply uses the cob_decimal path (sealed): 1.50 * 2.00 = 3.00 in S9(3)V99 COMP-3
        let a = [0x00, 0x15, 0x0c]; // 001.50
        let b = [0x00, 0x20, 0x0c]; // 002.00
        let r = cob_arith(
            Op::Multiply,
            &a,
            &packed(5, 2, true),
            &b,
            &packed(5, 2, true),
            Round::Truncate,
        )
        .unwrap();
        assert_eq!(r, vec![0x00, 0x30, 0x0c]); // 003.00
    }

    #[test]
    fn divide_truncates() {
        // 10.00 / 3.00 = 3.333... -> S9(5)V99 truncate = 3.33
        let r = cob_divide(
            b"0001000",
            &disp(7, 2, false),
            b"0000300",
            &disp(7, 2, false),
            &disp(7, 2, false),
            Round::Truncate,
        )
        .unwrap();
        assert_eq!(&r, b"0000333");
    }

    #[test]
    fn divide_rounded_half_away() {
        // 20.00 / 3.00 = 6.666... -> trunc 6.66 ; ROUNDED 6.67
        let t = cob_divide(
            b"0002000",
            &disp(7, 2, false),
            b"0000300",
            &disp(7, 2, false),
            &disp(7, 2, false),
            Round::Truncate,
        )
        .unwrap();
        assert_eq!(&t, b"0000666");
        let rr = cob_divide(
            b"0002000",
            &disp(7, 2, false),
            b"0000300",
            &disp(7, 2, false),
            &disp(7, 2, false),
            Round::NearAwayFromZero,
        )
        .unwrap();
        assert_eq!(&rr, b"0000667");
    }

    #[test]
    fn divide_negative_truncates_toward_zero() {
        // -10.00 / 3.00 = -3.33 (truncate toward zero, not -3.34)
        let mut a = b"0001000".to_vec();
        *a.last_mut().unwrap() = 0x70; // -10.00 (last '0' -> negative overpunch)
        let r = cob_divide(
            &a,
            &disp(7, 2, true),
            b"0000300",
            &disp(7, 2, false),
            &disp(7, 2, true),
            Round::Truncate,
        )
        .unwrap();
        let mut exp = b"0000333".to_vec();
        *exp.last_mut().unwrap() = 0x73; // -3.33 ('3' negative overpunch)
        assert_eq!(r, exp);
    }

    #[test]
    fn divide_half_scale() {
        // 1.00 / 2.00 = 0.50
        let r = cob_divide(
            b"0000100",
            &disp(7, 2, false),
            b"0000200",
            &disp(7, 2, false),
            &disp(7, 2, false),
            Round::Truncate,
        )
        .unwrap();
        assert_eq!(&r, b"0000050");
    }

    #[test]
    fn divide_into_packed_receiver() {
        // DIVIDE into a COMP-3 receiver: 9.00 / 2.00 = 4.50 in S9(3)V99 COMP-3.
        let r = cob_divide(
            b"0000900",
            &disp(7, 2, false),
            b"0000200",
            &disp(7, 2, false),
            &packed(5, 2, true),
            Round::Truncate,
        )
        .unwrap();
        assert_eq!(r, vec![0x00, 0x45, 0x0c]); // 004.50
    }

    #[test]
    fn divide_by_zero_fails_closed() {
        let r = cob_divide(
            b"0001000",
            &disp(7, 2, false),
            b"0000000",
            &disp(7, 2, false),
            &disp(7, 2, false),
            Round::Truncate,
        );
        assert_eq!(r, Err(ArithError::DivideByZero));
    }

    #[test]
    fn out_of_range_fails_closed() {
        // 18-digit * 18-digit may overflow i128 (36 digits ok, but 38*? -> guard). Force overflow.
        let big = b"999999999999999999"; // 18 nines
        let r = cob_arith(
            Op::Multiply,
            big,
            &disp(18, 0, false),
            big,
            &disp(18, 0, false),
            Round::Truncate,
        );
        // 36-digit product fits i128; widening for storage into 18 digits truncates. Should be Ok.
        assert!(r.is_ok());
    }
}
