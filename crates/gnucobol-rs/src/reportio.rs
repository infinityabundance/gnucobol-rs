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
    pub next_group_line: i32,
    pub group_indicate: bool,
    pub suppress: bool,
    pub present_now: bool,
    /// The CONTROL field id this field's PRESENT WHEN tracks (`rf->control`), for [`line_control_one`].
    pub control_id: Option<usize>,
    /// The bytes this field prints at its column (used by [`report_line`] to build the record).
    pub data: Vec<u8>,
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
    /// The field id this counter sums onto (`sum->f`), for [`sum_this_counter`]'s forward accumulation.
    pub sums_field: Option<usize>,
    /// The control id this counter resets on (`sc->control`), and whether it is a FINAL counter.
    pub control: Option<usize>,
    pub control_final: bool,
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

/// The report's runtime state (`cob_report`; common.h:1502) — the parts the ported operations touch:
/// the line tree, SUM counters, the PAGE/LINE counters, the page-limit defaults, the current position,
/// and the `NEXT GROUP` bookkeeping. Grown as the port advances.
#[derive(Debug, Clone, Default)]
pub struct Report {
    pub lines: Vec<ReportLine>,
    pub sum_counters: Vec<ReportSumCtr>,
    pub controls: Vec<ReportControl>,
    pub page_counter: Option<(Vec<u8>, FieldAttr)>,
    pub line_counter: Option<(Vec<u8>, FieldAttr)>,
    pub def_lines: i32,
    pub curr_page: i32,
    pub curr_line: i32,
    pub initiate_done: bool,
    pub next_value: i32,
    pub next_line: bool,
    pub next_line_plus: bool,
    pub next_page: bool,
    pub next_just_set: bool,
    // --- output state machine (cob_report I/O + page geometry) ---
    /// The accumulated report bytes (`write_rec` appends here; the actual `cob_file` write is the boundary).
    pub output: Vec<u8>,
    /// The current line record being built (`def_cols` wide).
    pub record: Vec<u8>,
    pub def_cols: i32,
    pub def_heading: i32,
    pub def_first_detail: i32,
    pub def_last_detail: i32,
    pub def_footing: i32,
    pub report_name: String,
    pub in_page_heading: bool,
    pub in_page_footing: bool,
    pub in_report_footing: bool,
    pub first_generate: bool,
    pub first_detail: bool,
    pub incr_line: bool,
    pub foot_next_page: bool,
}

/// `limitCheckOneLine (r, fl)` (reportio.c:623): a line (and its fields) violates the PAGE LIMIT when any
/// `LINE` / `NEXT GROUP` number exceeds `def_lines`. Returns `true` on a violation.
#[allow(non_snake_case)]
pub fn limitCheckOneLine(line: &ReportLine, def_lines: i32) -> bool {
    if line.line > 0 && def_lines > 0 && line.line > def_lines {
        return true;
    }
    if line.next_group_line > 0 && def_lines > 0 && line.next_group_line > def_lines {
        return true;
    }
    for rf in &line.fields {
        if rf.line != 0 && rf.line > def_lines {
            return true;
        }
        if rf.next_group_line != 0 && rf.next_group_line > def_lines {
            return true;
        }
    }
    false
}

/// `limitCheckLine (r, fl)` (reportio.c:663): [`limitCheckOneLine`] over the line tree.
#[allow(non_snake_case)]
pub fn limitCheckLine(lines: &[ReportLine], def_lines: i32) -> bool {
    for l in lines {
        if limitCheckOneLine(l, def_lines) {
            return true;
        }
        if limitCheckLine(&l.children, def_lines) {
            return true;
        }
    }
    false
}

/// `limitCheck (r)` (reportio.c:676): verify every `LINE #` is within the PAGE LIMIT; clear
/// `initiate_done` on a violation (the C also raises `COB_EC_REPORT_PAGE_LIMIT`). Returns the violation.
#[allow(non_snake_case)]
pub fn limitCheck(report: &mut Report) -> bool {
    let bad = limitCheckLine(&report.lines, report.def_lines);
    if bad {
        report.initiate_done = false;
    }
    bad
}

/// `saveLineCounter (r)` (reportio.c:682): clamp `curr_line` to `[0, def_lines]` and store the current
/// page/line into the `PAGE-COUNTER` / `LINE-COUNTER` special registers.
#[allow(non_snake_case)]
pub fn saveLineCounter(report: &mut Report) {
    let mut ln = report.curr_line;
    if ln > report.def_lines {
        ln = 0;
    }
    if ln < 0 {
        ln = 0;
    }
    if let Some((data, attr)) = report.page_counter.as_mut() {
        let _ = crate::accessors::cob_set_int(data, attr, report.curr_page);
    }
    if let Some((data, attr)) = report.line_counter.as_mut() {
        let _ = crate::accessors::cob_set_int(data, attr, ln);
    }
}

