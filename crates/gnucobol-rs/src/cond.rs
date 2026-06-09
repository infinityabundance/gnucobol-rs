//! LEVEL-88 condition names (`GNURUST.11`): evaluate whether a condition name is **true** given the
//! current bytes of its parent field, per GnuCOBOL `VALUE` literals/ranges, proven against `cobc`.
//!
//! **Doctrine.** GNURUST.11 admits LEVEL-88 only as a **parent-field byte predicate**: it proves
//! condition-name truth against current storage bytes, not Procedure Division control flow, `SET`
//! semantics, or business validity beyond the admitted `VALUE` clauses.
//!
//! Semantics (diagnosed from the oracle):
//! - **alphanumeric** parent: a value matches iff the parent bytes equal the literal **space-padded
//!   to the parent's length**; a range matches iff `padded(start) <= parent <= padded(end)` byte-wise
//!   (native ASCII order — collating-sequence-sensitive ranges are a non-claim);
//! - **numeric DISPLAY/COMP-3** parent: the parent's decoded **numeric value** is compared (scale-
//!   and sign-aware); a range is inclusive.
//!
//! **Sealed subset:** single/multiple literal values and a single `THRU` range, alphanumeric or
//! numeric, over an admitted DISPLAY/COMP-3 parent. The inverse — `SET condition-name TO TRUE`
//! ([`set_88_true`], `GNURUST.12`) — constructs the canonical parent bytes (first `VALUE` / range
//! lower bound). **Fails closed** (typed [`ConditionError`]/[`ConditionSetError`]) on a literal whose
//! category mismatches the parent, an unsupported parent category, and values beyond the i128
//! numeric range. `SET ... TO FALSE`, the `FALSE` clause, condition-name expressions, and Procedure
//! Division execution are **not** modelled.

use crate::attr::{FieldAttr, COB_TYPE_NUMERIC_DISPLAY, COB_TYPE_NUMERIC_PACKED};
use crate::pic::COB_TYPE_ALPHANUMERIC;
use crate::value::Decimal;
use core::cmp::Ordering;

/// A single `VALUE` literal in a LEVEL-88 clause.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CondLit {
    /// An alphanumeric literal (without quotes).
    Alpha(String),
    /// A numeric literal, e.g. `1`, `-1.5`.
    Num(String),
}

/// One `VALUE` entry: a single literal or a `start THRU end` range.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CondValue {
    Lit(CondLit),
    Range(CondLit, CondLit),
}

/// A LEVEL-88 condition name and its `VALUE` set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Condition {
    pub name: String,
    pub values: Vec<CondValue>,
}

/// Why a condition could not be evaluated (fail closed).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConditionError {
    /// The parent field's category is not an admitted DISPLAY/COMP-3 numeric or alphanumeric.
    UnsupportedParent,
    /// A `VALUE` literal's category does not match the parent (e.g. a numeric literal on an
    /// alphanumeric parent).
    LiteralCategoryMismatch,
    /// A numeric literal could not be parsed.
    BadLiteral(String),
    /// A value/parent magnitude exceeds the i128 comparison range.
    OutOfRange,
}

impl core::fmt::Display for ConditionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ConditionError::UnsupportedParent => write!(f, "unsupported parent field category"),
            ConditionError::LiteralCategoryMismatch => {
                write!(f, "VALUE literal category does not match the parent field")
            }
            ConditionError::BadLiteral(l) => write!(f, "bad numeric VALUE literal: {l}"),
            ConditionError::OutOfRange => write!(f, "value exceeds i128 comparison range"),
        }
    }
}
impl std::error::Error for ConditionError {}

/// A numeric value as `(signed magnitude, scale)`: value = `mag * 10^(-scale)`.
type Num = (i128, i32);

fn pow10(n: u32) -> Option<i128> {
    let mut r: i128 = 1;
    for _ in 0..n {
        r = r.checked_mul(10)?;
    }
    Some(r)
}

fn num_of_decimal(d: &Decimal) -> Result<Num, ConditionError> {
    let mut mag: i128 = 0;
    for &digit in &d.digits {
        mag = mag.checked_mul(10).ok_or(ConditionError::OutOfRange)?;
        mag = mag
            .checked_add(digit as i128)
            .ok_or(ConditionError::OutOfRange)?;
    }
    if d.negative {
        mag = -mag;
    }
    Ok((mag, d.scale as i32))
}

