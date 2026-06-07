//! PICTURE → field model (`GNURUST.3`). Parses a COBOL `PIC` clause + `USAGE` into the same
//! `(type, digits, scale, flags)` [`FieldAttr`] the decimal court uses, plus the storage `size`,
//! matching the GnuCOBOL compiler's own field-attribute computation (verified against `cobc`'s
//! generated `cob_field_attr` and runtime `LENGTH OF`).
//!
//! **Sealed subset:** `9`, `X`, `A`, `S`, `V`, fixed repeats `(n)`, the `SIGN [LEADING|TRAILING]
//! [SEPARATE]` clause, and `USAGE DISPLAY` / `COMP-3` (`PACKED-DECIMAL` / `COMPUTATIONAL-3`).
//! Everything else **fails closed** with a typed [`PicError`] — in particular the `P` scaling
//! symbol (whose leading/trailing digit/scale rules are asymmetric in GnuCOBOL) and every edited
//! symbol (`Z * $ , . + - CR DB B 0 /`) are deferred future courts, not silently mis-parsed.

use crate::attr::{
    FieldAttr, COB_FLAG_HAVE_SIGN, COB_FLAG_SIGN_LEADING, COB_FLAG_SIGN_SEPARATE,
    COB_TYPE_NUMERIC_DISPLAY, COB_TYPE_NUMERIC_PACKED,
};

/// `COB_TYPE_ALPHANUMERIC` (`common.h`): a `PIC X`/`A` field.
pub const COB_TYPE_ALPHANUMERIC: u16 = 0x21;

/// `USAGE` of a numeric field (the sealed subset).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Usage {
    /// `USAGE DISPLAY` (the default): zoned/display.
    Display,
    /// `USAGE COMP-3` / `PACKED-DECIMAL` / `COMPUTATIONAL-3`.
    Comp3,
}

/// A parsed field: the [`FieldAttr`] plus its storage `size` in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PicField {
    pub attr: FieldAttr,
    pub size: usize,
}

/// Why a `PIC` clause is outside the sealed subset (fail closed — never a silent mis-parse).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PicError {
    /// Empty picture string.
    Empty,
    /// An edited-picture or other symbol outside the sealed subset.
    UnsupportedSymbol(char),
    /// The `P` scaling symbol — deferred (`GNURUST.PIC-SCALING-P.0`): GnuCOBOL's leading vs
    /// trailing `P` digit/scale rules are asymmetric and not yet sealed.
    ScalingPDeferred,
    /// A `( n )` repeat count that is malformed or zero.
    BadRepeat,
    /// A numeric picture with no `9` digit positions.
    NoDigits,
    /// `X`/`A` mixed with `9`/`V`/`S` (a category clash the sealed subset does not admit).
    MixedCategory,
}

impl core::fmt::Display for PicError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PicError::Empty => write!(f, "empty PICTURE"),
            PicError::UnsupportedSymbol(c) => write!(
                f,
                "unsupported PICTURE symbol '{c}' (edited/usage outside sealed subset)"
            ),
            PicError::ScalingPDeferred => write!(
                f,
                "PICTURE scaling symbol 'P' is deferred (GNURUST.PIC-SCALING-P.0)"
            ),
            PicError::BadRepeat => write!(f, "malformed or zero ( n ) repeat count"),
            PicError::NoDigits => write!(f, "numeric PICTURE has no 9 digit positions"),
            PicError::MixedCategory => {
                write!(f, "mixed alphanumeric and numeric PICTURE positions")
            }
        }
    }
}

impl std::error::Error for PicError {}

/// A `( symbol, count )` term of a picture string. Streamed, never materialized — so a huge repeat
/// like `9(999999999)` costs O(1) memory instead of allocating a billion positions
/// (`GNURUST.DOS.0`: the parser is resource-bounded, not just panic-free).
struct PicTerm {
    sym: char,
    count: u64,
}

/// Total character positions a picture may declare. Far beyond any real COBOL field; a larger
/// declared size is rejected as `BadRepeat` (a resource guard, not a semantic claim).
const MAX_POSITIONS: u64 = 1_000_000;

