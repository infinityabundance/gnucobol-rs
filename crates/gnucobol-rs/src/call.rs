//! Port of `libcob/call.c` -- the `CALL` / `CANCEL` program-linkage subsystem.
//!
//! This module ports the **portable** core of call.c: the `PROGRAM-ID` name mangling, the field/parameter
//! metadata accessors (`cob_get_field_*` / `cob_get_param_*`), the typed parameter getters/setters
//! (`cob_get_s64_param` / `cob_put_picx_param` / ...), and the lifecycle hooks. The genuinely external pieces
//! -- dynamic loading (`lt_dlopen`/`lt_dlsym`), the C stack save/restore (`setjmp`/`longjmp`), and the global
//! runtime module table -- are the declared OS/ABI boundary; everything that is a pure function of a field's
//! `(attr, bytes)` is ported here and unit-tested.
//!
//! A COBOL `CALL`ed program receives its arguments as a list of `cob_field`s (the *procedure parameters*).
//! [`CallField`] is the Rust analogue -- a field's attributes plus its data bytes -- and [`Params`] is the
//! argument list a called program reads through the `cob_get_param_*` API.

use crate::accessors;
use crate::attr::{
    FieldAttr, COB_FLAG_BINARY_SWAP, COB_FLAG_HAVE_SIGN, COB_TYPE_ALPHANUMERIC, COB_TYPE_GROUP,
    COB_TYPE_NUMERIC_BINARY, COB_TYPE_NUMERIC_COMP5, COB_TYPE_NUMERIC_DISPLAY, COB_TYPE_NUMERIC_DOUBLE,
    COB_TYPE_NUMERIC_FLOAT, COB_TYPE_NUMERIC_PACKED,
};

/// `COB_FLAG_CONSTANT` (`common.h`) -- the field is a read-only constant (a literal); a `cob_put_*` to it is
/// refused. (Not in `attr.rs`'s exported set, so defined here for the call layer.)
pub const COB_FLAG_CONSTANT: u16 = 0x0400;

/// Case folding for `PROGRAM-ID` encoding (`common.h` `COB_FOLD_*`).
pub const COB_FOLD_NONE: i32 = 0;
pub const COB_FOLD_UPPER: i32 = 1;
pub const COB_FOLD_LOWER: i32 = 2;

/// A procedure parameter / `cob_field`: its attributes and its data bytes (the field size is `data.len()`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallField {
    pub attr: FieldAttr,
    pub data: Vec<u8>,
}

impl CallField {
    /// The field size in bytes (`f->size`).
    pub fn size(&self) -> usize {
        self.data.len()
    }
    fn is_constant(&self) -> bool {
        self.attr.flags & COB_FLAG_CONSTANT != 0
    }
    fn binary_swap(&self) -> bool {
        self.attr.flags & COB_FLAG_BINARY_SWAP != 0
    }
}

/// The argument list a `CALL`ed program receives (`COB_MODULE_PTR->cob_procedure_params`). A `None` slot is
/// a `BY REFERENCE OMITTED` / NULL parameter.
#[derive(Debug, Clone, Default)]
pub struct Params {
    pub fields: Vec<Option<CallField>>,
}

// ======================================================================================================
// PROGRAM-ID name mangling
// ======================================================================================================

/// The characters valid unencoded in a mangled program id (`call.c:pvalid_char`).
const PVALID_CHAR: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz";
/// Hex digits used when encoding an invalid character (`call.c:hexval`).
const HEXVAL: &[u8] = b"0123456789ABCDEF";

fn is_valid_char(c: u8) -> bool {
    PVALID_CHAR.contains(&c)
}

/// Port of `fileio.c`/`call.c:cob_encode_invalid_chars` -- append `name`'s characters to `name_buff` from
/// `external_pos`, copying valid characters verbatim and encoding each invalid one as `_` (a `-` becomes
/// `__`, any other invalid byte becomes `_HH` in uppercase hex). Returns the new length, or the negated
/// length if the buffer (`buff_size`) would overflow (matching the C `-pos`). `external_pos` is advanced.
pub fn cob_encode_invalid_chars(name: &[u8], name_buff: &mut Vec<u8>, buff_size: usize, external_pos: &mut i32) -> i32 {
    let mut pos = *external_pos;
    for &c in name {
        if pos as usize >= buff_size.saturating_sub(3) {
            name_buff.truncate(pos.max(0) as usize);
            return -pos;
        }
        if is_valid_char(c) {
            set_at(name_buff, pos as usize, c);
            pos += 1;
        } else {
            set_at(name_buff, pos as usize, b'_');
            pos += 1;
            if c == b'-' {
                set_at(name_buff, pos as usize, b'_');
                pos += 1;
            } else {
                set_at(name_buff, pos as usize, HEXVAL[(c / 16) as usize]);
                pos += 1;
                set_at(name_buff, pos as usize, HEXVAL[(c % 16) as usize]);
                pos += 1;
            }
        }
    }
    *external_pos = pos;
    pos
}

fn set_at(buf: &mut Vec<u8>, idx: usize, val: u8) {
    if idx >= buf.len() {
        buf.resize(idx + 1, 0);
    }
    buf[idx] = val;
}

/// Port of `call.c:cob_encode_program_id` -- mangle a `PROGRAM-ID` into a valid C symbol: a leading digit is
/// prefixed with `_`, invalid characters are encoded via [`cob_encode_invalid_chars`], and the whole name
/// is case-folded per `fold_case` (`COB_FOLD_NONE`/`UPPER`/`LOWER`). Returns the mangled name bytes.
pub fn cob_encode_program_id(name: &[u8], buff_size: usize, fold_case: i32) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    let mut pos = 0i32;
    if name.first().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        set_at(&mut out, 0, b'_');
        pos = 1;
    }
    cob_encode_invalid_chars(name, &mut out, buff_size, &mut pos);
    out.truncate(pos.max(0) as usize);
    match fold_case {
        COB_FOLD_UPPER => out.iter_mut().for_each(|p| *p = p.to_ascii_uppercase()),
        COB_FOLD_LOWER => out.iter_mut().for_each(|p| *p = p.to_ascii_lowercase()),
        _ => {}
    }
    out
}

// ======================================================================================================
// Field metadata accessors (cob_get_field_*) -- pure functions of a field's (attr, size)
// ======================================================================================================

