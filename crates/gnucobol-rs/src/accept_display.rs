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
}
