//! Edited-picture **decode** court (`GNURUST.16`, decode-only subset `16a`).
//!
//! **Doctrine.** GNURUST.16 admits edited pictures only as an oracle-proven decode boundary for
//! DISPLAY-shaped edited fields: it interprets admitted edited bytes and presentation markers without
//! claiming report execution, numeric-to-edited formatting, locale/currency generality, EBCDIC zoned
//! editing, or arithmetic semantics.
//!
//! **Admitted subset (`16a`, decode-only):** `Z` (zero suppression), `9` (digit), `,` (comma), `.`
//! (decimal point), and a leading **or** trailing `+`/`-` sign, with `(n)` repeats. Every other
//! edited symbol — `$ * CR DB B 0 /` (the financial decorations, deferred to `16b`) and anything
//! outside this set — **fails closed** with [`EditedError`]. This is *decode* only: it reads an
//! edited field's bytes (as produced by GnuCOBOL) back into a value + presentation text; it does not
//! produce edited output (`MOVE numeric → edited`), nor touch binary/packed storage.

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

/// The character width of an admitted edited picture (errors on any symbol outside `16a`).
pub fn edited_size(pic: &str) -> Result<usize, EditedError> {
    let chars: Vec<char> = pic.trim().chars().filter(|c| !c.is_whitespace()).collect();
    if chars.is_empty() {
        return Err(EditedError::Empty);
    }
    let mut total = 0usize;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i].to_ascii_uppercase();
        match c {
            'Z' | '9' | ',' | '.' | '+' | '-' => {}
            _ => return Err(EditedError::UnsupportedSymbol(c)),
        }
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
        total += count;
    }
    Ok(total)
}

/// Decode the bytes of an edited DISPLAY field under an admitted `16a` picture into its presentation
/// text and recovered numeric value. Insertion characters (`,`, suppression spaces) are skipped; the
/// `.` byte marks the decimal point; a `-` marks a negative value.
pub fn decode_edited(pic: &str, bytes: &[u8]) -> Result<EditedDecode, EditedError> {
    let size = edited_size(pic)?;
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
    for &b in bytes {
        match b {
            b'0'..=b'9' => {
                digits.push(b - b'0');
                if seen_dot {
                    scale += 1;
                }
            }
            b'.' => seen_dot = true,
            b'-' => negative = true,
            b'+' | b',' | b' ' => {} // sign-shown-positive / insertion / suppression
            other => return Err(EditedError::InvalidByte(other)),
        }
    }
    // All-suppressed (pure `Z` over zero) → value 0; otherwise the collected digits at `scale`.
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
        assert!(matches!(
            edited_size("$ZZ9"),
            Err(EditedError::UnsupportedSymbol('$'))
        ));
        assert!(matches!(
            edited_size("ZZ9CR"),
            Err(EditedError::UnsupportedSymbol('C'))
        ));
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
        // unsupported symbol
        assert!(matches!(
            decode_edited("$ZZ9", b" $ 5"),
            Err(EditedError::UnsupportedSymbol('$'))
        ));
        // size mismatch
        assert!(matches!(
            decode_edited("ZZ9", b"12"),
            Err(EditedError::SizeMismatch { .. })
        ));
        // invalid byte (a letter the 16a subset never produces)
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
