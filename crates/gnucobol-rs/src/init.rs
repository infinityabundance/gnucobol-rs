//! Initial record image from `VALUE` clauses (`GNURUST.8`): compute the WORKING-STORAGE bytes a
//! GnuCOBOL `01` record holds at program start, proven against `cobc`-initialized storage.
//!
//! **Diagnosed from the oracle (generated C + runtime), not guessed.** For a record that carries any
//! `VALUE`, `cobc` initializes each elementary field by type:
//! - **alphanumeric** `VALUE "lit"` → the literal left-justified, **space**-padded; unvalued → spaces;
//! - **numeric DISPLAY** `VALUE n` → zoned (digits at the field scale, trailing **overpunch** sign,
//!   e.g. `-7`→`"00w"`); unvalued or `ZERO` → `'0'` fill;
//! - **COMP-3** `VALUE n` → packed (the sealed [`crate::cob_move`] encode of the literal); an
//!   **unvalued COMP-3 is a canonical packed zero** — digit nibbles 0 with the proper sign nibble
//!   (`0x0C` signed / `0x0F` unsigned), not raw `0x00`.
//!
//! **Sealed subset:** a flat `01` record of elementary `9 X A S V` / `COMP-3` items, each optionally
//! `VALUE <numeric-literal | "alnum-literal" | ZERO | SPACE>`. **Fails closed** (typed
//! [`InitError`]) on edited/`P`/unsupported PICs, `OCCURS`/`REDEFINES` (deferred for VALUE),
//! and literals that do not fit the field.

use crate::attr::{FieldAttr, COB_FLAG_HAVE_SIGN, COB_TYPE_NUMERIC_DISPLAY};
use crate::layout::{lay_out, Item, LayoutError};
use crate::move_ops::cob_move;
use crate::pic::{build_field, Usage, COB_TYPE_ALPHANUMERIC};
use crate::sign;

/// A `VALUE` clause.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Val {
    /// A numeric literal, e.g. `-12.34` or `42`.
    Num(String),
    /// An alphanumeric literal (without quotes).
    Alpha(String),
    /// The figurative constant `ZERO`/`ZEROS`/`ZEROES`.
    Zero,
    /// The figurative constant `SPACE`/`SPACES`.
    Space,
}

/// One data item, with an optional `VALUE`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueItem {
    pub level: u16,
    pub name: String,
    /// `(picture, usage, sign_separate, sign_leading)`, or `None` for a group item.
    pub pic: Option<(String, Usage, bool, bool)>,
    pub value: Option<Val>,
}

/// Why the initial image could not be built.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum InitError {
    Layout(String),
    Pic(String),
    /// A numeric `VALUE` literal could not be parsed.
    BadLiteral(String),
    /// A `VALUE` literal does not fit its PIC (a `cobc` compile error).
    DoesNotFit(String),
}

impl core::fmt::Display for InitError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            InitError::Layout(e) => write!(f, "layout: {e}"),
            InitError::Pic(e) => write!(f, "pic: {e}"),
            InitError::BadLiteral(l) => write!(f, "bad numeric VALUE literal: {l}"),
            InitError::DoesNotFit(n) => write!(f, "VALUE does not fit field {n}"),
        }
    }
}
impl std::error::Error for InitError {}

impl From<LayoutError> for InitError {
    fn from(e: LayoutError) -> Self {
        InitError::Layout(format!("{e:?}"))
    }
}

/// Encode a numeric literal `(neg, digits, lit_scale)` into a field's bytes per its `attr`
/// (DISPLAY zoned, or COMP-3 via the sealed [`cob_move`]). Shared by `value_image` and the
/// LEVEL-88 `SET ... TO TRUE` constructor (`GNURUST.12`).
pub(crate) fn encode_numeric(
    attr: &FieldAttr,
    neg: bool,
    digits: &[u8],
    scale: i32,
) -> Result<Vec<u8>, InitError> {
    let signed = attr.have_sign();
    let z = zoned(
        neg,
        digits,
        scale,
        attr.digits as usize,
        attr.scale as i32,
        signed,
        "field",
    )?;
    if attr.field_type == COB_TYPE_NUMERIC_DISPLAY {
        Ok(z)
    } else {
        // COMP-3 (and any other numeric target): render via the sealed cob_move from the zoned temp.
        let zattr = FieldAttr {
            field_type: COB_TYPE_NUMERIC_DISPLAY,
            digits: attr.digits,
            scale: attr.scale,
            flags: if signed { COB_FLAG_HAVE_SIGN } else { 0 },
        };
        let mut out = vec![0u8; attr.digits as usize / 2 + 1];
        cob_move(&z, &zattr, &mut out, attr).map_err(|e| InitError::Pic(format!("{e:?}")))?;
        Ok(out)
    }
}

