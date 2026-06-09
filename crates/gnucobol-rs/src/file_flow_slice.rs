//! File × control-flow execution slice (`GNURUST.FILE.FLOW.SLICE.1`): the **read-loop** — where file I/O meets
//! control flow, the shape of essentially every COBOL batch program. A narrow interpreter *runs* the canonical
//! `OPEN INPUT … PERFORM UNTIL EOF: READ + accumulate … CLOSE` loop and returns the resulting WORKING-STORAGE,
//! proven against GnuCOBOL 3.2. It composes the sealed sequential READ (`GNURUST.FILE.SEQUENTIAL.1`) with the
//! looped numeric accumulation of the execution slices — completing the spine from bytes to a running program.
//!
//! **Sealed subset (deliberately narrow).** A single sequential file (RECORD or LINE) of fixed `record_len`
//! records is read to AT END; **each record is processed exactly once** (the priming-read + read-at-bottom
//! loop). The per-record body is a sequence of unsigned-`9(n)` accumulations into WORKING-STORAGE:
//! - **`Count(acc)`** — `ADD 1 TO acc` (count records),
//! - **`SumField { field, into }`** — `ADD <record.field> TO into` (sum a record field).
//!
//! **Non-claims:** indexed / relative organizations, signed / packed accumulators, numeric `SIZE ERROR` on the
//! accumulators, per-record `IF`/`MOVE`/general statements (compose `if_eval` for that), `WRITE`/`REWRITE`
//! inside the loop, multi-file loops, explicit file-status inspection beyond AT END, `READ … INTO`, and all
//! dialects.

use crate::file_seq::{read_sequential, FileOrg};
use crate::if_eval::{Relop, SliceField};
use std::cmp::Ordering;

/// One per-record accumulation into WORKING-STORAGE.
pub enum LoopOp<'a> {
    /// `ADD 1 TO <ws_field>` — count records.
    Count(&'a str),
    /// `ADD <record.field> TO <ws_field>` — sum a record field.
    SumField { field: &'a str, into: &'a str },
}

/// The read-loop result: the resulting WORKING-STORAGE bytes and the number of records processed.
pub struct ReadLoopResult {
    pub ws: Vec<u8>,
    pub records_processed: usize,
}

fn field<'a>(fields: &'a [SliceField], name: &str) -> Option<&'a SliceField<'a>> {
    fields.iter().find(|f| f.name == name)
}

fn read_num(buf: &[u8], fields: &[SliceField], name: &str) -> i64 {
    field(fields, name)
        .and_then(|f| buf.get(f.offset..f.offset + f.size))
        .map(|b| b.iter().fold(0i64, |a, &c| if c.is_ascii_digit() { a * 10 + (c - b'0') as i64 } else { a }))
        .unwrap_or(0)
}

fn add_num(ws: &mut [u8], fields: &[SliceField], name: &str, delta: i64) {
    if let Some(f) = field(fields, name) {
        if f.offset + f.size <= ws.len() {
            let m = 10i64.pow(f.size as u32);
            let v = (((read_num(ws, fields, name) + delta) % m) + m) % m; // wrap to field width (overflow is a non-claim)
            let s = format!("{:0width$}", v, width = f.size);
            ws[f.offset..f.offset + f.size].copy_from_slice(s.as_bytes());
        }
    }
}

/// Execute the canonical read-loop over `file`, returning the resulting WORKING-STORAGE and the record count.
/// `record_fields` describe fields within each record; `ws_init` / `ws_fields` the WORKING-STORAGE accumulators.
pub fn eval_read_loop(
    file: &[u8],
    org: FileOrg,
    record_len: usize,
    record_fields: &[SliceField],
    ws_init: &[u8],
    ws_fields: &[SliceField],
    body: &[LoopOp],
) -> ReadLoopResult {
    let mut ws = ws_init.to_vec();
    let mut processed = 0usize;
    for r in read_sequential(file, org, record_len) {
        if r.at_end {
            continue;
        }
        processed += 1;
        for op in body {
            match op {
                LoopOp::Count(acc) => add_num(&mut ws, ws_fields, acc, 1),
                LoopOp::SumField { field, into } => {
                    let v = read_num(&r.record, record_fields, field);
                    add_num(&mut ws, ws_fields, into, v);
                }
            }
        }
    }
    ReadLoopResult { ws, records_processed: processed }
}