/// Parse a picture string into its `(symbol, count)` terms (counts from `(n)` repeats), uppercased,
/// with the total declared positions bounded. Whitespace is ignored.
fn terms(pic: &str) -> Result<Vec<PicTerm>, PicError> {
    let chars: Vec<char> = pic.trim().chars().filter(|c| !c.is_whitespace()).collect();
    if chars.is_empty() {
        return Err(PicError::Empty);
    }
    let mut out = Vec::new();
    let mut total: u64 = 0;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i].to_ascii_uppercase();
        if c == '(' {
            return Err(PicError::BadRepeat); // '(' without a preceding symbol
        }
        i += 1;
        let mut count: u64 = 1;
        if i < chars.len() && chars[i] == '(' {
            let mut j = i + 1;
            let mut num = String::new();
            while j < chars.len() && chars[j] != ')' {
                if !chars[j].is_ascii_digit() {
                    return Err(PicError::BadRepeat);
                }
                num.push(chars[j]);
                if num.len() > 7 {
                    return Err(PicError::BadRepeat); // > 9_999_999, beyond MAX_POSITIONS
                }
                j += 1;
            }
            if j >= chars.len() || num.is_empty() {
                return Err(PicError::BadRepeat);
            }
            count = num.parse().map_err(|_| PicError::BadRepeat)?;
            if count == 0 {
                return Err(PicError::BadRepeat);
            }
            i = j + 1; // past ')'
        }
        total = total.saturating_add(count);
        if total > MAX_POSITIONS {
            return Err(PicError::BadRepeat);
        }
        out.push(PicTerm { sym: c, count });
    }
    Ok(out)
}

