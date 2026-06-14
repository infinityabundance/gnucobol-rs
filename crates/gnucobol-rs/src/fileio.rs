//! Line-sequential file WRITE byte-semantics (`GNURUST.FILEIO.LINESEQ.1`): a faithful port of
//! GnuCOBOL 3.2 `libcob/fileio.c` `lineseq_size` + `lineseq_write` — the bytes a `WRITE` appends to
//! an `ORGANIZATION IS LINE SEQUENTIAL` file and the resulting FILE STATUS — proven byte-identical
//! against the admitted `libcob` across the `COB_LS_*` runtime-config matrix.
//!
//! This is the first sealed sub-surface of `fileio.c` (libcob file 8/13, the keystone). In keeping
//! with the pure-kernel model (`#![forbid(unsafe_code)]`, no global state), the actual `FILE *`/`fd`
//! syscalls are the **declared OS boundary**: these functions model the *bytes a `WRITE` produces*
//! as a deterministic function of `(record bytes, runtime config)`. The caller (e.g. the
//! `gnucobol-rs-io` satellite) performs the I/O; this kernel decides what bytes go out.
//!
//! **Config matrix sealed** (the `cob_set_*` runtime settings that change the output bytes):
//! - **default (`COB_LS_VALIDATE=1`)** — trailing spaces stripped, record written raw, `\n` appended;
//!   but any byte `< 0x20` in the record makes the `WRITE` fail with status `71` and emit **nothing**.
//! - **`COB_LS_VALIDATE=0` (plain)** — trailing spaces stripped, record (incl. control bytes) written
//!   raw, `\n` appended.
//! - **`COB_LS_NULLS=1`** (with validate off) — each byte `< 0x20` is emitted as `0x00` then the byte;
//!   `\n` appended.
//! - **`COB_LS_FIXED=1`** — trailing spaces are **not** stripped; the full record area is written.
//!
//! **Forensic finding (binary is the authority).** `fileio.c`'s `IS_BAD_CHAR` macro excludes
//! `COB_CHAR_BS`/`ESC`/`FF`/`SI`/`TAB` from the bad set, but in the compiled 3.2 GA `libcob` those
//! exclusions are **dead**: every byte `< 0x20` (0x00–0x1F inclusive — verified across the full
//! 0x00–0xFF range) is rejected with status `71`. The effective rule is simply `byte < 0x20`.
//!
//! **Non-claims:** `WRITE ... ADVANCING` (`opt != 0`, the `cob_*_write_opt` family + LINAGE), the
//! Windows CR/LF text-mode path (`cob_ls_uses_cr` — a platform boundary, always off on Unix),
//! `COB_LS_VALIDATE>1` printable-check (`COB_EXPERIMENTAL`), CODE-SET conversion, variable-length
//! records (`f->variable_record`), the line-sequential READ and REWRITE paths, and all other organizations.

/// The `COB_LS_*` runtime settings that change line-sequential WRITE output bytes.
///
/// Mirrors the relevant `cobsetptr` fields read by `lineseq_size`/`lineseq_write`. `ls_validate`
/// defaults to **true** in libcob (`COB_LS_VALIDATE` default `"1"`); see [`LineSeqConfig::DEFAULT`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineSeqConfig {
    /// `COB_LS_FIXED` — write the full record area without stripping trailing spaces.
    pub ls_fixed: bool,
    /// `COB_LS_NULLS` — NULL-encode control bytes (`0x00` prefix before each byte `< 0x20`).
    pub ls_nulls: bool,
    /// `COB_LS_VALIDATE` — reject a record containing any byte `< 0x20` with status `71`.
    pub ls_validate: bool,
    /// `COB_LS_SPLIT` (READ only) — a line longer than the record is split into `06`/`00` records;
    /// when off, the overflow is status `04` and the rest of the line is discarded. Default `true`.
    pub ls_split: bool,
}

impl LineSeqConfig {
    /// libcob's out-of-the-box settings: `COB_LS_VALIDATE=1`, `COB_LS_SPLIT=1`, `COB_LS_FIXED=0`, `COB_LS_NULLS=0`.
    pub const DEFAULT: LineSeqConfig =
        LineSeqConfig { ls_fixed: false, ls_nulls: false, ls_validate: true, ls_split: true };
}

