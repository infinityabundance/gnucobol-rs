//! 1:1 port of strings.c's STRING / UNSTRING / INSPECT runtime API. The C uses module-global state
//! that the generated code drives through `init -> ... -> finish` call sequences; this port carries the
//! same state in explicit structs (no global mutable state, `#![forbid(unsafe_code)]`). The observable
//! results are the sealed `GNURUST.STRING.UNSTRING.1` / `GNURUST.INSPECT.1` byte courts.
#![forbid(unsafe_code)]

use crate::attr::{FieldAttr, COB_TYPE_ALPHANUMERIC, COB_TYPE_NUMERIC_DISPLAY};

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
    ptr: Option<(Vec<u8>, FieldAttr)>,
    dlm: Option<Vec<u8>>,
    offset: i32,
    /// `COB_EC_OVERFLOW_STRING` raised (the `ON OVERFLOW` condition).
    pub overflow: bool,
}

impl CobString {
    /// `cob_string_init (dst, ptr)` (strings.c:739): capture the receiver and optional `WITH POINTER`,
    /// seed the offset from the pointer (`-1` for 1-based), and flag overflow if it is out of range.
    pub fn cob_string_init(dst: &[u8], _dst_attr: &FieldAttr, ptr: Option<(&[u8], &FieldAttr)>) -> Self {
        let ptrv = ptr.map(|(d, a)| (d.to_vec(), *a));
        let mut offset = 0i32;
        let mut overflow = false;
        if let Some((pd, pa)) = &ptrv {
            offset = crate::accessors::cob_get_int(pd, pa) - 1;
            if offset < 0 || offset >= dst.len() as i32 {
                overflow = true;
            }
        }
        CobString { dst: dst.to_vec(), ptr: ptrv, dlm: None, offset, overflow }
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

/// Add `n` to an integer receiver field (the `cob_add_int` used by INSPECT TALLYING / UNSTRING; tally
/// counters are integer fields, so get + set is the faithful integer behaviour).
fn add_int_field(f: &mut [u8], attr: &FieldAttr, n: i32) {
    let cur = crate::accessors::cob_get_int(f, attr);
    let _ = crate::accessors::cob_set_int(f, attr, cur + n);
}

#[derive(Clone, Copy, PartialEq)]
enum InspectType {
    All,
    Leading,
    First,
    Trailing,
}

/// The INSPECT-statement state machine (strings.c:391-734): `cob_inspect_init[_converting]`, then per
/// clause `cob_inspect_start` + optional `before`/`after`, then a verb
/// (`characters`/`all`/`leading`/`first`/`trailing`/`converting`), then `cob_inspect_finish`. The
/// position marker (`mark`) and deferred replacement buffer (`repdata`) are carried here, not in globals.
pub struct CobInspect {
    data: Vec<u8>,
    off: usize, // data region offset within the field (sign-leading-separate)
    size: usize,
    start: usize,
    end: usize,
    mark: Vec<u8>,
    mark_min: usize,
    mark_max: usize,
    repdata: Vec<u8>,
    replacing: bool,
    // signed-numeric-display: strip the overpunch sign on init, restore on finish.
    sign: i32,
    var_attr: FieldAttr,
}

/// Expand a figurative `ALL x` pattern (`f` is a 1+ byte pattern) to `target_len` bytes (the
/// `alloc_figurative` behaviour for INSPECT REPLACING / CONVERTING).
fn alloc_figurative(f: &[u8], target_len: usize) -> Vec<u8> {
    if f.is_empty() {
        return vec![b' '; target_len];
    }
    (0..target_len).map(|i| f[i % f.len()]).collect()
}

impl CobInspect {
    /// `cob_inspect_init_common (var)` (strings.c:443): capture the field, strip a numeric-DISPLAY
    /// overpunch sign (restored at finish), and reset the window. The `replacing` flag selects the
    /// TALLYING vs REPLACING verb behaviour (the C uses the module-global `inspect_replacing`).
    fn cob_inspect_init_common(var: &[u8], var_attr: &FieldAttr, replacing: bool) -> Self {
        let mut data = var.to_vec();
        let off = var_attr.data_offset();
        let size = var_attr.data_size(var.len());
        // signed numeric DISPLAY (non-separate): strip the overpunch sign, restore at finish
        let mut sign = 0;
        if var_attr.have_sign() && !var_attr.sign_separate() && var_attr.field_type == COB_TYPE_NUMERIC_DISPLAY {
            sign = crate::sign::display_get_sign_strip(&mut data, var_attr);
        }
        CobInspect {
            data,
            off,
            size,
            start: 0,
            end: 0,
            mark: vec![0u8; size],
            mark_min: 0,
            mark_max: 0,
            repdata: vec![0u8; size],
            replacing,
            sign,
            var_attr: *var_attr,
        }
    }

    /// `cob_inspect_init (var, replacing)` (strings.c:471).
    pub fn cob_inspect_init(var: &[u8], var_attr: &FieldAttr, replacing: bool) -> Self {
        Self::cob_inspect_init_common(var, var_attr, replacing)
    }

    /// `cob_inspect_init_converting (var)` (strings.c:504).
    pub fn cob_inspect_init_converting(var: &[u8], var_attr: &FieldAttr) -> Self {
        Self::cob_inspect_init_common(var, var_attr, false)
    }

    /// `cob_inspect_start ()` (strings.c:510): the inspect window is the whole field.
    pub fn cob_inspect_start(&mut self) {
        self.start = 0;
        self.end = self.size;
    }

    /// Find `needle` in the data window `[start, end)`; returns the absolute index or `None`.
    fn inspect_find_data(&self, needle: &[u8]) -> Option<usize> {
        let len = needle.len();
        if len == 0 || self.start + len > self.end {
            return None;
        }
        let a = self.off;
        (self.start..=self.end - len).find(|&p| &self.data[a + p..a + p + len] == needle)
    }

    /// `cob_inspect_before (str)` (strings.c:518): bound the window before `str`.
    pub fn cob_inspect_before(&mut self, s: &[u8]) {
        if let Some(p) = self.inspect_find_data(s) {
            self.end = p;
        }
    }

    /// `cob_inspect_after (str)` (strings.c:527): bound the window after `str`.
    pub fn cob_inspect_after(&mut self, s: &[u8]) {
        if let Some(p) = self.inspect_find_data(s) {
            self.start = p + s.len();
        } else {
            self.start = self.end;
        }
    }

    fn is_marked(&self, pos: usize, length: usize) -> bool {
        if self.mark[self.mark_min] == 0 || self.mark_max < pos || self.mark_min >= pos + length {
            return false;
        }
        if self.mark_min >= pos || self.mark_max < pos + length {
            return true;
        }
        self.mark[pos..pos + length].iter().any(|&m| m != 0)
    }

    fn set_inspect_mark(&mut self, pos: usize, length: usize) {
        // strings.c `set_inspect_mark`: `pos_end = pos + length - 1`. C `size_t` underflows to SIZE_MAX
        // when called with length 0 (the LEADING `last_marker == 0` path), with a length-0 `memset`
        // writing nothing; `wrapping_sub` reproduces that exact behaviour (the empty slice is a no-op).
        let pos_end = (pos + length).wrapping_sub(1);
        for m in &mut self.mark[pos..pos + length] {
            *m = 1;
        }
        if (self.mark_min == 0 && self.mark[self.mark_min] == 0) || pos < self.mark_min {
            self.mark_min = pos;
        }
        if pos_end > self.mark_max {
            self.mark_max = pos_end;
        }
    }

    /// `setup_repdata ()` (strings.c:163): lazily ensure the deferred-replacement buffer is large enough.
    /// This port allocates `repdata` eagerly at `cob_inspect_init_common` (RAII sizing to the field), so
    /// the C's grow-on-first-use is already satisfied; kept as a named 1:1 counterpart. The C note confirms
    /// the buffer content "does not matter as we only used marked positions at end" — matching this port.
    fn setup_repdata(&mut self) {
        if self.repdata.len() < self.size {
            self.repdata.resize(self.size, 0);
        }
    }

    fn do_mark(&mut self, pos: usize, length: usize, replace: &[u8]) -> bool {
        if self.is_marked(pos, length) {
            return false;
        }
        self.setup_repdata();
        self.repdata[pos..pos + length].copy_from_slice(&replace[..length]);
        self.set_inspect_mark(pos, length);
        true
    }

    /// `cob_inspect_characters (f1)` (strings.c:538): TALLYING ... CHARACTERS (count unmarked) or
    /// REPLACING CHARACTERS BY (set repdata at unmarked positions).
    pub fn cob_inspect_characters(&mut self, f1: &mut [u8], f1_attr: &FieldAttr) {
        let pos = self.start;
        let inspect_len = self.end - self.start;
        if inspect_len == 0 {
            return;
        }
        if self.replacing {
            let repl_by = f1[0];
            if self.is_marked(pos, inspect_len) {
                for i in 0..inspect_len {
                    if self.mark[pos + i] == 0 {
                        self.repdata[pos + i] = repl_by;
                    }
                }
            } else {
                for b in &mut self.repdata[pos..pos + inspect_len] {
                    *b = repl_by;
                }
            }
        } else if self.is_marked(pos, inspect_len) {
            let n = self.mark[pos..pos + inspect_len].iter().filter(|&&m| m == 0).count() as i32;
            if n > 0 {
                add_int_field(f1, f1_attr, n);
            }
        } else {
            add_int_field(f1, f1_attr, inspect_len as i32);
        }
        self.set_inspect_mark(pos, inspect_len);
    }

    /// `inspect_common_no_replace (f1, f2, type, pos, inspect_len)` (strings.c:241): the TALLYING half —
    /// count non-overlapping `f2` matches in the window (marking to avoid double-count) and add the total
    /// to the counter `f1`.
    fn inspect_common_no_replace(&mut self, f1: &mut [u8], f1_attr: &FieldAttr, f2: &[u8], typ: InspectType) {
        let pos = self.start;
        let inspect_len = self.end - self.start;
        if inspect_len == 0 || f2.is_empty() || f2.len() > inspect_len {
            return;
        }
        let a = self.off;
        let f2s = f2.len();
        let mut n = 0i32;
        match typ {
            InspectType::Trailing => {
                let i_max = inspect_len - f2s;
                let mut first_marker = 0;
                let mut i = i_max as isize;
                loop {
                    let ii = i as usize;
                    if &self.data[a + self.start + ii..a + self.start + ii + f2s] == f2 {
                        if !self.is_marked(pos + ii, f2s) {
                            n += 1;
                            first_marker = ii;
                            i -= f2s as isize - 1;
                        }
                        if i == 0 {
                            break;
                        }
                    } else {
                        break;
                    }
                    i -= 1;
                    if i < 0 {
                        break;
                    }
                }
                if n != 0 {
                    self.set_inspect_mark(pos + first_marker, inspect_len - first_marker);
                }
            }
            InspectType::Leading => {
                let i_max = inspect_len - f2s + 1;
                let mut last_marker = 0;
                let mut i = 0;
                while i < i_max {
                    if &self.data[a + self.start + i..a + self.start + i + f2s] == f2 {
                        if !self.is_marked(pos + i, f2s) {
                            n += 1;
                            i += f2s - 1;
                            last_marker = i;
                        }
                    } else {
                        break;
                    }
                    i += 1;
                }
                if n != 0 {
                    self.set_inspect_mark(pos, last_marker);
                }
            }
            _ => {
                let i_max = inspect_len - f2s + 1;
                let mut i = 0;
                while i < i_max {
                    if &self.data[a + self.start + i..a + self.start + i + f2s] == f2 {
                        let checked = pos + i;
                        if !self.is_marked(checked, f2s) {
                            n += 1;
                            self.set_inspect_mark(checked, f2s);
                            if typ == InspectType::First {
                                break;
                            }
                            i += f2s - 1;
                        }
                    }
                    i += 1;
                }
            }
        }
        if n != 0 {
            add_int_field(f1, f1_attr, n);
        }
    }

    /// `inspect_common_replacing (f1, f2, type, pos, inspect_len)` (strings.c:340): the REPLACING half —
    /// `f1` is the BY value, deferred into `repdata` via `do_mark` at each non-overlapping `f2` match.
    fn inspect_common_replacing(&mut self, f1: &[u8], f2: &[u8], typ: InspectType) {
        let pos = self.start;
        let inspect_len = self.end - self.start;
        if inspect_len == 0 || f2.is_empty() || f2.len() > inspect_len {
            return;
        }
        let a = self.off;
        let f2s = f2.len();
        let repl = if f1.len() != f2s { alloc_figurative(f1, f2s) } else { f1.to_vec() };
        match typ {
            InspectType::Trailing => {
                let i_max = inspect_len - f2s;
                let mut i = i_max as isize;
                loop {
                    let ii = i as usize;
                    if &self.data[a + self.start + ii..a + self.start + ii + f2s] == f2 {
                        if self.do_mark(pos + ii, f2s, &repl) {
                            i -= f2s as isize - 1;
                        }
                        if i == 0 {
                            break;
                        }
                    } else {
                        break;
                    }
                    i -= 1;
                    if i < 0 {
                        break;
                    }
                }
            }
            InspectType::Leading => {
                let i_max = inspect_len - f2s + 1;
                let mut i = 0;
                while i < i_max {
                    if &self.data[a + self.start + i..a + self.start + i + f2s] == f2 {
                        if self.do_mark(pos + i, f2s, &repl) {
                            i += f2s - 1;
                        }
                    } else {
                        break;
                    }
                    i += 1;
                }
            }
            _ => {
                let i_max = inspect_len - f2s + 1;
                let mut i = 0;
                while i < i_max {
                    if &self.data[a + self.start + i..a + self.start + i + f2s] == f2
                        && self.do_mark(pos + i, f2s, &repl)
                    {
                        if typ == InspectType::First {
                            break;
                        }
                        i += f2s - 1;
                    }
                    i += 1;
                }
            }
        }
    }

    /// `cob_inspect_all/leading/first/trailing (f1, f2)` (strings.c:594-616). In TALLYING `f1` is the
    /// count receiver and `f2` the search; in REPLACING `f1` is the BY value and `f2` the search.
    pub fn cob_inspect_all(&mut self, f1: &mut [u8], f1_attr: &FieldAttr, f2: &[u8]) {
        self.inspect_common(f1, f1_attr, f2, InspectType::All);
    }
    pub fn cob_inspect_leading(&mut self, f1: &mut [u8], f1_attr: &FieldAttr, f2: &[u8]) {
        self.inspect_common(f1, f1_attr, f2, InspectType::Leading);
    }
    pub fn cob_inspect_first(&mut self, f1: &mut [u8], f1_attr: &FieldAttr, f2: &[u8]) {
        self.inspect_common(f1, f1_attr, f2, InspectType::First);
    }
    pub fn cob_inspect_trailing(&mut self, f1: &mut [u8], f1_attr: &FieldAttr, f2: &[u8]) {
        self.inspect_common(f1, f1_attr, f2, InspectType::Trailing);
    }

    /// `inspect_common (f1, f2, type)` (strings.c:392): the shared dispatcher for the
    /// ALL/LEADING/FIRST/TRAILING verbs — short-circuit on an empty window, then route to the TALLYING
    /// (`no_replace`) or REPLACING half (the C uses the module-global `inspect_replacing`). The
    /// `f2->size > inspect_len` precondition is enforced inside each half (kept there for safety under
    /// `#![forbid(unsafe_code)]`).
    fn inspect_common(&mut self, f1: &mut [u8], f1_attr: &FieldAttr, f2: &[u8], typ: InspectType) {
        if self.end - self.start == 0 {
            return;
        }
        if self.replacing {
            self.inspect_common_replacing(f1, f2, typ);
        } else {
            self.inspect_common_no_replace(f1, f1_attr, f2, typ);
        }
    }

    /// `cob_inspect_converting (f1, f2)` (strings.c:619): translate every byte in the window per the
    /// `CONVERTING f1 TO f2` table.
    pub fn cob_inspect_converting(&mut self, f1: &[u8], f2: &[u8]) {
        let inspect_len = self.end - self.start;
        if inspect_len == 0 {
            return;
        }
        let f2v = if f1.len() != f2.len() { alloc_figurative(f2, f1.len()) } else { f2.to_vec() };
        let mut conv_set = [false; 256];
        let mut conv_tab = [0u8; 256];
        for k in 0..f1.len() {
            let from = f1[k] as usize;
            if !conv_set[from] {
                conv_set[from] = true;
                conv_tab[from] = f2v[k];
            }
        }
        let a = self.off;
        for i in self.start..self.end {
            let c = self.data[a + i] as usize;
            if conv_set[c] {
                self.data[a + i] = conv_tab[c];
            }
        }
    }

    /// `cob_inspect_finish ()` (strings.c:706): apply the deferred REPLACING data and restore the sign.
    pub fn cob_inspect_finish(&mut self) {
        if self.replacing && self.mark[self.mark_min] != 0 {
            let a = self.off;
            for i in self.mark_min..=self.mark_max {
                if self.mark[i] != 0 {
                    self.data[a + i] = self.repdata[i];
                }
            }
        }
        if self.var_attr.have_sign() && !self.var_attr.sign_separate() && self.var_attr.field_type == COB_TYPE_NUMERIC_DISPLAY {
            crate::sign::display_put_sign(&mut self.data, &self.var_attr, self.sign);
        }
    }

    /// The inspected variable's bytes after the INSPECT.
    pub fn result(&self) -> &[u8] {
        &self.data
    }
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
    fn inspect_tally_replace_convert() {
        let a = alnum(5);
        let cnt_attr = FieldAttr { field_type: crate::attr::COB_TYPE_NUMERIC_BINARY, digits: 9, scale: 0, flags: crate::attr::COB_FLAG_HAVE_SIGN | crate::attr::COB_FLAG_REAL_BINARY };

        // TALLYING n FOR ALL "A" in "AABAA" -> 4
        let mut ins = CobInspect::cob_inspect_init(b"AABAA", &a, false);
        ins.cob_inspect_start();
        let mut n = 0i32.to_le_bytes();
        ins.cob_inspect_all(&mut n, &cnt_attr, b"A");
        ins.cob_inspect_finish();
        assert_eq!(crate::accessors::cob_get_int(&n, &cnt_attr), 4);

        // TALLYING LEADING "A" in "AABAA" -> 2
        let mut ins = CobInspect::cob_inspect_init(b"AABAA", &a, false);
        ins.cob_inspect_start();
        let mut n = 0i32.to_le_bytes();
        ins.cob_inspect_leading(&mut n, &cnt_attr, b"A");
        assert_eq!(crate::accessors::cob_get_int(&n, &cnt_attr), 2);

        // REPLACING ALL "A" BY "X" in "AABAA" -> "XXBXX"
        let mut ins = CobInspect::cob_inspect_init(b"AABAA", &a, true);
        ins.cob_inspect_start();
        let mut by = *b"X";
        ins.cob_inspect_all(&mut by, &a, b"A");
        ins.cob_inspect_finish();
        assert_eq!(ins.result(), b"XXBXX");

        // CONVERTING "ABC" TO "XYZ" in "CAB" -> "ZXY"
        let mut ins = CobInspect::cob_inspect_init_converting(b"CAB", &alnum(3));
        ins.cob_inspect_start();
        ins.cob_inspect_converting(b"ABC", b"XYZ");
        ins.cob_inspect_finish();
        assert_eq!(ins.result(), b"ZXY");

        // REPLACING ALL "A" BY "Z" AFTER "X" in "AXAA" -> only A's after X: "AXZZ"
        let mut ins = CobInspect::cob_inspect_init(b"AXAA", &alnum(4), true);
        ins.cob_inspect_start();
        ins.cob_inspect_after(b"X");
        let mut by = *b"Z";
        ins.cob_inspect_all(&mut by, &alnum(4), b"A");
        ins.cob_inspect_finish();
        assert_eq!(ins.result(), b"AXZZ");

        // REPLACING CHARACTERS BY "*" BEFORE "B" in "AABAA" -> "**BAA"
        let mut ins = CobInspect::cob_inspect_init(b"AABAA", &a, true);
        ins.cob_inspect_start();
        ins.cob_inspect_before(b"B");
        let mut star = *b"*";
        ins.cob_inspect_characters(&mut star, &a);
        ins.cob_inspect_finish();
        assert_eq!(ins.result(), b"**BAA");
    }

    #[test]
    fn inspect_crosscheck_against_sealed_court() {
        // CobInspect is the 1:1 stateful port of strings.c's INSPECT functions; inspect.rs is the
        // result-oriented byte-effect court already sealed against the GnuCOBOL 3.2 oracle. Driving a
        // broad grid through both and asserting identical results transitively oracle-verifies the port.
        use crate::inspect::{self, Region, TallyMode, ReplaceMode};
        let cnt_attr = FieldAttr {
            field_type: crate::attr::COB_TYPE_NUMERIC_BINARY,
            digits: 9,
            scale: 0,
            flags: crate::attr::COB_FLAG_HAVE_SIGN | crate::attr::COB_FLAG_REAL_BINARY,
        };
        let targets: &[&[u8]] = &[b"AABAA", b"XAXAAX", b"ABCABC", b"BBBBB", b"AXAA", b"A", b"ABAB"];
        let pats: &[&[u8]] = &[b"A", b"AB", b"X", b"AA"];
        let mut cases = 0u64;
        for &t in targets {
            let a = alnum(t.len());
            let regions: &[(Option<&[u8]>, Option<&[u8]>)] =
                &[(None, None), (Some(b"X"), None), (None, Some(b"X")), (Some(b"B"), None), (None, Some(b"B"))];
            for &(before, after) in regions {
                let r = match (before, after) {
                    (Some(d), None) => Region::Before(d),
                    (None, Some(d)) => Region::After(d),
                    _ => Region::All,
                };
                let setup = |ins: &mut CobInspect| {
                    ins.cob_inspect_start();
                    if let Some(d) = before { ins.cob_inspect_before(d); }
                    if let Some(d) = after { ins.cob_inspect_after(d); }
                };
                for &p in pats {
                    // TALLYING ALL
                    let mut ins = CobInspect::cob_inspect_init(t, &a, false);
                    setup(&mut ins);
                    let mut n = 0i32.to_le_bytes();
                    ins.cob_inspect_all(&mut n, &cnt_attr, p);
                    ins.cob_inspect_finish();
                    assert_eq!(crate::accessors::cob_get_int(&n, &cnt_attr) as u64,
                        inspect::inspect_tallying(t, TallyMode::All(p), r), "tally ALL {t:?} {p:?} {before:?}/{after:?}");
                    // TALLYING LEADING
                    let mut ins = CobInspect::cob_inspect_init(t, &a, false);
                    setup(&mut ins);
                    let mut n = 0i32.to_le_bytes();
                    ins.cob_inspect_leading(&mut n, &cnt_attr, p);
                    ins.cob_inspect_finish();
                    assert_eq!(crate::accessors::cob_get_int(&n, &cnt_attr) as u64,
                        inspect::inspect_tallying(t, TallyMode::Leading(p), r), "tally LEADING {t:?} {p:?}");
                    // REPLACING ALL (equal-length BY)
                    let by = vec![b'*'; p.len()];
                    let mut ins = CobInspect::cob_inspect_init(t, &a, true);
                    setup(&mut ins);
                    let mut byb = by.clone();
                    ins.cob_inspect_all(&mut byb, &alnum(by.len()), p);
                    ins.cob_inspect_finish();
                    assert_eq!(ins.result(), inspect::inspect_replacing(t, ReplaceMode::All(p, &by), r).as_slice(),
                        "repl ALL {t:?} {p:?}");
                    // REPLACING LEADING
                    let mut ins = CobInspect::cob_inspect_init(t, &a, true);
                    setup(&mut ins);
                    let mut byb = by.clone();
                    ins.cob_inspect_leading(&mut byb, &alnum(by.len()), p);
                    ins.cob_inspect_finish();
                    assert_eq!(ins.result(), inspect::inspect_replacing(t, ReplaceMode::Leading(p, &by), r).as_slice(),
                        "repl LEADING {t:?} {p:?}");
                    // REPLACING FIRST
                    let mut ins = CobInspect::cob_inspect_init(t, &a, true);
                    setup(&mut ins);
                    let mut byb = by.clone();
                    ins.cob_inspect_first(&mut byb, &alnum(by.len()), p);
                    ins.cob_inspect_finish();
                    assert_eq!(ins.result(), inspect::inspect_replacing(t, ReplaceMode::First(p, &by), r).as_slice(),
                        "repl FIRST {t:?} {p:?}");
                    cases += 5;
                }
                // TALLYING CHARACTERS
                let mut ins = CobInspect::cob_inspect_init(t, &a, false);
                setup(&mut ins);
                let mut n = 0i32.to_le_bytes();
                ins.cob_inspect_characters(&mut n, &cnt_attr);
                ins.cob_inspect_finish();
                assert_eq!(crate::accessors::cob_get_int(&n, &cnt_attr) as u64,
                    inspect::inspect_tallying(t, TallyMode::Characters, r), "tally CHARS {t:?} {before:?}/{after:?}");
                cases += 1;
            }
            // CONVERTING over the whole target
            for &(from, to) in &[(&b"AB"[..], &b"XY"[..]), (&b"ABC"[..], &b"123"[..]), (&b"X"[..], &b"Q"[..])] {
                let mut ins = CobInspect::cob_inspect_init_converting(t, &a);
                ins.cob_inspect_start();
                ins.cob_inspect_converting(from, to);
                ins.cob_inspect_finish();
                assert_eq!(ins.result(), inspect::inspect_converting(t, from, to, Region::All).as_slice(),
                    "convert {t:?} {from:?}->{to:?}");
                cases += 1;
            }
        }
        assert!(cases > 200, "expected a broad grid, got {cases}");
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

    fn cnt_attr() -> FieldAttr {
        FieldAttr { field_type: crate::attr::COB_TYPE_NUMERIC_BINARY, digits: 9, scale: 0, flags: crate::attr::COB_FLAG_HAVE_SIGN | crate::attr::COB_FLAG_REAL_BINARY }
    }

    #[test]
    fn inspect_trailing_tallies_only_trailing_run() {
        // INSPECT "AABAA" TALLYING n FOR TRAILING "A" -> 2 (the two trailing A's only).
        let a = alnum(5);
        let ca = cnt_attr();
        let mut ins = CobInspect::cob_inspect_init(b"AABAA", &a, false);
        ins.cob_inspect_start();
        let mut n = 0i32.to_le_bytes();
        ins.cob_inspect_trailing(&mut n, &ca, b"A");
        ins.cob_inspect_finish();
        assert_eq!(crate::accessors::cob_get_int(&n, &ca), 2);
    }

    #[test]
    fn do_mark_records_replacement_and_rejects_overlap() {
        // do_mark defers a replacement into repdata at an unmarked position and reports true;
        // a second call into the now-marked range reports false. cob_inspect_finish applies repdata.
        let a = alnum(4);
        let mut ins = CobInspect::cob_inspect_init(b"AAAA", &a, true);
        ins.cob_inspect_start();
        assert!(ins.do_mark(1, 2, b"XY"), "first mark of fresh range");
        assert!(!ins.do_mark(1, 2, b"ZZ"), "overlapping mark rejected");
        ins.cob_inspect_finish();
        assert_eq!(ins.result(), b"AXYA");
    }

    #[test]
    fn inspect_common_no_replace_counts_all_matches() {
        // The TALLYING half called directly: count ALL "A" in "AABAA" -> 4.
        let a = alnum(5);
        let ca = cnt_attr();
        let mut ins = CobInspect::cob_inspect_init(b"AABAA", &a, false);
        ins.cob_inspect_start();
        let mut n = 0i32.to_le_bytes();
        ins.inspect_common_no_replace(&mut n, &ca, b"A", InspectType::All);
        assert_eq!(crate::accessors::cob_get_int(&n, &ca), 4);
    }

    #[test]
    fn inspect_common_replacing_defers_all_matches() {
        // The REPLACING half called directly: ALL "A" BY "X" in "AABAA" -> "XXBXX".
        let a = alnum(5);
        let mut ins = CobInspect::cob_inspect_init(b"AABAA", &a, true);
        ins.cob_inspect_start();
        ins.inspect_common_replacing(b"X", b"A", InspectType::All);
        ins.cob_inspect_finish();
        assert_eq!(ins.result(), b"XXBXX");
    }

    #[test]
    fn unstring_tallying_adds_receiver_count() {
        // After two cob_unstring_into calls, cob_unstring_tallying adds 2 to the existing TALLYING value.
        let a3 = alnum(3);
        let ca = cnt_attr();
        let mut u = CobUnstring::cob_unstring_init(b"AB,CD", None, 1);
        u.cob_unstring_delimited(b",", false);
        let mut d = vec![b' '; 3];
        u.cob_unstring_into(&mut d, &a3, None, None);
        u.cob_unstring_into(&mut d, &a3, None, None);
        let mut tal = 10i32.to_le_bytes();
        u.cob_unstring_tallying(&mut tal, &ca);
        assert_eq!(crate::accessors::cob_get_int(&tal, &ca), 12);
    }
}
