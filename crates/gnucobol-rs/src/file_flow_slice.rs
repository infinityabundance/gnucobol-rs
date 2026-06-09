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
use crate::if_eval::SliceField;

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
}