/// The outcome of one `WRITE` to a LINE SEQUENTIAL file: the FILE STATUS and the bytes appended.
///
/// On a validation failure (`status == "71"`) `bytes` is empty — libcob writes nothing and the
/// `WRITE` fails before any output, matching the oracle (the file is left at its prior length).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineWrite {
    /// `"00"` success · `"71"` a byte `< 0x20` was rejected under `COB_LS_VALIDATE`.
    pub status: &'static str,
    /// The bytes this `WRITE` appends to the file (empty when `status != "00"`).
    pub bytes: Vec<u8>,
}

/// `IS_BAD_CHAR` as the compiled 3.2 GA `libcob` actually behaves: **every** byte below the space
/// (`0x20`) is bad. The source macro's `BS`/`ESC`/`FF`/`SI`/`TAB` exclusions are dead in this build
/// (verified empirically across 0x00–0xFF).
#[inline]
fn is_bad_char(b: u8) -> bool {
    b < b' '
}

/// Port of `fileio.c:lineseq_size` — the number of bytes of `record` that `WRITE` will emit.
///
/// `record` is the fixed-width FD record area (`f->record->data`, `record_max` wide, space-padded);
/// `record_min` is `f->record_min` (== `record.len()` for the sealed fixed-record subset). With
/// `ls_fixed` the full area length is returned; otherwise trailing spaces are stripped (an all-space
/// record yields `0`). Variable-length records (`f->variable_record`) are a declared non-claim.
pub fn lineseq_size(record: &[u8], record_min: usize, ls_fixed: bool) -> usize {
    if ls_fixed {
        return record.len();
    }
    let mut size = record.len();
    if size < record_min {
        size = record_min;
    }
    if size == 0 {
        return 0;
    }
    // strip trailing spaces: for (i = size-1; ; --i) { if data[i]!=' ' {i++; break;} if i==0 break; }
    let mut i = size - 1;
    loop {
        if record[i] != b' ' {
            i += 1;
            break;
        }
        if i == 0 {
            break;
        }
        i -= 1;
    }
    i
}

/// Port of `fileio.c:lineseq_write` for the unadvanced write (`opt == 0`, no LINAGE): the bytes a
/// single `WRITE record` appends to a LINE SEQUENTIAL file and the resulting FILE STATUS, byte-for-
/// byte as GnuCOBOL 3.2 `libcob`.
///
/// `record` is the fixed-width FD record area (space-padded to its declared length). The emitted
/// length is [`lineseq_size`]; then per `cfg`: validate (reject `<0x20` → `"71"`, else write raw),
/// else NULL-encode, else write raw — followed by a single `\n` (the `opt == 0`, non-LINAGE,
/// non-CR path). An all-space record under the default/plain paths emits just `\n`.
pub fn lineseq_write(record: &[u8], cfg: &LineSeqConfig) -> LineWrite {
    let size = lineseq_size(record, record.len(), cfg.ls_fixed);
    let data = &record[..size];
    let mut out = Vec::new();
    if size > 0 {
        if cfg.ls_validate {
            // validate && !flag_line_adv && !sort_collating (both declared off)
            if data.iter().any(|&b| is_bad_char(b)) {
                return LineWrite { status: "71", bytes: Vec::new() };
            }
            out.extend_from_slice(data);
        } else if cfg.ls_nulls {
            for &b in data {
                if b < b' ' {
                    out.push(0x00);
                }
                out.push(b);
            }
        } else {
            out.extend_from_slice(data);
        }
    }
    // opt == 0, not LINAGE, not cob_ls_uses_cr -> add exactly one LF.
    out.push(b'\n');
    LineWrite { status: "00", bytes: out }
}

/// `IS_BAD_CHAR` as the compiled 3.2 GA `libcob` behaves **on READ**: a byte below the space (`0x20`)
/// is bad *unless* it is one of `BS`/`TAB`/`FF`/`SI`/`ESC` (`0x08`/`0x09`/`0x0C`/`0x0F`/`0x1B`).
///
/// Forensic asymmetry (both verified across `0x00–0x1F`): the **same** source macro is honored here on
/// READ (those five are allowed → status `00`) but is **dead** on WRITE ([`is_bad_char`], where every
/// byte `< 0x20` is rejected → status `71`).
#[inline]
fn is_bad_char_read(b: u8) -> bool {
    b < b' ' && b != 0x08 && b != 0x09 && b != 0x0c && b != 0x0f && b != 0x1b
}

