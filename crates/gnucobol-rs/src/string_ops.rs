//! `STRING` / `UNSTRING` byte effects (`GNURUST.STRING.UNSTRING.1`): the receiver bytes, pointer, count,
//! delimiter, tally, and overflow produced by narrow `STRING` and `UNSTRING` statements, proven against
//! GnuCOBOL 3.2. The third Procedure-Division byte-mutation court (after `INITIALIZE` and `INSPECT`).
//!
//! **Witnessed rules (from the oracle):** `STRING` concatenates sources left-to-right at a **1-based**
//! `POINTER` — `DELIMITED BY SIZE` takes the whole operand, `DELIMITED BY lit` takes the operand up to (not
//! including) the first `lit`; the **unwritten target tail is preserved** (no space-fill); when the target
//! fills mid-write the partial writes are kept and `ON OVERFLOW` fires. `UNSTRING` splits the source by a
//! delimiter into fields — `COUNT IN` is the **source-field length before truncation**, `DELIMITER IN` is the
//! delimiter that ended the field (**space** when the field ended at source exhaustion), an **empty field**
//! between adjacent delimiters has count 0, `TALLYING IN` counts the filled fields, the `POINTER` is 1-based,
//! and leftover source after the last field is overflow.
//!
//! **Non-claims:** full Procedure Division execution, national/UTF-8 multibyte, multi-delimiter / `ALL`
//! delimiter generalization, locale/collation, business parsing correctness, and all-dialect parity.

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > hay.len() {
        return None;
    }
    hay.windows(needle.len()).position(|w| w == needle)
}

/// One `STRING` source operand and how it is delimited.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum StringSource<'a> {
    /// `DELIMITED BY SIZE` — the whole operand.
    Size(&'a [u8]),
    /// `DELIMITED BY lit` — the operand up to (not including) the first `lit` (whole operand if absent).
    Delimited(&'a [u8], &'a [u8]),
}

/// The result of a `STRING ... INTO` statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringResult {
    pub target: Vec<u8>,
    /// the 1-based `POINTER` value AFTER the statement.
    pub pointer: usize,
    pub overflow: bool,
}

/// `STRING <sources> INTO <target> [WITH POINTER p]` — `prefill` is the target before the statement,
/// `pointer` the 1-based start position. The unwritten target bytes are preserved.
pub fn string_into(prefill: &[u8], sources: &[StringSource], pointer: usize) -> StringResult {
    let mut out = prefill.to_vec();
    let mut pos = pointer.saturating_sub(1);
    let mut overflow = false;
    'outer: for src in sources {
        let bytes = match *src {
            StringSource::Size(b) => b,
            StringSource::Delimited(b, d) => &b[..find(b, d).unwrap_or(b.len())],
        };
        for &byte in bytes {
            if pos >= out.len() {
                overflow = true;
                break 'outer;
            }
            out[pos] = byte;
            pos += 1;
        }
    }
    StringResult { target: out, pointer: pos + 1, overflow }
}

/// One `UNSTRING` receiving field: its bytes (padded/truncated to the field size), the `COUNT IN` (source
/// length before truncation), and the `DELIMITER IN` (empty = ended at source exhaustion).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnstringField {
    pub data: Vec<u8>,
    pub count: usize,
    pub delimiter: Vec<u8>,
}

/// The result of an `UNSTRING ... INTO` statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnstringResult {
    pub fields: Vec<UnstringField>,
    /// the 1-based `POINTER` value AFTER the statement.
    pub pointer: usize,
    /// `TALLYING IN` — the number of receiving fields filled.
    pub tally: usize,
    pub overflow: bool,
}