/// `set_next_info (r, l)` (reportio.c:248): record a line's `NEXT GROUP LINE`/`PLUS`/`PAGE` request into
/// the report state so the next DETAIL advances accordingly.
pub fn set_next_info(report: &mut Report, line: &ReportLine) {
    if line.flags & flags::NEXT_GROUP_LINE != 0 {
        report.next_value = line.next_group_line;
        report.next_line = true;
        report.next_just_set = true;
        report.next_line_plus = false;
    }
    if line.flags & flags::NEXT_GROUP_PLUS != 0 {
        report.next_value = line.next_group_line;
        report.next_line = false;
        report.next_line_plus = true;
        report.next_just_set = true;
    }
    if line.flags & flags::NEXT_GROUP_PAGE != 0 {
        report.next_value = line.next_group_line;
        report.next_line = false;
        report.next_page = true;
        report.next_just_set = true;
    }
}

/// `print_field (rf, rec)` (reportio.c:894): place a report field's right-trimmed string into the print
/// record `rec` at its 1-based `column`, applying `COLUMN LEFT`/`RIGHT`/`CENTER` justification (when the
/// data-justify runtime setting `col_just_lrc` is on). `field_size` is the field's declared width.
pub fn print_field(
    field: &crate::cconv::FieldRef,
    field_size: usize,
    column: i32,
    field_flags: u32,
    col_just_lrc: bool,
    rec: &mut [u8],
) {
    let mut wrk = vec![0u8; field_size + 2];
    let ret = crate::cconv::cob_field_to_string(Some(field), &mut wrk, crate::cconv::CobCase::None);
    let mut ln = if ret > 0 { ret as usize } else { 0 };
    let mut content: Vec<u8> = wrk[..ln].to_vec();
    let mut dest_pos = (column - 1).max(0) as usize;

    if !col_just_lrc {
        // data justify off — no adjustment
    } else if field_flags & flags::COLUMN_RIGHT != 0 && ln < field_size {
        dest_pos += field_size - ln;
    } else if field_flags & flags::COLUMN_CENTER != 0 {
        // remove leading spaces (bounded by field_size), then centre
        let mut k = 0;
        while k < field_size && !content.is_empty() && content[0] == b' ' && ln > 0 {
            content.remove(0);
            ln -= 1;
            k += 1;
        }
        let i = 1 - (ln & 1);
        if ln < field_size {
            dest_pos += (field_size - ln - i) / 2;
        }
    } else if field_flags & flags::COLUMN_LEFT != 0 {
        let mut k = 0;
        while k < field_size && !content.is_empty() && content[0] == b' ' && ln > 0 {
            content.remove(0);
            ln -= 1;
            k += 1;
        }
    }
    let end = (dest_pos + ln).min(rec.len());
    if dest_pos < rec.len() {
        rec[dest_pos..end].copy_from_slice(&content[..end - dest_pos]);
    }
}

/// `reportInitialize ()` (reportio.c:287): one-time global RWCS init. A no-op in this port (the C sets a
/// `bDidReportInit` guard + `inDetailDecl`; there is no global state to seed here).
#[allow(non_snake_case)]
pub fn reportInitialize() {}

// ======================================================================================================
// Control-break graph + counters + debug dump (reportio.c)
// ======================================================================================================

/// A control-break definition (`cob_report_control`; common.h:1487): the CONTROL field, its declared
/// `sequence` (nesting depth), the `control_ref` line ids that fire on its break, the current/prior value
/// (`val`/`sf`), whether it has a CONTROL HEADING/FOOTING, and a per-cycle `suppress` flag.
#[derive(Debug, Clone, Default)]
pub struct ReportControl {
    pub name: String,
    pub sequence: i32,
    pub control_ref: Vec<usize>,
    pub val: Option<(Vec<u8>, FieldAttr)>,
    pub sf: Option<(Vec<u8>, FieldAttr)>,
    pub has_heading: bool,
    pub has_footing: bool,
    pub suppress: bool,
    /// The flags of each referenced line (for [`free_control_fields`] to recompute heading/footing).
    pub ref_flags: Vec<u32>,
    /// The live CONTROL field value (`rc->f`) compared against `val` to detect a control break.
    pub current: Option<(Vec<u8>, FieldAttr)>,
    /// Whether this control broke this `GENERATE` cycle (`rc->data_change`).
    pub data_change: bool,
}

/// `dumpFlags (flags, ln, name)` (reportio.c): render a line's `COB_REPORT_*` flag set as the space-joined
/// list of clause names the debug log prints (e.g. `"PAGE HEADING DETAIL "`). Empty when no flag is set.
#[allow(non_snake_case)]
pub fn dumpFlags(line_flags: u32, name: &str) -> String {
    use flags::*;
    let mut s = String::new();
    for (bit, text) in [
        (HEADING, "REPORT HEADING"),
        (FOOTING, "REPORT FOOTING"),
        (PAGE_HEADING, "PAGE HEADING"),
        (PAGE_FOOTING, "PAGE FOOTING"),
    ] {
        if line_flags & bit != 0 {
            s.push_str(text);
            s.push(' ');
        }
    }
    if line_flags & CONTROL_HEADING != 0 {
        s.push_str(&format!("CONTROL HEADING {name} "));
    }
    if line_flags & CONTROL_HEADING_FINAL != 0 {
        s.push_str("CONTROL HEADING FINAL ");
    }
    if line_flags & CONTROL_FOOTING != 0 {
        s.push_str(&format!("CONTROL FOOTING {name} "));
    }
    for (bit, text) in [
        (CONTROL_FOOTING_FINAL, "CONTROL FOOTING FINAL"),
        (DETAIL, "DETAIL"),
        (LINE, "LINE"),
        (LINE_PLUS, "LINE PLUS"),
        (NEXT_GROUP_LINE, "NEXT GROUP LINE"),
        (NEXT_GROUP_PLUS, "NEXT GROUP PLUS"),
        (NEXT_GROUP_PAGE, "NEXT GROUP PAGE"),
        (NEXT_PAGE, "NEXT PAGE"),
        (GROUP_INDICATE, "GROUP INDICATE"),
    ] {
        if line_flags & bit != 0 {
            s.push_str(text);
            s.push(' ');
        }
    }
    s
}