/// The outcome of one `READ NEXT` from a LINE SEQUENTIAL file: FILE STATUS, the `record_max`-wide record
/// area (space-filled past the bytes read), and the logical record length (`f->record->size`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineRead {
    pub at_end: bool,
    /// The record area (`record_max` bytes, space-filled past `size`); empty when `at_end`.
    pub record: Vec<u8>,
    /// `f->record->size` — the number of bytes actually read into the record (0 when `at_end`).
    pub size: usize,
    /// `"00"` success · `"04"` record truncated, line continues (split off) · `"06"` record truncated,
    /// remainder is the next read (split on) · `"09"` a bad control byte was read · `"10"` end of file.
    pub status: &'static str,
}

/// Port of `fileio.c:lineseq_read` for `READ ... NEXT RECORD` over a byte image: reads one logical line
/// from `data` starting at `*pos` (advancing it), byte-for-byte as GnuCOBOL 3.2 `libcob`.
///
/// `record_max` is `f->record_max` (the FD record width). Line termination is `\n`; a `\r\n` folds to
/// `\n`; a lone `\r` is kept as a data byte. Per `cfg`: `ls_validate` flags any [`is_bad_char_read`]
/// byte with status `09` (the line is still read); else `ls_nulls` decodes a `0x00`-prefixed control
/// byte (an unescaped control byte → status `71`, a declared non-claim); else bytes are stored raw.
/// When the record fills to `record_max`, `ls_split` peeks for the line end (consuming a trailing
/// `\n`/`\r\n`, status `00`) or reports `06` and leaves the remainder for the next read; with `ls_split`
/// off the overflow is status `04` and the rest of the line is consumed and discarded. At EOF with no
/// bytes read the result is `at_end` with status `10`.
///
/// **Non-claims:** the multi-file (concatenated-input) chain, CODE-SET conversion (`sort_collating`),
/// `ls_validate > 1` printable-check, the `ls_nulls` error-recovery path after status `71`, and the
/// actual `FILE *` reads (declared OS boundary).
pub fn lineseq_read(data: &[u8], pos: &mut usize, record_max: usize, cfg: &LineSeqConfig) -> LineRead {
    let mut rec = vec![b' '; record_max];
    let mut i = 0usize;
    let mut sts = "00";
    loop {
        if *pos >= data.len() {
            if i == 0 {
                return LineRead { at_end: true, record: Vec::new(), size: 0, status: "10" };
            }
            break;
        }
        let mut n = data[*pos];
        *pos += 1;
        if n == b'\r' {
            // \r\n -> fold to \n (consume the \n); lone \r -> keep as data (leave the next byte).
            if *pos < data.len() && data[*pos] == b'\n' {
                *pos += 1;
                n = b'\n';
            }
        }
        if n == b'\n' {
            break;
        }
        if cfg.ls_validate {
            if is_bad_char_read(n) {
                sts = "09";
            }
        } else if cfg.ls_nulls {
            if n == 0 {
                if *pos >= data.len() || data[*pos] >= b' ' {
                    // EOF or a non-control byte after 0x00 -> bad NULL encoding (declared non-claim).
                    return LineRead { at_end: false, record: rec, size: i, status: "71" };
                }
                n = data[*pos];
                *pos += 1;
            } else if n < b' ' {
                return LineRead { at_end: false, record: rec, size: i, status: "71" };
            }
        }
        if i < record_max {
            rec[i] = n;
            i += 1;
            if i == record_max && cfg.ls_split {
                // record full: peek for the line terminator, else put the byte(s) back and report 06.
                let start = *pos;
                let mut peek = if *pos < data.len() { let c = data[*pos]; *pos += 1; Some(c) } else { None };
                if peek == Some(b'\r') {
                    peek = if *pos < data.len() { let c = data[*pos]; *pos += 1; Some(c) } else { None };
                }
                if peek != Some(b'\n') {
                    *pos = start; // un-read the peeked byte(s)
                    sts = "06";
                }
                break;
            }
        } else if i == record_max {
            // split off: the line overflows the record -> status 04, discard the rest of the line.
            sts = "04";
        }
    }
    LineRead { at_end: false, record: rec, size: i, status: sts }
}

/// Replay `OPEN INPUT` + repeated `READ NEXT ... AT END` over `data` for the declared `record_max` and
/// config, returning every read event (the trailing event is always `at_end` with status `"10"`).
pub fn read_line_sequential(data: &[u8], record_max: usize, cfg: &LineSeqConfig) -> Vec<LineRead> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    loop {
        let r = lineseq_read(data, &mut pos, record_max, cfg);
        let end = r.at_end;
        out.push(r);
        if end {
            break;
        }
    }
    out
}