fn parse_num(lit: &str) -> Result<Num, ConditionError> {
    let t = lit.trim();
    let (neg, rest) = match t.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, t.strip_prefix('+').unwrap_or(t)),
    };
    let (int_part, frac_part) = rest.split_once('.').unwrap_or((rest, ""));
    if int_part.is_empty() && frac_part.is_empty() {
        return Err(ConditionError::BadLiteral(lit.to_string()));
    }
    let mut mag: i128 = 0;
    for c in int_part.chars().chain(frac_part.chars()) {
        if !c.is_ascii_digit() {
            return Err(ConditionError::BadLiteral(lit.to_string()));
        }
        mag = mag.checked_mul(10).ok_or(ConditionError::OutOfRange)?;
        mag = mag
            .checked_add((c as u8 - b'0') as i128)
            .ok_or(ConditionError::OutOfRange)?;
    }
    if neg {
        mag = -mag;
    }
    Ok((mag, frac_part.len() as i32))
}

/// Compare two numerics at a common scale.
fn cmp_num(a: Num, b: Num) -> Result<Ordering, ConditionError> {
    let scale = a.1.max(b.1);
    let av =
        a.0.checked_mul(pow10((scale - a.1) as u32).ok_or(ConditionError::OutOfRange)?)
            .ok_or(ConditionError::OutOfRange)?;
    let bv =
        b.0.checked_mul(pow10((scale - b.1) as u32).ok_or(ConditionError::OutOfRange)?)
            .ok_or(ConditionError::OutOfRange)?;
    Ok(av.cmp(&bv))
}

/// An alphanumeric literal space-padded (or truncated) to `len` bytes — the COBOL comparison form.
fn padded(lit: &str, len: usize) -> Vec<u8> {
    let mut v = lit.as_bytes().to_vec();
    v.resize(len, b' ');
    v.truncate(len);
    v
}

/// Decode the parent's numeric value for comparison.
fn parent_num(attr: &FieldAttr, bytes: &[u8]) -> Result<Num, ConditionError> {
    let d = match attr.field_type {
        COB_TYPE_NUMERIC_PACKED => Decimal::from_packed(bytes, attr),
        COB_TYPE_NUMERIC_DISPLAY => Decimal::from_display(bytes, attr),
        _ => return Err(ConditionError::UnsupportedParent),
    };
    num_of_decimal(&d)
}

/// Evaluate whether `cond` is **true** for a parent field of attributes `attr` holding `bytes`.
pub fn eval_88(attr: &FieldAttr, bytes: &[u8], cond: &Condition) -> Result<bool, ConditionError> {
    let is_alpha = attr.field_type == COB_TYPE_ALPHANUMERIC;
    let is_num =
        attr.field_type == COB_TYPE_NUMERIC_DISPLAY || attr.field_type == COB_TYPE_NUMERIC_PACKED;
    if !is_alpha && !is_num {
        return Err(ConditionError::UnsupportedParent);
    }

    for v in &cond.values {
        let hit = match v {
            CondValue::Lit(CondLit::Alpha(s)) if is_alpha => {
                bytes == padded(s, bytes.len()).as_slice()
            }
            CondValue::Range(CondLit::Alpha(a), CondLit::Alpha(b)) if is_alpha => {
                let (pa, pb) = (padded(a, bytes.len()), padded(b, bytes.len()));
                bytes >= pa.as_slice() && bytes <= pb.as_slice()
            }
            CondValue::Lit(CondLit::Num(s)) if is_num => {
                cmp_num(parent_num(attr, bytes)?, parse_num(s)?)? == Ordering::Equal
            }
            CondValue::Range(CondLit::Num(a), CondLit::Num(b)) if is_num => {
                let p = parent_num(attr, bytes)?;
                cmp_num(p, parse_num(a)?)? != Ordering::Less
                    && cmp_num(p, parse_num(b)?)? != Ordering::Greater
            }
            _ => return Err(ConditionError::LiteralCategoryMismatch),
        };
        if hit {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Why `SET condition-name TO TRUE` could not construct the parent bytes (fail closed).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConditionSetError {
    /// The condition has no `VALUE` to set from.
    NoValue,
    /// The chosen literal's category does not match the parent field.
    LiteralCategoryMismatch,
    /// The parent field category is not an admitted DISPLAY/COMP-3 numeric or alphanumeric.
    UnsupportedParent,
    /// The literal could not be encoded into the field (parse/fit/usage).
    Encode(String),
}

impl core::fmt::Display for ConditionSetError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ConditionSetError::NoValue => write!(f, "condition has no VALUE to SET TO TRUE"),
            ConditionSetError::LiteralCategoryMismatch => {
                write!(
                    f,
                    "chosen VALUE literal does not match the parent field category"
                )
            }
            ConditionSetError::UnsupportedParent => write!(f, "unsupported parent field category"),
            ConditionSetError::Encode(e) => write!(f, "could not encode SET value: {e}"),
        }
    }
}
impl std::error::Error for ConditionSetError {}

