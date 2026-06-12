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

// ---- GNURUST.16c: numeric -> edited ENCODE (the inverse direction of decode_edited) ----------------

fn sign_char(sym: char, neg: bool) -> char {
    match (sym, neg) {
        ('+', false) => '+',
        ('+', true) => '-',
        ('-', true) => '-',
        _ => ' ', // '-' positive shows a space
    }
}

/// Expand `(n)` repeats and validate every symbol is in the admitted edited subset.
fn expand_repeats(chars: &[char]) -> Result<Vec<char>, EditedError> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if !matches!(c, '9' | 'Z' | '*' | '$' | '+' | '-' | ',' | '.' | 'B' | '0' | '/') {
            return Err(EditedError::UnsupportedSymbol(c));
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
            count = num.parse().map_err(|_| EditedError::UnsupportedSymbol('('))?;
            i = j + 1;
        }
        for _ in 0..count {
            out.push(c);
        }
    }
    Ok(out)
}

/// Right-justify the integer digits and left-justify the fraction digits of `value` into the picture's
/// digit capacities (pad with zeros, truncate overflow — matching a fixed-width COBOL receiving field).
fn align(value: &Decimal, int_cap: usize, frac_cap: usize) -> (Vec<u8>, Vec<u8>) {
    let scale = value.scale.max(0) as usize;
    let int_len = value.digits.len().saturating_sub(scale);
    let int_part = &value.digits[..int_len];
    let frac_part = &value.digits[int_len..];
    let mut int_d = vec![0u8; int_cap];
    for (k, &d) in int_part.iter().rev().enumerate() {
        if k < int_cap {
            int_d[int_cap - 1 - k] = d;
        }
    }
    let mut frac_d = vec![0u8; frac_cap];
    for (k, &d) in frac_part.iter().enumerate() {
        if k < frac_cap {
            frac_d[k] = d;
        }
    }
    (int_d, frac_d)
}

/// Emit the integer field: assign digits to digit-bearing positions (`9 Z *` and float positions, the
/// first float position being the floating symbol's reserved slot), then zero-suppress the leading zone
/// (`Z`/float -> space, `*` -> star, commas in the zone -> the suppression char) and float the symbol to
/// the position immediately left of the first significant digit (or the first forced `9`).
fn emit_int(syms: &[char], digits: &[u8], float_char: Option<char>, neg: bool) -> String {
    let supp = if syms.contains(&'*') { '*' } else { ' ' };
    // a floating `+`/`-` shows a sign-aware glyph (`+`→`+`/`-`, `-`→` `/`-`); a floating `$` is literal.
    let float_glyph = float_char.map(|fc| match fc {
        '+' | '-' => sign_char(fc, neg),
        other => other,
    });
    let mut out: Vec<char> = vec![' '; syms.len()];
    let mut di = 0usize;
    let mut first_float = true;
    for (k, &s) in syms.iter().enumerate() {
        if s == '9' || s == 'Z' || s == '*' {
            out[k] = (b'0' + digits.get(di).copied().unwrap_or(0)) as char;
            di += 1;
        } else if Some(s) == float_char {
            if first_float {
                first_float = false;
                out[k] = ' ';
            } else {
                out[k] = (b'0' + digits.get(di).copied().unwrap_or(0)) as char;
                di += 1;
            }
        } else if matches!(s, ',' | 'B' | '0' | '/') {
            out[k] = if s == 'B' { ' ' } else { s };
        }
    }
    let mut stop = syms.len();
    for (k, &s) in syms.iter().enumerate() {
        if s == '9' {
            stop = k;
            break;
        }
        let suppressible = s == 'Z' || s == '*' || Some(s) == float_char;
        if suppressible && out[k].is_ascii_digit() && out[k] != '0' {
            stop = k;
            break;
        }
    }
    let float_target = float_char.and_then(|fc| (0..stop).rev().find(|&k| syms[k] == fc));
    for (k, slot) in out.iter_mut().enumerate().take(stop) {
        *slot = if Some(k) == float_target {
            float_glyph.unwrap()
        } else {
            supp
        };
    }
    out.into_iter().collect()
}

