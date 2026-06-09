//! Sequential file READ semantics (`GNURUST.FILE.SEQUENTIAL.1`): the record bytes and file status produced
//! by `READ ... NEXT RECORD` for `ORGANIZATION IS RECORD SEQUENTIAL` (fixed, binary) and
//! `ORGANIZATION IS LINE SEQUENTIAL` (text), proven byte-identical against GnuCOBOL 3.2's `libcob/fileio.c`.
//!
//! **Sealed subset.** OPEN INPUT → repeated READ NEXT → AT END, for a declared fixed record length, over a
//! flat byte buffer. Two organizations:
//! - **RECORD SEQUENTIAL** — fixed `record_len` chunks (status `00`). The record buffer **persists across
//!   reads**: a short final record overlays only the bytes available and **leaves the prior record's tail**
//!   in place (status `00`), then the next READ is AT END (`10`).
//! - **LINE SEQUENTIAL** — records delimited by `\n`; a short line is space-padded (`00`); a line longer
//!   than `record_len` is delivered `record_len` bytes at a time with status `06` until the remainder fits
//!   (`00`); a trailing newline does **not** create an empty record; a mid-file empty line **is** a record.
//!
//! **Non-claims:** indexed / relative / VSAM organizations, WRITE / REWRITE / START / DELETE, OPEN
//! I-O / EXTEND, file-status codes beyond `00`/`06`/`10`, locking, and any Procedure Division control flow.

/// The declared file organization for the sealed sequential-read subset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FileOrg {
    /// `ORGANIZATION IS RECORD SEQUENTIAL` — fixed binary records, no delimiters.
    RecordSequential,
    /// `ORGANIZATION IS LINE SEQUENTIAL` — newline-delimited text records.
    LineSequential,
}

/// One `READ NEXT` event: the `record_len`-byte record buffer (empty when `at_end`) and the file status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeqRead {
    pub at_end: bool,
    pub record: Vec<u8>,
    /// `"00"` success · `"06"` line longer than the record (delivered in chunks) · `"10"` end of file.
    pub status: &'static str,
}

/// Replay `OPEN INPUT` + repeated `READ NEXT ... AT END` over `data` for the declared organization and fixed
/// `record_len`, returning every read event (the trailing event is always `at_end` with status `"10"`),
/// byte-for-byte as GnuCOBOL's `libcob/fileio.c`.
pub fn read_sequential(data: &[u8], org: FileOrg, record_len: usize) -> Vec<SeqRead> {
    let mut out = Vec::new();
    let n = data.len();
    match org {
        FileOrg::RecordSequential => {
            // the FD record buffer persists across reads (initial: low-values / 0x00, per libcob).
            let mut buf = vec![0u8; record_len];
            let mut pos = 0usize;
            loop {
                if pos >= n {
                    out.push(SeqRead { at_end: true, record: Vec::new(), status: "10" });
                    break;
                }
                let rem = n - pos;
                if rem >= record_len {
                    buf.copy_from_slice(&data[pos..pos + record_len]);
                    pos += record_len;
                } else {
                    // short final record: overlay the available bytes, KEEP the prior tail.
                    buf[..rem].copy_from_slice(&data[pos..]);
                    pos = n;
                }
                out.push(SeqRead { at_end: false, record: buf.clone(), status: "00" });
            }
        }
        FileOrg::LineSequential => {
            let mut pos = 0usize;
            loop {
                if pos >= n {
                    out.push(SeqRead { at_end: true, record: Vec::new(), status: "10" });
                    break;
                }
                let line_end = data[pos..].iter().position(|&b| b == b'\n').map_or(n, |k| pos + k);
                let line_len = line_end - pos;
                if line_len > record_len {
                    // line longer than the record: deliver one record_len chunk, stay inside the line.
                    out.push(SeqRead { at_end: false, record: data[pos..pos + record_len].to_vec(), status: "06" });
                    pos += record_len;
                } else {
                    let mut rec = data[pos..line_end].to_vec();
                    rec.resize(record_len, b' ');
                    out.push(SeqRead { at_end: false, record: rec, status: "00" });
                    // consume the newline (if any); a trailing newline leaves pos == n -> next read is AT END.
                    pos = if line_end < n { line_end + 1 } else { n };
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    fn recs(data: &[u8], org: FileOrg) -> Vec<(String, &'static str)> {
        read_sequential(data, org, 8)
            .into_iter()
            .map(|r| (String::from_utf8_lossy(&r.record).into_owned(), r.status))
            .collect()
    }

    #[test]
    fn line_seq_short_long_and_trailing_newline() {
        // oracle: AB[00], CDEFGHIJ[06], KL[00], XY[00], END[10]  (a long line is chunked across reads)
        assert_eq!(
            recs(b"AB\nCDEFGHIJKL\nXY\n", FileOrg::LineSequential),
            vec![("AB      ".into(), "00"), ("CDEFGHIJ".into(), "06"), ("KL      ".into(), "00"), ("XY      ".into(), "00"), (String::new(), "10")]
        );
    }
    #[test]
    fn line_seq_no_trailing_newline_keeps_last_line() {
        assert_eq!(recs(b"AB\nXY", FileOrg::LineSequential), vec![("AB      ".into(), "00"), ("XY      ".into(), "00"), (String::new(), "10")]);
    }
    #[test]
    fn line_seq_mid_empty_line_is_a_record() {
        assert_eq!(recs(b"AB\n\nEF\n", FileOrg::LineSequential), vec![("AB      ".into(), "00"), ("        ".into(), "00"), ("EF      ".into(), "00"), (String::new(), "10")]);
    }
    #[test]
    fn record_seq_exact_multiple() {
        assert_eq!(recs(b"01234567ABCDEFGH", FileOrg::RecordSequential), vec![("01234567".into(), "00"), ("ABCDEFGH".into(), "00"), (String::new(), "10")]);
    }
    #[test]
    fn record_seq_partial_first_record_is_low_values() {
        // oracle: a 4-byte file < record_len -> WXYZ then the initial low-values (0x00), NOT spaces.
        let r = read_sequential(b"WXYZ", FileOrg::RecordSequential, 8);
        assert_eq!(r[0].record, vec![b'W', b'X', b'Y', b'Z', 0, 0, 0, 0]);
        assert_eq!(r[0].status, "00");
        assert!(r[1].at_end && r[1].status == "10");
    }
    #[test]
    fn record_seq_partial_final_leaks_prior_tail() {
        // oracle: the 4-byte final record overlays only WXYZ, leaving EFGH from the prior record.
        assert_eq!(recs(b"01234567ABCDEFGHWXYZ", FileOrg::RecordSequential), vec![("01234567".into(), "00"), ("ABCDEFGH".into(), "00"), ("WXYZEFGH".into(), "00"), (String::new(), "10")]);
    }
}
