//! Byte preservation.
//!
//! Original bytes are never overwritten. Four representations are kept distinct: the original
//! bytes, the decoded text (when decoding succeeds), a normalized analysis representation, and
//! any transformed source. The analysis records encoding, BOM, line endings, trailing
//! whitespace, sequence/indicator/text/identification areas (fixed format), and tab positions.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ByteAnalysis {
    /// Detected encoding ("UTF-8", "UTF-16LE", "UTF-16BE", "ASCII", "unknown").
    pub encoding: String,
    /// BOM bytes as hex ("" when none).
    pub bom_hex: String,
    /// "LF", "CRLF", "CR", or "mixed".
    pub line_endings: String,
    /// Any line with trailing whitespace (count; first line).
    pub trailing_whitespace_lines: usize,
    /// Fixed-format area facts (only meaningful for fixed-format source).
    pub sequence_area: String,
    pub indicator_area: String,
    /// Distinct tab-stop positions (0-based columns) seen in the source.
    pub tab_positions: Vec<usize>,
    /// 1-based line numbers containing an explicit tab character.
    pub tab_lines: Vec<usize>,
    pub bytes: usize,
    pub lines: usize,
}

/// Detect the encoding + BOM of a byte buffer.
fn detect_encoding(bytes: &[u8]) -> (String, String) {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        ("UTF-8".to_string(), "efbbbf".to_string())
    } else if bytes.starts_with(&[0xFF, 0xFE]) {
        ("UTF-16LE".to_string(), "fffe".to_string())
    } else if bytes.starts_with(&[0xFE, 0xFF]) {
        ("UTF-16BE".to_string(), "feff".to_string())
    } else if bytes.iter().all(|b| b.is_ascii()) {
        ("ASCII".to_string(), String::new())
    } else if std::str::from_utf8(bytes).is_ok() {
        ("UTF-8".to_string(), String::new())
    } else {
        ("unknown".to_string(), String::new())
    }
}

/// Analyze the original bytes of a source file.
pub fn analyze(bytes: &[u8]) -> ByteAnalysis {
    let (encoding, bom_hex) = detect_encoding(bytes);
    let mut crlf = 0;
    let mut lf = 0;
    let mut cr = 0;
    let mut trailing = 0usize;
    let mut tab_positions: Vec<usize> = Vec::new();
    let mut tab_lines: Vec<usize> = Vec::new();
    let mut line = 1usize;
    let mut col = 0usize;
    let mut i = 0usize;
    let mut first_line = String::new();
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\n' {
            lf += 1;
            line += 1;
            col = 0;
            i += 1;
            continue;
        }
        if b == b'\r' {
            if bytes.get(i + 1) == Some(&b'\n') {
                crlf += 1;
                i += 2;
            } else {
                cr += 1;
                i += 1;
            }
            line += 1;
            col = 0;
            continue;
        }
        if b == b'\t' {
            if !tab_positions.contains(&col) {
                tab_positions.push(col);
            }
            tab_lines.push(line);
            if line == 1 && first_line.is_empty() {
                first_line = format!("tab at col {col}");
            }
            col += 1;
            i += 1;
            continue;
        }
        if b == b' ' {
            // trailing whitespace detection happens at the next newline; count spaces here
            let mut j = i;
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                j += 1;
            }
            if j >= bytes.len() || bytes[j] == b'\n' || bytes[j] == b'\r' {
                trailing += 1;
            }
            col += j - i;
            i = j;
            continue;
        }
        col += 1;
        i += 1;
    }
    let line_endings = if crlf > 0 && lf == 0 && cr == 0 {
        "CRLF".to_string()
    } else if cr > 0 && lf == 0 && crlf == 0 {
        "CR".to_string()
    } else if crlf > 0 || cr > 0 {
        "mixed".to_string()
    } else {
        "LF".to_string()
    };
    // Fixed-format areas (first line): sequence area = cols 1-6, indicator = col 7.
    let first = bytes.split(|&b| b == b'\n').next().unwrap_or(&[]);
    let seq: String = first.iter().take(6).map(|&b| b as char).collect();
    let ind = first
        .get(6..7)
        .map(|s| (s[0] as char).to_string())
        .unwrap_or_else(|| " ".to_string());
    tab_positions.sort_unstable();
    tab_positions.dedup();
    tab_lines.sort_unstable();
    tab_lines.dedup();
    // Line count: number of line breaks, plus one when the file does not end with a break.
    let lines = if bytes.is_empty() {
        0
    } else if matches!(bytes.last(), Some(b'\n') | Some(b'\r')) {
        lf + crlf + cr
    } else {
        lf + crlf + cr + 1
    };
    ByteAnalysis {
        encoding,
        bom_hex,
        line_endings,
        trailing_whitespace_lines: trailing,
        sequence_area: seq.trim_end().to_string(),
        indicator_area: ind.to_string(),
        tab_positions,
        tab_lines,
        bytes: bytes.len(),
        lines,
    }
}