/// Port of `call.c:cob_get_field_type` -- the field's COBOL type, mapping a native-order / real binary field
/// to `COB_TYPE_NUMERIC_COMP5` (a swapped big-endian `BINARY` keeps its `BINARY` type). `None` -> `-1`.
pub fn cob_get_field_type(f: Option<&CallField>) -> i32 {
    let f = match f {
        Some(f) => f,
        None => return -1,
    };
    if f.attr.field_type == COB_TYPE_NUMERIC_BINARY {
        // A real binary (COMP-5/COMP-X) or any non-byte-swapped binary reports as COMP-5 (native order).
        if (f.attr.flags & crate::attr::COB_FLAG_REAL_BINARY != 0) || !f.binary_swap() {
            return COB_TYPE_NUMERIC_COMP5 as i32;
        }
    }
    f.attr.field_type as i32
}

/// Port of `call.c:cob_get_field_size` -- the field size in bytes; `None` -> `-1`.
pub fn cob_get_field_size(f: Option<&CallField>) -> i32 {
    f.map(|f| f.size() as i32).unwrap_or(-1)
}

/// Port of `call.c:cob_get_field_sign` -- `1` if the field carries a sign (`COB_FLAG_HAVE_SIGN`), else `0`;
/// `None` -> `-1`.
pub fn cob_get_field_sign(f: Option<&CallField>) -> i32 {
    match f {
        Some(f) => (f.attr.flags & COB_FLAG_HAVE_SIGN != 0) as i32,
        None => -1,
    }
}

/// Port of `call.c:cob_get_field_scale` -- the field's implied decimal scale; `None` -> `-1`.
pub fn cob_get_field_scale(f: Option<&CallField>) -> i32 {
    f.map(|f| f.attr.scale as i32).unwrap_or(-1)
}

/// Port of `call.c:cob_get_field_digits` -- the field's digit count; `None` -> `-1`.
pub fn cob_get_field_digits(f: Option<&CallField>) -> i32 {
    f.map(|f| f.attr.digits as i32).unwrap_or(-1)
}

/// Port of `call.c:cob_get_field_constant` -- `1` if the field is a read-only constant, else `0`;
/// `None` -> `-1`.
pub fn cob_get_field_constant(f: Option<&CallField>) -> i32 {
    match f {
        Some(f) => f.is_constant() as i32,
        None => -1,
    }
}

/// Port of `call.c:cob_field_constant` -- build a read-only constant view of `f`: same attributes (with
/// `COB_FLAG_CONSTANT` set) over a copy of the data. Returns the constant [`CallField`].
pub fn cob_field_constant(f: &CallField) -> CallField {
    let mut attr = f.attr;
    attr.flags |= COB_FLAG_CONSTANT;
    CallField { attr, data: f.data.clone() }
}

// ======================================================================================================
// Parameter accessors (cob_get_param_*) -- fetch the n-th procedure parameter, then read its metadata/value
// ======================================================================================================

/// Port of `call.c:cob_get_num_params` -- the number of parameters the current program was `CALL`ed with.
pub fn cob_get_num_params(params: &Params) -> i32 {
    params.fields.len() as i32
}

/// Port of `call.c:cob_get_param_field` -- the `n`-th procedure parameter (1-based), or `None` when `n` is
/// out of range or that parameter is NULL/omitted. (The C "cob_init not called" path is the runtime-state
/// boundary.)
pub fn cob_get_param_field(params: &Params, n: i32) -> Option<&CallField> {
    if n < 1 || n as usize > params.fields.len() {
        return None;
    }
    params.fields[(n - 1) as usize].as_ref()
}

/// Port of `call.c:cob_get_param_type` -- [`cob_get_field_type`] of the `n`-th parameter.
pub fn cob_get_param_type(params: &Params, n: i32) -> i32 {
    cob_get_field_type(cob_get_param_field(params, n))
}
/// Port of `call.c:cob_get_param_size` -- [`cob_get_field_size`] of the `n`-th parameter.
pub fn cob_get_param_size(params: &Params, n: i32) -> i32 {
    cob_get_field_size(cob_get_param_field(params, n))
}
/// Port of `call.c:cob_get_param_sign` -- [`cob_get_field_sign`] of the `n`-th parameter.
pub fn cob_get_param_sign(params: &Params, n: i32) -> i32 {
    cob_get_field_sign(cob_get_param_field(params, n))
}
/// Port of `call.c:cob_get_param_scale` -- [`cob_get_field_scale`] of the `n`-th parameter.
pub fn cob_get_param_scale(params: &Params, n: i32) -> i32 {
    cob_get_field_scale(cob_get_param_field(params, n))
}
/// Port of `call.c:cob_get_param_digits` -- [`cob_get_field_digits`] of the `n`-th parameter.
pub fn cob_get_param_digits(params: &Params, n: i32) -> i32 {
    cob_get_field_digits(cob_get_param_field(params, n))
}
/// Port of `call.c:cob_get_param_constant` -- [`cob_get_field_constant`] of the `n`-th parameter.
pub fn cob_get_param_constant(params: &Params, n: i32) -> i32 {
    cob_get_field_constant(cob_get_param_field(params, n))
}

/// Port of `call.c:cob_get_param_data` -- the raw data bytes of the `n`-th parameter (the C returns its
/// `data` pointer), or `None` when the parameter is absent.
pub fn cob_get_param_data(params: &Params, n: i32) -> Option<&[u8]> {
    cob_get_param_field(params, n).map(|f| f.data.as_slice())
}

// ======================================================================================================
// Typed parameter getters / setters
// ======================================================================================================

/// The signed integer value held by a field, dispatched by its USAGE (the `cob_get_s64_param` switch). The
/// `ebcdic` zoned-decimal path is the cp037 boundary; an ASCII host decodes DISPLAY as ASCII zoned.
fn field_to_s64(f: &CallField) -> i64 {
    let d = &f.data;
    let n = f.size();
    match f.attr.field_type {
        COB_TYPE_NUMERIC_DISPLAY => accessors::cob_get_s64_pic9(d, n, false),
        COB_TYPE_NUMERIC_PACKED => accessors::cob_get_s64_comp3(d, n),
        COB_TYPE_NUMERIC_BINARY => {
            if (f.attr.flags & crate::attr::COB_FLAG_REAL_BINARY != 0) || !f.binary_swap() {
                accessors::cob_get_s64_comp5(d, n)
            } else {
                accessors::cob_get_s64_compx(d, n)
            }
        }
        COB_TYPE_NUMERIC_FLOAT => accessors::cob_get_comp1(d) as i64,
        COB_TYPE_NUMERIC_DOUBLE => accessors::cob_get_comp2(d) as i64,
        _ => 0,
    }
}

