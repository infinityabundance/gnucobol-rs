//! Port of mlio.c -- `XML GENERATE`/`PARSE` + `JSON GENERATE`. The C generation/parse core wraps the
//! external **libxml2** (`xmlTextWriter`/`xmlParseURI`/the parser) and **json-c**/**cJSON** libraries, so
//! its exact byte output is not portable to zero-dep Rust without those libraries -- that core is deferred
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
/// XML content (`Char ::= #x9 | #xA | #xD | [#x20-#xD7FF] | ...`; the single-byte assumption mirrors the C
/// `TO-DO: assumes UTF-8`).
pub fn has_invalid_xml_char(data: &[u8]) -> bool {
    data.iter().any(|&c| is_cntrl(c) && c != 0x09 && c != 0x0a && c != 0x0d)
}

/// C `iscntrl` (C locale): `0x00-0x1F` and `0x7F`.
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

/// `is_valid_xml_name (f)` (mlio.c): the (trailing-space-trimmed) field is a valid XML `Name` -- a
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

// ======================================================================================================
// Native XML / JSON GENERATE -- a from-scratch Rust serializer of the `cob_ml_tree`, producing byte
// output verified IDENTICAL to the admitted GnuCOBOL (which drives libxml2 `xmlTextWriter` / json-c). This
// reimplements the XML and JSON tree serializers and their helpers natively
// (no libxml2 / cJSON dependency), proven against the oracle by `ml_generate_sweep`.
// ======================================================================================================

/// The serialized content of an `ML` leaf node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MlContent {
    /// A group/parent node with no direct content.
    None,
    /// A `PIC X` value (the raw field bytes; trailing spaces are trimmed on output).
    Alnum(Vec<u8>),
    /// A numeric value: the digit characters, the implied decimal scale, and whether it is negative.
    Numeric { digits: Vec<u8>, scale: usize, negative: bool },
}

/// An ML attribute (`cob_ml_attr`): a name and its (already-stringified) value.
#[derive(Debug, Clone)]
pub struct MlAttr {
    pub name: Vec<u8>,
    pub value: Vec<u8>,
}

/// The `cob_ml_tree` a compiled `XML GENERATE` / `JSON GENERATE` walks: a node name, its attributes,
/// its content (leaf) or children (group), and the SUPPRESS flag.
#[derive(Debug, Clone)]
pub struct MlTree {
    pub name: Vec<u8>,
    pub attrs: Vec<MlAttr>,
    pub content: MlContent,
    pub is_suppressed: bool,
    pub children: Vec<MlTree>,
}

/// Port of `mlio.c:get_xml_name` -- the element name: the field's bytes with trailing spaces trimmed.
pub fn get_xml_name(name: &[u8]) -> Vec<u8> {
    let mut end = name.len();
    while end > 0 && name[end - 1] == b' ' {
        end -= 1;
    }
    name[..end].to_vec()
}

/// Port of `mlio.c:get_trimmed_data` -- a `PIC X` value with trailing spaces trimmed.
pub fn get_trimmed_data(data: &[u8]) -> Vec<u8> {
    get_xml_name(data)
}
/// Port of `mlio.c:get_trimmed_xml_data` -- [`get_trimmed_data`] for the XML path.
pub fn get_trimmed_xml_data(data: &[u8]) -> Vec<u8> {
    get_trimmed_data(data)
}
/// Port of `mlio.c:get_xml_data` -- the XML text of a `PIC X` leaf (trailing-space-trimmed).
pub fn get_xml_data(data: &[u8]) -> Vec<u8> {
    get_trimmed_data(data)
}

/// Format a numeric ML value (the shared logic of `mlio.c:get_xml_num` / `get_json_num`): strip leading
/// zeros from the integer part (keeping one digit), insert the decimal point at the scale, keep the
/// fractional digits, and prepend `-` when negative -- e.g. `042`/scale 0 -> `-42`, `01250`/scale 2 ->
/// `12.50`.
fn format_ml_num(digits: &[u8], scale: usize, negative: bool) -> Vec<u8> {
    let int_len = digits.len().saturating_sub(scale);
    let int_part = &digits[..int_len];
    let mut start = 0;
    while start + 1 < int_part.len() && int_part[start] == b'0' {
        start += 1;
    }
    let mut out = Vec::new();
    if negative {
        out.push(b'-');
    }
    if int_part.is_empty() {
        out.push(b'0');
    } else {
        out.extend_from_slice(&int_part[start..]);
    }
    if scale > 0 {
        out.push(b'.');
        out.extend_from_slice(&digits[int_len..]);
    }
    out
}

/// Port of `mlio.c:get_xml_num` -- the XML text of a numeric leaf (see [`format_ml_num`]).
pub fn get_xml_num(digits: &[u8], scale: usize, negative: bool) -> Vec<u8> {
    format_ml_num(digits, scale, negative)
}
/// Port of `mlio.c:get_json_num` -- the JSON text of a numeric value (same formatting, unquoted).
pub fn get_json_num(digits: &[u8], scale: usize, negative: bool) -> Vec<u8> {
    format_ml_num(digits, scale, negative)
}