/// Parse a numeric literal into `(negative, digits, scale)` where the value is
/// `±(digits as integer) * 10^(-scale)`.
pub(crate) fn parse_num(lit: &str) -> Result<(bool, Vec<u8>, i32), InitError> {
    let t = lit.trim();
    let (neg, rest) = match t.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, t.strip_prefix('+').unwrap_or(t)),
    };
    let (int_part, frac_part) = match rest.split_once('.') {
        Some((i, f)) => (i, f),
        None => (rest, ""),
    };
    if int_part.is_empty() && frac_part.is_empty() {
        return Err(InitError::BadLiteral(lit.to_string()));
    }
    let mut digits = Vec::new();
    for c in int_part.chars().chain(frac_part.chars()) {
        if !c.is_ascii_digit() {
            return Err(InitError::BadLiteral(lit.to_string()));
        }
        digits.push(c as u8 - b'0');
    }
    Ok((neg, digits, frac_part.len() as i32))
}

/// Build the zoned DISPLAY bytes (`target_digits` long) for a value aligned to `target_scale`.
fn zoned(
    neg: bool,
    digits: &[u8],
    scale: i32,
    target_digits: usize,
    target_scale: i32,
    signed: bool,
    name: &str,
) -> Result<Vec<u8>, InitError> {
    // Align the fraction to the field scale (append zeros / drop low digits).
    let mut d: Vec<u8> = digits.to_vec();
    match target_scale.cmp(&scale) {
        std::cmp::Ordering::Greater => {
            d.resize(d.len() + (target_scale - scale) as usize, 0);
        }
        std::cmp::Ordering::Less => {
            let drop = (scale - target_scale) as usize;
            if drop > d.len() {
                d.clear();
            } else {
                d.truncate(d.len() - drop);
            }
        }
        std::cmp::Ordering::Equal => {}
    }
    // Now `d` is the integer magnitude at the field scale; left-pad / overflow-check to width.
    if d.len() > target_digits {
        // allow leading zeros to be trimmed before declaring overflow
        let extra = d.len() - target_digits;
        if d[..extra].iter().any(|&x| x != 0) {
            return Err(InitError::DoesNotFit(name.to_string()));
        }
        d.drain(0..extra);
    }
    while d.len() < target_digits {
        d.insert(0, 0);
    }
    let mut out: Vec<u8> = d.iter().map(|&x| sign::i2d(x)).collect();
    if signed && neg {
        if let Some(last) = out.last_mut() {
            *last = sign::put_sign_ascii(*last);
        }
    }
    Ok(out)
}

