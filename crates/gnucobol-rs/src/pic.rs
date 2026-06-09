//! PICTURE → field model (`GNURUST.3`). Parses a COBOL `PIC` clause + `USAGE` into the same
//! `(type, digits, scale, flags)` [`FieldAttr`] the decimal court uses, plus the storage `size`,
//! matching the GnuCOBOL compiler's own field-attribute computation (verified against `cobc`'s
//! generated `cob_field_attr` and runtime `LENGTH OF`).
//!
//! **Sealed subset:** `9`, `X`, `A`, `S`, `V`, the `P` scaling symbol (`GNURUST.9`: a single
//! contiguous run at one end, with the asymmetric leading/trailing digit/scale rule), fixed repeats
//! `(n)`, the `SIGN [LEADING|TRAILING] [SEPARATE]` clause,
//! (`PACKED-DECIMAL` / `COMPUTATIONAL-3`). Everything else **fails closed** with a typed
//! [`PicError`] — every edited symbol (`Z * $ , . + - CR DB B 0 /`), `V`+`P`, and `P` at both ends
//! are deferred future courts, not silently mis-parsed.

use crate::attr::{
    FieldAttr, COB_FLAG_BINARY_SWAP, COB_FLAG_BINARY_TRUNC, COB_FLAG_HAVE_SIGN,
    COB_FLAG_NO_SIGN_NIBBLE, COB_FLAG_REAL_BINARY, COB_FLAG_SIGN_LEADING, COB_FLAG_SIGN_SEPARATE,
    COB_TYPE_NUMERIC_BINARY, COB_TYPE_NUMERIC_DISPLAY, COB_TYPE_NUMERIC_PACKED,
};

/// `COB_TYPE_ALPHANUMERIC` (`common.h`): a `PIC X`/`A` field.
pub const COB_TYPE_ALPHANUMERIC: u16 = 0x21;

/// `USAGE` of a numeric field (the sealed subset). `#[non_exhaustive]`: new usages are added over time,
/// so downstream `match`es must include a wildcard arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Usage {
    /// `USAGE DISPLAY` (the default): zoned/display.
    Display,
    /// `USAGE COMP-3` / `PACKED-DECIMAL` / `COMPUTATIONAL-3`.
    Comp3,
    /// `USAGE COMP` / `BINARY` / `COMPUTATIONAL` (`GNURUST.14`): big-endian binary, truncated to the
    /// PIC digit range (`binary-truncate: yes`).
    Comp,
    /// `USAGE COMP-5` (`GNURUST.14`): native byte order, full binary range.
    Comp5,
    /// `USAGE COMP-X` (`GNURUST.14`): big-endian binary, full binary range (masked to the byte width).
    CompX,
    /// `USAGE COMP-6` (`GNURUST.18`): **unsigned** packed-decimal — two digits per byte, no sign nibble,
    /// size `ceil(digits/2)`. (GnuCOBOL warns and converts a *signed* `S9(n)` COMP-6 to COMP-3, so
    /// signed COMP-6 is **not** this court.)
    Comp6,
}

/// Storage bytes for `COMP`/`BINARY`/`COMP-5`, under the oracle's `binary-size: 1-2-4-8` config
/// (sizes are powers of two; same table for both).
pub(crate) fn binary_size(digits: u16) -> usize {
    match digits {
        0 => 0,
        1..=2 => 1,
        3..=4 => 2,
        5..=9 => 4,
        10..=18 => 8,
        _ => 16, // 19..=38
    }
}