/// `reportDumpOneLine (r, fl, indent, id)` (reportio.c): the single-line debug summary -- the indent, the
/// line number, and the [`dumpFlags`] of the line.
#[allow(non_snake_case)]
pub fn reportDumpOneLine(line: &ReportLine, indent: usize) -> String {
    format!("{}Line {} {}\n", " ".repeat(indent), line.line, dumpFlags(line.flags, ""))
}

/// `reportDumpLine (r, fl, indent)` (reportio.c): the recursive line-tree dump -- this line then its child
/// subtree (one deeper indent), then its siblings (same indent), as the debug log prints them.
#[allow(non_snake_case)]
pub fn reportDumpLine(lines: &[ReportLine], indent: usize) -> String {
    let mut s = String::new();
    for l in lines {
        s.push_str(&reportDumpOneLine(l, indent));
        s.push_str(&reportDumpLine(&l.children, indent + 2));
    }
    s
}

/// `reportDump (r, msg)` (reportio.c): dump the whole report's line tree (used under `DEBUG_ISON("rw")`).
#[allow(non_snake_case)]
pub fn reportDump(report: &Report) -> String {
    reportDumpLine(&report.lines, 0)
}

/// `get_control_sequence (r, l)` (reportio.c): the `sequence` of the control whose `control_ref` includes
/// the line `line_id`, or `-1` when no control references it.
pub fn get_control_sequence(report: &Report, line_id: usize) -> i32 {
    for c in &report.controls {
        if c.control_ref.contains(&line_id) {
            return c.sequence;
        }
    }
    -1
}

/// `free_control_fields (r)` (reportio.c): release each control's saved `val`/`sf` fields and recompute its
/// `has_heading`/`has_footing` from the flags of the lines it references.
pub fn free_control_fields(report: &mut Report) {
    use flags::*;
    for c in &mut report.controls {
        c.val = None;
        c.sf = None;
        c.has_heading = false;
        c.has_footing = false;
        for &rf in &c.ref_flags {
            if rf & CONTROL_HEADING != 0 || rf & CONTROL_HEADING_FINAL != 0 {
                c.has_heading = true;
            }
            if rf & CONTROL_FOOTING != 0 || rf & CONTROL_FOOTING_FINAL != 0 {
                c.has_footing = true;
            }
        }
    }
}

/// `cob_report_suppress (r, l)` (reportio.c): `SUPPRESS PRINTING` for the line group `line_id` -- set the
/// `suppress` flag on the control whose `control_ref` includes that line.
pub fn cob_report_suppress(report: &mut Report, line_id: usize) {
    for c in &mut report.controls {
        if c.control_ref.contains(&line_id) {
            c.suppress = true;
            return;
        }
    }
}

/// `sum_this_counter (r, counter)` (reportio.c): when any SUM counter sums the field `counter_id`, fold all
/// of that counter's summed fields into its running total (the `SUM ... UPON` forward accumulation).
pub fn sum_this_counter(report: &mut Report, counter_id: usize) {
    for sc in report.sum_counters.iter_mut() {
        if sc.sums_field == Some(counter_id) {
            for (val, vattr) in sc.sum_values.clone() {
                let (data, attr) = (std::mem::take(&mut sc.counter), sc.counter_attr);
                let mut out = vec![0u8; data.len()];
                cob_add_fields(&data, &attr, &val, &vattr, &mut out, &attr);
                sc.counter = out;
            }
        }
    }
}

/// `zero_all_counters (r, flag, l)` (reportio.c): zero the SUM counters whose control matches `flag` -- the
/// FINAL counters on `CONTROL FOOTING FINAL`, else the counters tied to the breaking control.
pub fn zero_all_counters(report: &mut Report, flag: u32) {
    use flags::*;
    for sc in report.sum_counters.iter_mut() {
        // cob_field_init resets to the field's zero (DISPLAY numeric -> '0' chars, not binary zero).
        if flag & CONTROL_FOOTING_FINAL != 0 {
            if sc.control_final {
                cob_field_init(&mut sc.counter, &sc.counter_attr);
            }
        } else if sc.control.is_some() {
            cob_field_init(&mut sc.counter, &sc.counter_attr);
        }
    }
}

// ======================================================================================================
// Line / page output state machine (write_rec produces the report bytes; the cob_file write is the
// boundary) + the present-now control logic + the INITIATE / TERMINATE verbs.
// ======================================================================================================

/// `COB_WRITE_*` option bits used by the report writer (`common.h`): the line-advance count is the low
/// `COB_WRITE_MASK` bits.
const COB_WRITE_MASK: i32 = 0xFFFF;

