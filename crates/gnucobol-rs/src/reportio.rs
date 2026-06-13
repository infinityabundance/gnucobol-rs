//! Port of reportio.c — the COBOL Report Writer Control System (RWCS): `INITIATE` / `GENERATE` /
//! `TERMINATE`, control breaks, SUM counters, page/line headings and footings. The C drives a large
//! compiler-built linked structure (`cob_report` → lines tree → fields / controls / sums) and writes
//! report lines to a file. This is a **subsystem** ported incrementally; this module currently holds the
//! standalone field/arithmetic helpers + the `COB_REPORT_*` line-flag vocabulary. The RWCS state machine
//! over the full `cob_report` model is the substantial remainder (a dedicated effort with a Report-Writer
//! program oracle); it is NOT yet ported, and `gnucobol-rs-port-index` reflects the honest partial.
#![forbid(unsafe_code)]

use crate::attr::{FieldAttr, COB_TYPE_ALPHANUMERIC, COB_TYPE_NUMERIC_DISPLAY};

/// `COB_REPORT_*` line/field flags (`common.h:1030-1064`) — the bit vocabulary the RWCS classifies lines
/// and fields with (REPORT/PAGE/CONTROL HEADING & FOOTING, DETAIL, LINE/PLUS, NEXT GROUP, …).
pub mod flags {
    pub const LINE: u32 = 1 << 0;
    pub const LINE_PLUS: u32 = 1 << 1;
    pub const COLUMN_PLUS: u32 = 1 << 2;
    pub const RESET_FINAL: u32 = 1 << 3;
    pub const HEADING: u32 = 1 << 4;
    pub const FOOTING: u32 = 1 << 5;
    pub const PAGE_HEADING: u32 = 1 << 6;
    pub const PAGE_FOOTING: u32 = 1 << 7;
    pub const CONTROL_HEADING: u32 = 1 << 8;
    pub const CONTROL_HEADING_FINAL: u32 = 1 << 9;
    pub const CONTROL_FOOTING: u32 = 1 << 10;
    pub const CONTROL_FOOTING_FINAL: u32 = 1 << 11;
    pub const DETAIL: u32 = 1 << 12;
    pub const NEXT_GROUP_LINE: u32 = 1 << 13;
    pub const NEXT_GROUP_PLUS: u32 = 1 << 14;
    pub const NEXT_GROUP_PAGE: u32 = 1 << 15;
    pub const LINE_NEXT_PAGE: u32 = 1 << 16;
    pub const NEXT_PAGE: u32 = 1 << 17;
    pub const GROUP_INDICATE: u32 = 1 << 18;
    pub const GROUP_ITEM: u32 = 1 << 19;
    pub const HAD_WHEN: u32 = 1 << 20;
    pub const COLUMN_LEFT: u32 = 1 << 21;
    pub const COLUMN_CENTER: u32 = 1 << 22;
    pub const COLUMN_RIGHT: u32 = 1 << 23;
    pub const PRESENT: u32 = 1 << 24;
    pub const BEFORE: u32 = 1 << 25;
    pub const PAGE: u32 = 1 << 26;
    pub const ALL: u32 = 1 << 27;
    pub const NEGATE: u32 = 1 << 28;
    pub const SUM_EMITTED: u32 = 1 << 29;
    pub const LINE_EMITTED: u32 = 1 << 30;
    pub const REF_EMITTED: u32 = 1 << 31;

    /// `ND1|ND2|ND3` (reportio.c:62-65): the non-DETAIL heading/footing flags — `NOTDETAIL(f)`.
    pub const NOT_DETAIL: u32 = HEADING
        | FOOTING
        | PAGE_HEADING
        | PAGE_FOOTING
        | CONTROL_HEADING
        | CONTROL_HEADING_FINAL
        | CONTROL_FOOTING
        | CONTROL_FOOTING_FINAL;
}

/// `NOTDETAIL(f)` (reportio.c:65): the line is a heading/footing (not a plain DETAIL line).
pub fn not_detail(line_flags: u32) -> bool {
    line_flags & flags::NOT_DETAIL != 0
}