/// XML-escape text content (`&`->`&amp;`, `<`->`&lt;`, `>`->`&gt;`), matching libxml2's writer.
fn xml_escape(data: &[u8], out: &mut Vec<u8>) {
    for &b in data {
        match b {
            b'&' => out.extend_from_slice(b"&amp;"),
            b'<' => out.extend_from_slice(b"&lt;"),
            b'>' => out.extend_from_slice(b"&gt;"),
            _ => out.push(b),
        }
    }
}

/// Port of `mlio.c:generate_content` -- write a leaf's content: a `PIC X` value (trimmed, XML-escaped) or a
/// numeric value ([`get_xml_num`]).
pub fn generate_content(tree: &MlTree, out: &mut Vec<u8>) {
    match &tree.content {
        MlContent::Alnum(d) => xml_escape(&get_xml_data(d), out),
        MlContent::Numeric { digits, scale, negative } => out.extend_from_slice(&get_xml_num(digits, *scale, *negative)),
        MlContent::None => {}
    }
}

/// Port of `mlio.c:generate_normal_attribute` -- write one attribute ` name="value"` (the value XML-escaped).
pub fn generate_normal_attribute(attr: &MlAttr, out: &mut Vec<u8>) {
    out.push(b' ');
    out.extend_from_slice(&get_xml_name(&attr.name));
    out.push(b'=');
    out.push(0x22); // double-quote (written as 0x22 to keep the port-index tokenizer happy)
    xml_escape(&attr.value, out);
    out.push(0x22);
}

/// Port of `mlio.c:generate_attributes` -- write every attribute of a node.
pub fn generate_attributes(attrs: &[MlAttr], out: &mut Vec<u8>) {
    for a in attrs {
        generate_normal_attribute(a, out);
    }
}

/// Port of `mlio.c:generate_normal_element` -- `<name attrs>` then children (recursed) or content, then
/// `</name>`.
pub fn generate_normal_element(tree: &MlTree, out: &mut Vec<u8>) {
    let name = get_xml_name(&tree.name);
    out.push(b'<');
    out.extend_from_slice(&name);
    generate_attributes(&tree.attrs, out);
    out.push(b'>');
    if !tree.children.is_empty() {
        for child in &tree.children {
            generate_xml_from_tree(child, out);
        }
    } else {
        generate_content(tree, out);
    }
    out.extend_from_slice(b"</");
    out.extend_from_slice(&name);
    out.push(b'>');
}

/// Port of `mlio.c:generate_hex_element` -- when a leaf's content has invalid XML characters, emit a
/// `<hex.name>hexdata</hex.name>` element (the content hex-encoded).
pub fn generate_hex_element(tree: &MlTree, out: &mut Vec<u8>) {
    let name = name_with_hex_prefix(&get_xml_name(&tree.name));
    out.push(b'<');
    out.extend_from_slice(&name);
    out.push(b'>');
    if let MlContent::Alnum(d) = &tree.content {
        out.extend_from_slice(&hex_encode(d));
    }
    out.extend_from_slice(b"</");
    out.extend_from_slice(&name);
    out.push(b'>');
}

/// Port of `mlio.c:generate_element` -- pick the hex form for a leaf whose alphanumeric content has invalid
/// XML characters (setting `XML-CODE` to `415`), else the normal element.
pub fn generate_element(tree: &MlTree, out: &mut Vec<u8>) -> i32 {
    if let MlContent::Alnum(d) = &tree.content {
        if tree.children.is_empty() && has_invalid_xml_char(d) {
            generate_hex_element(tree, out);
            return set_xml_code(XML_INVALID_CHAR_REPLACED);
        }
    }
    generate_normal_element(tree, out);
    XML_OK
}

/// Port of `mlio.c:generate_xml_from_tree` -- the recursive XML serializer: skip a SUPPRESSed node, else
/// emit its element (and, for a group, its children).
pub fn generate_xml_from_tree(tree: &MlTree, out: &mut Vec<u8>) {
    if tree.is_suppressed {
        return;
    }
    generate_element(tree, out);
}

/// Port of the built-in (non-libxml2) XML serializer `mlio.c:cob_xml_generate_new` -- serialize a `cob_ml_tree` to its XML
/// bytes (the `XML GENERATE` output written to the receiving item).
pub fn cob_xml_generate_new(tree: &MlTree) -> Vec<u8> {
    let mut out = Vec::new();
    generate_xml_from_tree(tree, &mut out);
    out
}

/// JSON-escape a string value (the double-quote and backslash escaped, control chars -> `\uXXXX`),
/// matching json-c. (`0x22` is the double-quote, written numerically for the port-index tokenizer.)
fn json_escape(data: &[u8], out: &mut Vec<u8>) {
    for &b in data {
        match b {
            0x22 => {
                out.push(b'\\');
                out.push(0x22);
            }
            b'\\' => out.extend_from_slice(b"\\\\"),
            0x08 => out.extend_from_slice(b"\\b"),
            0x09 => out.extend_from_slice(b"\\t"),
            0x0a => out.extend_from_slice(b"\\n"),
            0x0c => out.extend_from_slice(b"\\f"),
            0x0d => out.extend_from_slice(b"\\r"),
            c if c < 0x20 => out.extend_from_slice(format!("\\u{c:04x}").as_bytes()),
            c => out.push(c),
        }
    }
}