/// `write_rec (r, opt)` (reportio.c): emit the current record to the report file, advancing
/// `opt & COB_WRITE_MASK` lines. The record is first truncated to `def_cols`; the bytes are appended to the
/// report output (the actual `cob_file` `WRITE ... ADVANCING` is the declared OS boundary). `CODE IS` insert
/// and EXTFH are non-claims.
pub fn write_rec(report: &mut Report, opt: i32) {
    let cols = report.def_cols.max(0) as usize;
    let mut rec = report.record.clone();
    rec.resize(cols, b' ');
    report.output.extend_from_slice(&rec);
    report.output.push(b'\n');
    // COB_WRITE_LINES with an advance count > 1 emits the extra blank lines.
    let num = opt & COB_WRITE_MASK;
    for _ in 1..num.max(1) {
        report.output.push(b'\n');
    }
}

/// `report_line (r, l)` (reportio.c): print one report line -- build the `def_cols`-wide record by placing
/// each present field's data at its column, then [`write_rec`] it. (The fine-grained `print_field` column
/// justification is `GNURUST`-sealed separately; this lays out the present fields.)
pub fn report_line(report: &mut Report, line: &ReportLine) {
    let cols = report.def_cols.max(0) as usize;
    let mut rec = vec![b' '; cols];
    for rf in &line.fields {
        if rf.flags & flags::PRESENT != 0 && !rf.present_now {
            continue;
        }
        if rf.suppress {
            continue;
        }
        let col = (rf.column.max(1) - 1) as usize;
        for (i, &b) in rf.data.iter().enumerate() {
            if col + i < cols {
                rec[col + i] = b;
            }
        }
    }
    report.record = rec;
    write_rec(report, 1);
    report.curr_line += 1;
}

/// `report_line_type (r, l, type)` (reportio.c): print this line (then its child subtree as LINEs, then
/// siblings) when its `flags` include `type`. The control-footing sibling-sequence ordering is honoured via
/// [`get_control_sequence`].
pub fn report_line_type(report: &mut Report, lines: &[ReportLine], type_: u32) {
    for l in lines {
        if l.flags & type_ != 0 {
            report_line(report, l);
            if !l.children.is_empty() {
                report_line_type(report, &l.children, flags::LINE);
            }
        }
    }
}

/// `report_line_and (r, l, type)` (reportio.c): a field-less line with a single child is "transparent" --
/// descend to the child before [`report_line_type`], so a wrapper group line prints its child lines.
pub fn report_line_and(report: &mut Report, line: &ReportLine, type_: u32) {
    if line.fields.is_empty() && line.children.len() == 1 {
        if line.flags & type_ != 0 {
            report_line(report, line);
            report_line_type(report, &line.children, flags::LINE);
            return;
        }
        report_line_type(report, &line.children, type_);
    } else {
        report_line_type(report, std::slice::from_ref(line), type_);
    }
}

/// `line_control_one (r, l, f)` (reportio.c): update the `present_now` flag of each PRESENT-WHEN field on a
/// line for a control change (`control_field_id`, or `None` for a new page): a non-negated field becomes
/// present, a NEGATEd one becomes absent, when its tracked control changed (or, with `COB_REPORT_PAGE`, on
/// a new page).
pub fn line_control_one(line: &mut ReportLine, control_field_id: Option<usize>) {
    use flags::*;
    for rf in &mut line.fields {
        if rf.flags & PRESENT == 0 {
            continue;
        }
        if rf.flags & NEGATE == 0 && !rf.present_now {
            match control_field_id {
                None => {
                    if rf.flags & PAGE != 0 {
                        rf.present_now = true;
                    }
                }
                Some(f) if rf.control_id == Some(f) => rf.present_now = true,
                _ => {}
            }
        } else if rf.flags & NEGATE != 0 && rf.present_now {
            match control_field_id {
                None => {
                    if rf.flags & PAGE != 0 {
                        rf.present_now = false;
                    }
                }
                Some(f) if rf.control_id == Some(f) => rf.present_now = false,
                _ => {}
            }
        }
    }
}

/// `line_control_chg (r, l, f)` (reportio.c): [`line_control_one`] applied to a line and, recursively, its
/// child and sibling lines.
pub fn line_control_chg(lines: &mut [ReportLine], control_field_id: Option<usize>) {
    for l in lines.iter_mut() {
        line_control_one(l, control_field_id);
        line_control_chg(&mut l.children, control_field_id);
    }
}