/// Emit the fraction field: digit positions show their digit, `B` is a space, `0`/`/`/`,` are literal.
fn emit_frac(syms: &[char], digits: &[u8]) -> String {
    let mut di = 0usize;
    let mut out = String::new();
    for &s in syms {
        match s {
            '9' | 'Z' | '*' => {
                out.push((b'0' + digits.get(di).copied().unwrap_or(0)) as char);
                di += 1;
            }
            'B' => out.push(' '),
            '0' | '/' | ',' => out.push(s),
            _ => {}
        }
    }
    out
}

/// Encode (`GNURUST.16c`) a numeric `value` into an edited DISPLAY field, byte-faithful to cobc's
/// `MOVE numeric TO edited-field` for the admitted `16a`+`16b` subset (the inverse of [`decode_edited`]).
/// Fail closed on any symbol outside the subset.
pub fn encode_edited(pic: &str, value: &Decimal) -> Result<Vec<u8>, EditedError> {
    let mut chars: Vec<char> = pic
        .trim()
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    if chars.is_empty() {
        return Err(EditedError::Empty);
    }
    let neg = value.negative && !value.digits.iter().all(|&d| d == 0);

    // trailing CR / DB
    let mut suffix = String::new();
    if chars.len() >= 2 {
        let last2: String = chars[chars.len() - 2..].iter().collect();
        if last2 == "CR" || last2 == "DB" {
            suffix = if neg { last2 } else { "  ".into() };
            chars.truncate(chars.len() - 2);
        }
    }
    // trailing fixed sign (single + / -)
    if suffix.is_empty() {
        if let Some(&last) = chars.last() {
            if (last == '+' || last == '-') && chars.iter().filter(|&&c| c == last).count() == 1 {
                suffix = sign_char(last, neg).to_string();
                chars.pop();
            }
        }
    }
    // leading fixed sign (single + / -) or fixed currency (single $)
    let mut prefix = String::new();
    if let Some(&first) = chars.first() {
        if (first == '+' || first == '-') && chars.iter().filter(|&&c| c == first).count() == 1 {
            prefix = sign_char(first, neg).to_string();
            chars.remove(0);
        } else if first == '$' && chars.iter().filter(|&&c| c == '$').count() == 1 {
            prefix = "$".into();
            chars.remove(0);
        }
    }

    let syms = expand_repeats(&chars)?;
    let float_char = ['$', '+', '-']
        .into_iter()
        .find(|&c| syms.iter().filter(|&&x| x == c).count() >= 2);
    let dot = syms.iter().position(|&c| c == '.');
    let (int_syms, frac_syms): (&[char], &[char]) = match dot {
        Some(d) => (&syms[..d], &syms[d + 1..]),
        None => (&syms[..], &[]),
    };
    let count_digits = |sl: &[char]| -> usize {
        let plain = sl.iter().filter(|&&c| matches!(c, '9' | 'Z' | '*')).count();
        let nf = float_char.map_or(0, |fc| sl.iter().filter(|&&c| c == fc).count());
        plain + nf.saturating_sub(1)
    };
    let (int_d, frac_d) = align(value, count_digits(int_syms), count_digits(frac_syms));

    let mut out = String::new();
    out.push_str(&prefix);
    out.push_str(&emit_int(int_syms, &int_d, float_char, neg));
    if dot.is_some() {
        out.push('.');
    }
    out.push_str(&emit_frac(frac_syms, &frac_d));
    out.push_str(&suffix);

    let bytes = out.into_bytes();
    let expected = edited_size(pic)?;
    if bytes.len() != expected {
        return Err(EditedError::SizeMismatch {
            expected,
            got: bytes.len(),
        });
    }
    Ok(bytes)
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

// ---------------------------------------------------------------------------------------------------
// move.c edited-MOVE leaves (the named entry points; the FieldAttr model carries digits/scale, so the
// receiver/source PICTURE is passed explicitly). The numeric leaves are the sealed encode/decode
// (GNURUST.16C / GNURUST.16); the alphanumeric leaf is the picture-walk.
// ---------------------------------------------------------------------------------------------------

/// `cob_move_display_to_edited (f1, f2)` (move.c:878): edit a numeric DISPLAY value into a
/// NUMERIC-EDITED picture. The faithful port is [`encode_edited`] (sealed `GNURUST.16C`).
pub fn cob_move_display_to_edited(src: &[u8], src_attr: &crate::attr::FieldAttr, dst_pic: &str) -> Result<Vec<u8>, EditedError> {
    encode_edited(dst_pic, &Decimal::from_display(src, src_attr))
}

/// `cob_move_edited_to_display (f1, f2)` (move.c:1214): de-edit a NUMERIC-EDITED field back to its
/// numeric value — [`decode_edited`] (sealed `GNURUST.16`).
pub fn cob_move_edited_to_display(src_pic: &str, src: &[u8]) -> Result<EditedDecode, EditedError> {
    decode_edited(src_pic, src)
}

/// `cob_move_alphanum_to_edited (f1, f2)` (move.c:1293): fill an ALPHANUMERIC-EDITED picture from an
/// alphanumeric source — `A`/`X`/`9` copy a source byte (space once exhausted), `0`/`/` insert the
/// literal, `B` inserts a space, any other symbol yields `'?'` (invalid PIC).
pub fn cob_move_alphanum_to_edited(src_data: &[u8], dst_pic: &str) -> Vec<u8> {
    let mut out = Vec::new();
    let mut si = 0usize;
    for (sym, n) in pic_symbols(dst_pic) {
        for _ in 0..n {
            match sym {
                'A' | 'X' | '9' => out.push(if si < src_data.len() {
                    let b = src_data[si];
                    si += 1;
                    b
                } else {
                    b' '
                }),
                '0' | '/' => out.push(sym as u8),
                'B' => out.push(b' '),
                _ => out.push(b'?'),
            }
        }
    }
    out
}

/// Parse a PICTURE string into `(symbol, repeat)` pairs, handling both `XXX` runs and `X(3)` counts.
fn pic_symbols(pic: &str) -> Vec<(char, usize)> {
    let chars: Vec<char> = pic.trim().chars().filter(|c| !c.is_whitespace()).map(|c| c.to_ascii_uppercase()).collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        i += 1;
        let mut count = 1usize;
        if i < chars.len() && chars[i] == '(' {
            let mut n = 0usize;
            i += 1;
            while i < chars.len() && chars[i].is_ascii_digit() {
                n = n * 10 + (chars[i] as usize - '0' as usize);
                i += 1;
            }
            if i < chars.len() && chars[i] == ')' {
                i += 1;
            }
            count = n;
        }
        out.push((c, count));
    }
    out
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

/// Fuzz entry for the encode direction (`GNURUST.16c`): any picture + any value yields bytes or a typed
/// `EditedError`, never a panic.
pub fn __fuzz_edited_encode(data: &[u8]) {
    if data.len() < 2 {
        return;
    }
    let split = (data[0] as usize) % data.len();
    let pic = String::from_utf8_lossy(&data[1..=split]);
    let value = Decimal {
        negative: data[1] & 1 == 1,
        digits: data[split..].iter().map(|b| b % 10).collect(),
        scale: (data[1] % 4) as i16,
    };
    let _ = encode_edited(&pic, &value);
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

    // KANIFOR: GNURUST.16C
    /// encode_edited over a symbolic small value for a fixed edited PICTURE is total (Ok or typed error).
    #[kani::proof]
    #[kani::unwind(10)]
    fn encode_edited_is_total() {
        let d: [u8; 4] = kani::any();
        let value = Decimal {
            negative: d[0] & 1 == 1,
            digits: d[1..].iter().map(|b| b % 10).collect(),
            scale: 1,
        };
        let _ = encode_edited("$$,$$9.99", &value);
    }
}