/// A per-record filter condition (`GNURUST.FILE.FILTER.SLICE.1`): a single relation on a record field, either
/// **numeric** (compare the field's decoded unsigned value to an integer) or **alphanumeric** (compare the
/// field bytes, space-padded, in the ASCII collating sequence) — the choice that makes `5 < 10` numerically
/// but `"5" > "10"` alphanumerically.
pub enum FilterCond<'a> {
    Numeric { field: &'a str, op: Relop, value: i64 },
    Alpha { field: &'a str, op: Relop, value: &'a [u8] },
}

fn relop_int(a: i64, op: &Relop, b: i64) -> bool {
    match op {
        Relop::Eq => a == b,
        Relop::Ne => a != b,
        Relop::Gt => a > b,
        Relop::Lt => a < b,
        Relop::Ge => a >= b,
        Relop::Le => a <= b,
    }
}

fn relop_ord(ord: Ordering, op: &Relop) -> bool {
    match op {
        Relop::Eq => ord == Ordering::Equal,
        Relop::Ne => ord != Ordering::Equal,
        Relop::Gt => ord == Ordering::Greater,
        Relop::Lt => ord == Ordering::Less,
        Relop::Ge => ord != Ordering::Less,
        Relop::Le => ord != Ordering::Greater,
    }
}

fn alnum_cmp(a: &[u8], b: &[u8]) -> Ordering {
    let n = a.len().max(b.len());
    for i in 0..n {
        let av = a.get(i).copied().unwrap_or(b' ');
        let bv = b.get(i).copied().unwrap_or(b' ');
        match av.cmp(&bv) {
            Ordering::Equal => continue,
            o => return o,
        }
    }
    Ordering::Equal
}

fn passes(record: &[u8], record_fields: &[SliceField], cond: &FilterCond) -> bool {
    match cond {
        FilterCond::Numeric { field: fname, op, value } => relop_int(read_num(record, record_fields, fname), op, *value),
        FilterCond::Alpha { field: fname, op, value } => {
            let b = field(record_fields, fname)
                .and_then(|f| record.get(f.offset..f.offset + f.size))
                .unwrap_or(&[]);
            relop_ord(alnum_cmp(b, value), op)
        }
    }
}