/// `SET condition-name TO TRUE`: construct the canonical parent bytes GnuCOBOL writes to make
/// `condition` true. The chosen value is the **first** `VALUE` entry — its literal, or for a `THRU`
/// range its **lower bound** — encoded into a parent field of `attr` and storage `size` bytes
/// (proven against `cobc`). Pure byte producer; see [`apply_set_88_true`] for the mutating form.
///
/// **Doctrine (`GNURUST.12`).** This is an oracle-proven parent-byte construction only: it does not
/// claim Procedure Division execution, `SET ... TO FALSE` / the `FALSE` clause, condition
/// expressions, or business validity beyond the selected `VALUE` clause.
pub fn set_88_true(
    attr: &FieldAttr,
    size: usize,
    cond: &Condition,
) -> Result<Vec<u8>, ConditionSetError> {
    let mut buf = vec![0u8; size];
    apply_set_88_true(attr, &mut buf, cond)?;
    Ok(buf)
}

/// `SET condition-name TO TRUE`, writing the canonical bytes into `parent` in place (COBOL-like
/// mutation). The parent's storage size is `parent.len()`.
pub fn apply_set_88_true(
    attr: &FieldAttr,
    parent: &mut [u8],
    cond: &Condition,
) -> Result<(), ConditionSetError> {
    let chosen = cond.values.first().ok_or(ConditionSetError::NoValue)?;
    let lit = match chosen {
        CondValue::Lit(l) => l,
        CondValue::Range(start, _) => start, // SET TO TRUE picks the range lower bound (proven)
    };
    let is_alpha = attr.field_type == COB_TYPE_ALPHANUMERIC;
    let is_num =
        attr.field_type == COB_TYPE_NUMERIC_DISPLAY || attr.field_type == COB_TYPE_NUMERIC_PACKED;
    if !is_alpha && !is_num {
        return Err(ConditionSetError::UnsupportedParent);
    }
    match lit {
        CondLit::Alpha(s) if is_alpha => {
            let p = padded(s, parent.len());
            parent.copy_from_slice(&p);
        }
        CondLit::Num(s) if is_num => {
            let (neg, digits, scale) = crate::init::parse_num(s)
                .map_err(|e| ConditionSetError::Encode(format!("{e:?}")))?;
            let bytes = crate::init::encode_numeric(attr, neg, &digits, scale)
                .map_err(|e| ConditionSetError::Encode(format!("{e:?}")))?;
            if bytes.len() != parent.len() {
                return Err(ConditionSetError::Encode("size mismatch".into()));
            }
            parent.copy_from_slice(&bytes);
        }
        _ => return Err(ConditionSetError::LiteralCategoryMismatch),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attr::COB_FLAG_HAVE_SIGN;

    fn alpha() -> FieldAttr {
        // Alphanumeric comparison pads to the parent bytes' length, so the attr needs no size.
        FieldAttr {
            field_type: COB_TYPE_ALPHANUMERIC,
            digits: 0,
            scale: 0,
            flags: 0,
        }
    }
    fn num(d: u16, s: i16, signed: bool) -> FieldAttr {
        FieldAttr {
            field_type: COB_TYPE_NUMERIC_DISPLAY,
            digits: d,
            scale: s,
            flags: if signed { COB_FLAG_HAVE_SIGN } else { 0 },
        }
    }
    fn lit_a(s: &str) -> CondValue {
        CondValue::Lit(CondLit::Alpha(s.into()))
    }
    fn lit_n(s: &str) -> CondValue {
        CondValue::Lit(CondLit::Num(s.into()))
    }
    fn cond(values: Vec<CondValue>) -> Condition {
        Condition {
            name: "C".into(),
            values,
        }
    }

    #[test]
    fn alpha_padded_equality_and_range() {
        let a = alpha();
        assert!(eval_88(&a, b"A  ", &cond(vec![lit_a("A")])).unwrap());
        assert!(!eval_88(&a, b"B  ", &cond(vec![lit_a("A")])).unwrap());
        assert!(eval_88(&a, b"B  ", &cond(vec![lit_a("A"), lit_a("B"), lit_a("C")])).unwrap());
        let rng = cond(vec![CondValue::Range(
            CondLit::Alpha("A".into()),
            CondLit::Alpha("C".into()),
        )]);
        assert!(eval_88(&a, b"A  ", &rng).unwrap());
        assert!(!eval_88(&a, b"D  ", &rng).unwrap());
    }

    #[test]
    fn numeric_value_and_range() {
        let n = num(1, 0, false);
        assert!(!eval_88(&n, b"2", &cond(vec![lit_n("1")])).unwrap());
        let low = cond(vec![CondValue::Range(
            CondLit::Num("1".into()),
            CondLit::Num("3".into()),
        )]);
        assert!(eval_88(&n, b"2", &low).unwrap());
        assert!(!eval_88(&n, b"4", &low).unwrap());
        // signed scaled: parent S9V9 = -1.5 (byte '1' + overpunched '5' = 0x75), VALUE -1.5.
        let sn = num(2, 1, true);
        assert!(eval_88(&sn, &[b'1', 0x75], &cond(vec![lit_n("-1.5")])).unwrap());
    }

    #[test]
    fn category_mismatch_fails_closed() {
        assert_eq!(
            eval_88(&num(1, 0, false), b"1", &cond(vec![lit_a("A")])),
            Err(ConditionError::LiteralCategoryMismatch)
        );
    }

    #[test]
    fn set_true_picks_first_and_round_trips() {
        // alnum: first of multiple values, padded; the result must satisfy eval_88.
        let a = alpha();
        let c = cond(vec![lit_a("A"), lit_a("B"), lit_a("C")]);
        let bytes = set_88_true(&a, 3, &c).unwrap();
        assert_eq!(&bytes, b"A  ");
        assert!(eval_88(&a, &bytes, &c).unwrap());

        // alnum range -> lower bound.
        let r = cond(vec![CondValue::Range(
            CondLit::Alpha("AB".into()),
            CondLit::Alpha("AM".into()),
        )]);
        let rb = set_88_true(&a, 3, &r).unwrap();
        assert_eq!(&rb, b"AB ");
        assert!(eval_88(&a, &rb, &r).unwrap());

        // numeric range -> lower bound, encoded; round-trips.
        let n = num(1, 0, false);
        let nr = cond(vec![CondValue::Range(
            CondLit::Num("1".into()),
            CondLit::Num("3".into()),
        )]);
        let nb = set_88_true(&n, 1, &nr).unwrap();
        assert_eq!(&nb, b"1");
        assert!(eval_88(&n, &nb, &nr).unwrap());

        // COMP-3 signed S9(3): VALUE 1 THRU 5 -> packed +1 = [0x00, 0x1c].
        let packed = FieldAttr {
            field_type: COB_TYPE_NUMERIC_PACKED,
            digits: 3,
            scale: 0,
            flags: COB_FLAG_HAVE_SIGN,
        };
        let pr = cond(vec![CondValue::Range(
            CondLit::Num("1".into()),
            CondLit::Num("5".into()),
        )]);
        let pb = set_88_true(&packed, 2, &pr).unwrap();
        assert_eq!(pb, vec![0x00, 0x1c]);
        assert!(eval_88(&packed, &pb, &pr).unwrap());
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    // KANIFOR: GNURUST.11, GNURUST.12
    /// eval_88 is total over symbolic parent bytes for a fixed declared 88-condition: Ok(bool) or typed error.
    #[kani::proof]
    #[kani::unwind(6)]
    fn eval_88_is_total() {
        let bytes: [u8; 3] = kani::any();
        if let Ok(f) = crate::pic::build_field("9(3)", crate::Usage::Display, false, false) {
            let cond = Condition { name: "C".into(), values: vec![CondValue::Lit(CondLit::Num("5".into()))] };
            let _ = eval_88(&f.attr, &bytes, &cond);
        }
    }
}