/// `UNSTRING <source> [DELIMITED BY d] INTO <fields> [...] [WITH POINTER p]`. `delimiter` is `None` for the
/// no-delimiter form (each field takes its own size in chars); `field_sizes` are the receiving field widths.
pub fn unstring(source: &[u8], delimiter: Option<&[u8]>, field_sizes: &[usize], pointer: usize) -> UnstringResult {
    let mut pos = pointer.saturating_sub(1);
    let mut fields = Vec::new();
    let mut tally = 0usize;
    for &fsize in field_sizes {
        if pos >= source.len() {
            // source exhausted: remaining fields get spaces, count 0, no delimiter
            fields.push(UnstringField { data: vec![b' '; fsize], count: 0, delimiter: Vec::new() });
            continue;
        }
        let (field_bytes, delim): (&[u8], Vec<u8>) = match delimiter {
            None => {
                let end = (pos + fsize).min(source.len());
                let fb = &source[pos..end];
                pos = end;
                (fb, Vec::new())
            }
            Some(d) => match find(&source[pos..], d) {
                Some(rel) => {
                    let dp = pos + rel;
                    let fb = &source[pos..dp];
                    pos = dp + d.len();
                    (fb, d.to_vec())
                }
                None => {
                    let fb = &source[pos..];
                    pos = source.len();
                    (fb, Vec::new())
                }
            },
        };
        let count = field_bytes.len();
        let mut data = field_bytes.to_vec();
        data.resize(fsize, b' '); // pad with spaces / truncate to the field width
        fields.push(UnstringField { data, count, delimiter: delim });
        tally += 1;
    }
    let overflow = pos < source.len();
    UnstringResult { fields, pointer: pos + 1, tally, overflow }
}

#[cfg(test)]
mod tests {
    use super::*;
    use StringSource::*;

    #[test]
    fn string_size_preserves_tail() {
        // "AB"+"CDE" into "~~~~~~" -> "ABCDE~" (6th byte preserved)
        let r = string_into(b"~~~~~~", &[Size(b"AB"), Size(b"CDE")], 1);
        assert_eq!(r.target, b"ABCDE~");
        assert!(!r.overflow);
    }
    #[test]
    fn string_with_pointer_is_one_based() {
        let r = string_into(b"~~~~~~", &[Size(b"XY")], 2);
        assert_eq!(r.target, b"~XY~~~");
        assert_eq!(r.pointer, 4);
    }
    #[test]
    fn string_overflow_keeps_partial() {
        let r = string_into(b"~~~~~~", &[Size(b"ABCDEF"), Size(b"GH")], 1);
        assert_eq!(r.target, b"ABCDEF");
        assert!(r.overflow);
    }
    #[test]
    fn string_delimited_by_literal() {
        let r = string_into(b"~~~~~~", &[Delimited(b"HELLO,WORLD", b",")], 1);
        assert_eq!(r.target, b"HELLO~");
    }
    #[test]
    fn unstring_delimited_count_and_delimiter() {
        // "AB,CDE,F  " split by "," into 3x X(4): counts 2/3/3, delimiters ,/,/(space)
        let r = unstring(b"AB,CDE,F  ", Some(b","), &[4, 4, 4], 1);
        assert_eq!(r.fields[0], UnstringField { data: b"AB  ".to_vec(), count: 2, delimiter: b",".to_vec() });
        assert_eq!(r.fields[1], UnstringField { data: b"CDE ".to_vec(), count: 3, delimiter: b",".to_vec() });
        assert_eq!(r.fields[2], UnstringField { data: b"F   ".to_vec(), count: 3, delimiter: Vec::new() });
        assert_eq!(r.tally, 3);
    }
    #[test]
    fn unstring_empty_field_and_truncated_count() {
        // "A,,B      " -> "A"(1), ""(0), rest "B      "(7 chars, truncated into X(4))
        let r = unstring(b"A,,B      ", Some(b","), &[4, 4, 4], 1);
        assert_eq!(r.fields[0].count, 1);
        assert_eq!(r.fields[1].count, 0);
        assert_eq!(r.fields[1].data, b"    ");
        assert_eq!(r.fields[2].count, 7);
        assert_eq!(r.fields[2].data, b"B   ");
    }
    #[test]
    fn unstring_no_delimiter_one_based_pointer() {
        let r = unstring(b"ABCDEFGH  ", None, &[4], 3);
        assert_eq!(r.fields[0].data, b"CDEF");
        assert_eq!(r.pointer, 7);
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    // KANIFOR: GNURUST.STRING.UNSTRING.1
    /// STRING INTO never changes the target's length (it overwrites within the receiver, tail preserved).
    #[kani::proof]
    #[kani::unwind(9)]
    fn string_into_preserves_target_length() {
        let prefill: [u8; 6] = kani::any();
        let src: [u8; 4] = kani::any();
        let ptr: usize = kani::any();
        kani::assume(ptr >= 1 && ptr <= 7);
        let r = string_into(&prefill, &[StringSource::Size(&src)], ptr);
        assert_eq!(r.target.len(), prefill.len());
    }
}