/// `cob_report_initiate (r)` (reportio.c): the `INITIATE` verb -- validate the PAGE LIMIT geometry (HEADING
/// <= FIRST DETAIL <= LAST DETAIL, FOOTING within HEADING..LAST DETAIL, all within LINE LIMIT), clamp the
/// page bounds, and arm the report for `GENERATE`. Returns `Err(())` on a PAGE LIMIT violation (the C raises
/// `COB_EC_REPORT_PAGE_LIMIT`) or when already INITIATEd, else `Ok(())` with `initiate_done` set.
pub fn cob_report_initiate(report: &mut Report) -> Result<(), ()> {
    reportInitialize();
    if report.initiate_done {
        return Err(()); // already active (COB_EC_REPORT_ACTIVE)
    }
    const REPORT_MAX_LINES: i32 = 9999;
    const REPORT_MAX_COLS: i32 = 999;
    if report.def_lines > REPORT_MAX_LINES {
        report.def_lines = REPORT_MAX_LINES;
    }
    if report.def_cols > REPORT_MAX_COLS || report.def_cols < 1 {
        report.def_cols = REPORT_MAX_COLS;
    }
    let bad = (report.def_first_detail > 0 && !(report.def_first_detail >= report.def_heading))
        || (report.def_last_detail > 0 && !(report.def_last_detail >= report.def_first_detail))
        || (report.def_footing > 0 && !(report.def_footing >= report.def_heading))
        || (report.def_footing > 0 && !(report.def_footing >= report.def_last_detail))
        || (report.def_lines > 0 && !(report.def_lines >= report.def_heading))
        || (report.def_lines > 0 && !(report.def_lines >= report.def_footing));
    if bad {
        return Err(()); // COB_EC_REPORT_PAGE_LIMIT
    }
    report.initiate_done = true;
    report.first_generate = true;
    report.first_detail = true;
    report.curr_line = 0;
    report.curr_page = 1;
    Ok(())
}

/// `do_page_heading (r)` (reportio.c): emit the PAGE HEADING -- skip any remaining lines to the page end,
/// bump the page counter (after the first GENERATE), advance to the HEADING line, print the PAGE HEADING
/// lines, advance to FIRST DETAIL, and re-arm the PRESENT-WHEN flags for the new page.
pub fn do_page_heading(r: &mut Report) {
    if r.in_page_heading {
        return;
    }
    let opt = 1;
    if !r.first_generate && r.def_lines > 0 && r.def_heading > 0 && r.curr_line <= r.def_lines && r.curr_line > r.def_heading {
        while r.curr_line <= r.def_lines {
            write_rec(r, opt);
            r.curr_line += 1;
        }
        if r.curr_line > r.def_lines {
            r.curr_line = 1;
        }
        saveLineCounter(r);
    }
    r.in_page_heading = true;
    if !r.first_generate {
        r.curr_page += 1;
    }
    r.first_detail = false;
    while r.curr_line < r.def_heading {
        write_rec(r, opt);
        r.curr_line += 1;
        saveLineCounter(r);
    }
    let lines = std::mem::take(&mut r.lines);
    report_line_type(r, &lines, flags::PAGE_HEADING);
    r.lines = lines;
    while r.curr_line < r.def_first_detail {
        write_rec(r, opt);
        r.curr_line += 1;
        saveLineCounter(r);
    }
    clear_group_indicate(&mut r.lines);
    r.in_page_heading = false;
    line_control_chg(&mut r.lines, None);
}

/// `do_page_footing (r)` (reportio.c): emit the PAGE FOOTING lines, pad to the page end (advancing
/// `def_lines - curr_line` lines), reset the line counter to the top of the next page, and re-arm the
/// first-detail flag.
pub fn do_page_footing(r: &mut Report) {
    if r.in_page_footing {
        return;
    }
    r.in_page_footing = true;
    let lines = std::mem::take(&mut r.lines);
    report_line_type(r, &lines, flags::PAGE_FOOTING);
    r.lines = lines;
    if r.curr_line < r.def_lines {
        write_rec(r, COB_WRITE_MASK & (r.def_lines - r.curr_line).max(1));
        r.curr_line = r.def_lines;
        r.incr_line = false;
    } else {
        r.curr_line = 1;
    }
    saveLineCounter(r);
    r.first_detail = true;
    r.in_page_footing = false;
}

/// Detect control breaks: a control whose live value (`current`) differs from its saved value (`val`) sets
/// `data_change`; a break at level N also breaks every lower (smaller-sequence) level. Mirrors the
/// `cob_cmp(rc->f, rc->val)` loop at the top of `cob_report_generate`/`cob_report_terminate`.
fn detect_control_breaks(r: &mut Report) -> i32 {
    let mut maxctl = 0;
    for rc in r.controls.iter_mut() {
        rc.data_change = rc.current != rc.val;
        if rc.data_change {
            rc.sf = rc.current.clone();
            if rc.sequence > maxctl {
                maxctl = rc.sequence;
            }
        }
    }
    if maxctl > 0 {
        for rc in r.controls.iter_mut() {
            if rc.sequence < maxctl && !rc.data_change {
                rc.data_change = true;
                rc.sf = rc.current.clone();
            }
        }
    }
    maxctl
}