/// Replay `OPEN OUTPUT` + a sequence of `WRITE`s, concatenating the bytes each produces. Stops at the
/// first record that fails validation (status `"71"`) — as libcob does, the failing `WRITE` emits
/// nothing and the run aborts — returning the bytes written so far and the terminating status.
pub fn write_line_sequential(records: &[&[u8]], cfg: &LineSeqConfig) -> (Vec<u8>, &'static str) {
    let mut out = Vec::new();
    for &rec in records {
        let w = lineseq_write(rec, cfg);
        if w.status != "00" {
            return (out, w.status);
        }
        out.extend_from_slice(&w.bytes);
    }
    (out, "00")
}

// ======================================================================================================
// RECORD SEQUENTIAL organization (`GNURUST.FILEIO.SEQ.1`)
// ======================================================================================================

/// The record-length-prefix width for a variable-length RECORD SEQUENTIAL file (`cob_vsq_len`):
/// 2 bytes for `COB_VARSEQ_FORMAT=3`, otherwise 4.
pub fn cob_vsq_len(varseq_type: u8) -> usize {
    if varseq_type == 3 {
        2
    } else {
        4
    }
}

/// The variable-length size prefix `sequential_write` emits, per `COB_VARSEQ_FORMAT` (`cob_varseq_type`),
/// verified byte-exact against the oracle:
/// - `0` (default): `BE16(size)` then two `0x00` bytes (4 bytes total)
/// - `1`: `BE32(size)` (4 bytes) · `2`: native little-endian `LE32(size)` (4 bytes) · `3`: `BE16(size)` (2 bytes)
fn varseq_prefix(size: usize, varseq_type: u8) -> Vec<u8> {
    match varseq_type {
        1 => (size as u32).to_be_bytes().to_vec(),
        2 => (size as u32).to_le_bytes().to_vec(),
        3 => (size as u16).to_be_bytes().to_vec(),
        // 0 and any other: BE16 size in the first two bytes, the rest of the 4-byte field zero.
        _ => {
            let mut v = (size as u16).to_be_bytes().to_vec();
            v.extend_from_slice(&[0u8, 0u8]);
            v
        }
    }
}

/// Port of `fileio.c:sequential_write` for the unadvanced write (`opt == 0`): the bytes a `WRITE`
/// appends to a RECORD SEQUENTIAL file. For a **fixed** record (`record_min == record_max`, so
/// `variable` is false) the full record area is written with no prefix; for a **variable-length**
/// record a [`varseq_prefix`] size prefix precedes the `size` data bytes. The `record` buffer is the
/// FD record area (`size` bytes of which are live). No delimiter is ever added.
///
/// **Non-claims:** `WRITE ... ADVANCING` (the `opt != 0` advancing path) and CODE-SET conversion.
pub fn sequential_write(record: &[u8], size: usize, variable: bool, varseq_type: u8) -> Vec<u8> {
    let mut out = Vec::new();
    if variable {
        out.extend_from_slice(&varseq_prefix(size, varseq_type));
    }
    let n = size.min(record.len());
    out.extend_from_slice(&record[..n]);
    out
}

/// Port of `fileio.c:set_sequential_variable_length` — read the `cob_vsq_len`-byte record-length prefix
/// at `data[*pos..]` (advancing the cursor) and return the record size, or a FILE STATUS on error
/// (`"10"` EOF when no prefix bytes remain, `"39"` conflicting attribute on a malformed prefix).
pub fn set_sequential_variable_length(data: &[u8], pos: &mut usize, varseq_type: u8) -> Result<usize, &'static str> {
    let vlen = cob_vsq_len(varseq_type);
    if *pos >= data.len() {
        return Err("10"); // bytesread == 0 -> end of file
    }
    if *pos + vlen > data.len() {
        return Err("39"); // a partial prefix -> conflicting attribute
    }
    let buf = &data[*pos..*pos + vlen];
    *pos += vlen;
    let size = match varseq_type {
        0 => {
            if buf[2] != 0 || buf[3] != 0 {
                return Err("39"); // type 0 expects two trailing NULs
            }
            u16::from_be_bytes([buf[0], buf[1]]) as usize
        }
        1 => u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize,
        2 => u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize,
        _ => u16::from_be_bytes([buf[0], buf[1]]) as usize, // type 3 (and any default)
    };
    Ok(size)
}

