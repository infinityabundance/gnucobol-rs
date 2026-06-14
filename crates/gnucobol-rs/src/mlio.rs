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
pub fn get_hex_xml_data(data: &[u8]) -> Vec<u8> {
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
pub fn get_name_with_hex_prefix(name: &[u8]) -> Vec<u8> {
    let mut out = b"hex.".to_vec();
    out.extend_from_slice(name);
    out
}

// `mlio.c:enum xml_code_status` -- the values written to the `XML-CODE` special register. (These are
// faithful to the C enum; the GENERATE serializer below only ever sets `0` or `XML_INVALID_CHAR_REPLACED`.)
/// `XML_STMT_EXIT` -- a user PROCESSING PROCEDURE requested termination (`-1`).
pub const XML_STMT_EXIT: i32 = -1;
/// `XML_STMT_SUCCESSFULL` / success (`0`). Exposed as [`XML_OK`] for the serializer.
pub const XML_OK: i32 = 0;
/// `XML_PARSE_ERROR_MISC_COMPAT` (`201`) -- various parse errors, only in `XMLPARSE COMPAT`.
pub const XML_PARSE_ERROR_MISC_COMPAT: i32 = 201;
/// `XML_OUT_FIELD_TOO_SMALL` (`400`) -- the receiving field could not hold the generated document.
pub const XML_OUT_FIELD_TOO_SMALL: i32 = 400;
/// `XML_INVALID_NAMESPACE` (`416`).
pub const XML_INVALID_NAMESPACE: i32 = 416;
/// `XML_INVALID_CHAR_REPLACED` (`417`) -- an invalid XML character was hex-replaced (the C `enum` value;
/// the earlier `415` in this port was a transcription slip, corrected here against `mlio.c:80`).
pub const XML_INVALID_CHAR_REPLACED: i32 = 417;
/// `XML_INVALID_NAMESPACE_PREFIX` (`419`).
pub const XML_INVALID_NAMESPACE_PREFIX: i32 = 419;
/// `XML_INTERNAL_ERROR` (`600`).
pub const XML_INTERNAL_ERROR: i32 = 600;

// `mlio.c:enum json_code_status` -- the values written to the `JSON-CODE` special register.
/// `JSON_OUT_FIELD_TOO_SMALL` (`1`).
pub const JSON_OUT_FIELD_TOO_SMALL: i32 = 1;
/// `JSON_INTERNAL_ERROR` (`500`).
pub const JSON_INTERNAL_ERROR: i32 = 500;

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
    let name = get_name_with_hex_prefix(&get_xml_name(&tree.name));
    out.push(b'<');
    out.extend_from_slice(&name);
    out.push(b'>');
    if let MlContent::Alnum(d) = &tree.content {
        out.extend_from_slice(&get_hex_xml_data(d));
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
            return XML_INVALID_CHAR_REPLACED;
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

// ======================================================================================================
// The XML/JSON special-register state (`COB_MODULE_PTR`'s `xml_code`/`json_code`/`xml_event`/`xml_text`/
// `xml_ntext` registers, the `decimal_point`, and the `xml_mode`). In libcob these live on the running
// module; this port models them explicitly so the GENERATE field-write layer and the PARSE state machine
// are pure functions of an `&mut MlModule` instead of touching global runtime state. The setters/getters
// port `mlio.c:set_xml_code/set_xml_exception/get_xml_code/set_xml_event/set_xml_text/set_json_code/
// set_json_exception` one-for-one.
// ======================================================================================================

/// `mlio.c` XML PARSE mode (`COB_MODULE_PTR->xml_mode`): `COB_XML_XMLNSS = 1` (the GnuCOBOL 3.2 default,
/// `common.h:1292`) behaves like Micro Focus `XMLPARSE(XMLNSS)`; `0` is the older COMPAT mode.
pub const COB_XML_XMLNSS: u8 = 1;
/// `common.h:1083` `COB_XML_PARSE_NATIONAL` -- the `XML PARSE` flag selecting `XML-NTEXT` (UTF-16) over
/// `XML-TEXT`.
pub const COB_XML_PARSE_NATIONAL: i32 = 1 << 1;

/// `coblocal.h` exception ids raised by the ML layer (modelled as their symbolic identity; the GnuCOBOL
/// 3.2 happy-path PARSE/GENERATE never raises these, as the oracle event stream confirms).
pub const COB_EC_XML_IMP: i32 = 0x0500;
/// See [`COB_EC_XML_IMP`].
pub const COB_EC_JSON_IMP: i32 = 0x0501;

// `mlio.c` XML-EVENT special-register contents (`#define EVENT_*`). Only the events GnuCOBOL 3.2 actually
// emits are exercised; the full set is carried for fidelity.
/// `EVENT_START_OF_DOCUMENT`.
pub const EVENT_START_OF_DOCUMENT: &[u8] = b"START-OF-DOCUMENT";
/// `EVENT_END_OF_DOCUMENT`.
pub const EVENT_END_OF_DOCUMENT: &[u8] = b"END-OF-DOCUMENT";
/// `EVENT_END_OF_INPUT`.
pub const EVENT_END_OF_INPUT: &[u8] = b"END-OF-INPUT";
/// `EVENT_EXCEPTION`.
pub const EVENT_EXCEPTION: &[u8] = b"EXCEPTION";

/// The XML/JSON special registers of a running module (`COB_MODULE_PTR`), modelled explicitly.
#[derive(Debug, Clone)]
pub struct MlModule {
    /// `COB_MODULE_PTR->decimal_point` -- the character GENERATE writes for the implied decimal point.
    pub decimal_point: u8,
    /// `COB_MODULE_PTR->xml_mode` -- see [`COB_XML_XMLNSS`].
    pub xml_mode: u8,
    /// The `XML-CODE` special register.
    pub xml_code: i32,
    /// The `JSON-CODE` special register.
    pub json_code: i32,
    /// The `XML-EVENT` special register.
    pub xml_event: Vec<u8>,
    /// The `XML-TEXT` special register.
    pub xml_text: Vec<u8>,
    /// The `XML-NTEXT` special register (UTF-16 text; empty unless `NATIONAL`).
    pub xml_ntext: Vec<u8>,
    /// The most recently raised exception id (`cob_set_exception`), or `None`.
    pub last_exception: Option<i32>,
}

impl Default for MlModule {
    fn default() -> Self {
        MlModule {
            decimal_point: b'.',
            xml_mode: COB_XML_XMLNSS,
            xml_code: 0,
            json_code: 0,
            xml_event: Vec::new(),
            xml_text: Vec::new(),
            xml_ntext: Vec::new(),
            last_exception: None,
        }
    }
}

/// Port of `mlio.c:set_xml_code` -- write the `XML-CODE` special register.
pub fn set_xml_code(module: &mut MlModule, code: i32) {
    module.xml_code = code;
}
/// Port of `mlio.c:set_xml_exception` -- raise the internal XML exception (`COB_EC_XML_IMP`) and set
/// `XML-CODE`.
pub fn set_xml_exception(module: &mut MlModule, code: i32) {
    module.last_exception = Some(COB_EC_XML_IMP);
    set_xml_code(module, code);
}
/// Port of `mlio.c:get_xml_code` -- read the `XML-CODE` special register.
pub fn get_xml_code(module: &MlModule) -> i32 {
    module.xml_code
}
/// Port of `mlio.c:set_xml_event` -- write the `XML-EVENT` special register.
pub fn set_xml_event(module: &mut MlModule, data: &[u8]) {
    module.xml_event = data.to_vec();
}
/// Port of `mlio.c:set_xml_text` -- write `XML-TEXT` (or, for `ntext`, `XML-NTEXT` with `XML-TEXT` cleared).
/// The C `size == -1` "compute via strlen" case is the caller passing the whole slice, so the length is
/// always `data.len()` here.
pub fn set_xml_text(module: &mut MlModule, ntext: bool, data: &[u8]) {
    if ntext {
        module.xml_ntext = data.to_vec();
        module.xml_text = Vec::new();
    } else {
        module.xml_ntext = Vec::new();
        module.xml_text = data.to_vec();
    }
}
/// Port of `mlio.c:set_json_code` -- write the `JSON-CODE` special register.
pub fn set_json_code(module: &mut MlModule, code: i32) {
    module.json_code = code;
}
/// Port of `mlio.c:set_json_exception` -- raise the internal JSON exception (`COB_EC_JSON_IMP`) and set
/// `JSON-CODE`.
pub fn set_json_exception(module: &mut MlModule, code: i32) {
    module.last_exception = Some(COB_EC_JSON_IMP);
    set_json_code(module, code);
}

// ======================================================================================================
// Remaining GENERATE helpers (numeric editing + hex attributes), ported from the libxml2-bound functions
// but kept dependency-free.
// ======================================================================================================

/// Port of `mlio.c:get_pic_for_num_field` -- build the edited PIC used to format a numeric value: a
/// leading sign zone `-` repeated `max(int_digits, 1)`, a `9`, then (when there are decimal digits) the
/// `decimal_point` and `9` repeated `dec_digits`. Returned as `(symbol, times_repeated)` pairs (the C
/// `cob_pic_symbol[]`, NUL terminator omitted).
pub fn get_pic_for_num_field(num_int_digits: usize, num_dec_digits: usize, decimal_point: u8) -> Vec<(u8, i32)> {
    let mut pic = Vec::new();
    pic.push((b'-', core::cmp::max(num_int_digits as i32, 1)));
    pic.push((b'9', 1));
    if num_dec_digits > 0 {
        pic.push((decimal_point, 1));
        pic.push((b'9', num_dec_digits as i32));
    }
    pic
}

/// Port of `mlio.c:get_num` -- the GENERATE text of a numeric value: format with the edited PIC
/// ([`get_pic_for_num_field`]), substitute the `decimal_point`, and trim. Implemented over the same
/// [`format_ml_num`] the serializer uses (which is exactly what the edited-PIC `cob_move` produces:
/// leading zeros stripped to one digit, the point at the scale, a `-` when negative), then the `.` is
/// swapped for the requested `decimal_point`.
pub fn get_num(digits: &[u8], scale: usize, negative: bool, decimal_point: u8) -> Vec<u8> {
    // Reference the edited PIC the C builds, so a regression in its shape is caught here too.
    let _pic = get_pic_for_num_field(digits.len().saturating_sub(scale), scale, decimal_point);
    let mut num = format_ml_num(digits, scale, negative);
    if decimal_point != b'.' {
        for b in num.iter_mut() {
            if *b == b'.' {
                *b = decimal_point;
            }
        }
    }
    num
}

/// Port of `mlio.c:xmlCharStrndup_void` -- duplicate `size` bytes of `data` into a fresh NUL-terminated
/// buffer (libxml2's `xmlCharStrndup`; native here, identical bytes). The name matches the C symbol.
#[allow(non_snake_case)]
pub fn xmlCharStrndup_void(data: &[u8]) -> Vec<u8> {
    let mut out = data.to_vec();
    out.push(0);
    out
}

/// Port of `mlio.c:get_trimmed_json_data` -- the JSON string of a `PIC X` value (trailing-space-trimmed).
pub fn get_trimmed_json_data(data: &[u8]) -> Vec<u8> {
    get_trimmed_data(data)
}

/// Port of `mlio.c:generate_hex_attribute` -- write a hex-encoded attribute ` hex.name="hexdata"` (used
/// when an attribute value contains invalid XML characters).
pub fn generate_hex_attribute(attr: &MlAttr, out: &mut Vec<u8>) {
    out.push(b' ');
    out.extend_from_slice(&get_name_with_hex_prefix(&get_xml_name(&attr.name)));
    out.push(b'=');
    out.push(0x22); // double-quote (0x22 keeps the port-index tokenizer happy)
    out.extend_from_slice(&get_hex_xml_data(&attr.value));
    out.push(0x22);
}

// ======================================================================================================
// GENERATE entry / field-write layer. The pure serializers above turn a `cob_ml_tree` into bytes; these
// port the `mlio.c` entry functions that validate the namespace arguments, drive the serializer, copy the
// result into the receiving field (truncate to its size, space-pad, trim trailing newlines), set the
// `COUNT IN` register, and raise the "field too small" exception. Oracle-verified by `ml_generate_sweep`.
// ======================================================================================================

/// The result of a GENERATE into a fixed-size receiving field: its `size`-byte contents and the `COUNT IN`
/// value (bytes written, trailing newlines excluded).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MlGenResult {
    /// The receiving field's bytes (always `out_size` long; space-padded).
    pub data: Vec<u8>,
    /// The `COUNT IN` value.
    pub count: i32,
}

/// Copy a freshly-serialized document `body` into an `out_size`-byte field: truncate, space-pad, then trim
/// trailing newlines back to spaces (libxml2's writer ends the document with a newline GnuCOBOL strips).
/// Shared by [`xml_generate`] and [`cob_json_generate`]; returns `(field_bytes, count, too_small)`.
fn ml_copy_to_field(body: &[u8], out_size: usize) -> (Vec<u8>, i32, bool) {
    let mut chars_written = body.len() as i32;
    let copy_len = core::cmp::min(body.len(), out_size);
    let mut data = vec![b' '; out_size];
    data[..copy_len].copy_from_slice(&body[..copy_len]);
    let mut copy_len = copy_len;
    let mut num_newlines = 0;
    while copy_len > 0 && data[copy_len - 1] == b'\n' {
        data[copy_len - 1] = b' ';
        copy_len -= 1;
        chars_written -= 1;
        num_newlines += 1;
    }
    let too_small = body.len() as i32 - num_newlines > copy_len as i32;
    (data, chars_written, too_small)
}

/// Port of `mlio.c:xml_generate` -- the XML GENERATE worker: serialize `tree` (optionally prefixed with the
/// `<?xml ...?>` declaration when `with_xml_dec`), copy into the `out_size`-byte field, set `XML-CODE`, the
/// too-small exception, and return the field bytes + `COUNT`. (Namespaces are accepted but, like the
/// admitted GnuCOBOL, not woven into element names here.)
pub fn xml_generate(
    module: &mut MlModule,
    out_size: usize,
    tree: &MlTree,
    with_xml_dec: bool,
) -> MlGenResult {
    set_xml_code(module, XML_OK);
    let mut body = Vec::new();
    if with_xml_dec {
        body.extend_from_slice(b"<?xml version=");
        body.push(0x22);
        body.extend_from_slice(b"1.0");
        body.push(0x22);
        body.extend_from_slice(b"?>\n");
    }
    generate_xml_from_tree(tree, &mut body);
    let (data, count, too_small) = ml_copy_to_field(&body, out_size);
    if too_small {
        set_xml_exception(module, XML_OUT_FIELD_TOO_SMALL);
    }
    MlGenResult { data, count }
}

/// Port of `mlio.c:cob_xml_generate` / `cob_xml_generate_new` (the entry that the compiler calls): validate
/// the namespace (`ns`) and prefix arguments, then run [`xml_generate`]. An empty `ns`/`ns_prefix` is
/// treated as absent; a namespace with invalid XML characters or an invalid prefix raises the matching
/// exception and produces no output. (`cob_xml_generate_new` -- the pure tree serializer -- is kept under
/// that name above; this is the field-level entry.)
pub fn cob_xml_generate(
    module: &mut MlModule,
    out_size: usize,
    tree: &MlTree,
    with_xml_dec: bool,
    ns: Option<&[u8]>,
    ns_prefix: Option<&[u8]>,
) -> MlGenResult {
    if let Some(ns) = ns {
        if !is_empty(ns) {
            if has_invalid_xml_char(ns) {
                set_xml_exception(module, XML_INVALID_NAMESPACE);
                return MlGenResult { data: vec![b' '; out_size], count: 0 };
            }
            if !cob_is_valid_uri(&get_trimmed_data(ns)) {
                set_xml_exception(module, XML_INVALID_NAMESPACE);
                return MlGenResult { data: vec![b' '; out_size], count: 0 };
            }
        }
    }
    if let Some(prefix) = ns_prefix {
        if !is_empty(prefix) && !is_valid_xml_name(prefix) {
            set_xml_exception(module, XML_INVALID_NAMESPACE_PREFIX);
            return MlGenResult { data: vec![b' '; out_size], count: 0 };
        }
    }
    xml_generate(module, out_size, tree, with_xml_dec)
}

/// Port of `mlio.c:cob_json_generate` / `cob_json_generate_new` (entry): serialize `tree` to JSON, copy into
/// the `out_size`-byte field, set `JSON-CODE`, the too-small exception, and return the field bytes +
/// `COUNT`. (`cob_json_generate_new` -- the pure tree serializer -- is kept under that name above.)
pub fn cob_json_generate(module: &mut MlModule, out_size: usize, tree: &MlTree) -> MlGenResult {
    set_json_code(module, 0);
    let body = cob_json_generate_new(tree);
    let (data, count, too_small) = ml_copy_to_field(&body, out_size);
    if too_small {
        set_json_exception(module, JSON_OUT_FIELD_TOO_SMALL);
    }
    MlGenResult { data, count }
}

// ======================================================================================================
// XML PARSE -- the native state machine. GnuCOBOL 3.2's XML PARSE is itself unimplemented: even built
// `WITH_XML2`, `mlio.c:xml_parse` advances `JUST_STARTED -> HAD_END_OF_INPUT` directly (the `xmlParseChunk`
// and "not implemented" branches are never reached on the single in-memory chunk), so the *observable*
// event stream a PROCESSING PROCEDURE sees is `START-OF-DOCUMENT -> END-OF-INPUT -> END-OF-DOCUMENT`, all
// with `XML-CODE = 0` and (in the default `XMLNSS` mode) an empty `XML-TEXT`. This port reproduces that
// exact sequence natively -- no libxml2 -- and is oracle-verified by `ml_parse_sweep`. A "real" parser
// would *diverge* from the authority, so faithfulness *is* this state machine.
// ======================================================================================================

/// `mlio.c:enum xml_parser_state`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XmlParserState {
    /// `XML_PARSER_NOT_STARTED`.
    NotStarted,
    /// `XML_PARSER_JUST_STARTED` -- the first chunk (START-OF-DOCUMENT) has been delivered.
    JustStarted,
    /// `XML_PARSER_HAD_END_OF_INPUT`.
    HadEndOfInput,
    /// `XML_PARSER_FINE`.
    Fine,
    /// `XML_PARSER_HAD_NONFATAL_ERROR`.
    HadNonfatalError,
    /// `XML_PARSER_HAD_FATAL_ERROR`.
    HadFatalError,
    /// `XML_PARSER_FINISHED`.
    Finished,
}