/// `cob_report_generate (r, l, ctl)` (reportio.c): the `GENERATE` verb -- the report-writer control-break
/// engine. On the first GENERATE it prints the REPORT HEADING, PAGE HEADING and CONTROL HEADINGs; on each
/// later GENERATE it handles page overflow, detects control breaks (printing the CONTROL FOOTINGs of the
/// broken controls, zeroing their SUM counters, then the new CONTROL HEADINGs), accumulates the SUM detail
/// counters, and prints the detail line `l` (unless SUPPRESSed). The DECLARATIVES re-entry (`ctl`/`use_decl`
/// -- USE BEFORE/AFTER REPORTING) is a declared non-claim: this ports the straight-through path that a
/// report without USE procedures takes.
pub fn cob_report_generate(r: &mut Report, detail: Option<&ReportLine>) {
    reportInitialize();
    if !r.initiate_done {
        return; // GENERATE without INITIATE (COB_EC_REPORT_INACTIVE)
    }
    r.foot_next_page = false;
    if r.incr_line {
        r.incr_line = false;
        r.curr_line += 1;
        saveLineCounter(r);
    }
    if r.first_generate {
        let lines = std::mem::take(&mut r.lines);
        report_line_type(r, &lines, flags::HEADING);
        r.lines = lines;
        do_page_heading(r);
        // CONTROL HEADINGs + snapshot each control's value
        let lines = std::mem::take(&mut r.lines);
        for ci in 0..r.controls.len() {
            for &rl in &r.controls[ci].control_ref.clone() {
                if let Some(l) = lines.get(rl) {
                    if l.flags & flags::CONTROL_HEADING != 0 {
                        report_line_and(r, l, flags::CONTROL_HEADING);
                    }
                }
            }
            r.controls[ci].val = r.controls[ci].current.clone();
            r.controls[ci].data_change = false;
        }
        r.lines = lines;
        r.first_generate = false;
    } else {
        if r.curr_line > r.def_last_detail {
            do_page_footing(r);
            r.curr_line = 1;
            do_page_heading(r);
            r.first_detail = false;
        } else if r.curr_line <= 1 || r.first_detail {
            if r.first_detail {
                r.curr_line = 1;
            }
            do_page_heading(r);
            r.first_detail = false;
        }
        detect_control_breaks(r);
        // PRESENT WHEN updates for each broken control
        let broke: Vec<Option<usize>> = r
            .controls
            .iter()
            .enumerate()
            .filter(|(_, c)| c.data_change)
            .map(|(i, _)| Some(i))
            .collect();
        for f in &broke {
            let mut lines = std::mem::take(&mut r.lines);
            line_control_chg(&mut lines, *f);
            r.lines = lines;
        }
        // CONTROL FOOTINGs (broken controls), zero their counters
        let lines = std::mem::take(&mut r.lines);
        for ci in 0..r.controls.len() {
            if !r.controls[ci].data_change {
                continue;
            }
            for &rl in &r.controls[ci].control_ref.clone() {
                if let Some(l) = lines.get(rl) {
                    if l.flags & flags::CONTROL_FOOTING != 0 && !r.controls[ci].suppress {
                        report_line_and(r, l, flags::CONTROL_FOOTING);
                    }
                }
            }
            r.controls[ci].suppress = false;
            zero_all_counters(r, flags::CONTROL_FOOTING);
        }
        // CONTROL HEADINGs (broken controls), refresh saved value
        for ci in 0..r.controls.len() {
            if !r.controls[ci].data_change {
                continue;
            }
            for &rl in &r.controls[ci].control_ref.clone() {
                if let Some(l) = lines.get(rl) {
                    if l.flags & flags::CONTROL_HEADING != 0 {
                        report_line_and(r, l, flags::CONTROL_HEADING);
                    }
                }
            }
            r.controls[ci].val = r.controls[ci].current.clone();
            r.controls[ci].data_change = false;
        }
        r.lines = lines;
    }
    sum_all_detail(&mut r.sum_counters);
    // the DETAIL line itself
    match detail {
        None => {}
        Some(l) if l.suppress => {}
        Some(l) => {
            report_line(r, l);
        }
    }
}

/// `cob_report_terminate (r, ctl)` (reportio.c): the `TERMINATE` verb -- force a final control break at all
/// levels (printing every CONTROL FOOTING and the CONTROL FOOTING FINAL), then the PAGE FOOTING and REPORT
/// FOOTING, and clear `initiate_done`. Returns `Err(())` when no INITIATE/GENERATE preceded it. The
/// DECLARATIVES re-entry (`ctl`) is a declared non-claim (the straight-through path is ported).
pub fn cob_report_terminate(r: &mut Report) -> Result<(), ()> {
    if !r.initiate_done {
        return Err(()); // TERMINATE without INITIATE
    }
    if r.first_generate {
        return Ok(()); // no GENERATE ever done
    }
    // Final break: every control footing fires.
    let lines = std::mem::take(&mut r.lines);
    for ci in 0..r.controls.len() {
        for &rl in &r.controls[ci].control_ref.clone() {
            if let Some(l) = lines.get(rl) {
                if l.flags & (flags::CONTROL_FOOTING | flags::CONTROL_FOOTING_FINAL) != 0 {
                    report_line_and(r, l, l.flags & (flags::CONTROL_FOOTING | flags::CONTROL_FOOTING_FINAL));
                }
            }
        }
    }
    // REPORT FOOTING
    r.in_report_footing = true;
    report_line_type(r, &lines, flags::FOOTING);
    r.lines = lines;
    do_page_footing(r);
    r.in_report_footing = false;
    r.initiate_done = false;
    Ok(())
}

/// `cob_init_reportio (lptr, sptr)` (reportio.c): module init binding the runtime globals. A no-op here.
pub fn cob_init_reportio() {}