/// The outcome of one `READ NEXT` from a RECORD SEQUENTIAL file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeqReadResult {
    pub at_end: bool,
    /// `f->record->size` after the read (bytes actually delivered into the record area).
    pub size: usize,
    /// `"00"` success · `"04"` record truncated/short · `"10"` end of file · `"39"` malformed prefix.
    pub status: &'static str,
}

/// Port of `fileio.c:sequential_read`: read one record from `data` at `*pos` into `record_buf`
/// (the persistent `record_max`-wide FD record area, kept across reads), advancing the cursor.
///
/// For a **fixed** record (`record_min == record_max`) it reads `record_max` bytes; a short final read
/// overwrites only the bytes available and **leaves the prior record's tail** in `record_buf` (status
/// `"00"`, `size = bytesread`), and a read of zero bytes is end of file (`"10"`). For a **variable**
/// record it first parses the [`set_sequential_variable_length`] prefix, then reads that many data
/// bytes (clamped to `record_max`; an over/under-length record yields status `"04"`).
///
/// **Non-claims:** CODE-SET conversion and the over-long `bytes_to_skip` seek-past on a record whose
/// declared length exceeds `record_max` (the prefix is honored; the trailing data handling is a
/// declared boundary).
pub fn sequential_read(
    data: &[u8],
    pos: &mut usize,
    record_buf: &mut [u8],
    record_min: usize,
    record_max: usize,
    varseq_type: u8,
) -> SeqReadResult {
    let mut ret = "00";
    let mut want = record_max;
    let variable = record_min != record_max;
    if variable {
        match set_sequential_variable_length(data, pos, varseq_type) {
            Err(s) => return SeqReadResult { at_end: s == "10", size: 0, status: s },
            Ok(sz) => {
                want = sz;
                if sz < record_min || sz > record_max {
                    ret = "04";
                    want = want.min(record_max);
                }
            }
        }
    }
    let avail = data.len() - *pos;
    let bytesread = avail.min(want);
    if bytesread == 0 {
        if !variable {
            return SeqReadResult { at_end: true, size: 0, status: "10" };
        }
        return SeqReadResult { at_end: false, size: 0, status: "04" };
    }
    record_buf[..bytesread].copy_from_slice(&data[*pos..*pos + bytesread]);
    *pos += bytesread;
    let size = if bytesread != want { bytesread } else { want };
    SeqReadResult { at_end: false, size, status: ret }
}