/// Execute the **filter** read-loop: the canonical read-loop, but each record's `body` accumulation is gated by
/// a per-record condition (`IF <cond> ... END-IF`). `records_processed` is the count of records that **passed**
/// the filter (had the body applied). This is the selective-accumulation workhorse of COBOL batch.
#[allow(clippy::too_many_arguments)]
pub fn eval_filter_loop(
    file: &[u8],
    org: FileOrg,
    record_len: usize,
    record_fields: &[SliceField],
    ws_init: &[u8],
    ws_fields: &[SliceField],
    cond: &FilterCond,
    body: &[LoopOp],
) -> ReadLoopResult {
    let mut ws = ws_init.to_vec();
    let mut processed = 0usize;
    for r in read_sequential(file, org, record_len) {
        if r.at_end || !passes(&r.record, record_fields, cond) {
            continue;
        }
        processed += 1;
        for op in body {
            match op {
                LoopOp::Count(acc) => add_num(&mut ws, ws_fields, acc, 1),
                LoopOp::SumField { field, into } => {
                    let v = read_num(&r.record, record_fields, field);
                    add_num(&mut ws, ws_fields, into, v);
                }
            }
        }
    }
    ReadLoopResult { ws, records_processed: processed }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_loop_counts_and_sums() {
        // file: AA100 BB025 CC050 DD200 (RECORD SEQUENTIAL, record_len 5); R-ID X(2)@0, R-AMT 9(3)@2
        let file = b"AA100BB025CC050DD200";
        let record_fields = [
            SliceField { name: "R-ID", offset: 0, size: 2 },
            SliceField { name: "R-AMT", offset: 2, size: 3 },
        ];
        // WS: CNT 9(3)@0, SM 9(5)@3  (8 bytes)
        let ws_fields = [
            SliceField { name: "CNT", offset: 0, size: 3 },
            SliceField { name: "SM", offset: 3, size: 5 },
        ];
        let body = [LoopOp::Count("CNT"), LoopOp::SumField { field: "R-AMT", into: "SM" }];
        let r = eval_read_loop(file, FileOrg::RecordSequential, 5, &record_fields, b"00000000", &ws_fields, &body);
        assert_eq!(r.records_processed, 4);
        assert_eq!(read_num(&r.ws, &ws_fields, "CNT"), 4);
        assert_eq!(read_num(&r.ws, &ws_fields, "SM"), 375); // 100+25+50+200
        assert_eq!(&r.ws, b"00400375");
    }

    #[test]
    fn empty_file_processes_nothing() {
        let ws_fields = [SliceField { name: "CNT", offset: 0, size: 3 }];
        let r = eval_read_loop(b"", FileOrg::RecordSequential, 5, &[], b"000", &ws_fields, &[LoopOp::Count("CNT")]);
        assert_eq!(r.records_processed, 0);
        assert_eq!(&r.ws, b"000");
    }

    #[test]
    fn filter_loop_numeric_and_alpha() {
        // file: A100 B025 A050 A200 B007 (record_len 4); R-ST X(1)@0, R-AMT 9(3)@1
        let file = b"A100B025A050A200B007";
        let rf = [
            SliceField { name: "R-ST", offset: 0, size: 1 },
            SliceField { name: "R-AMT", offset: 1, size: 3 },
        ];
        let wf = [SliceField { name: "CNT", offset: 0, size: 3 }, SliceField { name: "SM", offset: 3, size: 5 }];
        let body = [LoopOp::Count("CNT"), LoopOp::SumField { field: "R-AMT", into: "SM" }];
        // numeric: R-AMT >= 50 -> 100, 50, 200 -> count 3, sum 350 (25 and 7 excluded -- a numeric test, not "5">"50")
        let num = eval_filter_loop(file, FileOrg::RecordSequential, 4, &rf, b"00000000", &wf, &FilterCond::Numeric { field: "R-AMT", op: Relop::Ge, value: 50 }, &body);
        assert_eq!((num.records_processed, read_num(&num.ws, &wf, "CNT"), read_num(&num.ws, &wf, "SM")), (3, 3, 350));
        // alphanumeric: R-ST = "A" -> A100, A050, A200 -> count 3, sum 350
        let alpha = eval_filter_loop(file, FileOrg::RecordSequential, 4, &rf, b"00000000", &wf, &FilterCond::Alpha { field: "R-ST", op: Relop::Eq, value: b"A" }, &body);
        assert_eq!((alpha.records_processed, read_num(&alpha.ws, &wf, "SM")), (3, 350));
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    use crate::if_eval::SliceField;
    // KANIFOR: GNURUST.FILE.FLOW.SLICE.1, GNURUST.FILE.FILTER.SLICE.1
    /// The read-loop never changes the WORKING-STORAGE length, whatever the file contents.
    #[kani::proof]
    #[kani::unwind(5)]
    fn read_loop_preserves_ws_length() {
        let file: [u8; 8] = kani::any();
        let rf = [SliceField { name: "R-AMT", offset: 1, size: 3 }];
        let wf = [SliceField { name: "CNT", offset: 0, size: 3 }, SliceField { name: "SM", offset: 3, size: 5 }];
        let ws = [b'0'; 8];
        let body = [LoopOp::Count("CNT")];
        let r = eval_read_loop(&file, crate::file_seq::FileOrg::RecordSequential, 4, &rf, &ws, &wf, &body);
        assert_eq!(r.ws.len(), 8);
    }
}