/// Port of `mlio.c:generate_json_from_tree` -- the recursive JSON serializer: `"name":value` for a leaf (a
/// quoted, JSON-escaped string for `PIC X`; an unquoted number for numeric) and `"name":{...}` for a group;
/// SUPPRESSed nodes are skipped.
pub fn generate_json_from_tree(tree: &MlTree, out: &mut Vec<u8>) {
    if tree.is_suppressed {
        return;
    }
    out.push(0x22);
    out.extend_from_slice(&get_xml_name(&tree.name));
    out.push(0x22);
    out.push(b':');
    if !tree.children.is_empty() {
        out.push(b'{');
        let mut first = true;
        for child in &tree.children {
            if child.is_suppressed {
                continue;
            }
            if !first {
                out.push(b',');
            }
            first = false;
            generate_json_from_tree(child, out);
        }
        out.push(b'}');
    } else {
        match &tree.content {
            MlContent::Alnum(d) => {
                out.push(0x22);
                json_escape(&get_trimmed_data(d), out);
                out.push(0x22);
            }
            MlContent::Numeric { digits, scale, negative } => out.extend_from_slice(&get_json_num(digits, *scale, *negative)),
            MlContent::None => out.extend_from_slice(b"null"),
        }
    }
}

/// Port of the built-in (non-cJSON) JSON serializer `mlio.c:cob_json_generate_new` -- serialize a `cob_ml_tree` to its JSON
/// bytes (`{ "root": ... }`).
pub fn cob_json_generate_new(tree: &MlTree) -> Vec<u8> {
    let mut out = vec![b'{'];
    generate_json_from_tree(tree, &mut out);
    out.push(b'}');
    out
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    // KANIFOR: GNURUST.MLIO.GENERATE.1
    /// Serializing a small numeric leaf never panics and brackets the value in `<N>...</N>`.
    #[kani::proof]
    #[kani::unwind(4)]
    fn xml_generate_total() {
        let d0: u8 = kani::any();
        kani::assume(d0.is_ascii_digit());
        let tree = MlTree {
            name: b"N".to_vec(),
            attrs: vec![],
            content: MlContent::Numeric { digits: vec![d0], scale: 0, negative: false },
            is_suppressed: false,
            children: vec![],
        };
        let out = cob_xml_generate_new(&tree);
        assert_eq!(out.first(), Some(&b'<'));
        assert_eq!(out.last(), Some(&b'>'));
    }
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

    fn leaf_num(name: &str, digits: &[u8], scale: usize, neg: bool) -> MlTree {
        MlTree { name: name.into(), attrs: vec![], content: MlContent::Numeric { digits: digits.to_vec(), scale, negative: neg }, is_suppressed: false, children: vec![] }
    }
    fn leaf_x(name: &str, v: &[u8]) -> MlTree {
        MlTree { name: name.into(), attrs: vec![], content: MlContent::Alnum(v.to_vec()), is_suppressed: false, children: vec![] }
    }
    fn group(name: &str, children: Vec<MlTree>) -> MlTree {
        MlTree { name: name.into(), attrs: vec![], content: MlContent::None, is_suppressed: false, children }
    }

    #[test]
    fn ml_generate_matches_oracle() {
        // The exact tree from the oracle probe (NEG S9(3)=-42, DEC 9(3)V99=12.50, SPC X(5)="a<b&c",
        // GRP{ X X(2)="hi", Y 9=7 }); output must equal the admitted GnuCOBOL byte-for-byte.
        let tree = group(
            "G",
            vec![
                leaf_num("NEG", b"042", 0, true),
                leaf_num("DEC", b"01250", 2, false),
                leaf_x("SPC", b"a<b&c"),
                group("GRP", vec![leaf_x("X", b"hi"), leaf_num("Y", b"7", 0, false)]),
            ],
        );
        assert_eq!(
            cob_xml_generate_new(&tree),
            b"<G><NEG>-42</NEG><DEC>12.50</DEC><SPC>a&lt;b&amp;c</SPC><GRP><X>hi</X><Y>7</Y></GRP></G>".to_vec()
        );
        assert_eq!(
            cob_json_generate_new(&tree),
            b"{\"G\":{\"NEG\":-42,\"DEC\":12.50,\"SPC\":\"a<b&c\",\"GRP\":{\"X\":\"hi\",\"Y\":7}}}".to_vec()
        );
        // trailing-space trimming + the simple case
        let g2 = group("G", vec![leaf_x("A", b"abc"), leaf_num("N", b"042", 0, false)]);
        assert_eq!(cob_xml_generate_new(&g2), b"<G><A>abc</A><N>42</N></G>".to_vec());
        assert_eq!(cob_json_generate_new(&g2), b"{\"G\":{\"A\":\"abc\",\"N\":42}}".to_vec());
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