/// Compute the initial bytes of the `01` record described by `items`.
pub fn value_image(items: &[ValueItem]) -> Result<Vec<u8>, InitError> {
    let lay_items: Vec<Item> = items
        .iter()
        .map(|it| Item {
            level: it.level,
            name: it.name.clone(),
            pic: it.pic.clone(),
            occurs: None,
            redefines: None,
            odo: None,
        })
        .collect();
    let laid = lay_out(&lay_items)?;
    let total = laid.iter().map(|l| l.offset + l.size).max().unwrap_or(0);
    let mut buf = vec![0u8; total]; // static zero default

    for it in items {
        let Some((pic, usage, sep, lead)) = &it.pic else {
            continue; // group: no bytes of its own
        };
        let l = laid
            .iter()
            .find(|x| x.name == it.name)
            .ok_or_else(|| InitError::Layout(format!("missing laid field {}", it.name)))?;
        let pf =
            build_field(pic, *usage, *sep, *lead).map_err(|e| InitError::Pic(format!("{e:?}")))?;
        let attr = pf.attr;
        // A P-scaled numeric field (scale < 0 or scale > digits) has attr.digits != stored size; its
        // VALUE image is a separate court (`GNURUST.VALUE-P.0`). Fail closed rather than mis-place
        // a digits-wide rendering into the smaller stored field.
        if matches!(
            attr.field_type,
            COB_TYPE_NUMERIC_DISPLAY | crate::attr::COB_TYPE_NUMERIC_PACKED
        ) && (attr.scale < 0 || attr.scale as i32 > attr.digits as i32)
        {
            return Err(InitError::Pic(format!(
                "P-scaled VALUE deferred: {}",
                it.name
            )));
        }
        let field = &mut buf[l.offset..l.offset + l.size];

        match attr.field_type {
            COB_TYPE_ALPHANUMERIC => {
                // default + VALUE: spaces, left-justified literal.
                field.fill(b' ');
                match &it.value {
                    Some(Val::Alpha(s)) => {
                        for (slot, b) in field.iter_mut().zip(s.bytes()) {
                            *slot = b;
                        }
                    }
                    Some(Val::Zero) => field.fill(b'0'),
                    Some(Val::Space) | None => {}
                    Some(Val::Num(_)) => return Err(InitError::DoesNotFit(it.name.clone())),
                }
            }
            COB_TYPE_NUMERIC_DISPLAY | crate::attr::COB_TYPE_NUMERIC_PACKED => {
                // Unvalued / ZERO -> 0; this also yields the observed defaults (DISPLAY '0'-fill,
                // COMP-3 canonical packed zero) since encoding 0 produces exactly those bytes.
                let (neg, digits, scale) = match &it.value {
                    Some(Val::Num(lit)) => parse_num(lit)?,
                    Some(Val::Zero) | None => (false, vec![0u8], 0),
                    Some(Val::Space) | Some(Val::Alpha(_)) => {
                        return Err(InitError::DoesNotFit(it.name.clone()))
                    }
                };
                let bytes = encode_numeric(&attr, neg, &digits, scale)?;
                field.copy_from_slice(&bytes);
            }
            _ => {
                return Err(InitError::Pic(format!(
                    "unsupported field type for VALUE: {}",
                    it.name
                )))
            }
        }
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn it(
        level: u16,
        name: &str,
        pic: Option<(&str, Usage, bool, bool)>,
        value: Option<Val>,
    ) -> ValueItem {
        ValueItem {
            level,
            name: name.to_string(),
            pic: pic.map(|(p, u, s, l)| (p.to_string(), u, s, l)),
            value,
        }
    }

    #[test]
    fn matches_oracle_example() {
        // The probed val.cob record (without the oversized REDEFINES).
        let items = vec![
            it(1, "REC", None, None),
            it(
                5,
                "A",
                Some(("9(3)", Usage::Display, false, false)),
                Some(Val::Num("42".into())),
            ),
            it(
                5,
                "B",
                Some(("X(4)", Usage::Display, false, false)),
                Some(Val::Alpha("HI".into())),
            ),
            it(
                5,
                "C",
                Some(("S9(3)V99", Usage::Comp3, false, false)),
                Some(Val::Num("-12.34".into())),
            ),
            it(5, "D", Some(("9(2)", Usage::Display, false, false)), None),
            it(5, "E", Some(("X(3)", Usage::Display, false, false)), None),
        ];
        let img = value_image(&items).unwrap();
        // "042" + "HI  " + 01234d + "00" + "   "
        let mut want = Vec::new();
        want.extend_from_slice(b"042");
        want.extend_from_slice(b"HI  ");
        want.extend_from_slice(&[0x01, 0x23, 0x4d]);
        want.extend_from_slice(b"00");
        want.extend_from_slice(b"   ");
        assert_eq!(img, want);
    }

    #[test]
    fn signed_display_overpunch_and_unvalued_packed() {
        let items = vec![
            it(1, "R", None, None),
            it(
                5,
                "F",
                Some(("S9(3)", Usage::Display, false, false)),
                Some(Val::Num("-7".into())),
            ),
            it(5, "G", Some(("S9(3)V99", Usage::Comp3, false, false)), None), // unvalued COMP-3 -> zeros
        ];
        let img = value_image(&items).unwrap();
        let mut want = vec![b'0', b'0', sign::put_sign_ascii(b'7')]; // "00w"
        want.extend_from_slice(&[0x00, 0x00, 0x0c]); // unvalued signed COMP-3 = canonical packed zero
        assert_eq!(img, want);
    }
}
