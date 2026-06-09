//! Edited-picture **decode** court (`GNURUST.16`, decode-only; `16a` + `16b`).
//!
//! **Doctrine.** GNURUST.16 admits edited pictures only as an oracle-proven decode boundary for
//! DISPLAY-shaped edited fields: it interprets admitted edited bytes and presentation markers without
//! claiming report execution, numeric-to-edited formatting, locale/currency generality, EBCDIC zoned
//! editing, or arithmetic semantics.
//!
//! **Admitted subset (decode-only):**
//! - `16a`: `Z` (zero suppression), `9` (digit), `,`, `.`, leading/trailing `+`/`-`, `(n)` repeats.
//! - `16b`: the **financial decorations** — `$` (currency, fixed or floating), `*` (check protection /
//!   star fill), `CR`/`DB` (trailing credit/debit sign), `B` (blank insertion), `0` (zero insertion),
//!   `/` (slash insertion).
//!
//! Decode is **slot-based** (picture-position-aware), which is required for `0` insertion: a literal
//! inserted `0` is a fixed character, not a value digit, and only the picture distinguishes them.
//! Everything outside the subset, wrong width, and corrupt bytes **fail closed** with [`EditedError`].
//! This is *decode* only: it does not produce edited output (`MOVE numeric → edited`), nor touch
//! binary/packed storage.

use crate::value::Decimal;

/// Why an edited field could not be decoded (fail closed — never a silent mis-read).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EditedError {
    /// A picture symbol outside the admitted `16a` subset (e.g. `$ * C D B 0 /` or a usage symbol).
    UnsupportedSymbol(char),
    /// The byte length does not match the picture's character width.
    SizeMismatch { expected: usize, got: usize },
    /// A byte in a position the `16a` subset never produces (corrupt/foreign edited data).
    InvalidByte(u8),
    /// Empty picture.
    Empty,
}

impl core::fmt::Display for EditedError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            EditedError::UnsupportedSymbol(c) => {
                write!(
                    f,
                    "edited PICTURE symbol '{c}' outside the admitted 16a decode subset"
                )
            }
            EditedError::SizeMismatch { expected, got } => {
                write!(
                    f,
                    "edited field is {got} bytes, picture is {expected} positions"
                )
            }
            EditedError::InvalidByte(b) => write!(f, "invalid byte {b:#04x} in edited field"),
            EditedError::Empty => write!(f, "empty edited PICTURE"),
        }
    }
}
impl std::error::Error for EditedError {}

/// A decoded edited field: the presentation `raw_text` exactly as stored, plus the recovered
/// `numeric_value` (when the admitted subset can recover it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditedDecode {
    /// The field bytes as text (Latin-1), exactly as stored (leading suppression spaces preserved).
    pub raw_text: String,
    /// The recovered numeric value, or `None` if it could not be recovered within the subset.
    pub numeric_value: Option<Decimal>,
    /// Whether the field carried a negative sign marker.
    pub negative: bool,
}

/// A picture slot for decoding. `Value` positions may carry a digit (`9 Z * $ + -`); `Literal`
/// positions hold a fixed insertion char (`, B 0 /`); `Decimal` is the `.`; `Crdb` is a 2-byte
/// trailing `CR`/`DB` sign.
#[derive(Clone, Copy, PartialEq)]
enum Slot {
    Value,
    Literal,
    Decimal,
    Crdb,
}

/// Parse an admitted edited picture into its decode slots (errors on any symbol outside the subset).
fn parse_slots(pic: &str) -> Result<Vec<Slot>, EditedError> {
    let chars: Vec<char> = pic.trim().chars().filter(|c| !c.is_whitespace()).collect();
    if chars.is_empty() {
        return Err(EditedError::Empty);
    }
    let mut slots = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i].to_ascii_uppercase();
        // CR / DB are two-letter trailing sign symbols (one slot, two byte positions).
        let next = chars.get(i + 1).map(|c| c.to_ascii_uppercase());
        if (c == 'C' && next == Some('R')) || (c == 'D' && next == Some('B')) {
            slots.push(Slot::Crdb);
            i += 2;
            continue;
        }
        let kind = match c {
            // a digit may appear here (or space / currency / star-fill / sign):
            '9' | 'Z' | '*' | '$' | '+' | '-' => Slot::Value,
            // a fixed insertion (comma, blank, literal '0', slash):
            ',' | 'B' | '0' | '/' => Slot::Literal,
            '.' => Slot::Decimal,
            _ => return Err(EditedError::UnsupportedSymbol(c)),
        };
        i += 1;
        let mut count = 1usize;
        if i < chars.len() && chars[i] == '(' {
            let mut j = i + 1;
            let mut num = String::new();
            while j < chars.len() && chars[j] != ')' {
                if !chars[j].is_ascii_digit() || num.len() > 6 {
                    return Err(EditedError::UnsupportedSymbol('('));
                }
                num.push(chars[j]);
                j += 1;
            }
            if j >= chars.len() || num.is_empty() {
                return Err(EditedError::UnsupportedSymbol('('));
            }
            count = num
                .parse()
                .map_err(|_| EditedError::UnsupportedSymbol('('))?;
            i = j + 1;
        }
        for _ in 0..count {
            slots.push(kind);
        }
    }
    Ok(slots)
}