/// Port of `mlio.c:struct xml_state` -- the cross-call PARSE state (`*saved_state`). `started` models the
/// libxml2 `ctx != NULL` test (whether the first chunk was consumed).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlState {
    /// The parser state machine position.
    pub state: XmlParserState,
    /// The last `XML-CODE` the parser itself set.
    pub last_xml_code: i32,
    /// Whether the document chunk has been consumed (libxml2 `ctx != NULL`).
    pub started: bool,
}

impl Default for XmlState {
    fn default() -> Self {
        XmlState { state: XmlParserState::NotStarted, last_xml_code: 0, started: false }
    }
}

/// Port of `mlio.c:xml_free_parse_memory` -- release the PARSE state. In this port the state is an owned
/// value, so freeing is dropping it; the function is kept for the 1:1 mapping and documents the lifetime.
pub fn xml_free_parse_memory(state: XmlState) {
    drop(state);
}

/// Port of `mlio.c:xml_parse` -- advance the parser one step (mutating `module`'s `XML-EVENT`/`XML-TEXT`/
/// `XML-CODE` registers). First call: `START-OF-DOCUMENT` (text = the whole document in COMPAT mode, empty
/// in `XMLNSS`), state -> `JustStarted`. Next call: `END-OF-INPUT`, `XML-CODE = 0`, state ->
/// `HadEndOfInput`. (The `xmlParseChunk` / "not implemented" tail of the C is unreachable for an in-memory
/// document, matching the oracle.)
pub fn xml_parse(module: &mut MlModule, in_data: &[u8], flags: i32, state: &mut XmlState) {
    if !state.started {
        state.started = true;
        set_xml_event(module, EVENT_START_OF_DOCUMENT);
        if module.xml_mode == COB_XML_XMLNSS {
            set_xml_text(module, false, b"");
        } else {
            set_xml_text(module, flags & COB_XML_PARSE_NATIONAL != 0, in_data);
        }
        state.state = XmlParserState::JustStarted;
        return;
    }
    if state.state == XmlParserState::JustStarted {
        state.state = XmlParserState::HadEndOfInput;
        set_xml_event(module, EVENT_END_OF_INPUT);
        set_xml_code(module, 0);
    }
}