/// The four preserved representations of one source file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreservedSource {
    /// Blob SHA-256 of the ORIGINAL bytes (never overwritten).
    pub original_sha256: String,
    /// Blob SHA-256 of the decoded text (same as original when ASCII/UTF-8).
    pub decoded_sha256: String,
    /// Blob SHA-256 of the normalized analysis representation (line endings + trailing ws
    /// normalized to LF, no trailing whitespace).
    pub normalized_sha256: String,
    /// Blob SHA-256 of any transformed source ("" when none).
    pub transformed_sha256: String,
    pub analysis: ByteAnalysis,
}

/// Normalize bytes: CRLF/CR -> LF, strip trailing whitespace per line. Used for the normalized
/// analysis representation only -- the original is always preserved separately.
pub fn normalize(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\r' || b == b'\n' {
            // strip trailing whitespace of the line being terminated
            while matches!(out.last(), Some(b' ') | Some(b'\t')) {
                out.pop();
            }
            out.push(b'\n');
            if b == b'\r' && bytes.get(i + 1) == Some(&b'\n') {
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }
        out.push(b);
        i += 1;
    }
    while matches!(out.last(), Some(b' ') | Some(b'\t')) {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_encoding_and_bom() {
        let ascii = analyze(b"IDENTIFICATION DIVISION.\n");
        assert_eq!(ascii.encoding, "ASCII");
        assert_eq!(ascii.bom_hex, "");
        let utf8 = analyze("àéî".as_bytes());
        assert_eq!(utf8.encoding, "UTF-8");
        let mut bom = vec![0xEF, 0xBB, 0xBF];
        bom.extend_from_slice(b"ID");
        assert_eq!(analyze(&bom).encoding, "UTF-8");
        assert_eq!(analyze(&bom).bom_hex, "efbbbf");
        let utf16 = vec![0xFF, 0xFE, 0x00, 0x41];
        assert_eq!(analyze(&utf16).encoding, "UTF-16LE");
    }

    #[test]
    fn detects_line_endings() {
        assert_eq!(analyze(b"a\nb\n").line_endings, "LF");
        assert_eq!(analyze(b"a\r\nb\r\n").line_endings, "CRLF");
        assert_eq!(analyze(b"a\rb\r").line_endings, "CR");
        assert_eq!(analyze(b"a\nb\r\n").line_endings, "mixed");
    }

    #[test]
    fn detects_trailing_whitespace_and_tabs() {
        let a = analyze(b"a   \nb\n");
        assert_eq!(a.trailing_whitespace_lines, 1);
        let t = analyze(b"\tA\tB\n");
        assert!(t.tab_positions.contains(&0));
        assert!(t.tab_positions.contains(&2));
        assert_eq!(t.tab_lines, vec![1]);
    }

    #[test]
    fn fixed_format_areas() {
        // fixed-format: cols 1-6 sequence area, col 7 indicator ('*' = comment), text from col 8.
        let src = b"000100*comment text here\n000200 IDENTIFICATION DIVISION.\n";
        let a = analyze(src);
        assert_eq!(a.sequence_area, "000100");
        assert_eq!(a.indicator_area, "*");
        assert_eq!(a.lines, 2);
    }

    #[test]
    fn normalize_preserves_content_only() {
        let src = b"a  \r\nb\t\r\nc\n";
        let n = normalize(src);
        assert_eq!(n, b"a\nb\nc\n");
        // original untouched
        assert_eq!(src, b"a  \r\nb\t\r\nc\n");
    }
}
