//! # gnucobol-rs-ffi
//!
//! A **C-ABI shim** that exposes the native-Rust `libcob` algorithms of [`gnucobol_rs`] with `extern "C"`
//! linkage, so a C program can link gnucobol-rs as a drop-in **`libcob` replacement** -- the same
//! `cob_field` / `cob_field_attr` structs and the same entry points (`cob_move`, `cob_get_int`,
//! `cob_set_int`, `cob_get_llint`, `cob_get_long_long`, ...), backed by the byte-identical Rust port.
//!
//! This is the **only** crate in the workspace that uses `unsafe` -- it is the C boundary, where raw
//! `cob_field` pointers cross from C. The [`gnucobol_rs`] core stays `#![forbid(unsafe_code)]`: every
//! function here just (1) reads the C struct, (2) calls the safe-Rust algorithm, (3) writes the result
//! back. No COBOL semantics live here.
//!
//! The C header is `include/gnucobol-rs-ffi.h`. Build a `cdylib` (`libgnucobol_rs_ffi.so`) or `staticlib`
//! and link it where you would link `libcob`.

use gnucobol_rs::attr::FieldAttr;
use std::os::raw::{c_int, c_longlong};
use std::slice;

/// C `cob_field_attr` (libcob `common.h:1097`): the field's type, digit count, scale, flags, and a
/// pointer to its picture string. `#[repr(C)]` matches the C struct layout exactly.
#[repr(C)]
pub struct CobFieldAttr {
    /// `unsigned short type` -- the `COB_TYPE_*` field type.
    pub type_: u16,
    /// `unsigned short digits` -- the digit count.
    pub digits: u16,
    /// `signed short scale` -- the field scale.
    pub scale: i16,
    /// `unsigned short flags` -- the `COB_FLAG_*` bits.
    pub flags: u16,
    /// `const cob_pic_symbol *pic` -- the picture string (opaque to this shim).
    pub pic: *const u8,
}

/// C `cob_field` (libcob `common.h:1107`): a byte buffer (`size`/`data`) tagged with its attributes.
#[repr(C)]
pub struct CobField {
    /// `size_t size` -- the field's byte length.
    pub size: usize,
    /// `unsigned char *data` -- the field's data bytes.
    pub data: *mut u8,
    /// `const cob_field_attr *attr` -- the field's attributes.
    pub attr: *const CobFieldAttr,
}

/// Read a C [`CobFieldAttr`] into the safe-Rust [`FieldAttr`] the core algorithms take.
///
/// # Safety
/// `a` must be a valid, non-null pointer to a `CobFieldAttr`.
unsafe fn rust_attr(a: *const CobFieldAttr) -> FieldAttr {
    let a = &*a;
    FieldAttr { field_type: a.type_, digits: a.digits, scale: a.scale, flags: a.flags }
}

/// `void cob_move (cob_field *src, cob_field *dst)` -- the libcob `MOVE`, native-Rust.
///
/// Decodes `src` (its bytes + attr), converts to `dst`'s category/usage, and writes `dst`'s bytes. A
/// null pointer or a failed conversion leaves `dst` unchanged.
///
/// # Safety
/// `src`/`dst` must be valid `cob_field` pointers whose `data`/`attr` are valid for `size` bytes.
#[no_mangle]
pub unsafe extern "C" fn cob_move(src: *const CobField, dst: *mut CobField) {
    if src.is_null() || dst.is_null() {
        return;
    }
    let s = &*src;
    let d = &*dst;
    if s.data.is_null() || s.attr.is_null() || d.data.is_null() || d.attr.is_null() {
        return;
    }
    let src_bytes = slice::from_raw_parts(s.data, s.size);
    let src_attr = rust_attr(s.attr);
    let dst_attr = rust_attr(d.attr);
    // Convert into a private buffer first, then copy back -- so a partial/failed move never half-writes
    // `dst`, and `src`/`dst` aliasing is harmless.
    let mut buf = vec![0u8; d.size];
    if gnucobol_rs::move_ops::cob_move(src_bytes, &src_attr, &mut buf, &dst_attr).is_ok() {
        let dst_bytes = slice::from_raw_parts_mut(d.data, d.size);
        dst_bytes.copy_from_slice(&buf);
    }
}

/// `int cob_get_int (cob_field *f)` -- the field's value as a 32-bit int (libcob `cob_get_int`).
///
/// # Safety
/// `f` must be a valid `cob_field` pointer.
#[no_mangle]
pub unsafe extern "C" fn cob_get_int(f: *const CobField) -> c_int {
    if f.is_null() {
        return 0;
    }
    let f = &*f;
    if f.data.is_null() || f.attr.is_null() {
        return 0;
    }
    let bytes = slice::from_raw_parts(f.data, f.size);
    gnucobol_rs::accessors::cob_get_int(bytes, &rust_attr(f.attr))
}

/// `cob_s64_t cob_get_llint (cob_field *f)` -- the field's value as a 64-bit int (libcob `cob_get_llint`).
///
/// # Safety
/// `f` must be a valid `cob_field` pointer.
#[no_mangle]
pub unsafe extern "C" fn cob_get_llint(f: *const CobField) -> c_longlong {
    if f.is_null() {
        return 0;
    }
    let f = &*f;
    if f.data.is_null() || f.attr.is_null() {
        return 0;
    }
    let bytes = slice::from_raw_parts(f.data, f.size);
    gnucobol_rs::accessors::cob_get_llint(bytes, &rust_attr(f.attr))
}

/// `void cob_set_int (cob_field *f, int n)` -- store the int `n` into `f` (libcob `cob_set_int`).
///
/// # Safety
/// `f` must be a valid, writable `cob_field` pointer.
#[no_mangle]
pub unsafe extern "C" fn cob_set_int(f: *mut CobField, n: c_int) {
    if f.is_null() {
        return;
    }
    let f = &*f;
    if f.data.is_null() || f.attr.is_null() {
        return;
    }
    let attr = rust_attr(f.attr);
    let mut buf = vec![0u8; f.size];
    if gnucobol_rs::accessors::cob_set_int(&mut buf, &attr, n).is_ok() {
        let dst = slice::from_raw_parts_mut(f.data, f.size);
        dst.copy_from_slice(&buf);
    }
}

/// `const char *cobrs_version (void)` -- the GnuCOBOL version this runtime reproduces (e.g. `"3.2.0"`),
/// as a static NUL-terminated string. (Not a libcob symbol -- a convenience for FFI consumers.)
#[no_mangle]
pub extern "C" fn cobrs_version() -> *const u8 {
    use gnucobol_rs::common::{LIBCOB_VERSION, LIBCOB_VERSION_MINOR, LIBCOB_VERSION_PATCHLEVEL};
    // A leaked, NUL-terminated static -- valid for the process lifetime.
    use std::sync::OnceLock;
    static VER: OnceLock<Vec<u8>> = OnceLock::new();
    VER.get_or_init(|| {
        let mut v = format!("{LIBCOB_VERSION}.{LIBCOB_VERSION_MINOR}.{LIBCOB_VERSION_PATCHLEVEL}").into_bytes();
        v.push(0);
        v
    })
    .as_ptr()
}