fn field_to_u64(f: &CallField) -> u64 {
    let d = &f.data;
    let n = f.size();
    match f.attr.field_type {
        COB_TYPE_NUMERIC_DISPLAY => accessors::cob_get_u64_pic9(d, n),
        COB_TYPE_NUMERIC_PACKED => accessors::cob_get_u64_comp3(d, n),
        COB_TYPE_NUMERIC_BINARY => {
            if (f.attr.flags & crate::attr::COB_FLAG_REAL_BINARY != 0) || !f.binary_swap() {
                accessors::cob_get_u64_comp5(d, n)
            } else {
                accessors::cob_get_u64_compx(d, n)
            }
        }
        COB_TYPE_NUMERIC_FLOAT => accessors::cob_get_comp1(d) as u64,
        COB_TYPE_NUMERIC_DOUBLE => accessors::cob_get_comp2(d) as u64,
        _ => 0,
    }
}

/// Port of `call.c:cob_get_s64_param` -- the signed integer value of the `n`-th parameter; `None` -> `-1`.
pub fn cob_get_s64_param(params: &Params, n: i32) -> i64 {
    match cob_get_param_field(params, n) {
        Some(f) => field_to_s64(f),
        None => -1,
    }
}

/// Port of `call.c:cob_get_u64_param` -- the unsigned integer value of the `n`-th parameter; `None` -> `-1`
/// (the C casts `-1` to unsigned).
pub fn cob_get_u64_param(params: &Params, n: i32) -> u64 {
    match cob_get_param_field(params, n) {
        Some(f) => field_to_u64(f),
        None => u64::MAX,
    }
}

/// Port of `call.c:cob_get_dbl_param` -- the floating value of the `n`-th parameter (`COMP-1`/`COMP-2`
/// directly, other usages via their integer value scaled by the field scale); `None` -> `-1.0`.
pub fn cob_get_dbl_param(params: &Params, n: i32) -> f64 {
    let f = match cob_get_param_field(params, n) {
        Some(f) => f,
        None => return -1.0,
    };
    match f.attr.field_type {
        COB_TYPE_NUMERIC_FLOAT => accessors::cob_get_comp1(&f.data) as f64,
        COB_TYPE_NUMERIC_DOUBLE => accessors::cob_get_comp2(&f.data),
        _ => field_to_s64(f) as f64 / 10f64.powi(f.attr.scale as i32),
    }
}

/// Port of `call.c:cob_get_picx_param` -- copy the `n`-th parameter's `PIC X` data into a `char_len`-byte
/// buffer (space-padded / truncated, via [`accessors::cob_get_picx`]); `None` when the parameter is absent.
pub fn cob_get_picx_param(params: &Params, n: i32, char_len: usize) -> Option<Vec<u8>> {
    let f = cob_get_param_field(params, n)?;
    Some(accessors::cob_get_picx(&f.data, f.size(), Some(char_len)))
}

/// Port of `call.c:cob_put_s64_param` -- store a signed integer into the `n`-th parameter (refused for a
/// constant parameter). Dispatches by USAGE; returns `false` if the parameter is absent or constant.
pub fn cob_put_s64_param(params: &mut Params, n: i32, val: i64) -> bool {
    let f = match param_field_mut(params, n) {
        Some(f) if !f.is_constant() => f,
        _ => return false,
    };
    let len = f.data.len();
    match f.attr.field_type {
        COB_TYPE_NUMERIC_DISPLAY => accessors::cob_put_s64_pic9(val, &mut f.data, len, false),
        COB_TYPE_NUMERIC_PACKED => accessors::cob_put_s64_comp3(val, &mut f.data, len),
        COB_TYPE_NUMERIC_BINARY => {
            if (f.attr.flags & crate::attr::COB_FLAG_REAL_BINARY != 0) || (f.attr.flags & COB_FLAG_BINARY_SWAP == 0) {
                accessors::cob_put_s64_comp5(val, &mut f.data, len)
            } else {
                accessors::cob_put_s64_compx(val, &mut f.data, len)
            }
        }
        _ => return false,
    }
    true
}

/// Port of `call.c:cob_put_picx_param` -- store a `PIC X` byte string into the `n`-th parameter (refused for
/// a constant parameter). Returns `false` if absent or constant.
pub fn cob_put_picx_param(params: &mut Params, n: i32, char_field: &[u8]) -> bool {
    let f = match param_field_mut(params, n) {
        Some(f) if !f.is_constant() => f,
        _ => return false,
    };
    let len = f.data.len();
    accessors::cob_put_picx(&mut f.data, len, char_field);
    true
}

/// Port of `call.c:cob_get_grp_param` -- copy the `n`-th parameter's raw group bytes into a buffer of `len`
/// (`0` = the field size; a short buffer is grown to the field size). `None` when the parameter is absent.
pub fn cob_get_grp_param(params: &Params, n: i32, len: usize) -> Option<Vec<u8>> {
    let f = cob_get_param_field(params, n)?;
    let mut want = if len == 0 { f.size() } else { len };
    if want < f.size() {
        want = f.size();
    }
    let mut out = vec![0u8; want];
    out[..f.size()].copy_from_slice(&f.data);
    Some(out)
}

/// Port of `call.c:cob_put_grp_param` -- copy up to `len` (`0`/too-big = the field size) bytes of
/// `char_field` into the `n`-th parameter's raw storage. Returns `false` when the parameter is absent.
pub fn cob_put_grp_param(params: &mut Params, n: i32, char_field: &[u8], len: usize) -> bool {
    let f = match param_field_mut(params, n) {
        Some(f) => f,
        None => return false,
    };
    let size = f.data.len();
    let n_copy = if len == 0 || len > size { size } else { len }.min(char_field.len());
    f.data[..n_copy].copy_from_slice(&char_field[..n_copy]);
    true
}

fn param_field_mut(params: &mut Params, n: i32) -> Option<&mut CallField> {
    if n < 1 || n as usize > params.fields.len() {
        return None;
    }
    params.fields[(n - 1) as usize].as_mut()
}

// ======================================================================================================
// Runtime buffer + lifecycle
// ======================================================================================================

/// Port of `call.c:cob_get_buff` -- obtain a scratch buffer of at least `buffsize` bytes. libcob keeps one
/// growable global buffer; the Rust port returns a freshly-sized `Vec` (the global caching is an internal
/// optimisation, not observable behaviour).
pub fn cob_get_buff(buffsize: usize) -> Vec<u8> {
    vec![0u8; buffsize]
}