/// `cob_str_move (dst, src, size)` (reportio.c:78): `MOVE` `size` alphanumeric bytes of `src` into the
/// `dst` field (so the receiver's editing/padding applies).
pub fn cob_str_move(dst: &mut [u8], dst_attr: &FieldAttr, src: &[u8], size: usize) {
    let sattr = FieldAttr { field_type: COB_TYPE_ALPHANUMERIC, digits: 0, scale: 0, flags: 0 };
    let n = size.min(src.len());
    let _ = crate::move_ops::cob_move(&src[..n], &sattr, dst, dst_attr);
}

/// `cob_field_init (f)` (reportio.c:92): `MOVE ZERO` to a numeric field, `MOVE SPACES` to an alphanumeric
/// one — the RWCS "reset to empty" used before re-summing / re-printing.
pub fn cob_field_init(data: &mut [u8], attr: &FieldAttr) {
    if attr.is_numeric() {
        let sattr = FieldAttr { field_type: crate::attr::COB_TYPE_NUMERIC, digits: 0, scale: 0, flags: 0 };
        let _ = crate::move_ops::cob_move(b"0", &sattr, data, attr);
    } else {
        let sattr = FieldAttr { field_type: COB_TYPE_ALPHANUMERIC, digits: 0, scale: 0, flags: 0 };
        let _ = crate::move_ops::cob_move(b" ", &sattr, data, attr);
    }
}

/// `cob_field_dup (f, incr)` (reportio.c:115): allocate a new field the same format as `f` but `incr`
/// bytes larger, initialised to ZERO/SPACES. Returns the owned `(data, attr)` (Rust RAII replaces the
/// `cob_malloc`; [`cob_field_free`] is the matching no-op drop).
pub fn cob_field_dup(attr: &FieldAttr, size: usize, incr: i32) -> (Vec<u8>, FieldAttr) {
    let dsize = (size as i32 + incr).max(0) as usize;
    let mut data = vec![0u8; dsize];
    cob_field_init(&mut data, attr);
    (data, *attr)
}

/// `cob_field_free (f)` (reportio.c:141): free a [`cob_field_dup`] allocation. A no-op under Rust RAII
/// (the owned `Vec` drops); kept as the named 1:1 counterpart.
pub fn cob_field_free(_data: Vec<u8>) {}

/// `cob_add_fields (op1, op2, rslt)` (reportio.c:299): add two numeric fields giving `rslt`. The C copies
/// each operand to a local `PIC 9 DISPLAY` (because `cob_add` handles `NUMERIC EDITED` poorly), adds, then
/// `MOVE`s the sum into `rslt` (which applies any editing). This port mirrors that: convert via DISPLAY,
/// add through the sealed decimal path, then `cob_move` into the receiver.
pub fn cob_add_fields(
    op1: &[u8],
    a1: &FieldAttr,
    op2: &[u8],
    a2: &FieldAttr,
    rslt: &mut [u8],
    rslt_attr: &FieldAttr,
) {
    let d1 = display_of(op1, a1);
    let d2 = display_of(op2, a2);
    let d1attr = FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: a1.digits, scale: a1.scale, flags: a1.flags };
    let d2attr = FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: a2.digits, scale: a2.scale, flags: a2.flags };
    if let Ok(sum) = crate::cob_decimal::cob_add(&d1, &d1attr, &d2, &d2attr, crate::arith::Round::Truncate) {
        let _ = crate::move_ops::cob_move(&sum, &d1attr, rslt, rslt_attr);
    }
}

/// Convert a numeric field to its `PIC 9 DISPLAY` image (the `cob_add_fields` intermediate).
fn display_of(data: &[u8], attr: &FieldAttr) -> Vec<u8> {
    let dattr = FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: attr.digits, scale: attr.scale, flags: attr.flags };
    let size = attr.digits as usize + if attr.have_sign() && attr.sign_separate() { 1 } else { 0 };
    let mut out = vec![b'0'; size.max(1)];
    let _ = crate::move_ops::cob_move(data, attr, &mut out, &dattr);
    out
}

// ---- cob_report data model (the compiler-built RD tables; common.h:1429-1545) ------------------
// The C uses linked lists (`sister`/`child` line tree, `next` field/control/sum chains). This port
// models the line tree with nested `Vec`s; cross-references that the C resolves by pointer identity are
// indices. Only the parts the ported operations touch are modelled (grown as the port advances).