/// Port of `fileio.c:sequential_rewrite`: overwrite the just-read fixed record in place with
/// `record[..size]`, returning the updated file bytes. (`REWRITE` seeks back `record->size` and writes
/// the same length.) An out-of-range offset leaves the file unchanged.
pub fn sequential_rewrite(file: &[u8], record_off: usize, record: &[u8], size: usize) -> Vec<u8> {
    let mut out = file.to_vec();
    let n = size.min(record.len());
    if record_off + n <= out.len() {
        out[record_off..record_off + n].copy_from_slice(&record[..n]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLAIN: LineSeqConfig = LineSeqConfig { ls_fixed: false, ls_nulls: false, ls_validate: false, ls_split: true };
    const NULLS: LineSeqConfig = LineSeqConfig { ls_fixed: false, ls_nulls: true, ls_validate: false, ls_split: true };
    const FIXED: LineSeqConfig = LineSeqConfig { ls_fixed: true, ls_nulls: false, ls_validate: false, ls_split: true };
    const FIXED_NULLS: LineSeqConfig = LineSeqConfig { ls_fixed: true, ls_nulls: true, ls_validate: false, ls_split: true };

    #[test]
    fn size_strips_trailing_spaces() {
        assert_eq!(lineseq_size(b"AB      ", 8, false), 2);
        assert_eq!(lineseq_size(b"HELLO123", 8, false), 8);
        assert_eq!(lineseq_size(b"        ", 8, false), 0);
        assert_eq!(lineseq_size(b"AB      ", 8, true), 8); // fixed: no strip
    }

    #[test]
    fn default_validate_strips_and_adds_lf() {
        // oracle: AB -> "AB\n", HELLO123 -> "HELLO123\n", SPACES -> "\n"
        assert_eq!(lineseq_write(b"AB      ", &LineSeqConfig::DEFAULT).bytes, b"AB\n");
        assert_eq!(lineseq_write(b"HELLO123", &LineSeqConfig::DEFAULT).bytes, b"HELLO123\n");
        assert_eq!(lineseq_write(b"        ", &LineSeqConfig::DEFAULT).bytes, b"\n");
    }

    #[test]
    fn default_validate_rejects_control_byte() {
        // oracle: "A\tB" under validate=1 -> status 71, nothing written (every byte <0x20 is bad)
        let w = lineseq_write(b"A\x09B     ", &LineSeqConfig::DEFAULT);
        assert_eq!(w.status, "71");
        assert!(w.bytes.is_empty());
    }

    #[test]
    fn plain_passes_control_bytes_raw() {
        // oracle (validate=0): "A\tB" -> "A\tB\n"
        assert_eq!(lineseq_write(b"A\x09B     ", &PLAIN).bytes, b"A\x09B\n");
        assert_eq!(lineseq_write(b"A\x09B     ", &PLAIN).status, "00");
    }

    #[test]
    fn nulls_encodes_control_bytes() {
        // oracle (validate=0, nulls=1): "A\tB" -> 41 00 09 42 0a
        assert_eq!(lineseq_write(b"A\x09B     ", &NULLS).bytes, &[0x41, 0x00, 0x09, 0x42, 0x0a]);
    }

    #[test]
    fn fixed_writes_full_area() {
        // oracle (validate=0, fixed=1): "AB" -> full 8 bytes + \n
        assert_eq!(lineseq_write(b"AB      ", &FIXED).bytes, b"AB      \n");
    }

    #[test]
    fn fixed_nulls_full_area_encoded() {
        // oracle: fixed+nulls: "A\tB     " -> 41 00 09 42 20 20 20 20 20 0a
        assert_eq!(
            lineseq_write(b"A\x09B     ", &FIXED_NULLS).bytes,
            &[0x41, 0x00, 0x09, 0x42, 0x20, 0x20, 0x20, 0x20, 0x20, 0x0a]
        );
    }

    #[test]
    fn write_sequence_stops_at_validation_failure() {
        let (bytes, sts) = write_line_sequential(&[b"AB      ", b"A\x09B     ", b"XY      "], &LineSeqConfig::DEFAULT);
        assert_eq!(bytes, b"AB\n"); // wrote AB, then the tab record failed
        assert_eq!(sts, "71");
    }

    // ---- lineseq_read ----
    fn rd(data: &[u8], cfg: &LineSeqConfig) -> Vec<(Vec<u8>, &'static str)> {
        read_line_sequential(data, 8, cfg)
            .into_iter()
            .filter(|r| !r.at_end)
            .map(|r| (r.record, r.status))
            .collect()
    }

    #[test]
    fn read_basic_and_crlf_fold() {
        assert_eq!(rd(b"AB\nCD\n", &LineSeqConfig::DEFAULT), vec![(b"AB      ".to_vec(), "00"), (b"CD      ".to_vec(), "00")]);
        // \r\n folds to one line break
        assert_eq!(rd(b"AB\r\nCD\n", &LineSeqConfig::DEFAULT), vec![(b"AB      ".to_vec(), "00"), (b"CD      ".to_vec(), "00")]);
    }

    #[test]
    fn read_validate_tab_ok_cr_bad() {
        // TAB (0x09) is an IS_BAD_CHAR exclusion -> status 00; lone CR (0x0d) is bad -> status 09 (kept as data)
        assert_eq!(rd(b"A\x09B\n", &LineSeqConfig::DEFAULT), vec![(b"A\x09B     ".to_vec(), "00")]);
        assert_eq!(rd(b"A\x0dB\n", &LineSeqConfig::DEFAULT), vec![(b"A\x0dB     ".to_vec(), "09")]);
    }

    #[test]
    fn read_plain_passes_control_raw() {
        assert_eq!(rd(b"A\x0dB\n", &PLAIN), vec![(b"A\x0dB     ".to_vec(), "00")]);
    }

    #[test]
    fn read_nulls_decode() {
        // 41 00 09 42 00 09 42 00 00 0a -> "A\tB\tB\0" decoded
        assert_eq!(rd(b"A\x00\x09B\x00\x09B\x00\x00\n", &NULLS), vec![(b"A\x09B\x09B\x00  ".to_vec(), "00")]);
    }

    #[test]
    fn read_split_long_line() {
        // "ABCDEFGHIJ\n" -> split on: 06 "ABCDEFGH", 00 "IJ"
        assert_eq!(rd(b"ABCDEFGHIJ\n", &LineSeqConfig::DEFAULT), vec![(b"ABCDEFGH".to_vec(), "06"), (b"IJ      ".to_vec(), "00")]);
        // exactly record_max + \n -> single 00 (the \n is consumed at the boundary peek)
        assert_eq!(rd(b"ABCDEFGH\n", &LineSeqConfig::DEFAULT), vec![(b"ABCDEFGH".to_vec(), "00")]);
    }

    #[test]
    fn read_no_split_truncates_with_04() {
        // split OFF: "ABCDEFGHIJ\n" -> 04 "ABCDEFGH", the rest of the line is discarded (one record)
        let nosplit = LineSeqConfig { ls_fixed: false, ls_nulls: false, ls_validate: true, ls_split: false };
        assert_eq!(rd(b"ABCDEFGHIJ\n", &nosplit), vec![(b"ABCDEFGH".to_vec(), "04")]);
    }

    #[test]
    fn read_trailing_and_empty() {
        // no trailing newline keeps the last line
        assert_eq!(rd(b"AB", &LineSeqConfig::DEFAULT), vec![(b"AB      ".to_vec(), "00")]);
        // empty file -> immediate AT END (no records)
        assert!(rd(b"", &LineSeqConfig::DEFAULT).is_empty());
        // a mid-file empty line is a record (all spaces)
        assert_eq!(rd(b"AB\n\nCD\n", &LineSeqConfig::DEFAULT), vec![(b"AB      ".to_vec(), "00"), (b"        ".to_vec(), "00"), (b"CD      ".to_vec(), "00")]);
    }

    // ---- RECORD SEQUENTIAL ----
    #[test]
    fn varseq_prefix_all_formats() {
        // oracle: WRITE "AB" (size 2) variable -> the prefix per COB_VARSEQ_FORMAT
        assert_eq!(varseq_prefix(2, 0), vec![0x00, 0x02, 0x00, 0x00]); // BE16 + 00 00 (default)
        assert_eq!(varseq_prefix(2, 1), vec![0x00, 0x00, 0x00, 0x02]); // BE32
        assert_eq!(varseq_prefix(2, 2), vec![0x02, 0x00, 0x00, 0x00]); // native LE32
        assert_eq!(varseq_prefix(2, 3), vec![0x00, 0x02]); // BE16 (2 bytes)
    }

    #[test]
    fn sequential_write_variable_matches_oracle() {
        // oracle (VARSEQ=0): "AB"/2, "HELLO"/5 -> 0002000041420005000048454c4c4f
        let mut out = Vec::new();
        out.extend_from_slice(&sequential_write(b"AB      ", 2, true, 0));
        out.extend_from_slice(&sequential_write(b"HELLO   ", 5, true, 0));
        assert_eq!(out, b"\x00\x02\x00\x00AB\x00\x05\x00\x00HELLO");
        // VARSEQ=3 (2-byte BE16): "AB"/2 -> 0002 4142
        assert_eq!(sequential_write(b"AB      ", 2, true, 3), b"\x00\x02AB");
    }

    #[test]
    fn sequential_write_fixed_is_full_record() {
        // fixed: no prefix, full record area (record_max=8)
        assert_eq!(sequential_write(b"AB      ", 8, false, 0), b"AB      ");
    }

    #[test]
    fn set_varlen_roundtrips_every_format() {
        for ty in [0u8, 1, 2, 3] {
            let bytes = sequential_write(b"HELLO123", 5, true, ty);
            let mut pos = 0usize;
            let sz = set_sequential_variable_length(&bytes, &mut pos, ty).unwrap();
            assert_eq!(sz, 5, "format {ty}");
            assert_eq!(pos, cob_vsq_len(ty), "format {ty} prefix width");
            assert_eq!(&bytes[pos..pos + sz], b"HELLO", "format {ty} data");
        }
    }

    #[test]
    fn set_varlen_eof_and_malformed() {
        let mut p = 0usize;
        assert_eq!(set_sequential_variable_length(b"", &mut p, 0), Err("10")); // no bytes -> EOF
        let mut p = 0usize;
        assert_eq!(set_sequential_variable_length(b"\x00\x02\x01\x00", &mut p, 0), Err("39")); // type 0 non-zero tail
    }

    #[test]
    fn sequential_read_fixed_chunks_and_eof() {
        let data = b"01234567ABCDEFGH";
        let mut buf = vec![0u8; 8];
        let mut pos = 0usize;
        let r1 = sequential_read(data, &mut pos, &mut buf, 8, 8, 0);
        assert_eq!((&buf[..], r1.status, r1.size), (&b"01234567"[..], "00", 8));
        let r2 = sequential_read(data, &mut pos, &mut buf, 8, 8, 0);
        assert_eq!((&buf[..], r2.status), (&b"ABCDEFGH"[..], "00"));
        let r3 = sequential_read(data, &mut pos, &mut buf, 8, 8, 0);
        assert!(r3.at_end && r3.status == "10");
    }

    #[test]
    fn sequential_read_short_final_leaks_prior_tail() {
        // a 4-byte final record overlays only WXYZ, leaving EFGH from the prior record
        let data = b"ABCDEFGHWXYZ";
        let mut buf = vec![0u8; 8];
        let mut pos = 0usize;
        let _ = sequential_read(data, &mut pos, &mut buf, 8, 8, 0); // "ABCDEFGH"
        let r = sequential_read(data, &mut pos, &mut buf, 8, 8, 0);
        assert_eq!(&buf[..], b"WXYZEFGH");
        assert_eq!((r.status, r.size), ("00", 4));
    }

    #[test]
    fn sequential_read_variable_roundtrip() {
        // write 3 variable records, read them back (record_min=1 record_max=8)
        let mut file = Vec::new();
        for (d, s) in [(&b"AB      "[..], 2usize), (b"HELLO   ", 5), (b"XYZ12678", 8)] {
            file.extend_from_slice(&sequential_write(d, s, true, 0));
        }
        let mut buf = vec![b' '; 8];
        let mut pos = 0usize;
        let r1 = sequential_read(&file, &mut pos, &mut buf, 1, 8, 0);
        assert_eq!((r1.status, r1.size, &buf[..2]), ("00", 2, &b"AB"[..]));
        let r2 = sequential_read(&file, &mut pos, &mut buf, 1, 8, 0);
        assert_eq!((r2.status, r2.size, &buf[..5]), ("00", 5, &b"HELLO"[..]));
        let r3 = sequential_read(&file, &mut pos, &mut buf, 1, 8, 0);
        assert_eq!((r3.status, r3.size, &buf[..]), ("00", 8, &b"XYZ12678"[..]));
        assert!(sequential_read(&file, &mut pos, &mut buf, 1, 8, 0).at_end);
    }

    #[test]
    fn sequential_rewrite_in_place() {
        assert_eq!(sequential_rewrite(b"AAAABBBBCCCC", 4, b"X1X1", 4), b"AAAAX1X1CCCC");
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    // KANIFOR: GNURUST.FILEIO.LINESEQ.1
    /// A successful default-validate WRITE always ends in exactly one LF and never contains a byte < 0x20.
    #[kani::proof]
    #[kani::unwind(9)]
    fn validate_output_is_clean_and_lf_terminated() {
        let rec: [u8; 8] = kani::any();
        let w = lineseq_write(&rec, &LineSeqConfig::DEFAULT);
        if w.status == "00" {
            assert_eq!(*w.bytes.last().unwrap(), b'\n');
            // every byte before the terminating LF is >= 0x20 (validate rejected anything below)
            for &b in &w.bytes[..w.bytes.len() - 1] {
                assert!(b >= b' ');
            }
        } else {
            assert!(w.bytes.is_empty());
        }
    }
    // KANIFOR: GNURUST.FILEIO.LINESEQ.2
    /// A non-AT-END read always fills the record area to exactly record_max and reports a known status;
    /// the cursor never runs backwards past where it started.
    #[kani::proof]
    #[kani::unwind(6)]
    fn read_record_width_and_status() {
        let data: [u8; 4] = kani::any();
        let mut pos = 0usize;
        let r = lineseq_read(&data, &mut pos, 4, &LineSeqConfig::DEFAULT);
        if !r.at_end {
            assert_eq!(r.record.len(), 4);
            assert!(matches!(r.status, "00" | "04" | "06" | "09" | "71"));
            assert!(r.size <= 4);
        }
        assert!(pos <= data.len());
    }
    // KANIFOR: GNURUST.FILEIO.SEQ.1
    /// A variable-length write/read round-trip never panics; the parsed size is bounded by record_max.
    #[kani::proof]
    #[kani::unwind(6)]
    fn seqrec_varlen_roundtrip_total() {
        let rec: [u8; 4] = kani::any();
        let size: usize = kani::any();
        kani::assume(size <= 4);
        let ty: u8 = kani::any();
        kani::assume(ty <= 3);
        let bytes = sequential_write(&rec, size, true, ty);
        let mut pos = 0usize;
        if let Ok(sz) = set_sequential_variable_length(&bytes, &mut pos, ty) {
            assert_eq!(sz, size);
        }
    }
}