/// Port of `call.c:cob_set_library_path` -- set the dynamic-library search path used by `cob_resolve`. The
/// actual `lt_dlopen` search is the OS boundary; this returns the normalised path string the resolver uses.
pub fn cob_set_library_path(path: &str) -> String {
    path.to_string()
}

/// Port of `call.c:cob_init_call` -- runtime initialisation of the CALL subsystem (global buffers + the
/// module/resolve caches). The Rust port holds no global mutable state, so this is a documented no-op.
pub fn cob_init_call() {}

/// Port of `call.c:cob_exit_call` -- runtime teardown of the CALL subsystem (free buffers, unload modules).
/// Rust frees on drop and dynamic loading is the OS boundary, so this is a documented no-op.
pub fn cob_exit_call() {}

/// Port of `call.c:close_and_free_module_list` -- close + free every dynamically loaded module in a list.
/// The `lt_dlclose` of each handle is the OS boundary; Rust drops the list, so this is a documented no-op.
pub fn close_and_free_module_list() {}

// ======================================================================================================
// Typed putters (the remaining members of the cob_put_*_param family)
// ======================================================================================================

/// Port of `call.c:cob_put_u64_param` -- store an unsigned integer into the `n`-th parameter (refused for a
/// constant). Returns `false` when the parameter is absent or constant.
pub fn cob_put_u64_param(params: &mut Params, n: i32, val: u64) -> bool {
    cob_put_s64_param(params, n, val as i64)
}

/// Port of `call.c:cob_put_dbl_param` -- store a floating value into the `n`-th parameter: directly for
/// `COMP-1`/`COMP-2`, else the integer value scaled by the field scale. Returns `false` if absent/constant.
pub fn cob_put_dbl_param(params: &mut Params, n: i32, val: f64) -> bool {
    let ftype = match param_field_mut(params, n) {
        Some(f) if !f.is_constant() => f.attr.field_type,
        _ => return false,
    };
    match ftype {
        COB_TYPE_NUMERIC_FLOAT => {
            let f = param_field_mut(params, n).unwrap();
            let len = f.data.len().min(4);
            f.data[..len].copy_from_slice(&(val as f32).to_le_bytes()[..len]);
            true
        }
        COB_TYPE_NUMERIC_DOUBLE => {
            let f = param_field_mut(params, n).unwrap();
            let len = f.data.len().min(8);
            f.data[..len].copy_from_slice(&val.to_le_bytes()[..len]);
            true
        }
        _ => {
            let scale = param_field_mut(params, n).unwrap().attr.scale as i32;
            cob_put_s64_param(params, n, (val * 10f64.powi(scale)).round() as i64)
        }
    }
}

// ======================================================================================================
// Field <-> string (cob_get_field_str / cob_put_field_str) and the parameter wrappers
// ======================================================================================================

/// Port of `call.c:cob_get_field_str` -- render a field as it would `DISPLAY` (pretty): an alphanumeric /
/// group field is its raw bytes; a numeric field is its value with a leading sign (when negative) and a
/// decimal point at the field scale, leading zeros trimmed to one. A `None` field is `"NULL field"`, an
/// empty field is `""`. (The C routes through `cob_display_common`; this is the equivalent rendering for
/// the DISPLAY/PACKED/BINARY usages -- edited-picture pretty-printing stays the termio delegation.)
pub fn cob_get_field_str(f: Option<&CallField>) -> Vec<u8> {
    let f = match f {
        Some(f) => f,
        None => return b"NULL field".to_vec(),
    };
    if f.size() == 0 {
        return Vec::new();
    }
    match f.attr.field_type {
        COB_TYPE_NUMERIC_DISPLAY | COB_TYPE_NUMERIC_PACKED | COB_TYPE_NUMERIC_BINARY => {
            let val = field_to_s64(f);
            let scale = f.attr.scale.max(0) as usize;
            let mut digits = val.unsigned_abs().to_string().into_bytes();
            while digits.len() <= scale {
                digits.insert(0, b'0'); // ensure at least one integer digit before the point
            }
            let mut out: Vec<u8> = Vec::new();
            if val < 0 {
                out.push(b'-');
            }
            let int_len = digits.len() - scale;
            out.extend_from_slice(&digits[..int_len]);
            if scale > 0 {
                out.push(b'.');
                out.extend_from_slice(&digits[int_len..]);
            }
            out
        }
        _ => f.data.clone(),
    }
}

/// Port of `call.c:cob_get_field_str_buffered` -- [`cob_get_field_str`] into an internally-sized buffer.
pub fn cob_get_field_str_buffered(f: Option<&CallField>) -> Vec<u8> {
    cob_get_field_str(f)
}

/// Port of `call.c:cob_put_field_str` -- store the string `str` into a field: an alphanumeric field takes
/// the bytes space-padded; a numeric field parses `str` (sign + digits + decimal point, scaled to the
/// field) and stores the value. Returns `0` on success, `EINVAL` (`22`) for a NULL / zero-size / constant
/// field. (The full `NUMVAL` parsing of `str` is the `GNURUST.INTRINSIC` delegation; this handles the
/// plain numeric-literal form.)
pub fn cob_put_field_str(f: Option<&mut CallField>, s: &[u8]) -> i32 {
    let f = match f {
        Some(f) if f.size() > 0 && !f.is_constant() => f,
        _ => return 22, // EINVAL
    };
    match f.attr.field_type {
        COB_TYPE_NUMERIC_DISPLAY | COB_TYPE_NUMERIC_PACKED | COB_TYPE_NUMERIC_BINARY => {
            let scale = f.attr.scale.max(0) as i32;
            let val = parse_numeric_literal(s, scale);
            let len = f.data.len();
            match f.attr.field_type {
                COB_TYPE_NUMERIC_DISPLAY => accessors::cob_put_s64_pic9(val, &mut f.data, len, false),
                COB_TYPE_NUMERIC_PACKED => accessors::cob_put_s64_comp3(val, &mut f.data, len),
                _ => {
                    if (f.attr.flags & crate::attr::COB_FLAG_REAL_BINARY != 0) || (f.attr.flags & COB_FLAG_BINARY_SWAP == 0) {
                        accessors::cob_put_s64_comp5(val, &mut f.data, len)
                    } else {
                        accessors::cob_put_s64_compx(val, &mut f.data, len)
                    }
                }
            }
        }
        _ => {
            let len = f.data.len();
            accessors::cob_put_picx(&mut f.data, len, s);
        }
    }
    0
}