/// A field on a report line (`cob_report_field`; common.h:1429): the RWCS flags + the per-field state.
#[derive(Debug, Clone, Default)]
pub struct ReportField {
    pub flags: u32,
    pub line: i32,
    pub column: i32,
    pub level: u8,
    pub group_indicate: bool,
    pub suppress: bool,
    pub present_now: bool,
}

/// A report line (`cob_report_line`; common.h:1448): its fields + the nested child lines (the C's
/// `child`) — sibling lines (`sister`) are the slice elements at one level.
#[derive(Debug, Clone, Default)]
pub struct ReportLine {
    pub fields: Vec<ReportField>,
    pub children: Vec<ReportLine>,
    pub flags: u32,
    pub line: i32,
    pub next_group_line: i32,
    pub suppress: bool,
}

/// A SUM counter (`cob_report_sum_ctr`; common.h:1474): the running `counter` field plus the numeric
/// fields summed into it.
#[derive(Debug, Clone)]
pub struct ReportSumCtr {
    pub name: String,
    pub counter: Vec<u8>,
    pub counter_attr: FieldAttr,
    /// The fields summed into `counter` (`sum->f` chain).
    pub sum_values: Vec<(Vec<u8>, FieldAttr)>,
    pub subtotal: bool,
}

/// `clear_group_indicate (l)` (reportio.c:192): clear the `group_indicate` flag on every field of the
/// line tree.
pub fn clear_group_indicate(lines: &mut [ReportLine]) {
    for l in lines.iter_mut() {
        for f in &mut l.fields {
            f.group_indicate = false;
        }
        clear_group_indicate(&mut l.children);
    }
}

/// `clear_suppress (l)` (reportio.c:208): clear the `suppress` flag on each line and its non-GROUP-ITEM
/// fields, recursively.
pub fn clear_suppress(lines: &mut [ReportLine]) {
    for l in lines.iter_mut() {
        l.suppress = false;
        for f in &mut l.fields {
            if f.flags & flags::GROUP_ITEM != 0 {
                continue;
            }
            f.suppress = false;
        }
        clear_suppress(&mut l.children);
    }
}

/// `get_print_line (l)` (reportio.c:274): descend through field-less single-child lines to the line that
/// actually carries data fields.
pub fn get_print_line(line: &ReportLine) -> &ReportLine {
    let mut l = line;
    while l.fields.is_empty() && l.children.len() == 1 {
        l = &l.children[0];
    }
    l
}

/// `get_line_type (r, l, type)` (reportio.c:1142): the first line in the tree (this line, then its child
/// subtree, then its siblings) whose `flags` include `type` — or `None`.
pub fn get_line_type(lines: &[ReportLine], type_: u32) -> Option<&ReportLine> {
    for l in lines {
        if l.flags & type_ != 0 {
            return Some(l);
        }
        if let Some(t) = get_line_type(&l.children, type_) {
            return Some(t);
        }
    }
    None
}

/// `sum_all_detail (r)` (reportio.c:1203): add every (non-subtotal) SUM counter's summed fields into its
/// running counter — the per-`GENERATE` detail accumulation.
pub fn sum_all_detail(sum_counters: &mut [ReportSumCtr]) {
    for sc in sum_counters.iter_mut() {
        if sc.subtotal {
            continue;
        }
        for (val, vattr) in sc.sum_values.clone() {
            let (data, attr) = (std::mem::take(&mut sc.counter), sc.counter_attr);
            let mut out = vec![0u8; data.len()];
            cob_add_fields(&data, &attr, &val, &vattr, &mut out, &attr);
            sc.counter = out;
        }
    }
}

/// `reportInitialize ()` (reportio.c:287): one-time global RWCS init. A no-op in this port (the C sets a
/// `bDidReportInit` guard + `inDetailDecl`; there is no global state to seed here).
pub fn report_initialize() {}

/// `cob_init_reportio (lptr, sptr)` (reportio.c): module init binding the runtime globals. A no-op here.
pub fn cob_init_reportio() {}