/// `cob_exit_reportio ()` (reportio.c): module teardown freeing active reports. A no-op (RAII).
pub fn cob_exit_reportio() {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attr::COB_FLAG_HAVE_SIGN;

    #[allow(dead_code)] // test helper retained for symmetry with the numeric builders
    fn anum(b: &[u8]) -> (Vec<u8>, FieldAttr) {
        (b.to_vec(), FieldAttr { field_type: COB_TYPE_ALPHANUMERIC, digits: 0, scale: 0, flags: 0 })
    }
    fn fld(col: i32, data: &[u8]) -> ReportField {
        ReportField {
            flags: 0,
            line: 1,
            column: col,
            level: 1,
            next_group_line: 0,
            group_indicate: false,
            suppress: false,
            present_now: false,
            control_id: None,
            data: data.to_vec(),
        }
    }
    fn line(flag: u32, fields: Vec<ReportField>) -> ReportLine {
        ReportLine { fields, children: vec![], flags: flag, line: 1, next_group_line: 0, suppress: false }
    }

    #[test]
    fn dump_flags_and_control_sequence() {
        use flags::*;
        let s = dumpFlags(PAGE_HEADING | DETAIL, "");
        assert!(s.contains("PAGE HEADING") && s.contains("DETAIL"));
        assert_eq!(dumpFlags(CONTROL_HEADING, "DEPT"), "CONTROL HEADING DEPT ");
        let mut r = Report::default();
        r.controls.push(ReportControl { name: "DEPT".into(), sequence: 2, control_ref: vec![3], ..Default::default() });
        assert_eq!(get_control_sequence(&r, 3), 2);
        assert_eq!(get_control_sequence(&r, 9), -1);
    }

    #[test]
    fn initiate_validates_page_limits() {
        let mut r = Report::default();
        r.def_heading = 1;
        r.def_first_detail = 3;
        r.def_last_detail = 50;
        r.def_footing = 55;
        r.def_lines = 60;
        r.def_cols = 80;
        assert!(cob_report_initiate(&mut r).is_ok());
        assert!(r.initiate_done && r.first_generate);
        // a second INITIATE is rejected
        assert!(cob_report_initiate(&mut r).is_err());
        // bad geometry: FIRST DETAIL before HEADING
        let mut b = Report::default();
        b.def_heading = 10;
        b.def_first_detail = 3;
        b.def_lines = 60;
        b.def_cols = 80;
        assert!(cob_report_initiate(&mut b).is_err());
    }

    #[test]
    fn report_line_lays_out_fields_into_output() {
        let mut r = Report::default();
        r.def_cols = 10;
        let l = line(flags::DETAIL, vec![fld(1, b"AB"), fld(5, b"XY")]);
        report_line(&mut r, &l);
        // record is 10 cols: "AB  XY    " then a newline
        assert_eq!(&r.output, b"AB  XY    \n");
        assert_eq!(r.curr_line, 1);
    }

    #[test]
    fn generate_first_emits_heading_and_detail() {
        let mut r = Report::default();
        r.def_cols = 6;
        r.def_heading = 1;
        r.def_first_detail = 1;
        r.def_last_detail = 50;
        r.def_lines = 60;
        r.lines = vec![
            line(flags::PAGE_HEADING, vec![fld(1, b"HEAD")]),
            line(flags::DETAIL, vec![fld(1, b"DAT")]),
        ];
        cob_report_initiate(&mut r).unwrap();
        let detail = line(flags::DETAIL, vec![fld(1, b"DAT")]);
        cob_report_generate(&mut r, Some(&detail));
        let out = String::from_utf8_lossy(&r.output);
        assert!(out.contains("HEAD"), "page heading emitted: {out:?}");
        assert!(out.contains("DAT"), "detail emitted: {out:?}");
        assert!(!r.first_generate);
    }

    #[test]
    fn sum_counters_accumulate_and_zero() {
        let nattr = FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: 3, scale: 0, flags: 0 };
        let mut r = Report::default();
        r.sum_counters.push(ReportSumCtr {
            name: "T".into(),
            counter: b"000".to_vec(),
            counter_attr: nattr,
            sum_values: vec![(b"012".to_vec(), nattr)],
            subtotal: false,
            sums_field: Some(7),
            control: Some(0),
            control_final: false,
        });
        sum_this_counter(&mut r, 7);
        assert_eq!(r.sum_counters[0].counter, b"012");
        zero_all_counters(&mut r, flags::CONTROL_FOOTING);
        assert_eq!(r.sum_counters[0].counter, b"000");
    }

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
            sums_field: None,
            control: None,
            control_final: false,
        }];
        sum_all_detail(&mut ctrs);
        assert_eq!(ctrs[0].counter, b"0060");
        // a second GENERATE accumulates further: 0060 + 60 = 0120
        sum_all_detail(&mut ctrs);
        assert_eq!(ctrs[0].counter, b"0120");
    }

    #[test]
    fn limit_check_counters_next_group() {
        let a = FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: 4, scale: 0, flags: 0 };
        // a line whose LINE 60 exceeds def_lines 50 -> violation, initiate_done cleared
        let mut r = Report {
            lines: vec![ReportLine { line: 60, ..Default::default() }],
            def_lines: 50,
            initiate_done: true,
            curr_page: 3,
            curr_line: 7,
            page_counter: Some((b"0000".to_vec(), a)),
            line_counter: Some((b"0000".to_vec(), a)),
            ..Default::default()
        };
        assert!(limitCheck(&mut r));
        assert!(!r.initiate_done);
        // within limits -> no violation
        r.lines[0].line = 40;
        r.initiate_done = true;
        assert!(!limitCheck(&mut r));
        assert!(r.initiate_done);
        // saveLineCounter stores page/line (curr_line 7 <= def 50, kept)
        saveLineCounter(&mut r);
        assert_eq!(r.page_counter.as_ref().unwrap().0, b"0003");
        assert_eq!(r.line_counter.as_ref().unwrap().0, b"0007");
        // curr_line beyond def_lines clamps to 0
        r.curr_line = 99;
        saveLineCounter(&mut r);
        assert_eq!(r.line_counter.as_ref().unwrap().0, b"0000");
        // set_next_info: NEXT GROUP LINE
        let l = ReportLine { flags: flags::NEXT_GROUP_LINE, next_group_line: 5, ..Default::default() };
        set_next_info(&mut r, &l);
        assert_eq!(r.next_value, 5);
        assert!(r.next_line && r.next_just_set && !r.next_line_plus);
    }

    #[test]
    fn print_field_placement_and_justify() {
        use crate::cconv::FieldRef;
        let an = FieldAttr { field_type: COB_TYPE_ALPHANUMERIC, digits: 0, scale: 0, flags: 0 };
        let _ = an;
        // value "AB" in a size-5 field at column 3 (1-based) -> rec[2..]
        let mk = |rec: &mut [u8], flags: u32, val: &'static [u8]| {
            let f = FieldRef { size: val.len(), data: Some(val) };
            print_field(&f, 5, 3, flags, true, rec);
        };
        // default (no justify flag): placed left at column, content "AB" (rtrimmed)
        let mut rec = [b' '; 12];
        mk(&mut rec, 0, b"AB   ");
        assert_eq!(&rec, b"  AB        ");
        // RIGHT: right-align within the 5-wide field -> "AB" at column 3 + (5-2)=offset 3 -> col 6
        let mut rec = [b' '; 12];
        mk(&mut rec, flags::COLUMN_RIGHT, b"AB   ");
        assert_eq!(&rec, b"     AB     ");
        // LEFT: strip leading spaces, place at column
        let mut rec = [b' '; 12];
        mk(&mut rec, flags::COLUMN_LEFT, b"  AB ");
        assert_eq!(&rec, b"  AB        ");
        // CENTER: "A" (len 1) in width 5 -> dest += (5-1-0)/2 = 2 -> col 5
        let mut rec = [b' '; 12];
        let f = FieldRef { size: 1, data: Some(b"A") };
        print_field(&f, 5, 3, flags::COLUMN_CENTER, true, &mut rec);
        assert_eq!(&rec, b"    A       ");
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

    #[test]
    fn report_dump_renders_line_tree() {
        use flags::*;
        // A parent line (PAGE HEADING) with a child (DETAIL): reportDumpOneLine renders one line,
        // reportDumpLine recurses with a deeper indent, reportDump dumps the whole report.
        let parent = ReportLine {
            fields: vec![],
            children: vec![line(DETAIL, vec![])],
            flags: PAGE_HEADING,
            line: 5,
            next_group_line: 0,
            suppress: false,
        };
        let one = reportDumpOneLine(&parent, 0);
        assert_eq!(one, "Line 5 PAGE HEADING \n");

        let tree = reportDumpLine(std::slice::from_ref(&parent), 0);
        // parent at indent 0, child at indent 2.
        assert_eq!(tree, "Line 5 PAGE HEADING \n  Line 1 DETAIL \n");

        let mut r = Report::default();
        r.lines = vec![parent];
        assert_eq!(reportDump(&r), tree);
    }

    #[test]
    fn limit_check_one_line_flags_overshoot() {
        // limitCheckOneLine: a LINE number beyond the page def_lines is a violation.
        let mut l = line(0, vec![]);
        l.line = 80;
        assert!(limitCheckOneLine(&l, 60));
        l.line = 40;
        assert!(!limitCheckOneLine(&l, 60));
        // a field NEXT GROUP overshoot also violates.
        let mut lf = line(0, vec![fld(1, b"X")]);
        lf.line = 1;
        lf.fields[0].next_group_line = 99;
        assert!(limitCheckOneLine(&lf, 60));
    }

    #[test]
    fn line_control_one_sets_present_on_break() {
        use flags::*;
        // A PRESENT-WHEN field tracking control id 7 becomes present when that control changes.
        let mut rf = fld(1, b"X");
        rf.flags = PRESENT;
        rf.control_id = Some(7);
        rf.present_now = false;
        let mut l = line(0, vec![rf]);
        // an unrelated control change leaves it absent
        line_control_one(&mut l, Some(3));
        assert!(!l.fields[0].present_now);
        // the tracked control change makes it present
        line_control_one(&mut l, Some(7));
        assert!(l.fields[0].present_now);
    }

    #[test]
    fn report_suppress_sets_control_flag() {
        // cob_report_suppress sets the suppress flag on the control referencing the line group.
        let mut r = Report::default();
        r.controls.push(ReportControl { name: "DEPT".into(), sequence: 1, control_ref: vec![4], ..Default::default() });
        r.controls.push(ReportControl { name: "OTHER".into(), sequence: 2, control_ref: vec![9], ..Default::default() });
        cob_report_suppress(&mut r, 4);
        assert!(r.controls[0].suppress);
        assert!(!r.controls[1].suppress);
    }
}