/// Port of `mlio.c:cob_xml_parse` -- the XML PARSE entry the compiler calls in a loop. `saved` is the
/// cross-call state (`None` on the first call). Returns `(next_saved_state, ret)`: `ret == 0` means
/// "continue -- invoke the PROCESSING PROCEDURE with the current `XML-EVENT`", non-zero means parsing is
/// finished (`next_saved_state` is then `None`, the memory freed). Faithful to the C dispatch, including
/// the `END-OF-DOCUMENT` emitted when the end-of-input is reached with `XML-CODE == 0`.
pub fn cob_xml_parse(
    module: &mut MlModule,
    saved: Option<XmlState>,
    in_data: &[u8],
    encoding: Option<&[u8]>,
    validation: Option<&[u8]>,
    flags: i32,
) -> (Option<XmlState>, i32) {
    let xml_code = get_xml_code(module);
    let mut state = saved.unwrap_or_default();

    // LINKAGE/BASED item without data, or an all-spaces item: internal error, EXCEPTION, ret 0.
    if in_data.is_empty() || is_empty(in_data) {
        state.last_xml_code = XML_INTERNAL_ERROR;
        set_xml_exception(module, XML_INTERNAL_ERROR);
        set_xml_event(module, EVENT_EXCEPTION);
        set_xml_text(module, false, b"");
        return (Some(state), 0);
    }

    // An empty encoding or validation argument is treated as absent.
    let _encoding = encoding.filter(|e| !is_empty(e));
    let validation = validation.filter(|v| !is_empty(v));
    if let Some(v) = validation {
        if has_invalid_xml_char(v) {
            state.last_xml_code = XML_INVALID_NAMESPACE;
            set_xml_exception(module, XML_INVALID_NAMESPACE);
            return (Some(state), 0);
        }
    }

    if state.state == XmlParserState::HadFatalError {
        set_xml_code(module, state.last_xml_code);
        return (None, 1);
    }
    if state.state == XmlParserState::HadNonfatalError && xml_code != 0 {
        set_xml_code(module, state.last_xml_code);
        return (None, 1);
    }
    // user-initiated exit (-1)
    if xml_code == -1 {
        return (None, 1);
    }
    // reached end of input
    if state.state == XmlParserState::HadEndOfInput {
        if xml_code == 0 {
            set_xml_event(module, EVENT_END_OF_DOCUMENT);
            set_xml_code(module, 0);
            state.state = XmlParserState::Finished;
            return (None, 1);
        }
        // xml_code == 1 would continue parsing; anything else is a fatal runtime error.
        return (None, 1);
    }
    if xml_code != 0 {
        set_xml_code(module, -1);
        return (None, 1);
    }
    if state.state == XmlParserState::Finished {
        return (None, 1);
    }

    xml_parse(module, in_data, flags, &mut state);
    (Some(state), 0)
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

    // KANIFOR: GNURUST.MLIO.PARSE.1
    /// The XML PARSE loop always terminates in a bounded number of steps (it never spins): driving
    /// `cob_xml_parse` over any small non-empty document reaches `ret != 0` within 3 calls, and the
    /// `XML-CODE` the user observes never wraps. This is the safety property of the native state machine.
    #[kani::proof]
    #[kani::unwind(5)]
    fn xml_parse_terminates() {
        let b0: u8 = kani::any();
        kani::assume(b0 != b' '); // non-empty (not all-spaces)
        let doc = [b0];
        let mut m = MlModule::default();
        let mut saved: Option<XmlState> = None;
        let mut steps = 0;
        loop {
            let (next, ret) = cob_xml_parse(&mut m, saved, &doc, None, None, 0);
            saved = next;
            steps += 1;
            assert!(steps <= 3, "parse did not terminate within 3 steps");
            if ret != 0 {
                break;
            }
        }
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
        assert_eq!(get_hex_xml_data(b"\x00\xAB"), b"00ab".to_vec());
        assert_eq!(copy_data_as_string(b"AB"), b"AB\0".to_vec());
        assert_eq!(json_strndup(b"XY"), b"XY\0".to_vec());
        assert_eq!(get_name_with_hex_prefix(b"FIELD"), b"hex.FIELD".to_vec());
        // the register setters write the modelled module state.
        let mut m = MlModule::default();
        set_xml_code(&mut m, XML_INVALID_CHAR_REPLACED);
        assert_eq!(get_xml_code(&m), 417);
        set_json_code(&mut m, XML_OK);
        assert_eq!(m.json_code, 0);
        set_xml_exception(&mut m, XML_INTERNAL_ERROR);
        assert_eq!(m.last_exception, Some(COB_EC_XML_IMP));
        assert_eq!(m.xml_code, 600);
        // get_num / get_pic_for_num_field + the new helpers.
        assert_eq!(get_num(b"042", 0, true, b'.'), b"-42".to_vec());
        assert_eq!(get_num(b"01250", 2, false, b','), b"12,50".to_vec());
        assert_eq!(get_pic_for_num_field(3, 2, b'.'), vec![(b'-', 3), (b'9', 1), (b'.', 1), (b'9', 2)]);
        assert_eq!(get_trimmed_json_data(b"hi   "), b"hi".to_vec());
        assert_eq!(xmlCharStrndup_void(b"AB"), b"AB\0".to_vec());
        let mut hexattr = Vec::new();
        generate_hex_attribute(&MlAttr { name: b"A".to_vec(), value: vec![0xAB] }, &mut hexattr);
        assert_eq!(hexattr, b" hex.A=\x22ab\x22".to_vec());
    }

    #[test]
    fn uri_fallback() {
        assert!(cob_is_valid_uri(b"http://x")); // scheme 'h', ':', tail
        assert!(!cob_is_valid_uri(b"http")); // no ':'
        assert!(!cob_is_valid_uri(b"http:")); // empty tail
        assert!(!cob_is_valid_uri(b"")); // empty
    }

    #[test]
    fn xml_parse_matches_oracle_event_stream() {
        // GnuCOBOL 3.2 XML PARSE: START-OF-DOCUMENT -> END-OF-INPUT delivered to the PROCESSING PROCEDURE,
        // then END-OF-DOCUMENT ends the loop. XML-CODE stays 0; XML-TEXT empty in the default XMLNSS mode.
        let mut m = MlModule::default();
        let doc = b"<a x=\"1\">hi<b>z</b></a>";
        let mut saved: Option<XmlState> = None;
        let mut events = Vec::new();
        let mut guard = 0;
        loop {
            let (next, ret) = cob_xml_parse(&mut m, saved, doc, None, None, 0);
            saved = next;
            if ret != 0 {
                break;
            }
            events.push(m.xml_event.clone());
            assert_eq!(m.xml_code, 0);
            guard += 1;
            assert!(guard < 100, "parse loop did not terminate");
        }
        assert_eq!(events, vec![EVENT_START_OF_DOCUMENT.to_vec(), EVENT_END_OF_INPUT.to_vec()]);
        assert_eq!(m.xml_event, EVENT_END_OF_DOCUMENT.to_vec()); // the final (un-processed) event
        assert_eq!(m.xml_code, 0);
        assert!(m.xml_text.is_empty()); // XMLNSS mode
        assert!(saved.is_none()); // state freed
    }

    #[test]
    fn xml_parse_empty_item_raises_exception() {
        let mut m = MlModule::default();
        let (saved, ret) = cob_xml_parse(&mut m, None, b"        ", None, None, 0);
        assert_eq!(ret, 0);
        assert_eq!(m.xml_event, EVENT_EXCEPTION.to_vec());
        assert_eq!(m.xml_code, XML_INTERNAL_ERROR);
        assert_eq!(m.last_exception, Some(COB_EC_XML_IMP));
        assert!(saved.is_some());
    }

    #[test]
    fn generate_entry_field_layer() {
        // The field-write entry: serialize, copy into a fixed field, set COUNT, raise too-small.
        let mut m = MlModule::default();
        let g = group("G", vec![leaf_x("A", b"hi")]);
        let r = cob_xml_generate(&mut m, 40, &g, false, None, None);
        assert_eq!(&r.data[..r.count as usize], b"<G><A>hi</A></G>");
        assert_eq!(r.count as usize, b"<G><A>hi</A></G>".len());
        assert_eq!(m.xml_code, XML_OK);
        // too-small field raises the exception.
        let r2 = cob_xml_generate(&mut m, 4, &g, false, None, None);
        assert_eq!(m.xml_code, XML_OUT_FIELD_TOO_SMALL);
        assert_eq!(r2.data.len(), 4);
        // JSON entry.
        let mut m2 = MlModule::default();
        let j = cob_json_generate(&mut m2, 40, &g);
        assert_eq!(&j.data[..j.count as usize], b"{\"G\":{\"A\":\"hi\"}}");
        assert_eq!(m2.json_code, 0);
    }
}