/// Storage bytes for `COMP-X` — the **tight** byte count: the smallest `k` with `256^k >= 10^digits`
/// (e.g. `9(6)` → 3, `9(10)` → 5), proven against `cobc`. Distinct from the `1-2-4-8` table.
pub(crate) fn comp_x_size(digits: u16) -> usize {
    if digits == 0 {
        return 0;
    }
    let mut ten_pow: u128 = 1;
    for _ in 0..digits {
        ten_pow = ten_pow.saturating_mul(10);
    }
    let mut k = 0usize;
    let mut cap: u128 = 1;
    while cap < ten_pow {
        cap = cap.saturating_mul(256);
        k += 1;
    }
    k
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
    /// A `P` scaling form outside the sealed `GNURUST.9` subset: `V` combined with `P`, or `P` at
    /// both ends of the `9`s. (A single contiguous `P` run at one end is supported.)
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
                "unsupported 'P' scaling form (V+P, or P at both ends) outside the sealed subset"
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
    // P scaling positions (not stored): leading P (before the 9s) vs trailing P (after).
    let mut p_count: u64 = 0;
    let mut p_leading = false;
    let mut p_trailing = false;
    let mut nine_seen = false;

    for t in &parsed {
        match t.sym {
            'S' => has_sign = true,
            'V' => seen_v = true,
            '9' => {
                numeric = true;
                nine_seen = true;
                nines += t.count;
                if seen_v {
                    after_v += t.count;
                }
            }
            'X' | 'A' => {
                alpha = true;
                alnum_positions += t.count;
            }
            'P' => {
                numeric = true;
                p_count += t.count;
                if nine_seen {
                    p_trailing = true;
                } else {
                    p_leading = true;
                }
            }
            other => return Err(PicError::UnsupportedSymbol(other)),
        }
    }

    // P is a numeric-only scaling symbol; the sealed subset admits a single contiguous run of P at
    // one end of the 9s, without V (V+P is deferred). Anything else fails closed.
    if p_count > 0 && (alpha || seen_v || (p_leading && p_trailing)) {
        return Err(PicError::ScalingPDeferred);
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
    // A field with P scaling must still carry at least one stored digit (a 9).
    if nines == 0 {
        return Err(PicError::NoDigits);
    }
    // A numeric field's digit count fits the field model's u16; an absurd declared count is a
    // resource reject (fail closed), not a truncated mis-claim.
    if nines > u16::MAX as u64 || after_v > u16::MAX as u64 || p_count > u16::MAX as u64 {
        return Err(PicError::BadRepeat);
    }
    let nines = nines as u16;
    let after_v = after_v as u16;
    let p_count = p_count as u16;

    // P-scaling overrides the attr digits/scale (the asymmetric GnuCOBOL rule, proven against
    // `cobc`); storage `size` still uses only the stored `9`s. trailing: digits = 9s + P, scale = -P;
    // leading: digits = 9s, scale = 9s + P.
    let (attr_digits, attr_scale): (u16, i16) = if p_count == 0 {
        (nines, after_v as i16)
    } else if p_trailing {
        (nines + p_count, -(p_count as i16))
    } else {
        (nines, (nines + p_count) as i16)
    };

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

    let (field_type, size, bin_flags) = match usage {
        Usage::Display => {
            let sep = if has_sign && sign_separate { 1 } else { 0 };
            (COB_TYPE_NUMERIC_DISPLAY, nines as usize + sep, 0)
        }
        // COMP-3 always carries a sign nibble (0x0C/0x0D, or 0x0F for unsigned): size = n/2 + 1.
        Usage::Comp3 => (COB_TYPE_NUMERIC_PACKED, nines as usize / 2 + 1, 0),
        // Binary families (GNURUST.14): type BINARY, size from the 1-2-4-8 table; the usage flags
        // distinguish endian + truncation (proven against `cobc`'s emitted attrs).
        Usage::Comp => (
            COB_TYPE_NUMERIC_BINARY,
            binary_size(nines),
            COB_FLAG_BINARY_SWAP | COB_FLAG_BINARY_TRUNC,
        ),
        Usage::Comp5 => (
            COB_TYPE_NUMERIC_BINARY,
            binary_size(nines),
            COB_FLAG_REAL_BINARY,
        ),
        Usage::CompX => (
            COB_TYPE_NUMERIC_BINARY,
            comp_x_size(nines),
            COB_FLAG_BINARY_SWAP,
        ),
        // COMP-6 (GNURUST.18): unsigned packed, two digits/byte, NO sign nibble; size = ceil(n/2).
        Usage::Comp6 => (
            COB_TYPE_NUMERIC_PACKED,
            (nines as usize).div_ceil(2),
            COB_FLAG_NO_SIGN_NIBBLE,
        ),
    };
    flags |= bin_flags;
    // COMP-6 is unsigned regardless of a `PIC S` — clear any sign flags (GnuCOBOL converts a *signed*
    // COMP-6 to COMP-3, which is a separate non-claim).
    if matches!(usage, Usage::Comp6) {
        flags &= !(COB_FLAG_HAVE_SIGN | COB_FLAG_SIGN_SEPARATE | COB_FLAG_SIGN_LEADING);
    }

    Ok(PicField {
        attr: FieldAttr {
            field_type,
            digits: attr_digits,
            scale: attr_scale,
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
    fn binary_field_model_matches_oracle() {
        use crate::attr::{
            COB_FLAG_BINARY_SWAP, COB_FLAG_BINARY_TRUNC, COB_FLAG_HAVE_SIGN, COB_FLAG_REAL_BINARY,
            COB_TYPE_NUMERIC_BINARY,
        };
        // COMP: type BINARY, 1-2-4-8 size, SWAP|TRUNC (+sign).
        let c = f("9(4)", Usage::Comp, false, false);
        assert_eq!(
            (c.attr.field_type, c.attr.flags, c.size),
            (
                COB_TYPE_NUMERIC_BINARY,
                COB_FLAG_BINARY_SWAP | COB_FLAG_BINARY_TRUNC,
                2
            )
        );
        let cs = f("S9(9)", Usage::Comp, false, false);
        assert_eq!(
            (cs.attr.flags, cs.size),
            (
                COB_FLAG_BINARY_SWAP | COB_FLAG_BINARY_TRUNC | COB_FLAG_HAVE_SIGN,
                4
            )
        );
        // COMP-5: native, full-range (REAL_BINARY), same 1-2-4-8 size.
        let c5 = f("9(4)", Usage::Comp5, false, false);
        assert_eq!((c5.attr.flags, c5.size), (COB_FLAG_REAL_BINARY, 2));
        // COMP-X: SWAP, tight size (9(6) -> 3, 9(10) -> 5).
        let cx = f("9(6)", Usage::CompX, false, false);
        assert_eq!((cx.attr.flags, cx.size), (COB_FLAG_BINARY_SWAP, 3));
        assert_eq!(f("9(10)", Usage::CompX, false, false).size, 5);
    }

    #[test]
    fn p_scaling_matches_oracle() {
        // trailing P: digits = 9s + P, scale = -P, size = 9s (proven vs cobc).
        let a = f("999PPP", Usage::Display, false, false);
        assert_eq!(
            (a.attr.field_type, a.attr.digits, a.attr.scale, a.size),
            (0x10, 6, -3, 3)
        );
        let c = f("9PPP", Usage::Display, false, false);
        assert_eq!((c.attr.digits, c.attr.scale, c.size), (4, -3, 1));
        let ff = f("99P", Usage::Display, false, false);
        assert_eq!((ff.attr.digits, ff.attr.scale, ff.size), (3, -1, 2));
        // leading P: digits = 9s, scale = 9s + P, size = 9s.
        let b = f("PPP999", Usage::Display, false, false);
        assert_eq!((b.attr.digits, b.attr.scale, b.size), (3, 6, 3));
        let gg = f("P99", Usage::Display, false, false);
        assert_eq!((gg.attr.digits, gg.attr.scale, gg.size), (2, 3, 2));
        // COMP-3: digits carries P but size uses only the stored 9s (n/2+1).
        let cp = f("999PPP", Usage::Comp3, false, false);
        assert_eq!(
            (cp.attr.field_type, cp.attr.digits, cp.attr.scale, cp.size),
            (0x12, 6, -3, 2)
        );
        // signed trailing P keeps the sign flag.
        let s = f("S999PP", Usage::Display, false, false);
        assert_eq!(
            (s.attr.digits, s.attr.scale, s.attr.flags, s.size),
            (5, -2, 0x0001, 3)
        );
    }

    #[test]
    fn p_scaling_fails_closed() {
        // V+P, P at both ends, and P-only (no 9) are outside the sealed subset.
        assert_eq!(
            build_field("9PV9", Usage::Display, false, false),
            Err(PicError::ScalingPDeferred)
        );
        assert_eq!(
            build_field("P9P", Usage::Display, false, false),
            Err(PicError::ScalingPDeferred)
        );
        assert_eq!(
            build_field("PPP", Usage::Display, false, false),
            Err(PicError::NoDigits)
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

#[cfg(test)]
mod comp6_tests {
    use super::*;
    use crate::attr::{COB_FLAG_NO_SIGN_NIBBLE, COB_TYPE_NUMERIC_PACKED};
    #[test]
    fn comp6_field_model() {
        // 9(3) COMP-6: PACKED, NO_SIGN_NIBBLE, ceil(3/2)=2 bytes (proven vs cobc {0x12,3,0,0x0100}).
        let a = build_field("9(3)", Usage::Comp6, false, false).unwrap();
        assert_eq!(
            (a.attr.field_type, a.attr.digits, a.attr.flags, a.size),
            (COB_TYPE_NUMERIC_PACKED, 3, COB_FLAG_NO_SIGN_NIBBLE, 2)
        );
        assert_eq!(
            build_field("9(4)", Usage::Comp6, false, false)
                .unwrap()
                .size,
            2
        );
        assert_eq!(
            build_field("9(5)", Usage::Comp6, false, false)
                .unwrap()
                .size,
            3
        );
        assert_eq!(
            build_field("9(1)", Usage::Comp6, false, false)
                .unwrap()
                .size,
            1
        );
        // S is ignored (COMP-6 is unsigned): no HAVE_SIGN.
        assert_eq!(
            build_field("S9(4)", Usage::Comp6, false, false)
                .unwrap()
                .attr
                .flags,
            COB_FLAG_NO_SIGN_NIBBLE
        );
    }

    #[test]
    fn comp6_decodes_via_from_packed() {
        // 9(4) COMP-6 bytes 0x12 0x34 -> 1234 (unsigned, no sign nibble).
        let pf = build_field("9(4)", Usage::Comp6, false, false).unwrap();
        let d = crate::Decimal::from_packed(&[0x12, 0x34], &pf.attr);
        assert_eq!(d.unscaled_i128(), Some(1234));
        // 9(3) COMP-6 bytes 0x01 0x23 -> 123 (odd digits, leading 0 nibble).
        let pf3 = build_field("9(3)", Usage::Comp6, false, false).unwrap();
        assert_eq!(
            crate::Decimal::from_packed(&[0x01, 0x23], &pf3.attr).unscaled_i128(),
            Some(123)
        );
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    // KANIFOR: GNURUST.3, GNURUST.9
    /// build_field is TOTAL over a bounded symbolic PICTURE string + usage: it returns Ok or a typed PicError,
    /// never a panic; and a valid Ok field has a positive size.
    #[kani::proof]
    #[kani::unwind(6)]
    fn build_field_is_total() {
        let raw: [u8; 4] = kani::any();
        if let Ok(pic) = core::str::from_utf8(&raw) {
            let u: u8 = kani::any();
            let usage = match u % 5 { 0 => Usage::Display, 1 => Usage::Comp3, 2 => Usage::Comp, 3 => Usage::Comp5, _ => Usage::CompX };
            if let Ok(f) = build_field(pic, usage, kani::any(), kani::any()) {
                assert!(f.size >= 1);
            }
        }
    }
}
