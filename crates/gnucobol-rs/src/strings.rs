//! 1:1 port of strings.c's STRING / UNSTRING / INSPECT runtime API. The C uses module-global state
//! that the generated code drives through `init -> ... -> finish` call sequences; this port carries the
//! same state in explicit structs (no global mutable state, `#![forbid(unsafe_code)]`). The observable
//! results are the sealed `GNURUST.STRING.UNSTRING.1` / `GNURUST.INSPECT.1` byte courts.
#![forbid(unsafe_code)]

use crate::attr::{FieldAttr, COB_TYPE_ALPHANUMERIC};

/// `cob_str_memcpy (dst, src, size)` (strings.c:104): move `size` alphanumeric bytes into `dst` via
/// `cob_move` (so the receiver's JUSTIFIED/padding semantics apply).
pub fn cob_str_memcpy(dst: &mut [u8], dst_attr: &FieldAttr, src: &[u8], size: usize) {
    let sattr = FieldAttr { field_type: COB_TYPE_ALPHANUMERIC, digits: 0, scale: 0, flags: 0 };
    let _ = crate::move_ops::cob_move(&src[..size.min(src.len())], &sattr, dst, dst_attr);
}

/// `cob_init_strings` (strings.c): module init binding the runtime global. A no-op in this port.
pub fn cob_init_strings() {}
/// `cob_exit_strings` (strings.c): module teardown freeing the scratch buffers. A no-op (RAII).
pub fn cob_exit_strings() {}

/// The STRING-statement state machine (strings.c:739-816): `cob_string_init` then one or more
/// `cob_string_delimited`/`cob_string_append`, then `cob_string_finish`. State that the C keeps in
/// module globals (`string_dst`, `string_ptr`, `string_dlm`, `string_offset`) lives here.
pub struct CobString {
    dst: Vec<u8>,
    dst_attr: FieldAttr,
    ptr: Option<(Vec<u8>, FieldAttr)>,
    dlm: Option<Vec<u8>>,
    offset: i32,
    /// `COB_EC_OVERFLOW_STRING` raised (the `ON OVERFLOW` condition).
    pub overflow: bool,
}

impl CobString {
    /// `cob_string_init (dst, ptr)` (strings.c:739): capture the receiver and optional `WITH POINTER`,
    /// seed the offset from the pointer (`-1` for 1-based), and flag overflow if it is out of range.
    pub fn cob_string_init(dst: &[u8], dst_attr: &FieldAttr, ptr: Option<(&[u8], &FieldAttr)>) -> Self {
        let ptrv = ptr.map(|(d, a)| (d.to_vec(), *a));
        let mut offset = 0i32;
        let mut overflow = false;
        if let Some((pd, pa)) = &ptrv {
            offset = crate::accessors::cob_get_int(pd, pa) - 1;
            if offset < 0 || offset >= dst.len() as i32 {
                overflow = true;
            }
        }
        CobString { dst: dst.to_vec(), dst_attr: *dst_attr, ptr: ptrv, dlm: None, offset, overflow }
    }

    /// `cob_string_delimited (dlm)` (strings.c:761): set the active `DELIMITED BY` (or `None` for SIZE).
    pub fn cob_string_delimited(&mut self, dlm: Option<&[u8]>) {
        self.dlm = dlm.map(|d| d.to_vec());
    }

    /// `cob_string_append (src)` (strings.c:771): append `src` (up to the delimiter) into the receiver
    /// at the running offset; raise `OVERFLOW` and stop when the receiver fills.
    pub fn cob_string_append(&mut self, src: &[u8]) {
        if self.overflow {
            return;
        }
        let mut src_size = src.len();
        if src_size == 0 {
            return;
        }
        if let Some(dlm) = &self.dlm {
            if !dlm.is_empty() && dlm.len() <= src_size {
                let size = src_size - dlm.len() + 1;
                for i in 0..size {
                    if &src[i..i + dlm.len()] == dlm.as_slice() {
                        src_size = i;
                        break;
                    }
                }
            }
        }
        let off = self.offset as usize;
        if src_size as i32 <= self.dst.len() as i32 - self.offset {
            self.dst[off..off + src_size].copy_from_slice(&src[..src_size]);
            self.offset += src_size as i32;
        } else {
            let size = (self.dst.len() as i32 - self.offset).max(0) as usize;
            self.dst[off..off + size].copy_from_slice(&src[..size]);
            self.offset += size as i32;
            self.overflow = true;
        }
    }

    /// `cob_string_finish ()` (strings.c:808): write the final position back to the `WITH POINTER`.
    pub fn cob_string_finish(&mut self) {
        if let Some((pd, pa)) = &mut self.ptr {
            let _ = crate::accessors::cob_set_int(pd, pa, self.offset + 1);
        }
    }

    /// The receiver bytes after the STRING.
    pub fn result(&self) -> &[u8] {
        &self.dst
    }
    /// The `WITH POINTER` bytes after the STRING (if any).
    pub fn pointer(&self) -> Option<&[u8]> {
        self.ptr.as_ref().map(|(d, _)| d.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alnum(n: usize) -> FieldAttr {
        let _ = n;
        FieldAttr { field_type: COB_TYPE_ALPHANUMERIC, digits: 0, scale: 0, flags: 0 }
    }

    #[test]
    fn string_concatenates_with_delimiter_and_pointer() {
        // STRING "AB" "CDE" DELIMITED BY SIZE INTO dst(8) -> "ABCDE   "
        let mut s = CobString::cob_string_init(&[b' '; 8], &alnum(8), None);
        s.cob_string_delimited(None);
        s.cob_string_append(b"AB");
        s.cob_string_append(b"CDE");
        s.cob_string_finish();
        assert_eq!(&s.result()[..5], b"ABCDE");
        assert!(!s.overflow);

        // DELIMITED BY "X": "ABXYZ" delimited by "X" appends only "AB"
        let mut s = CobString::cob_string_init(&[0u8; 8], &alnum(8), None);
        s.cob_string_delimited(Some(b"X"));
        s.cob_string_append(b"ABXYZ");
        assert_eq!(&s.result()[..2], b"AB");

        // overflow: into dst(3), append "ABCDE" -> fills 3, overflow set
        let mut s = CobString::cob_string_init(&[0u8; 3], &alnum(3), None);
        s.cob_string_delimited(None);
        s.cob_string_append(b"ABCDE");
        assert_eq!(s.result(), b"ABC");
        assert!(s.overflow);

        // WITH POINTER: start at position 3 (1-based) in dst(8)
        let ptr = 3i32.to_le_bytes(); // COMP-5 9(9)
        let pa = FieldAttr { field_type: crate::attr::COB_TYPE_NUMERIC_BINARY, digits: 9, scale: 0, flags: crate::attr::COB_FLAG_HAVE_SIGN | crate::attr::COB_FLAG_REAL_BINARY };
        let mut s = CobString::cob_string_init(&[b'.'; 8], &alnum(8), Some((&ptr, &pa)));
        s.cob_string_delimited(None);
        s.cob_string_append(b"XY");
        s.cob_string_finish();
        assert_eq!(s.result(), b"..XY....");
        // pointer now 5 (started 3, appended 2)
        assert_eq!(crate::accessors::cob_get_int(s.pointer().unwrap(), &pa), 5);
    }
}
