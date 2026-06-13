//! Port of mlio.c — `XML GENERATE`/`PARSE` + `JSON GENERATE`. The C generation/parse core wraps the
//! external **libxml2** (`xmlTextWriter`/`xmlParseURI`/the parser) and **json-c**/**cJSON** libraries, so
//! its exact byte output is not portable to zero-dep Rust without those libraries — that core is deferred
//! (a dedicated effort, or an optional libxml2/json-c feature). This module ports the **self-contained,
//! pure** subset that needs no external library: the XML name/char validation (the W3C XML 1.0
//! productions), the all-spaces / invalid-control-char checks, and the module lifecycle. The libxml2/
//! json-c-bound functions are tracked as the honest remainder by `gnucobol-rs-port-index`.
#![forbid(unsafe_code)]

/// `is_empty (f)` (mlio.c): the field is all spaces.
pub fn is_empty(data: &[u8]) -> bool {
    data.iter().all(|&b| b == b' ')
}

/// `has_invalid_xml_char (f)` (mlio.c): any single-byte control character other than TAB/LF/CR is invalid
/// XML content (`Char ::= #x9 | #xA | #xD | [#x20-#xD7FF] | …`; the single-byte assumption mirrors the C
/// `TO-DO: assumes UTF-8`).
pub fn has_invalid_xml_char(data: &[u8]) -> bool {
    data.iter().any(|&c| is_cntrl(c) && c != 0x09 && c != 0x0a && c != 0x0d)
}

/// C `iscntrl` (C locale): `0x00–0x1F` and `0x7F`.
fn is_cntrl(c: u8) -> bool {
    c < 0x20 || c == 0x7f
}

/// `cob_is_xml_namestartchar (c)` (mlio.c): the XML 1.0 `NameStartChar` production (single-byte subset):
/// `[A-Za-z_]`, `[#xC0-#xD6]`, `[#xD8-#xF6]`, or `>= #xF8`.
pub fn cob_is_xml_namestartchar(c: i32) -> bool {
    let u = c & 0xff;
    (c >= 0 && (c as u8).is_ascii_alphabetic())
        || c == b'_' as i32
        || (0xc0..=0xd6).contains(&u)
        || (0xd8..=0xf6).contains(&u)
        || u >= 0xf8
}

/// `cob_is_xml_namechar (c)` (mlio.c): a `NameStartChar`, or `-` `.` `[0-9]` `#xB7`.
pub fn cob_is_xml_namechar(c: i32) -> bool {
    cob_is_xml_namechar_inner(c)
}
fn cob_is_xml_namechar_inner(c: i32) -> bool {
    cob_is_xml_namestartchar(c)
        || c == b'-' as i32
        || c == b'.' as i32
        || (c >= 0 && (c as u8).is_ascii_digit())
        || (c & 0xff) == 0xb7
}

/// `is_valid_xml_name (f)` (mlio.c): the (trailing-space-trimmed) field is a valid XML `Name` — a
/// `NameStartChar` followed by `NameChar`s.
pub fn is_valid_xml_name(data: &[u8]) -> bool {
    if data.is_empty() || !cob_is_xml_namestartchar(data[0] as i32) {
        return false;
    }
    // get_trimmed_data: drop trailing spaces.
    let mut end = data.len();
    while end > 0 && data[end - 1] == b' ' {
        end -= 1;
    }
    data[1..end].iter().all(|&c| cob_is_xml_namechar(c as i32))
}

/// `cob_is_valid_uri (str)` (mlio.c): in the admitted build (`WITH_XML2`) this delegates to libxml2's
/// `xmlParseURI`; this port implements the non-`WITH_XML2` fallback (a lowercase scheme, a `:`, then a
/// non-empty remainder). The libxml2 path is the documented non-claim.
pub fn cob_is_valid_uri(s: &[u8]) -> bool {
    // C fallback: `if (!str || *str <= 'a' || *str >= 'z') return 0;` then scan to ':' with a tail.
    let first = match s.first() {
        Some(&b) => b,
        None => return false,
    };
    if first <= b'a' || first >= b'z' {
        return false;
    }
    let mut i = 1;
    while i < s.len() && s[i] != b':' {
        i += 1;
    }
    i < s.len() && s[i] == b':' && i + 1 < s.len()
}

/// `cob_init_mlio (lptr)` (mlio.c): module init binding the runtime global. A no-op in this port.
pub fn cob_init_mlio() {}

/// `cob_exit_mlio ()` (mlio.c): module teardown. A no-op (RAII).
pub fn cob_exit_mlio() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xml_name_validation() {
        assert!(cob_is_xml_namestartchar(b'A' as i32));
        assert!(cob_is_xml_namestartchar(b'_' as i32));
        assert!(!cob_is_xml_namestartchar(b'-' as i32));
        assert!(!cob_is_xml_namestartchar(b'0' as i32));
        assert!(cob_is_xml_namechar(b'-' as i32));
        assert!(cob_is_xml_namechar(b'0' as i32));
        assert!(cob_is_xml_namechar(b'.' as i32));
        assert!(!cob_is_xml_namechar(b'/' as i32));
        // valid / invalid names (trailing spaces trimmed)
        assert!(is_valid_xml_name(b"my-element_1   "));
        assert!(is_valid_xml_name(b"_nsfoo"));
        assert!(!is_valid_xml_name(b"ns:foo")); // ':' is NOT a namechar in the C impl (excludes ':')
        assert!(!is_valid_xml_name(b"1bad"));
        assert!(!is_valid_xml_name(b"has space"));
        assert!(!is_valid_xml_name(b""));
    }

    #[test]
    fn empty_and_invalid_chars() {
        assert!(is_empty(b"      "));
        assert!(!is_empty(b"  x   "));
        assert!(has_invalid_xml_char(b"ok\x01bad")); // 0x01 control
        assert!(!has_invalid_xml_char(b"ok\ttab\nlf\rcr")); // tab/lf/cr allowed
        assert!(!has_invalid_xml_char(b"plain text"));
    }

    #[test]
    fn uri_fallback() {
        assert!(cob_is_valid_uri(b"http://x")); // scheme 'h', ':', tail
        assert!(!cob_is_valid_uri(b"http")); // no ':'
        assert!(!cob_is_valid_uri(b"http:")); // empty tail
        assert!(!cob_is_valid_uri(b"")); // empty
    }
}
