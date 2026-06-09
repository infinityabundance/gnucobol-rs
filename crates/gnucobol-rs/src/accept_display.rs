//! `DISPLAY` / `ACCEPT` byte effects (`GNURUST.ACCEPT.DISPLAY.1`): the emitted text of `DISPLAY` and the
//! received-field bytes of `ACCEPT`, proven against GnuCOBOL 3.2. For a forensic port, emitted text is
//! evidence too.
//!
//! **Witnessed rules (from the oracle):** `DISPLAY` emits its operands' bytes **concatenated with no
//! separator**, followed by a single newline (`\n`); a literal emits its text, an alphanumeric field emits
//! its (space-padded) bytes, and an unsigned `9(n)` field emits its zoned digit bytes. `ACCEPT field FROM
//! CONSOLE` reads one input line and **moves it into the field left-justified, space-padded, truncated** to
//! the field width — exactly a `MOVE` of the line into an alphanumeric receiver.
//!
//! **Non-claims:** `DISPLAY` of a **signed** numeric (GnuCOBOL prefixes `+`/`-`) or a `V`-scaled / edited
//! numeric (it inserts a `.` and reformats) — those reformat and are deferred; `DISPLAY UPON`/`WITH NO
//! ADVANCING`, `ACCEPT FROM DATE/TIME/environment/screen`, device/console specifics, and all dialects.

/// `DISPLAY <operands>` — the emitted bytes: every operand's display bytes concatenated, then one `\n`. Each
/// operand's bytes are its display form (literal text, alphanumeric field bytes, or unsigned `9(n)` digits).
pub fn display_line(operands: &[&[u8]]) -> Vec<u8> {
    let mut out = Vec::new();
    for op in operands {
        out.extend_from_slice(op);
    }
    out.push(b'\n');
    out
}

/// `ACCEPT <field> FROM CONSOLE` — the received field bytes for one input `line`: left-justified into the
/// field, space-padded, truncated to `field_size` (a `MOVE` of the line into an alphanumeric receiver).
pub fn accept_field(line: &[u8], field_size: usize) -> Vec<u8> {
    let mut out: Vec<u8> = line.iter().take(field_size).copied().collect();
    out.resize(field_size, b' ');
    out
}

/// `DISPLAY <numeric>` — the emitted text of a numeric field (`GNURUST.ACCEPT.DISPLAY.2`). GnuCOBOL reformats
/// a numeric on DISPLAY: a **signed** field (`S9`) gets a leading `+`/`-` (positive zero is `+`), and a
/// `V`-scaled field gets a `.` inserted at the implied decimal point. `digits` are the `n` magnitude digit
/// bytes (`'0'..'9'`, e.g. `b"01234"` for `S9(3)V99 = 12.34`), `scale` the number of fractional digits,
/// `signed` whether the PIC has `S`, and `negative` the sign of the value.
///
/// Examples: `S9(3) = -42` → `-042`; `S9(3) = 0` → `+000`; `9(3)V99 = 12.34` → `012.34`;
/// `S9(3)V99 = -12.34` → `-012.34`. (An unsigned non-`V` field emits its digits unchanged — see
/// [`display_line`].) **Non-claims:** numeric-edited PICs (`Z`/`,`/`*`/`$`/`CR`/`DB` — see `GNURUST.16`),
/// `BLANK WHEN ZERO`, `JUSTIFIED`, floating-point `USAGE`, and all dialects.
pub fn display_numeric(digits: &[u8], scale: usize, signed: bool, negative: bool) -> Vec<u8> {
    let mut out = Vec::new();
    if signed {
        out.push(if negative { b'-' } else { b'+' });
    }
    let split = digits.len() - scale;
    out.extend_from_slice(&digits[..split]);
    if scale > 0 {
        out.push(b'.');
        out.extend_from_slice(&digits[split..]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn display_concatenates_operands_plus_newline() {
        assert_eq!(display_line(&[b"[", b"ABC", b"]"]), b"[ABC]\n");
        assert_eq!(display_line(&[b"[", b"HEL  ", b"]"]), b"[HEL  ]\n"); // X(5) padded
        assert_eq!(display_line(&[b"[", b"042", b"]"]), b"[042]\n"); // unsigned 9(3)=42
        assert_eq!(display_line(&[b"[", b"X", b"Y", b"Z", b"]"]), b"[XYZ]\n");
        assert_eq!(display_line(&[b"[", b"HEL  ", b"042", b"]"]), b"[HEL  042]\n");
    }
    #[test]
    fn accept_moves_line_into_field() {
        assert_eq!(accept_field(b"HI", 6), b"HI    "); // short -> space-padded
        assert_eq!(accept_field(b"ABCDEFGH", 6), b"ABCDEF"); // long -> truncated
        assert_eq!(accept_field(b"EXACT6", 6), b"EXACT6");
    }
    #[test]
    fn display_numeric_signed_and_v_scaled() {
        assert_eq!(display_numeric(b"042", 0, true, true), b"-042"); // S9(3) = -42
        assert_eq!(display_numeric(b"042", 0, true, false), b"+042"); // S9(3) = +42
        assert_eq!(display_numeric(b"000", 0, true, false), b"+000"); // S9(3) = 0 -> +
        assert_eq!(display_numeric(b"01234", 2, false, false), b"012.34"); // 9(3)V99
        assert_eq!(display_numeric(b"01234", 2, true, true), b"-012.34"); // S9(3)V99 = -12.34
        assert_eq!(display_numeric(b"01234", 2, true, false), b"+012.34"); // S9(3)V99 = +12.34
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    // KANIFOR: GNURUST.ACCEPT.DISPLAY.1
    /// DISPLAY emits exactly the concatenated operand bytes plus one newline; ACCEPT fills exactly the width.
    #[kani::proof]
    #[kani::unwind(10)]
    fn display_and_accept_lengths() {
        let a: [u8; 3] = kani::any();
        let b: [u8; 2] = kani::any();
        let out = display_line(&[&a, &b]);
        assert_eq!(out.len(), a.len() + b.len() + 1);
        let size: usize = kani::any();
        kani::assume(size <= 8);
        assert_eq!(accept_field(&a, size).len(), size);
    }
    // KANIFOR: GNURUST.ACCEPT.DISPLAY.2
    /// DISPLAY of a signed/V numeric has width = sign? + int_digits + (scale>0 ? 1+scale : 0).
    #[kani::proof]
    #[kani::unwind(12)]
    fn display_numeric_width() {
        let digits: [u8; 5] = kani::any();
        let scale: usize = kani::any();
        let signed: bool = kani::any();
        let negative: bool = kani::any();
        kani::assume(scale < digits.len());
        let out = display_numeric(&digits, scale, signed, negative);
        let expect = (signed as usize) + (digits.len() - scale) + if scale > 0 { 1 + scale } else { 0 };
        assert_eq!(out.len(), expect);
    }
}
