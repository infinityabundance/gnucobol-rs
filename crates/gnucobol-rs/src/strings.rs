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

fn is_numeric(attr: &FieldAttr) -> bool {
    matches!(attr.field_type, 0x10..=0x1B | 0x24)
}

/// The UNSTRING-statement state machine (strings.c:828-1018): `cob_unstring_init`, then
/// `cob_unstring_delimited` per `DELIMITED BY`, then `cob_unstring_into` per receiver, then optionally
/// `cob_unstring_tallying`, then `cob_unstring_finish`.
pub struct CobUnstring {
    src: Vec<u8>,
    ptr: Option<(Vec<u8>, FieldAttr)>,
    dlms: Vec<(Vec<u8>, bool)>, // (delimiter, ALL)
    offset: i32,
    count: i32,
    /// `COB_EC_OVERFLOW_UNSTRING` raised.
    pub overflow: bool,
}

impl CobUnstring {
    /// `cob_unstring_init (src, ptr, num_dlm)` (strings.c:828): capture the source + optional
    /// `WITH POINTER` (seeding the offset), reset counters; `num_dlm` is a C allocation hint (unused).
    pub fn cob_unstring_init(src: &[u8], ptr: Option<(&[u8], &FieldAttr)>, _num_dlm: usize) -> Self {
        let ptrv = ptr.map(|(d, a)| (d.to_vec(), *a));
        let mut offset = 0i32;
        let mut overflow = false;
        if let Some((pd, pa)) = &ptrv {
            offset = crate::accessors::cob_get_int(pd, pa) - 1;
            if offset < 0 || offset >= src.len() as i32 {
                overflow = true;
            }
        }
        CobUnstring { src: src.to_vec(), ptr: ptrv, dlms: Vec::new(), offset, count: 0, overflow }
    }

    /// `cob_unstring_delimited (dlm, all)` (strings.c:863): register a `DELIMITED BY [ALL]` delimiter.
    pub fn cob_unstring_delimited(&mut self, dlm: &[u8], all: bool) {
        self.dlms.push((dlm.to_vec(), all));
    }

    /// `cob_unstring_into (dst, dlm, cnt)` (strings.c:871): extract the next sub-field (up to the first
    /// matching delimiter, or `DELIMITED BY SIZE`) into `dst`, set `DELIMITER IN`/`COUNT IN`, advance.
    pub fn cob_unstring_into(
        &mut self,
        dst: &mut [u8],
        dst_attr: &FieldAttr,
        dlm_out: Option<(&mut [u8], &FieldAttr)>,
        cnt: Option<(&mut [u8], &FieldAttr)>,
    ) {
        if self.overflow || self.offset >= self.src.len() as i32 {
            return;
        }
        let start = self.offset as usize;
        let srsize = self.src.len();
        let mut dlm_data: Option<Vec<u8>> = None;
        let mut match_size: i32;
        if self.dlms.is_empty() {
            // DELIMITED BY SIZE
            match_size = (dst.len() as i32).min(srsize as i32 - self.offset);
            cob_str_memcpy(dst, dst_attr, &self.src[start..], match_size as usize);
            self.offset += match_size;
        } else {
            let mut found = false;
            match_size = 0;
            let mut p = start;
            'outer: while p < srsize {
                for (dlm, all) in &self.dlms {
                    let dlsize = dlm.len();
                    if dlsize == 0 || p + dlsize > srsize {
                        continue;
                    }
                    if &self.src[p..p + dlsize] == dlm.as_slice() {
                        match_size = (p - start) as i32;
                        cob_str_memcpy(dst, dst_attr, &self.src[start..], match_size as usize);
                        self.offset += match_size + dlsize as i32;
                        dlm_data = Some(dlm.clone());
                        if *all {
                            let mut q = p + dlsize;
                            while q + dlsize <= srsize && &self.src[q..q + dlsize] == dlm.as_slice() {
                                self.offset += dlsize as i32;
                                q += dlsize;
                            }
                        }
                        found = true;
                        break 'outer;
                    }
                }
                p += 1;
            }
            if !found {
                match_size = srsize as i32 - self.offset;
                cob_str_memcpy(dst, dst_attr, &self.src[start..], match_size as usize);
                self.offset = srsize as i32;
            }
        }
        self.count += 1;
        if let Some((dd, da)) = dlm_out {
            if let Some(ddata) = &dlm_data {
                cob_str_memcpy(dd, da, ddata, ddata.len());
            } else if is_numeric(da) {
                let _ = crate::accessors::cob_set_int(dd, da, 0);
            } else {
                for b in dd.iter_mut() {
                    *b = b' ';
                }
            }
        }
        if let Some((cd, ca)) = cnt {
            let _ = crate::accessors::cob_set_int(cd, ca, match_size);
        }
    }

    /// `cob_unstring_tallying (f)` (strings.c:1006): add the number of receivers filled to `TALLYING`.
    pub fn cob_unstring_tallying(&self, f: &mut [u8], attr: &FieldAttr) {
        let cur = crate::accessors::cob_get_int(f, attr);
        let _ = crate::accessors::cob_set_int(f, attr, cur + self.count);
    }

    /// `cob_unstring_finish ()` (strings.c:1012): flag `OVERFLOW` if the source was not exhausted, and
    /// write the final position back to `WITH POINTER`.
    pub fn cob_unstring_finish(&mut self) {
        if self.offset < self.src.len() as i32 {
            self.overflow = true;
        }
        if let Some((pd, pa)) = &mut self.ptr {
            let _ = crate::accessors::cob_set_int(pd, pa, self.offset + 1);
        }
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
    fn unstring_splits_by_delimiter() {
        let a3 = alnum(3);
        // UNSTRING "AB,CD,EF" DELIMITED BY "," INTO d1 d2 d3
        let mut u = CobUnstring::cob_unstring_init(b"AB,CD,EF", None, 1);
        u.cob_unstring_delimited(b",", false);
        let outs: &[&[u8]] = &[b"AB ", b"CD ", b"EF "];
        for &want in outs {
            let mut d = vec![b' '; 3];
            let mut cnt = 9i32.to_le_bytes();
            let ca = FieldAttr { field_type: crate::attr::COB_TYPE_NUMERIC_BINARY, digits: 9, scale: 0, flags: crate::attr::COB_FLAG_HAVE_SIGN | crate::attr::COB_FLAG_REAL_BINARY };
            u.cob_unstring_into(&mut d, &a3, None, Some((&mut cnt, &ca)));
            assert_eq!(&d, want, "unstring field");
        }
        u.cob_unstring_finish();
        assert!(!u.overflow);

        // DELIMITED BY SIZE: fixed 3-byte chunks
        let mut u = CobUnstring::cob_unstring_init(b"ABCDEF", None, 0);
        let mut d = vec![0u8; 3];
        u.cob_unstring_into(&mut d, &a3, None, None);
        assert_eq!(&d, b"ABC");
        u.cob_unstring_into(&mut d, &a3, None, None);
        assert_eq!(&d, b"DEF");

        // DELIMITED BY ALL " ": collapse runs of spaces (cob_move space-pads the receiver)
        let mut u = CobUnstring::cob_unstring_init(b"A   B", None, 1);
        u.cob_unstring_delimited(b" ", true);
        let mut d = vec![b'.'; 3];
        u.cob_unstring_into(&mut d, &a3, None, None);
        assert_eq!(&d, b"A  ");
        let mut d2 = vec![b'.'; 3];
        u.cob_unstring_into(&mut d2, &a3, None, None);
        assert_eq!(&d2, b"B  ");
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