/// Build the field model for `pic` under `usage` and the `SIGN [LEADING] [SEPARATE]` clause.
///
/// `sign_separate`/`sign_leading` come from a `SIGN` clause (orthogonal to the picture's `S`).
/// For an alphanumeric (`X`/`A`) field, `usage`/sign are ignored.
pub fn build_field(
    pic: &str,
    usage: Usage,
    sign_separate: bool,
    sign_leading: bool,
) -> Result<PicField, PicError> {
    let parsed = terms(pic)?;

    let mut nines: u64 = 0;
    let mut after_v: u64 = 0;
    let mut seen_v = false;
    let mut has_sign = false;
    let mut alnum_positions: u64 = 0;
    let mut numeric = false;
    let mut alpha = false;

    for t in &parsed {
        match t.sym {
            'S' => has_sign = true,
            'V' => seen_v = true,
            '9' => {
                numeric = true;
                nines += t.count;
                if seen_v {
                    after_v += t.count;
                }
            }
            'X' | 'A' => {
                alpha = true;
                alnum_positions += t.count;
            }
            'P' => return Err(PicError::ScalingPDeferred),
            other => return Err(PicError::UnsupportedSymbol(other)),
        }
    }

    if numeric && alpha {
        return Err(PicError::MixedCategory);
    }

    if alpha {
        // Alphanumeric: count every character position (X/A). digits/scale = 0.
        return Ok(PicField {
            attr: FieldAttr {
                field_type: COB_TYPE_ALPHANUMERIC,
                digits: 0,
                scale: 0,
                flags: 0,
            },
            size: alnum_positions as usize,
        });
    }

    if !numeric {
        return Err(PicError::NoDigits);
    }
    // A numeric field's digit count fits the field model's u16; an absurd declared count is a
    // resource reject (fail closed), not a truncated mis-claim.
    if nines > u16::MAX as u64 || after_v > u16::MAX as u64 {
        return Err(PicError::BadRepeat);
    }
    let nines = nines as u16;
    let after_v = after_v as u16;

    let mut flags = 0u16;
    if has_sign {
        flags |= COB_FLAG_HAVE_SIGN;
        if sign_separate {
            flags |= COB_FLAG_SIGN_SEPARATE;
        }
        if sign_leading {
            flags |= COB_FLAG_SIGN_LEADING;
        }
    }

    let (field_type, size) = match usage {
        Usage::Display => {
            let sep = if has_sign && sign_separate { 1 } else { 0 };
            (COB_TYPE_NUMERIC_DISPLAY, nines as usize + sep)
        }
        // COMP-3 always carries a sign nibble (0x0C/0x0D, or 0x0F for unsigned): size = n/2 + 1.
        Usage::Comp3 => (COB_TYPE_NUMERIC_PACKED, nines as usize / 2 + 1),
    };

    Ok(PicField {
        attr: FieldAttr {
            field_type,
            digits: nines,
            scale: after_v as i16,
            flags,
        },
        size,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(pic: &str, u: Usage, sep: bool, lead: bool) -> PicField {
        build_field(pic, u, sep, lead).unwrap()
    }

    #[test]
    fn matches_oracle_samples() {
        // (oracle values captured from cobc -C generated cob_field_attr + size)
        let a = f("9(5)", Usage::Display, false, false);
        assert_eq!(
            (
                a.attr.field_type,
                a.attr.digits,
                a.attr.scale,
                a.attr.flags,
                a.size
            ),
            (0x10, 5, 0, 0x0000, 5)
        );
        let b = f("S9(5)V99", Usage::Display, false, false);
        assert_eq!(
            (
                b.attr.field_type,
                b.attr.digits,
                b.attr.scale,
                b.attr.flags,
                b.size
            ),
            (0x10, 7, 2, 0x0001, 7)
        );
        let c = f("S9(5)V99", Usage::Comp3, false, false);
        assert_eq!(
            (
                c.attr.field_type,
                c.attr.digits,
                c.attr.scale,
                c.attr.flags,
                c.size
            ),
            (0x12, 7, 2, 0x0001, 4)
        );
        let d = f("9(4)", Usage::Comp3, false, false);
        assert_eq!(
            (
                d.attr.field_type,
                d.attr.digits,
                d.attr.scale,
                d.attr.flags,
                d.size
            ),
            (0x12, 4, 0, 0x0000, 3)
        );
        let e = f("X(10)", Usage::Display, false, false);
        assert_eq!((e.attr.field_type, e.size), (0x21, 10));
        let g = f("S9(4)", Usage::Display, true, true);
        assert_eq!(
            (g.attr.field_type, g.attr.digits, g.attr.flags, g.size),
            (0x10, 4, 0x0007, 5)
        );
        let h = f("9(8)V9(4)", Usage::Comp3, false, false);
        assert_eq!(
            (h.attr.field_type, h.attr.digits, h.attr.scale, h.size),
            (0x12, 12, 4, 7)
        );
    }

    #[test]
    fn p_scaling_fails_closed() {
        assert_eq!(
            build_field("99P", Usage::Comp3, false, false),
            Err(PicError::ScalingPDeferred)
        );
        assert_eq!(
            build_field("PPP9(3)", Usage::Display, false, false),
            Err(PicError::ScalingPDeferred)
        );
    }

    #[test]
    fn edited_and_garbage_fail_closed() {
        assert!(matches!(
            build_field("ZZ9.99", Usage::Display, false, false),
            Err(PicError::UnsupportedSymbol(_))
        ));
        assert!(matches!(
            build_field("$,$$9", Usage::Display, false, false),
            Err(PicError::UnsupportedSymbol(_))
        ));
        assert_eq!(
            build_field("", Usage::Display, false, false),
            Err(PicError::Empty)
        );
        assert_eq!(
            build_field("9(0)", Usage::Display, false, false),
            Err(PicError::BadRepeat)
        );
        assert_eq!(
            build_field("X9", Usage::Display, false, false),
            Err(PicError::MixedCategory)
        );
    }

    #[test]
    fn huge_repeat_is_resource_rejected_not_oom() {
        // Regression (GNURUST.DOS.0): a giant repeat must reject in O(1), never allocate.
        assert_eq!(
            build_field("9(999999999)", Usage::Display, false, false),
            Err(PicError::BadRepeat)
        );
        assert_eq!(
            build_field("X(2000000)", Usage::Display, false, false),
            Err(PicError::BadRepeat)
        );
    }
}