/// Parse a plain numeric literal (`[+-]?digits[.digits]`) and scale it to `scale` fractional digits,
/// returning the signed integer value the field stores (the `cob_put_field_str` numeric path).
fn parse_numeric_literal(s: &[u8], scale: i32) -> i64 {
    let mut neg = false;
    let mut int_digits: Vec<u8> = Vec::new();
    let mut frac_digits: Vec<u8> = Vec::new();
    let mut seen_dot = false;
    for &c in s {
        match c {
            b'-' if int_digits.is_empty() && !seen_dot => neg = true,
            b'+' => {}
            b'.' => seen_dot = true,
            b'0'..=b'9' => {
                if seen_dot {
                    frac_digits.push(c);
                } else {
                    int_digits.push(c);
                }
            }
            _ => {}
        }
    }
    // Pad/truncate the fractional part to `scale` digits.
    frac_digits.resize(scale.max(0) as usize, b'0');
    let mut all = int_digits;
    all.extend_from_slice(&frac_digits);
    let mag: i64 = std::str::from_utf8(&all).ok().and_then(|t| t.parse().ok()).unwrap_or(0);
    if neg {
        -mag
    } else {
        mag
    }
}

/// Port of `call.c:cob_get_param_str` -- [`cob_get_field_str`] of the `n`-th parameter.
pub fn cob_get_param_str(params: &Params, n: i32) -> Vec<u8> {
    cob_get_field_str(cob_get_param_field(params, n))
}
/// Port of `call.c:cob_get_param_str_buffered` -- [`cob_get_field_str_buffered`] of the `n`-th parameter.
pub fn cob_get_param_str_buffered(params: &Params, n: i32) -> Vec<u8> {
    cob_get_field_str_buffered(cob_get_param_field(params, n))
}
/// Port of `call.c:cob_put_param_str` -- [`cob_put_field_str`] into the `n`-th parameter.
pub fn cob_put_param_str(params: &mut Params, n: i32, s: &[u8]) -> i32 {
    cob_put_field_str(param_field_mut(params, n), s)
}

// ======================================================================================================
// The resolve cache (call.c's call_table hash table) + name resolution
// ======================================================================================================

/// `call.c:HASH_SIZE` -- the number of buckets in the resolve cache.
pub const HASH_SIZE: u32 = 131;

/// Port of `call.c:hash` -- the resolve-cache bucket for a name: the sum of its bytes modulo [`HASH_SIZE`].
pub fn hash(s: &[u8]) -> u32 {
    let mut val: u32 = 0;
    for &c in s {
        val = val.wrapping_add(c as u32);
    }
    val % HASH_SIZE
}

/// Port of `call.c`'s `call_table` -- the name -> resolved-entry-point cache. The entry point itself is an
/// opaque handle (the real `lt_dlsym` address is the OS boundary); here it is a stable id assigned at
/// insert, which is enough to model lookup hits/misses and the `hash` bucketing.
#[derive(Debug, Default)]
pub struct CallCache {
    buckets: std::collections::HashMap<u32, Vec<(String, usize)>>,
}

impl CallCache {
    /// A fresh, empty resolve cache.
    pub fn new() -> CallCache {
        CallCache::default()
    }

    /// Port of `call.c:insert` -- add (or replace) a name -> handle mapping in its [`hash`] bucket.
    pub fn insert(&mut self, name: &str, func: usize) {
        let b = self.buckets.entry(hash(name.as_bytes())).or_default();
        if let Some(slot) = b.iter_mut().find(|(n, _)| n == name) {
            slot.1 = func;
        } else {
            b.push((name.to_string(), func));
        }
    }

    /// Port of `call.c:lookup` -- the cached handle for `name`, or `None` if it is not resolved.
    pub fn lookup(&self, name: &str) -> Option<usize> {
        self.buckets
            .get(&hash(name.as_bytes()))
            .and_then(|b| b.iter().find(|(n, _)| n == name).map(|(_, f)| *f))
    }

    /// Drop a name from its bucket (used when a module is `CANCEL`led). Returns whether it was present.
    fn remove(&mut self, name: &str) -> bool {
        if let Some(b) = self.buckets.get_mut(&hash(name.as_bytes())) {
            let before = b.len();
            b.retain(|(n, _)| n != name);
            return b.len() != before;
        }
        false
    }
}

/// Port of `call.c:cob_resolve` -- resolve a (already case-correct) program name to its entry point: a
/// cache hit returns the handle; a miss triggers a dynamic load (`lt_dlopen`/`lt_dlsym` -- the OS boundary),
/// modelled here as `None` (unresolved) when not cached.
pub fn cob_resolve(cache: &CallCache, name: &str) -> Option<usize> {
    cache.lookup(name)
}

/// Port of `call.c:cob_resolve_cobol` -- resolve a COBOL `PROGRAM-ID`: mangle it ([`cob_encode_program_id`])
/// with the requested case folding, then [`cob_resolve`] the mangled name. Returns the (mangled name,
/// resolved handle).
pub fn cob_resolve_cobol(cache: &CallCache, name: &[u8], fold_case: i32) -> (Vec<u8>, Option<usize>) {
    let mangled = cob_encode_program_id(name, 256, fold_case);
    let handle = cob_resolve(cache, &String::from_utf8_lossy(&mangled));
    (mangled, handle)
}

/// Port of `call.c:cob_resolve_func` -- resolve a user-defined `FUNCTION` name; same mechanism as
/// [`cob_resolve`] over the function cache (the dynamic load on a miss is the OS boundary).
pub fn cob_resolve_func(cache: &CallCache, name: &str) -> Option<usize> {
    cache.lookup(name)
}

/// Port of `call.c:set_resolve_error` / `cob_resolve_error` -- the message left after a failed
/// `cob_resolve` (the C stores it in a static buffer for the caller to read). Returns the formatted text.
pub fn set_resolve_error(name: &str) -> String {
    format!("module '{name}' not found")
}
/// Port of `call.c:cob_resolve_error` -- the last resolve error message (see [`set_resolve_error`]).
pub fn cob_resolve_error(name: &str) -> String {
    set_resolve_error(name)
}
/// Port of `call.c:cob_call_error` -- the message for a `CALL` to an unresolved program.
pub fn cob_call_error(name: &str) -> String {
    format!("CALL to '{name}' which could not be resolved")
}

// ======================================================================================================
// setjmp / longjmp and dynamic-loader boundary
// ======================================================================================================