/// `cob_exit_reportio ()` (reportio.c): module teardown freeing active reports. A no-op (RAII).
pub fn cob_exit_reportio() {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attr::COB_FLAG_HAVE_SIGN;

    #[test]
    fn str_move_and_field_init() {
        let dattr = FieldAttr { field_type: COB_TYPE_ALPHANUMERIC, digits: 0, scale: 0, flags: 0 };
        let mut dst = [0u8; 5];
        cob_str_move(&mut dst, &dattr, b"HI", 2);
        assert_eq!(&dst, b"HI   "); // space-padded
        // numeric init -> ZERO
        let nattr = FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: 3, scale: 0, flags: 0 };
        let mut num = [b'9'; 3];
        cob_field_init(&mut num, &nattr);
        assert_eq!(&num, b"000");
        // alnum init -> SPACES
        let mut al = [b'X'; 4];
        cob_field_init(&mut al, &dattr);
        assert_eq!(&al, b"    ");
    }

    #[test]
    fn add_fields_sums_into_receiver() {
        let a = FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: 4, scale: 0, flags: 0 };
        let mut rslt = [b'0'; 4];
        cob_add_fields(b"0012", &a, b"0030", &a, &mut rslt, &a);
        assert_eq!(&rslt, b"0042");
        // scaled add: 12.34 + 1.66 = 14.00
        let s = FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: 4, scale: 2, flags: 0 };
        let mut r2 = [b'0'; 4];
        cob_add_fields(b"1234", &s, b"0166", &s, &mut r2, &s);
        assert_eq!(&r2, b"1400");
    }

    #[test]
    fn tree_walk_and_sum_accumulation() {
        // line tree: a heading line (no fields, one child detail line with fields)
        let detail = ReportLine {
            fields: vec![
                ReportField { flags: 0, group_indicate: true, suppress: true, ..Default::default() },
                ReportField { flags: flags::GROUP_ITEM, suppress: true, ..Default::default() },
            ],
            flags: flags::DETAIL,
            ..Default::default()
        };
        let mut lines = vec![ReportLine {
            fields: vec![],
            children: vec![detail],
            flags: flags::PAGE_HEADING,
            ..Default::default()
        }];
        // get_print_line descends to the detail line
        assert_eq!(get_print_line(&lines[0]).flags, flags::DETAIL);
        // get_line_type finds the heading (this line) and the detail (child)
        assert_eq!(get_line_type(&lines, flags::PAGE_HEADING).unwrap().flags, flags::PAGE_HEADING);
        assert_eq!(get_line_type(&lines, flags::DETAIL).unwrap().flags, flags::DETAIL);
        assert!(get_line_type(&lines, flags::CONTROL_HEADING).is_none());
        // clear_group_indicate clears the flag everywhere
        clear_group_indicate(&mut lines);
        assert!(!lines[0].children[0].fields[0].group_indicate);
        // clear_suppress clears line + non-GROUP_ITEM fields, but skips GROUP_ITEM fields
        lines[0].children[0].suppress = true;
        clear_suppress(&mut lines);
        assert!(!lines[0].children[0].suppress);
        assert!(!lines[0].children[0].fields[0].suppress); // plain field cleared
        assert!(lines[0].children[0].fields[1].suppress); // GROUP_ITEM field NOT cleared

        // SUM accumulation: counter 0000 += 0010 + 0020 + 0030 = 0060
        let a = FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: 4, scale: 0, flags: 0 };
        let mut ctrs = vec![ReportSumCtr {
            name: "C".into(),
            counter: b"0000".to_vec(),
            counter_attr: a,
            sum_values: vec![(b"0010".to_vec(), a), (b"0020".to_vec(), a), (b"0030".to_vec(), a)],
            subtotal: false,
        }];
        sum_all_detail(&mut ctrs);
        assert_eq!(ctrs[0].counter, b"0060");
        // a second GENERATE accumulates further: 0060 + 60 = 0120
        sum_all_detail(&mut ctrs);
        assert_eq!(ctrs[0].counter, b"0120");
    }

    #[test]
    fn dup_and_flags() {
        let a = FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: 3, scale: 0, flags: COB_FLAG_HAVE_SIGN };
        let (data, attr) = cob_field_dup(&a, 3, 2);
        assert_eq!(data.len(), 5);
        assert_eq!(attr.digits, 3);
        cob_field_free(data);
        assert!(not_detail(flags::PAGE_HEADING));
        assert!(not_detail(flags::CONTROL_FOOTING));
        assert!(!not_detail(flags::DETAIL));
    }
}
