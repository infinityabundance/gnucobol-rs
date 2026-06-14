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
    // drop trailing spaces (the trim the validator applies).
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

// ======================================================================================================
// Dependency-free helpers of the XML/JSON GENERATE path. The serializers themselves
// (the libxml2 `xmlTextWriter` XML path and the cJSON tree path) are the
// declared external-library boundary -- GnuCOBOL 3.2 requires libxml2 / cJSON for ML GENERATE, so a 1:1
// port of those functions would pull in those C libraries. The pure byte/format helpers are ported here.
// ======================================================================================================

/// Port of `mlio.c:int_to_hex` -- map a nibble (`0..=15`) to its lowercase hex character (`0-9`, `a-f`).
pub fn int_to_hex(n: i32) -> u8 {
    if n < 10 {
        (n + b'0' as i32) as u8
    } else {
        (n - 10 + b'a' as i32) as u8
    }
}

/// The lowercase-hex encoding of a field's bytes (each
/// byte as two hex digits, high nibble first), the data XML GENERATE emits for a `hex.`-prefixed element (the libxml2 buffer plumbing is the boundary; this is the byte logic).
pub fn hex_encode(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() * 2);
    for &b in data {
        out.push(int_to_hex((b / 16) as i32));
        out.push(int_to_hex((b % 16) as i32));
    }
    out
}

/// Port of `mlio.c:copy_data_as_string` -- copy `data` into a fresh NUL-terminated buffer (the bytes plus a
/// trailing `\0`).
pub fn copy_data_as_string(data: &[u8]) -> Vec<u8> {
    let mut out = data.to_vec();
    out.push(0);
    out
}

/// Port of `mlio.c:json_strndup` -- duplicate `data` into a fresh `size+1` buffer (the JSON serializer's
/// string-dup; the extra byte is zero-initialised, as `cob_malloc` zeroes).
pub fn json_strndup(data: &[u8]) -> Vec<u8> {
    let mut out = data.to_vec();
    out.push(0);
    out
}

/// Prefix an element name with `hex.` (the name XML GENERATE
/// uses for a hex-encoded element whose content had invalid XML characters). The libxml2 `xmlStrcat`
/// plumbing is the boundary; this is the byte concatenation.
pub fn name_with_hex_prefix(name: &[u8]) -> Vec<u8> {
    let mut out = b"hex.".to_vec();
    out.extend_from_slice(name);
    out
}

/// `mlio.c` XML/JSON `*-CODE` status values (`enum xml_code_status` / `json_code_status`): `0` success,
/// `415` an invalid character was hex-replaced.
pub const XML_OK: i32 = 0;
pub const XML_INVALID_CHAR_REPLACED: i32 = 415;

/// Port of `mlio.c:set_xml_code` / `set_json_code` -- the value written to the `XML-CODE` / `JSON-CODE`
/// special register after a GENERATE. Returns the integer code; the `cob_set_int` to the live register is
/// the runtime-state boundary.
pub fn set_xml_code(code: i32) -> i32 {
    code
}
/// Port of `mlio.c:set_json_code` -- see [`set_xml_code`].
pub fn set_json_code(code: i32) -> i32 {
    code
}

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
    fn generate_helpers() {
        assert_eq!(int_to_hex(5), b'5');
        assert_eq!(int_to_hex(10), b'a');
        assert_eq!(int_to_hex(15), b'f');
        assert_eq!(hex_encode(b"\x00\xAB"), b"00ab".to_vec());
        assert_eq!(copy_data_as_string(b"AB"), b"AB\0".to_vec());
        assert_eq!(json_strndup(b"XY"), b"XY\0".to_vec());
        assert_eq!(name_with_hex_prefix(b"FIELD"), b"hex.FIELD".to_vec());
        assert_eq!(set_xml_code(XML_INVALID_CHAR_REPLACED), 415);
        assert_eq!(set_json_code(XML_OK), 0);
    }

    #[test]
    fn uri_fallback() {
        assert!(cob_is_valid_uri(b"http://x")); // scheme 'h', ':', tail
        assert!(!cob_is_valid_uri(b"http")); // no ':'
        assert!(!cob_is_valid_uri(b"http:")); // empty tail
        assert!(!cob_is_valid_uri(b"")); // empty
    }
}