/// Port of `call.c:cob_savenv` / `cob_savenv2` -- prime the `cob_setjmp` save point. The actual `setjmp`
/// capture of the C call stack is the OS boundary; the ported logic is the "already primed?" guard.
/// `primed` is the runtime `cob_jmp_primed` flag; returns the new flag (`true` once primed).
pub fn cob_savenv(primed: bool) -> bool {
    // The C hard-fails on a double save; the observable state transition is unprimed -> primed.
    let _ = primed;
    true
}
/// Port of `call.c:cob_savenv2` -- the two-argument save variant; same prime semantics as [`cob_savenv`].
pub fn cob_savenv2(primed: bool, _arg: i32) -> bool {
    cob_savenv(primed)
}
/// Port of `call.c:cob_longjmp` -- consume the `cob_setjmp` save point. The actual `longjmp` unwind is the
/// OS boundary; the ported logic clears the prime flag. Returns the new flag (`false`).
pub fn cob_longjmp(_primed: bool) -> bool {
    false
}

/// Port of the libtool `lt_dlopen` wrapper -- load a shared module by path. Dynamic loading is the declared
/// OS boundary (and `#![forbid(unsafe_code)]` precludes `dlopen` here); returns `None` (not loaded).
pub fn lt_dlopen(_path: &str) -> Option<usize> {
    None
}
/// Port of the libtool `lt_dlsym` wrapper -- look up a symbol in a loaded module. The OS boundary; `None`.
pub fn lt_dlsym(_handle: usize, _symbol: &str) -> Option<usize> {
    None
}
/// Port of the libtool `lt_dlerror` wrapper -- the last dynamic-loader error string. The OS boundary.
pub fn lt_dlerror() -> Option<String> {
    None
}

// ======================================================================================================
// CALL / CANCEL dispatch + path / preload management
// ======================================================================================================

/// Port of `call.c:cob_chk_dirp` -- the basename of a name: the part after the last `/` or `\\`, or the
/// whole name when it has no directory separator.
pub fn cob_chk_dirp(name: &str) -> &str {
    let mut q = None;
    for (i, c) in name.char_indices() {
        if c == '/' || c == '\\' {
            q = Some(i + 1);
        }
    }
    match q {
        Some(i) => &name[i..],
        None => name,
    }
}

/// Port of `call.c:cob_chk_call_path` -- split a name into `(basename, directory)`: when the name contains
/// a path separator the directory prefix (through the last separator) is returned as `Some`, else `None`.
pub fn cob_chk_call_path(name: &str) -> (String, Option<String>) {
    let mut q = None;
    for (i, c) in name.char_indices() {
        if c == '/' || c == '\\' {
            q = Some(i + 1);
        }
    }
    match q {
        Some(i) => (name[i..].to_string(), Some(name[..i].to_string())),
        None => (name.to_string(), None),
    }
}

/// The dynamic-load / preload list (`call.c`'s `base_dynload_ptr` / `base_preload_ptr`) -- the loaded module
/// paths and their handles. The actual `lt_dlopen` is the OS boundary; this models the dedup + ordering.
#[derive(Debug, Default)]
pub struct ModuleList {
    entries: Vec<(String, Option<usize>)>,
}