fn slots_width(slots: &[Slot]) -> usize {
    slots
        .iter()
        .map(|s| if *s == Slot::Crdb { 2 } else { 1 })
        .sum()
}

/// The character width of an admitted edited picture (`CR`/`DB` count as two).
pub fn edited_size(pic: &str) -> Result<usize, EditedError> {
    Ok(slots_width(&parse_slots(pic)?))
}

/// Decode the bytes of an edited DISPLAY field into its presentation text and recovered numeric value,
/// **slot by slot**: value positions collect digits (or skip spaces/`$`/`*`/sign), literal positions
/// (including an inserted `0`) are skipped, the `.` slot marks the decimal point, and a trailing
/// `CR`/`DB` (or a `-`) marks a negative value.
pub fn decode_edited(pic: &str, bytes: &[u8]) -> Result<EditedDecode, EditedError> {
    let slots = parse_slots(pic)?;
    let size = slots_width(&slots);
    if bytes.len() != size {
        return Err(EditedError::SizeMismatch {
            expected: size,
            got: bytes.len(),
        });
    }
    let raw_text: String = bytes.iter().map(|&b| b as char).collect();

    let mut negative = false;
    let mut digits: Vec<u8> = Vec::new();
    let mut scale: i16 = 0;
    let mut seen_dot = false;
    let mut bi = 0usize;
    for slot in &slots {
        match slot {
            Slot::Value => {
                let b = bytes[bi];
                bi += 1;
                match b {
                    b'0'..=b'9' => {
                        digits.push(b - b'0');
                        if seen_dot {
                            scale += 1;
                        }
                    }
                    b'-' => negative = true,
                    b' ' | b'$' | b'*' | b'+' => {} // suppression / currency / shown-positive
                    other => return Err(EditedError::InvalidByte(other)),
                }
            }
            Slot::Literal => bi += 1, // fixed insertion (comma / B / inserted 0 / slash): not a value
            Slot::Decimal => {
                bi += 1;
                seen_dot = true;
            }
            Slot::Crdb => {
                let pair = [bytes[bi], bytes[bi + 1]];
                bi += 2;
                if pair == *b"CR" || pair == *b"DB" {
                    negative = true;
                }
            }
        }
    }
    let numeric_value = Some(Decimal {
        negative,
        digits,
        scale,
    });
    Ok(EditedDecode {
        raw_text,
        numeric_value,
        negative,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes() {
        assert_eq!(edited_size("ZZ,ZZ9.99").unwrap(), 9);
        assert_eq!(edited_size("Z(4)9").unwrap(), 5);
        assert_eq!(edited_size("-ZZ9").unwrap(), 4);
        // 16b: financial decorations now sized (CR/DB are two positions).
        assert_eq!(edited_size("$ZZ9").unwrap(), 4);
        assert_eq!(edited_size("ZZ9CR").unwrap(), 5);
        assert_eq!(edited_size("$$,$$9.99").unwrap(), 9);
        assert_eq!(edited_size("99/99/99").unwrap(), 8);
        // still rejects out-of-subset symbols.
        assert!(matches!(
            edited_size("Z%9"),
            Err(EditedError::UnsupportedSymbol('%'))
        ));
    }

    #[test]
    fn decodes_financial_16b() {
        // fixed currency.
        assert_eq!(
            decode_edited("$ZZ9", b"$  5")
                .unwrap()
                .numeric_value
                .unwrap()
                .unscaled_i128(),
            Some(5)
        );
        // floating currency + comma + decimal.
        assert_eq!(
            decode_edited("$$,$$9.99", b"   $12.34")
                .unwrap()
                .numeric_value
                .unwrap(),
            Decimal {
                negative: false,
                digits: vec![1, 2, 3, 4],
                scale: 2
            }
        );
        // star (check protection) fill.
        assert_eq!(
            decode_edited("***9.99", b"***5.00")
                .unwrap()
                .numeric_value
                .unwrap(),
            Decimal {
                negative: false,
                digits: vec![5, 0, 0],
                scale: 2
            }
        );
        // CR / DB trailing sign (negative); positive shows spaces.
        let cr = decode_edited("ZZ9CR", b"  5CR").unwrap();
        assert!(cr.negative && cr.numeric_value.unwrap().unscaled_i128() == Some(-5));
        let pos = decode_edited("ZZ9CR", b"  5  ").unwrap();
        assert!(!pos.negative && pos.numeric_value.unwrap().unscaled_i128() == Some(5));
        let db = decode_edited("ZZ9DB", b" 12DB").unwrap();
        assert!(db.negative && db.numeric_value.unwrap().unscaled_i128() == Some(-12));
        // the subtle one: literal '0' insertion is NOT a value digit (slot-aware).
        // PIC "9990" over value 123 displays "1230"; decode must recover 123, not 1230.
        assert_eq!(
            decode_edited("9990", b"1230")
                .unwrap()
                .numeric_value
                .unwrap()
                .unscaled_i128(),
            Some(123)
        );
        // blank + slash insertions (e.g. a date-shaped edit) are skipped.
        assert_eq!(
            decode_edited("99/99", b"12/34")
                .unwrap()
                .numeric_value
                .unwrap()
                .unscaled_i128(),
            Some(1234)
        );
    }

    #[test]
    fn decodes_common_edits() {
        assert_eq!(
            decode_edited("ZZ,ZZ9.99", b" 1,234.56")
                .unwrap()
                .numeric_value
                .unwrap(),
            Decimal {
                negative: false,
                digits: vec![1, 2, 3, 4, 5, 6],
                scale: 2
            }
        );
        // zero suppression → leading spaces; value recovered.
        assert_eq!(
            decode_edited("ZZZ", b"  5")
                .unwrap()
                .numeric_value
                .unwrap()
                .unscaled_i128(),
            Some(5)
        );
        // pure-Z zero → all blank → 0.
        assert_eq!(
            decode_edited("ZZZ", b"   ")
                .unwrap()
                .numeric_value
                .unwrap()
                .unscaled_i128(),
            Some(0)
        );
        // trailing minus.
        let d = decode_edited("ZZ9-", b"  5-").unwrap();
        assert!(d.negative && d.numeric_value.unwrap().unscaled_i128() == Some(-5));
    }

    #[test]
    fn fails_closed() {
        // unsupported symbol (outside the 16a+16b subset)
        assert!(matches!(
            decode_edited("Z%9", b" %5"),
            Err(EditedError::UnsupportedSymbol('%'))
        ));
        // size mismatch
        assert!(matches!(
            decode_edited("ZZ9", b"12"),
            Err(EditedError::SizeMismatch { .. })
        ));
        // invalid byte in a value position (a letter the subset never produces there)
        assert!(matches!(
            decode_edited("ZZ9", b"X12"[..3].as_ref()),
            Err(EditedError::InvalidByte(b'X'))
        ));
    }
}

/// Fuzz entry: edited decode is panic-free over arbitrary picture/byte pairs.
#[cfg(feature = "fuzzing")]
pub fn __fuzz_edited(data: &[u8]) {
    if data.len() < 2 {
        return;
    }
    let split = (data[0] as usize) % data.len();
    let pic = String::from_utf8_lossy(&data[1..=split]);
    let _ = decode_edited(&pic, &data[split..]);
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    // KANIFOR: GNURUST.16
    /// decode_edited over symbolic field bytes for a fixed edited PICTURE is total (Ok or typed error).
    #[kani::proof]
    #[kani::unwind(10)]
    fn decode_edited_is_total() {
        let bytes: [u8; 6] = kani::any();
        let _ = decode_edited("ZZ9.99", &bytes);
        let _ = edited_size("ZZ9.99");
    }
}