impl ModuleList {
    pub fn new() -> ModuleList {
        ModuleList::default()
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn contains(&self, path: &str) -> bool {
        self.entries.iter().any(|(p, _)| p == path)
    }
}

/// Port of `call.c:add_to_preload` -- append a `(path, handle)` entry to the preload list (the C inserts in
/// load order; the `lt_dlopen` that produced `handle` is the OS boundary).
pub fn add_to_preload(list: &mut ModuleList, path: &str, handle: Option<usize>) {
    list.entries.push((path.to_string(), handle));
}

/// Port of `call.c:cache_preload` -- ensure `path` is in the preload list: `2` when it is already present
/// (a duplicate, no reload), `1` after loading + adding it, `0` when the module could not be loaded. The
/// path's `access(R_OK)` test and the `lt_dlopen` are the declared OS boundary (here a load is modelled as
/// succeeding so the dedup is observable); `add_to_preload` records the entry.
pub fn cache_preload(list: &mut ModuleList, path: &str) -> i32 {
    if list.contains(path) {
        return 2;
    }
    add_to_preload(list, path, lt_dlopen(path));
    1
}

/// `call.c:PATHSEP_CHAR` (the library-path separator) and `SLASH_CHAR` / `COB_MODULE_EXT` on this host.
const PATHSEP_CHAR: u8 = b':';
const SLASH_CHAR: char = '/';
const COB_MODULE_EXT: &str = "so";

/// Port of `call.c:last_entry_is_working_directory` -- given a path buffer and the current write position
/// `pos` (`pstr - buff`), is the last entry the working directory `.` (i.e. `buff[pos-1]=='.'` preceded by
/// the path separator at `buff[pos-2]`)? Returns `1`/`0`.
pub fn last_entry_is_working_directory(buff: &[u8], pos: usize) -> i32 {
    if pos >= 2 && buff.get(pos - 1) == Some(&b'.') && buff.get(pos - 2) == Some(&PATHSEP_CHAR) {
        1
    } else {
        0
    }
}

/// Port of `call.c:cob_try_preload` -- try to preload `module_name` from each resolved library path
/// (`<path>/<module_name>.<ext>`), returning the first non-zero [`cache_preload`] result; failing that, try
/// the bare name. Returns the final preload status (`0` when nothing could be loaded). The directory scan
/// uses the resolved `resolve_paths`; the actual load is the declared OS boundary.
pub fn cob_try_preload(list: &mut ModuleList, resolve_paths: &[String], module_name: &str) -> usize {
    for rp in resolve_paths {
        let buff = format!("{rp}{SLASH_CHAR}{module_name}.{COB_MODULE_EXT}");
        let ret = cache_preload(list, &buff);
        if ret != 0 {
            return ret as usize;
        }
    }
    cache_preload(list, module_name) as usize
}

/// Port of `call.c:cache_dynload` -- record a dynamically-loaded module: update the handle of an existing
/// path-entry whose handle is empty, or append a new entry. The `lt_dlopen` is the OS boundary.
pub fn cache_dynload(list: &mut ModuleList, path: &str, handle: Option<usize>) {
    if let Some(e) = list.entries.iter_mut().find(|(p, h)| p == path && h.is_none()) {
        e.1 = handle;
        return;
    }
    list.entries.push((path.to_string(), handle));
}

/// Port of `call.c:cob_resolve_internal` -- the core name resolver: search the resolve cache (when
/// `cache_check`), and on a miss fall through to a dynamic load (`lt_dlopen` + `lt_dlsym` -- the OS
/// boundary). Returns the cached handle, or `None` when unresolved. The name is taken already case-folded.
pub fn cob_resolve_internal(cache: &CallCache, name: &str, _fold_case: i32, _module_type: i32, cache_check: bool) -> Option<usize> {
    if cache_check {
        if let Some(h) = cache.lookup(name) {
            return Some(h);
        }
    }
    // miss -> dynamic load (the OS boundary); unresolved here.
    None
}

/// Port of `call.c:cob_set_cancel` -- register a (just-loaded) module's entry point under its name in the
/// resolve cache, so a later `CANCEL` can find and unload it.
pub fn cob_set_cancel(cache: &mut CallCache, module_name: &str, entry: usize) {
    cache.insert(module_name, entry);
}

/// Port of `call.c:do_cancel_module` -- run a module's `CANCEL` handler then drop it from the cache. Calling
/// the module's cancel entry point (a function pointer) is the OS boundary; the cache removal is ported.
pub fn do_cancel_module(cache: &mut CallCache, name: &str) {
    cache.remove(name);
}

/// Port of `call.c:cob_cancel` -- `CANCEL name`: unload the named module (its cancel handler + `lt_dlclose`
/// are the OS boundary) and drop it from the resolve cache. A no-op when the name is not loaded.
pub fn cob_cancel(cache: &mut CallCache, name: &str) {
    do_cancel_module(cache, name);
}

/// Trim a `cob_field`'s string content the way `cob_call_field`/`cob_cancel_field` do: drop trailing spaces
/// (`cob_field_to_string`) and a single uncommon leading space.
fn field_call_name(f: &CallField) -> String {
    let mut s: &[u8] = &f.data;
    while s.last() == Some(&b' ') || s.last() == Some(&0) {
        s = &s[..s.len() - 1];
    }
    if s.first() == Some(&b' ') {
        s = &s[1..];
    }
    String::from_utf8_lossy(s).into_owned()
}

/// Port of `call.c:cob_call_field` -- `CALL` with the program name held in a `cob_field`: extract + trim the
/// name, then resolve and dispatch. Returns the resolved program name (the actual call through the resolved
/// function pointer is the OS boundary).
pub fn cob_call_field(f: &CallField, _fold_case: i32) -> String {
    field_call_name(f)
}

/// Port of `call.c:cob_cancel_field` -- `CANCEL` with the name held in a `cob_field`: extract + trim the
/// name, then [`cob_cancel`] it. A no-op for an empty field.
pub fn cob_cancel_field(cache: &mut CallCache, f: &CallField) {
    if f.size() == 0 {
        return;
    }
    let name = field_call_name(f);
    cob_cancel(cache, &name);
}

/// Port of `call.c:cob_call` -- `CALL name`: resolve the program and invoke it, returning its `RETURN-CODE`.
/// Resolution (cache + name) is ported; the actual call through the resolved function pointer is the OS
/// boundary, so a resolved program yields `0` and an unresolved one yields `-1` (with the exception set by
/// the runtime). `argc` is the argument count the program is invoked with.
pub fn cob_call(cache: &CallCache, name: &str, _argc: i32) -> i32 {
    match cob_resolve_internal(cache, name, COB_FOLD_NONE, 0, true) {
        Some(_) => 0, // RETURN-CODE from the (boundary) invocation
        None => -1,
    }
}

/// Port of `call.c:cob_func` -- invoke a program once and immediately `CANCEL` it (the `CALL ... RETURNING`
/// of a one-shot function): [`cob_call`] then [`cob_cancel`], returning the call's `RETURN-CODE`.
pub fn cob_func(cache: &mut CallCache, name: &str, argc: i32) -> i32 {
    let ret = cob_call(cache, name, argc);
    cob_cancel(cache, name);
    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    fn af(field_type: u16, digits: u16, scale: i16, flags: u16) -> FieldAttr {
        FieldAttr { field_type, digits, scale, flags }
    }

    #[test]
    fn program_id_mangling() {
        // valid chars pass; '-' -> '__'; other invalid -> '_HH'; leading digit -> '_' prefix; folding
        assert_eq!(cob_encode_program_id(b"FOO-BAR", 64, COB_FOLD_NONE), b"FOO__BAR".to_vec());
        assert_eq!(cob_encode_program_id(b"9PROG", 64, COB_FOLD_NONE), b"_9PROG".to_vec());
        assert_eq!(cob_encode_program_id(b"a.b", 64, COB_FOLD_NONE), b"a_2Eb".to_vec()); // '.' = 0x2E
        assert_eq!(cob_encode_program_id(b"MixEd", 64, COB_FOLD_UPPER), b"MIXED".to_vec());
        assert_eq!(cob_encode_program_id(b"MixEd", 64, COB_FOLD_LOWER), b"mixed".to_vec());
    }

    #[test]
    fn field_and_param_metadata() {
        // a signed PIC S9(5)V99 DISPLAY field: digits 7, scale 2, signed
        let f = CallField { attr: af(COB_TYPE_NUMERIC_DISPLAY, 7, 2, COB_FLAG_HAVE_SIGN), data: vec![0u8; 7] };
        assert_eq!(cob_get_field_type(Some(&f)), COB_TYPE_NUMERIC_DISPLAY as i32);
        assert_eq!(cob_get_field_size(Some(&f)), 7);
        assert_eq!(cob_get_field_sign(Some(&f)), 1);
        assert_eq!(cob_get_field_scale(Some(&f)), 2);
        assert_eq!(cob_get_field_digits(Some(&f)), 7);
        assert_eq!(cob_get_field_constant(Some(&f)), 0);
        // NULL field -> -1 across the board
        assert_eq!(cob_get_field_type(None), -1);
        assert_eq!(cob_get_field_size(None), -1);
        // a non-swapped BINARY reports as COMP-5
        let b = CallField { attr: af(COB_TYPE_NUMERIC_BINARY, 9, 0, 0), data: vec![0u8; 4] };
        assert_eq!(cob_get_field_type(Some(&b)), COB_TYPE_NUMERIC_COMP5 as i32);
        // constant view
        let c = cob_field_constant(&f);
        assert_eq!(cob_get_field_constant(Some(&c)), 1);
    }

    #[test]
    fn param_accessors_and_values() {
        // two params: a COMP-5 binary holding 1000, and a PIC X(3) "ABC"
        let p1 = CallField { attr: af(COB_TYPE_NUMERIC_BINARY, 9, 0, 0), data: 1000u32.to_le_bytes().to_vec() };
        let p2 = CallField { attr: af(COB_TYPE_ALPHANUMERIC, 0, 0, 0), data: b"ABC".to_vec() };
        let mut params = Params { fields: vec![Some(p1), Some(p2), None] };
        assert_eq!(cob_get_num_params(&params), 3);
        assert_eq!(cob_get_param_size(&params, 1), 4);
        assert_eq!(cob_get_s64_param(&params, 1), 1000);
        assert_eq!(cob_get_u64_param(&params, 1), 1000);
        assert_eq!(cob_get_param_data(&params, 2), Some(&b"ABC"[..]));
        assert_eq!(cob_get_param_field(&params, 3), None); // omitted
        assert_eq!(cob_get_s64_param(&params, 9), -1); // out of range
        // get/put PIC X: cob_get_picx returns the trailing-space-trimmed content (capped to char_len-1)
        assert_eq!(cob_get_picx_param(&params, 2, 5), Some(b"ABC".to_vec()));
        assert!(cob_put_picx_param(&mut params, 2, b"XY"));
        assert_eq!(cob_get_param_data(&params, 2), Some(&b"XY "[..]));
        // put into the binary param, read back
        assert!(cob_put_s64_param(&mut params, 1, 42));
        assert_eq!(cob_get_s64_param(&params, 1), 42);
        // group get/put
        assert_eq!(cob_get_grp_param(&params, 2, 0).unwrap().len(), 3);
        assert!(cob_put_grp_param(&mut params, 2, b"ZZ", 2));
        assert_eq!(&cob_get_param_data(&params, 2).unwrap()[..2], b"ZZ");
    }

    #[test]
    fn field_str_get_and_put() {
        // a signed PIC S9(3)V99 DISPLAY field; pretty render + parse round-trip
        let mut f = CallField { attr: af(COB_TYPE_NUMERIC_DISPLAY, 5, 2, COB_FLAG_HAVE_SIGN), data: vec![b'0'; 5] };
        assert_eq!(cob_put_field_str(Some(&mut f), b"-12.34"), 0);
        assert_eq!(cob_get_field_str(Some(&f)), b"-12.34".to_vec());
        // alphanumeric: raw bytes in/out
        let mut a = CallField { attr: af(COB_TYPE_ALPHANUMERIC, 0, 0, 0), data: vec![b' '; 4] };
        assert_eq!(cob_put_field_str(Some(&mut a), b"Hi"), 0);
        assert_eq!(cob_get_field_str(Some(&a)), b"Hi  ".to_vec());
        // NULL field, and a constant field is refused
        assert_eq!(cob_get_field_str(None), b"NULL field".to_vec());
        let mut c = cob_field_constant(&a);
        assert_eq!(cob_put_field_str(Some(&mut c), b"X"), 22);
    }

    #[test]
    fn resolve_cache_and_hash() {
        // hash is the byte-sum mod 131; insert/lookup round-trip; cob_resolve_cobol mangles then resolves
        assert_eq!(hash(b"AB"), (b'A' as u32 + b'B' as u32) % HASH_SIZE);
        let mut cache = CallCache::new();
        cache.insert("FOO__BAR", 7);
        assert_eq!(cache.lookup("FOO__BAR"), Some(7));
        assert_eq!(cache.lookup("NOPE"), None);
        assert_eq!(cob_resolve(&cache, "FOO__BAR"), Some(7));
        // a COBOL name "FOO-BAR" folds-upper to "FOO__BAR" and resolves
        let (mangled, handle) = cob_resolve_cobol(&cache, b"FOO-BAR", COB_FOLD_UPPER);
        assert_eq!(mangled, b"FOO__BAR".to_vec());
        assert_eq!(handle, Some(7));
        // the dynamic-load boundary is unresolved
        assert_eq!(lt_dlopen("libfoo.so"), None);
        assert_eq!(lt_dlsym(0, "sym"), None);
    }

    #[test]
    fn call_cancel_path_and_preload() {
        // basename / path split
        assert_eq!(cob_chk_dirp("dir/sub/PROG"), "PROG");
        assert_eq!(cob_chk_dirp("PROG"), "PROG");
        assert_eq!(cob_chk_call_path("d/PROG"), ("PROG".to_string(), Some("d/".to_string())));
        assert_eq!(cob_chk_call_path("PROG"), ("PROG".to_string(), None));
        // preload list: fresh load returns 1, a duplicate returns 2
        let mut pl = ModuleList::new();
        assert_eq!(cache_preload(&mut pl, "libA"), 1);
        assert_eq!(cache_preload(&mut pl, "libA"), 2);
        assert_eq!(pl.len(), 1);
        cache_dynload(&mut pl, "libB", Some(9));
        assert!(pl.contains("libB"));
        // cob_try_preload scans each resolve path then the bare name
        let mut pl2 = ModuleList::new();
        assert_eq!(cob_try_preload(&mut pl2, &["/opt/cob".to_string()], "MOD"), 1);
        assert!(pl2.contains("/opt/cob/MOD.so"));
        // last_entry_is_working_directory: ":." at the tail is the cwd entry
        assert_eq!(last_entry_is_working_directory(b" :.", 3), 1);
        assert_eq!(last_entry_is_working_directory(b" :x", 3), 0);
        // CALL resolves a cached program (0) vs an unresolved one (-1); CANCEL drops it from the cache
        let mut cache = CallCache::new();
        cob_set_cancel(&mut cache, "PROG", 3);
        assert_eq!(cob_call(&cache, "PROG", 0), 0);
        assert_eq!(cob_call(&cache, "GHOST", 0), -1);
        cob_cancel(&mut cache, "PROG");
        assert_eq!(cob_call(&cache, "PROG", 0), -1); // gone after CANCEL
        // CALL/CANCEL with the name in a field (trailing spaces trimmed)
        let namef = CallField { attr: af(COB_TYPE_ALPHANUMERIC, 0, 0, 0), data: b"SUBPROG ".to_vec() };
        assert_eq!(cob_call_field(&namef, COB_FOLD_NONE), "SUBPROG");
        cob_cancel_field(&mut cache, &namef); // no panic on an unloaded name
        // cob_func calls then cancels
        cob_set_cancel(&mut cache, "ONESHOT", 5);
        assert_eq!(cob_func(&mut cache, "ONESHOT", 0), 0);
        assert_eq!(cob_call(&cache, "ONESHOT", 0), -1); // cancelled by cob_func
    }
}
