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

use std::cmp::Ordering;

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

// COB_WRITE_* option bits (common.h): the WRITE ADVANCING encoding.
const COB_WRITE_MASK: i32 = 0x0000_FFFF;
const COB_WRITE_LINES: i32 = 0x0001_0000;
const COB_WRITE_PAGE: i32 = 0x0002_0000;

/// Port of `fileio.c:cob_seq_write_opt` — the advancing bytes a RECORD SEQUENTIAL `WRITE ... ADVANCING`
/// emits: `ADVANCING n LINES` → `n` newlines (or a single `\r` when `n == 0`), `ADVANCING PAGE` → a
/// form-feed (`\f`), otherwise nothing.
pub fn cob_seq_write_opt(opt: i32) -> Vec<u8> {
    if opt & COB_WRITE_LINES != 0 {
        let n = opt & COB_WRITE_MASK;
        if n == 0 {
            vec![b'\r']
        } else {
            vec![b'\n'; n as usize]
        }
    } else if opt & COB_WRITE_PAGE != 0 {
        vec![0x0c]
    } else {
        Vec::new()
    }
}

/// Port of `fileio.c:cob_file_write_opt` for a non-LINAGE file — identical advancing bytes to
/// [`cob_seq_write_opt`] (`\n`×n / `\r` / `\f`). The LINAGE-clause advancing path is [`cob_linage_write_opt`].
pub fn cob_file_write_opt(opt: i32) -> Vec<u8> {
    cob_seq_write_opt(opt)
}

/// A LINAGE page geometry + the current line counter, as read by `cob_linage_write_opt`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Linage {
    pub lin_lines: i32,
    pub lin_top: i32,
    pub lin_bot: i32,
    /// the live `LINAGE-COUNTER`.
    pub counter: i32,
}

/// Port of `fileio.c:cob_linage_write_opt` — the page-advancing newlines for a `WRITE ADVANCING` on a
/// LINAGE file, returning the emitted bytes, the updated `LINAGE-COUNTER`, and the FILE STATUS (`"00"`
/// or `"57"` when the linage geometry is invalid). `ADVANCING PAGE` finishes the page (newlines to the
/// bottom + footer + top margin, counter → 1); `ADVANCING n LINES` advances the counter, rolling to a
/// fresh page (bottom + top margins) when it passes `lin_lines`, else emitting `n-1` newlines.
pub fn cob_linage_write_opt(lin: Linage, opt: i32) -> (Vec<u8>, i32, &'static str) {
    let mut out = Vec::new();
    let mut counter = lin.counter;
    if opt & COB_WRITE_PAGE != 0 {
        if counter == 0 {
            return (out, counter, "57");
        }
        let mut i = counter;
        while i < lin.lin_lines {
            out.push(b'\n');
            i += 1;
        }
        for _ in 0..lin.lin_bot {
            out.push(b'\n');
        }
        if lin.lin_lines < 1 {
            return (out, counter, "57");
        }
        for _ in 0..lin.lin_top {
            out.push(b'\n');
        }
        counter = 1;
    } else if opt & COB_WRITE_LINES != 0 {
        if counter == 0 {
            return (out, counter, "57");
        }
        counter += opt & COB_WRITE_MASK;
        if counter > lin.lin_lines {
            let mut n = lin.counter;
            while n < lin.lin_lines {
                out.push(b'\n');
                n += 1;
            }
            for _ in 0..lin.lin_bot {
                out.push(b'\n');
            }
            if lin.lin_lines < 1 {
                return (out, counter, "57");
            }
            counter = 1;
            for _ in 0..lin.lin_top {
                out.push(b'\n');
            }
        } else {
            for _ in 0..((opt & COB_WRITE_MASK) - 1) {
                out.push(b'\n');
            }
        }
    }
    (out, counter, "00")
}

/// Port of `fileio.c:cob_copy_check` — copy `from` into a fresh `to_size`-byte record: when the target
/// is wider the source is copied and the remainder space-filled, otherwise the source is truncated to
/// fit. (The SORT/MERGE record-move semantics.)
pub fn cob_copy_check(from: &[u8], to_size: usize) -> Vec<u8> {
    let mut to = vec![b' '; to_size];
    let n = from.len().min(to_size);
    to[..n].copy_from_slice(&from[..n]);
    to
}

/// Port of `fileio.c:file_linage_check` — validate a LINAGE clause's geometry, returning the resolved
/// `(lin_lines, lin_foot, lin_top, lin_bot)` or `Err(())` (the C status `1`, which zeroes the counter).
/// `lin_lines` must be `>= 1`; a present FOOTING must be `1..=lin_lines`; a present TOP/BOTTOM must be
/// `>= 0`; an absent FOOTING/TOP/BOTTOM resolves to `0`.
pub fn file_linage_check(lin_lines: i32, lin_foot: Option<i32>, lin_top: Option<i32>, lin_bot: Option<i32>) -> Result<(i32, i32, i32, i32), ()> {
    if lin_lines < 1 {
        return Err(());
    }
    let foot = match lin_foot {
        Some(v) if v < 1 || v > lin_lines => return Err(()),
        Some(v) => v,
        None => 0,
    };
    let top = match lin_top {
        Some(v) if v < 0 => return Err(()),
        Some(v) => v,
        None => 0,
    };
    let bot = match lin_bot {
        Some(v) if v < 0 => return Err(()),
        Some(v) => v,
        None => 0,
    };
    Ok((lin_lines, foot, top, bot))
}

/// Port of the byte-core of `fileio.c:is_suppressed_key_value` — is key `idx`'s value entirely the
/// SUPPRESS character (so the key is treated as absent)? Returns `1` if `tf_suppress` and every byte of
/// the key field equals `suppress_char`, else `0`.
pub fn is_suppressed_key_value(key_field: &[u8], suppress_char: u8, tf_suppress: bool) -> i32 {
    if tf_suppress && !key_field.is_empty() && key_field.iter().all(|&b| b == suppress_char) {
        1
    } else {
        0
    }
}

/// Port of the byte-core of `fileio.c:lineseq_rewrite` — `REWRITE` a LINE SEQUENTIAL record in place.
/// `slotlen` is the original line's length (without its newline); the new `record[..size]` (NUL-encoded
/// when `ls_nulls`) must fit, else status `"44"` (record overflow). It is written over the slot and the
/// remainder is space-padded so the surrounding bytes are undisturbed.
pub fn lineseq_rewrite(file: &[u8], record_off: usize, slotlen: usize, record: &[u8], size: usize, cfg: &LineSeqConfig) -> RelWrite {
    let data = &record[..size.min(record.len())];
    // build the bytes to write (validate -> 71; nulls -> 0x00 prefix; else raw)
    let mut body = Vec::new();
    if cfg.ls_validate {
        if data.iter().any(|&b| is_bad_char(b)) {
            return RelWrite { file: file.to_vec(), status: "71" };
        }
        body.extend_from_slice(data);
    } else if cfg.ls_nulls {
        for &b in data {
            if b < b' ' {
                body.push(0);
            }
            body.push(b);
        }
    } else {
        body.extend_from_slice(data);
    }
    if body.len() > slotlen {
        return RelWrite { file: file.to_vec(), status: "44" };
    }
    let mut out = file.to_vec();
    if record_off + slotlen <= out.len() {
        out[record_off..record_off + body.len()].copy_from_slice(&body);
        // pad the rest of the slot with spaces
        for b in out.iter_mut().skip(record_off + body.len()).take(slotlen - body.len()) {
            *b = b' ';
        }
    }
    RelWrite { file: out, status: "00" }
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

// ======================================================================================================
// Path / filename + status helpers (structural 1:1 ports of the pure `fileio.c` helpers)
//
// These are faithful ports of `fileio.c`'s pure string/status helpers (no I/O, no runtime config), each
// verified by unit tests against the source semantics. They are the decision logic the OS-facing open/
// map routines call; the syscalls themselves remain the declared OS boundary.
// ======================================================================================================

/// Port of `fileio.c:has_directory_separator` — does the name contain a `/` or `\` path separator?
pub fn has_directory_separator(src: &[u8]) -> bool {
    src.iter().any(|&c| c == b'/' || c == b'\\')
}

/// Port of `fileio.c:looks_absolute` — does the name (after an optional surrounding quote) begin with a
/// path separator? (The Win32 drive-letter case is a declared platform boundary.)
pub fn looks_absolute(src: &[u8]) -> bool {
    let s = if src.first() == Some(&0x22) || src.first() == Some(&0x27) { &src[1..] } else { src };
    s.first() == Some(&b'/') || s.first() == Some(&b'\\')
}

/// Port of `fileio.c:is_absolute` (non-Windows) — an absolute path begins with `/`.
pub fn is_absolute(filename: &[u8]) -> bool {
    filename.first() == Some(&b'/')
}

/// Port of `fileio.c:has_acu_hyphen` — the ACUCOBOL special case: a name beginning `-F`/`-D`/`-f`/`-d`
/// followed by whitespace (the device-assignment form, no path translation).
pub fn has_acu_hyphen(src: &[u8]) -> bool {
    src.len() >= 3
        && src[0] == b'-'
        && matches!(src[1], b'F' | b'D' | b'f' | b'd')
        && src[2].is_ascii_whitespace()
}

/// Port of `fileio.c:do_acu_hyphen_translation` — for an [`has_acu_hyphen`] name, the actual filename is
/// what follows the `-F `/`-D ` prefix after the first non-space, with surrounding matching quotes dropped.
pub fn do_acu_hyphen_translation(src: &[u8]) -> Vec<u8> {
    // skip the "-F"/"-D" (2 chars) then any whitespace (the C starts at src+3 then skips spaces; src[2]
    // is already known whitespace, so from index 2 skip all whitespace).
    let mut i = 2usize.min(src.len());
    while i < src.len() && src[i].is_ascii_whitespace() {
        i += 1;
    }
    let mut s = &src[i..];
    if s.len() >= 2 && (s[0] == 0x22 || s[0] == 0x27) && s[0] == s[s.len() - 1] {
        s = &s[1..s.len() - 1];
    }
    s.to_vec()
}

/// An `errno` value, as the subset `fileio.c:errno_cob_sts` distinguishes when mapping a failed syscall
/// to a FILE STATUS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileErrno {
    /// `ENOSPC` / `EDQUOT` — out of space / over quota.
    NoSpaceOrQuota,
    /// `EPERM` / `EACCES` / `EISDIR` — permission denied / is a directory.
    PermissionOrIsDir,
    /// `ENOENT` — no such file.
    NotExist,
    /// any other `errno`.
    Other,
}

/// Port of `fileio.c:errno_cob_sts` — map a failed syscall's `errno` to a FILE STATUS, falling back to
/// `default_status` for unrecognised errors. `ENOSPC`/`EDQUOT` → `"34"`, `EPERM`/`EACCES`/`EISDIR` →
/// `"37"`, `ENOENT` → `"35"`.
pub fn errno_cob_sts(err: FileErrno, default_status: &'static str) -> &'static str {
    match err {
        FileErrno::NoSpaceOrQuota => "34",
        FileErrno::PermissionOrIsDir => "37",
        FileErrno::NotExist => "35",
        FileErrno::Other => default_status,
    }
}

/// Port of `fileio.c:dummy_delete` — the DELETE handler for a file organization that does not support it:
/// always status `"91"` (not available). (`dummy_read`/`dummy_start` are the matching no-op handlers.)
pub fn dummy_delete() -> &'static str {
    "91"
}
/// Port of `fileio.c:dummy_read` — the READ handler for an unsupported organization: status `"91"`.
pub fn dummy_read() -> &'static str {
    "91"
}
/// Port of `fileio.c:dummy_start` — the START handler for an unsupported organization: status `"91"`.
pub fn dummy_start() -> &'static str {
    "91"
}

/// Port of `fileio.c:dummy_rnxt_rewrite` — the disabled (`#if 0`) READ-NEXT/REWRITE no-op handler, ported
/// for completeness; not wired (status `"91"` not available).
#[allow(dead_code)]
pub fn dummy_rnxt_rewrite() -> &'static str {
    "91"
}

// ======================================================================================================
// CBL_* system file/directory routines (`GNURUST.FILEIO.SYS.1`)
//
// Faithful ports of `fileio.c`'s `cob_sys_*` library routines (the `CBL_DELETE_FILE` / `CBL_CREATE_DIR`
// / ... entry points a COBOL program CALLs). Each takes the already-resolved name(s) (the C extracts
// them from the COBOL parameters and applies filename mapping, the calling-convention boundary) and
// returns the routine's documented status: `0` success, `128` operation failed, `35` the
// source does not exist, `-1` a missing parameter. The actual syscall is performed via `std::fs` /
// `std::env`, matching libcob's `unlink`/`rename`/`mkdir`/`chdir`/`getcwd`/copy.
// ======================================================================================================

/// Port of `fileio.c:cob_sys_delete_file` (`CBL_DELETE_FILE`): remove `name`; `0` on success, `128` on
/// failure.
pub fn cob_sys_delete_file(name: &[u8]) -> i32 {
    match std::str::from_utf8(name) {
        Ok(p) if std::fs::remove_file(p).is_ok() => 0,
        _ => 128,
    }
}

/// Port of `fileio.c:cob_sys_copy_file` (`CBL_COPY_FILE`): copy `from` to `to`; `35` if the source does
/// not exist, `-1` on a write failure, `0` on success.
pub fn cob_sys_copy_file(from: &[u8], to: &[u8]) -> i32 {
    let (Ok(src), Ok(dst)) = (std::str::from_utf8(from), std::str::from_utf8(to)) else {
        return -1;
    };
    let data = match std::fs::read(src) {
        Ok(d) => d,
        Err(_) => return 35,
    };
    match std::fs::write(dst, &data) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// Port of `fileio.c:cob_sys_rename_file` (`CBL_RENAME_FILE`): rename `from` to `to`; `0` / `128`.
pub fn cob_sys_rename_file(from: &[u8], to: &[u8]) -> i32 {
    match (std::str::from_utf8(from), std::str::from_utf8(to)) {
        (Ok(a), Ok(b)) if std::fs::rename(a, b).is_ok() => 0,
        _ => 128,
    }
}

/// Port of `fileio.c:cob_sys_create_dir` (`CBL_CREATE_DIR`): create directory `name`; `0` / `128`.
pub fn cob_sys_create_dir(name: &[u8]) -> i32 {
    match std::str::from_utf8(name) {
        Ok(p) if std::fs::create_dir(p).is_ok() => 0,
        _ => 128,
    }
}

/// Port of `fileio.c:cob_sys_delete_dir` (`CBL_DELETE_DIR`): remove directory `name`; `0` / `128`.
pub fn cob_sys_delete_dir(name: &[u8]) -> i32 {
    match std::str::from_utf8(name) {
        Ok(p) if std::fs::remove_dir(p).is_ok() => 0,
        _ => 128,
    }
}

/// Port of `fileio.c:cob_sys_change_dir` (`CBL_CHANGE_DIR`): change the working directory to `name`;
/// `0` / `128`.
pub fn cob_sys_change_dir(name: &[u8]) -> i32 {
    match std::str::from_utf8(name) {
        Ok(p) if std::env::set_current_dir(p).is_ok() => 0,
        _ => 128,
    }
}

/// Port of `fileio.c:cob_sys_get_current_dir` (`CBL_GET_CURRENT_DIR`): write the working directory into a
/// `dir_length`-wide field (space-filled, double-quoted when it contains a space). `flags != 0` → `129`,
/// `dir_length < 1` or a name that does not fit → `128`. Returns `(status, field_bytes)`.
pub fn cob_sys_get_current_dir(flags: i32, dir_length: usize) -> (i32, Vec<u8>) {
    if dir_length < 1 {
        return (128, Vec::new());
    }
    if flags != 0 {
        return (129, vec![b' '; dir_length]);
    }
    let mut dir = vec![b' '; dir_length];
    let cwd = match std::env::current_dir() {
        Ok(p) => p.to_string_lossy().into_owned().into_bytes(),
        Err(_) => return (128, dir),
    };
    let has_space = if cwd.contains(&b' ') { 2usize } else { 0 };
    if cwd.len() + has_space > dir_length {
        return (128, dir);
    }
    if has_space != 0 {
        dir[0] = 0x22;
        dir[1..1 + cwd.len()].copy_from_slice(&cwd);
        dir[1 + cwd.len()] = 0x22;
    } else {
        dir[..cwd.len()].copy_from_slice(&cwd);
    }
    (0, dir)
}

// ======================================================================================================
// SORT/MERGE record comparison (`GNURUST.FILEIO.SORT.1`)
// ======================================================================================================

/// Port of `fileio.c:sort_cmps` — compare two equal-length keys byte-by-byte, optionally through a
/// 256-entry collating table; returns the signed difference of the first differing (translated) byte,
/// or `0` if equal.
pub fn sort_cmps(s1: &[u8], s2: &[u8], col: Option<&[u8; 256]>) -> i32 {
    for i in 0..s1.len().min(s2.len()) {
        let (a, b) = match col {
            Some(c) => (c[s1[i] as usize] as i32, c[s2[i] as usize] as i32),
            None => (s1[i] as i32, s2[i] as i32),
        };
        if a != b {
            return a - b;
        }
    }
    0
}

/// One `SORT ... ON {ASCENDING|DESCENDING} KEY` key: a byte range `[offset, offset+size)` of the record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SortKey {
    pub offset: usize,
    pub size: usize,
    /// `COB_ASCENDING` vs `COB_DESCENDING`.
    pub ascending: bool,
}

/// Port of `fileio.c:cob_file_sort_init_key` — append a key to the sort key list (in declaration order).
pub fn cob_file_sort_init_key(keys: &mut Vec<SortKey>, offset: usize, size: usize, ascending: bool) {
    keys.push(SortKey { offset, size, ascending });
}

/// Port of the alphanumeric path of `fileio.c:cob_file_sort_compare` — order two records by the sort
/// keys (each compared via [`sort_cmps`], negated for DESCENDING). A full key tie breaks by the records'
/// insertion order (`u1`/`u2`, the `unique` field), giving a **stable** sort. (Numeric keys, which the C
/// routes through `cob_numeric_cmp`, are a declared composition with `GNURUST.NUMCMP.1`.)
pub fn cob_file_sort_compare(rec1: &[u8], u1: usize, rec2: &[u8], u2: usize, keys: &[SortKey], col: Option<&[u8; 256]>) -> Ordering {
    for k in keys {
        let a = &rec1[k.offset.min(rec1.len())..(k.offset + k.size).min(rec1.len())];
        let b = &rec2[k.offset.min(rec2.len())..(k.offset + k.size).min(rec2.len())];
        let cmp = sort_cmps(a, b, col);
        if cmp != 0 {
            return if k.ascending { cmp } else { -cmp }.cmp(&0);
        }
    }
    u1.cmp(&u2)
}

/// Replay a `SORT` over `records` with the given keys: return the records' indices in sorted order
/// (a stable sort by [`cob_file_sort_compare`]).
pub fn sort_records(records: &[&[u8]], keys: &[SortKey], col: Option<&[u8; 256]>) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..records.len()).collect();
    idx.sort_by(|&i, &j| cob_file_sort_compare(records[i], i, records[j], j, keys, col));
    idx
}

// ======================================================================================================
// Verb-layer preconditions (`GNURUST.FILEIO.VERB.1`)
//
// The open-mode / access-mode / record-state checks the public `cob_*` verbs apply *before* dispatching
// to an organization handler — the FILE STATUS a WRITE/READ/REWRITE/DELETE/START produces when the file
// is in the wrong mode for the operation. Each returns `Some(status)` for an early precondition failure
// or `None` when the preconditions pass and the verb proceeds to its (separately-sealed) handler.
// ======================================================================================================

/// `OPEN` mode (`f->open_mode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenMode {
    Closed,
    Input,
    Output,
    Io,
    Extend,
}

/// `ACCESS MODE` (`f->access_mode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMode {
    Sequential,
    Random,
    Dynamic,
}

/// File `ORGANIZATION` (`f->organization`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Organization {
    Sequential,
    LineSequential,
    Relative,
    Indexed,
    Sort,
}

/// Port of the precondition layer of `fileio.c:cob_write` — the FILE STATUS before the handler runs.
/// In SEQUENTIAL access the file must be `OPEN OUTPUT`/`EXTEND`, otherwise `OUTPUT`/`I-O`, else `"48"`;
/// a record outside `[record_min, record_max]` is `"44"`. `None` means proceed to the write handler.
pub fn cob_write(open: OpenMode, access: AccessMode, rec_size: usize, record_min: usize, record_max: usize) -> Option<&'static str> {
    let ok = if access == AccessMode::Sequential {
        matches!(open, OpenMode::Output | OpenMode::Extend)
    } else {
        matches!(open, OpenMode::Output | OpenMode::Io)
    };
    if !ok {
        return Some("48");
    }
    if rec_size < record_min || rec_size > record_max {
        return Some("44");
    }
    None
}

/// Port of the precondition layer of `fileio.c:cob_rewrite`: the file must be `OPEN I-O` (`"49"`), a
/// SEQUENTIAL-access REWRITE requires a prior successful READ (`"43"`), and a RECORD SEQUENTIAL rewrite
/// must keep the record length (`"44"`). `None` means proceed.
pub fn cob_rewrite(open: OpenMode, access: AccessMode, org: Organization, read_done: bool, rec_size: usize, record_size: usize) -> Option<&'static str> {
    if open != OpenMode::Io {
        return Some("49");
    }
    if access == AccessMode::Sequential && !read_done {
        return Some("43");
    }
    if org == Organization::Sequential && record_size != rec_size {
        return Some("44");
    }
    None
}

/// Port of the precondition layer of `fileio.c:cob_delete`: the file must be `OPEN I-O` (`"49"`) and a
/// SEQUENTIAL-access DELETE requires a prior successful READ (`"43"`). `None` means proceed.
pub fn cob_delete(open: OpenMode, access: AccessMode, read_done: bool) -> Option<&'static str> {
    if open != OpenMode::Io {
        return Some("49");
    }
    if access == AccessMode::Sequential && !read_done {
        return Some("43");
    }
    None
}

/// Port of the precondition layer of `fileio.c:cob_start`: the file must be `OPEN INPUT`/`I-O` and not
/// RANDOM-access (`"47"`), must exist (`"23"`), and a supplied key size must be `1..=key.size` (`"23"`).
/// `None` means proceed.
pub fn cob_start(open: OpenMode, access: AccessMode, nonexistent: bool, keysize_valid: bool) -> Option<&'static str> {
    if !matches!(open, OpenMode::Io | OpenMode::Input) {
        return Some("47");
    }
    if access == AccessMode::Random {
        return Some("47");
    }
    if nonexistent {
        return Some("23");
    }
    if !keysize_valid {
        return Some("23");
    }
    None
}

/// Port of the precondition layer of `fileio.c:cob_read` (keyed or sequential): the file must be `OPEN
/// INPUT`/`I-O` (`"47"`); a nonexistent optional file is `"10"` on the first read else `"23"`; a
/// sequential read past end-of-file (or before begin, reading backwards) is `"46"`. `None` = proceed.
#[allow(clippy::too_many_arguments)]
pub fn cob_read(open: OpenMode, nonexistent: bool, first_read: bool, key_based: bool, end_of_file: bool, begin_of_file: bool, read_previous: bool) -> Option<&'static str> {
    if !matches!(open, OpenMode::Input | OpenMode::Io) {
        return Some("47");
    }
    if nonexistent {
        return Some(if first_read { "10" } else { "23" });
    }
    if !key_based {
        if end_of_file && !read_previous {
            return Some("46");
        }
        if begin_of_file && read_previous {
            return Some("46");
        }
    }
    None
}

/// Port of the precondition layer of `fileio.c:cob_read_next`: like [`cob_read`] but a nonexistent file
/// after the first read is `"46"` (not `"23"`), and the end/begin-of-file check always applies. `None` = proceed.
pub fn cob_read_next(open: OpenMode, nonexistent: bool, first_read: bool, end_of_file: bool, begin_of_file: bool, read_previous: bool) -> Option<&'static str> {
    if !matches!(open, OpenMode::Input | OpenMode::Io) {
        return Some("47");
    }
    if nonexistent {
        return Some(if first_read { "10" } else { "46" });
    }
    if end_of_file && !read_previous {
        return Some("46");
    }
    if begin_of_file && read_previous {
        return Some("46");
    }
    None
}

// ======================================================================================================
// INDEXED key-descriptor handling (structural 1:1 ports of the pure `fileio.c` key helpers)
//
// Faithful ports of the ISAM key-extraction helpers, which operate purely on a key descriptor (a set of
// (start, length) parts within the record) — the byte logic the indexed backend uses to build, save,
// restore and compare keys. The backing ISAM/BDB store itself is the declared OS boundary.
// ======================================================================================================

/// One component of a multi-part key: a byte range `[start, start+leng)` within the record
/// (`struct keydesc`'s `k_part`: `kp_start` / `kp_leng`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyPart {
    pub start: usize,
    pub leng: usize,
}

/// A key descriptor (`struct keydesc`): the parts that make up the key and whether duplicates are allowed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyDesc {
    pub parts: Vec<KeyPart>,
    /// `k_flags`: duplicates allowed (`ISDUPS`) vs not (`ISNODUPS`).
    pub duplicates: bool,
}

/// Port of `fileio.c:indexed_keylen` — the total key length (sum of all part lengths).
pub fn indexed_keylen(kd: &KeyDesc) -> usize {
    kd.parts.iter().map(|p| p.leng).sum()
}

/// Port of `fileio.c:indexed_savekey` — extract the key from `data` into a contiguous buffer by copying
/// each part's `[start, start+leng)` range in order. Returns the saved key (its length is [`indexed_keylen`]).
pub fn indexed_savekey(data: &[u8], kd: &KeyDesc) -> Vec<u8> {
    let mut out = Vec::with_capacity(indexed_keylen(kd));
    for p in &kd.parts {
        let end = (p.start + p.leng).min(data.len());
        let start = p.start.min(end);
        out.extend_from_slice(&data[start..end]);
        // pad if the record was shorter than the part (faithful to the C memcpy over the record area)
        out.resize(out.len() + (p.leng - (end - start)), 0);
    }
    out
}

/// Port of `fileio.c:indexed_restorekey` — copy a saved key back into `data` at each part's range (the
/// inverse of [`indexed_savekey`]).
pub fn indexed_restorekey(data: &mut [u8], savekey: &[u8], kd: &KeyDesc) {
    let mut totlen = 0usize;
    for p in &kd.parts {
        let n = p.leng.min(savekey.len().saturating_sub(totlen));
        let dend = (p.start + n).min(data.len());
        let dn = dend.saturating_sub(p.start);
        if dn > 0 {
            data[p.start..p.start + dn].copy_from_slice(&savekey[totlen..totlen + dn]);
        }
        totlen += p.leng;
    }
}

/// Port of `fileio.c:indexed_cmpkey` — compare the key extracted from `data` against `savekey`,
/// part by part (`memcmp`), up to `partlen` bytes (`<= 0` means the whole key). Returns the sign of the
/// first differing byte (negative / zero / positive), like the C `memcmp` chain.
pub fn indexed_cmpkey(data: &[u8], savekey: &[u8], kd: &KeyDesc, partlen: i32) -> i32 {
    let mut remaining = if partlen <= 0 { indexed_keylen(kd) as i32 } else { partlen };
    let mut totlen = 0usize;
    for p in &kd.parts {
        if remaining <= 0 {
            break;
        }
        let cl = (remaining as usize).min(p.leng);
        let a_end = (p.start + cl).min(data.len());
        let a = &data[p.start.min(a_end)..a_end];
        let b_end = (totlen + cl).min(savekey.len());
        let b = &savekey[totlen.min(b_end)..b_end];
        match a.cmp(b) {
            std::cmp::Ordering::Less => return -1,
            std::cmp::Ordering::Greater => return 1,
            std::cmp::Ordering::Equal => {}
        }
        totlen += p.leng;
        remaining -= p.leng as i32;
    }
    0
}

/// Port of `fileio.c:indexed_keycmp` — are two key descriptors identical (same flags, parts, and each
/// part's start+length)? Returns `0` if equal, `1` otherwise (matching the C convention).
pub fn indexed_keycmp(k1: &KeyDesc, k2: &KeyDesc) -> i32 {
    if k1.duplicates != k2.duplicates || k1.parts.len() != k2.parts.len() {
        return 1;
    }
    if k1.parts == k2.parts {
        0
    } else {
        1
    }
}

/// A `cob_file_key`: a key as declared on the file — either a single contiguous field at `offset` of
/// `field_size` bytes, or, when `components` is non-empty, a composite of those `(offset, size)` ranges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CobFileKey {
    pub duplicates: bool,
    pub offset: usize,
    pub field_size: usize,
    pub components: Vec<(usize, usize)>,
}

/// Port of `fileio.c:indexed_keydesc` — build a [`KeyDesc`] from a [`CobFileKey`]. With no declared
/// components the key is one part `(offset, field_size)`; otherwise each component becomes a part.
pub fn indexed_keydesc(key: &CobFileKey) -> KeyDesc {
    let parts = if key.components.is_empty() {
        vec![KeyPart { start: key.offset, leng: key.field_size }]
    } else {
        key.components.iter().map(|&(s, l)| KeyPart { start: s, leng: l }).collect()
    };
    KeyDesc { parts, duplicates: key.duplicates }
}

/// Port of `fileio.c:cob_savekey` — extract key `idx`'s bytes from the record. A single-component key
/// copies the contiguous field; a multi-component key concatenates its parts in order.
pub fn cob_savekey(record: &[u8], key: &CobFileKey) -> Vec<u8> {
    if key.components.len() <= 1 {
        let end = (key.offset + key.field_size).min(record.len());
        return record[key.offset.min(end)..end].to_vec();
    }
    let mut out = Vec::new();
    for &(s, l) in &key.components {
        let end = (s + l).min(record.len());
        out.extend_from_slice(&record[s.min(end)..end]);
    }
    out
}

/// Port of the byte-core of `fileio.c:save_status` — format an integer FILE STATUS into its 2 display
/// bytes: `0` → `"00"`, otherwise the tens and units digits (e.g. `23` → `"23"`, `10` → `"10"`). The
/// exception/sync/FCD side effects of the full routine are the declared runtime boundary.
pub fn save_status(status: u8) -> [u8; 2] {
    if status == 0 {
        [b'0', b'0']
    } else {
        [b'0' + status / 10, b'0' + status % 10]
    }
}

/// Port of `fileio.c:cob_str_from_fld` — the right-trimmed string content of a field: trailing spaces
/// **and** NUL bytes are dropped, embedded `"` (0x22) quote characters are removed, and an all-space /
/// all-NUL field yields an empty string.
pub fn cob_str_from_fld(field: &[u8]) -> Vec<u8> {
    if field.is_empty() {
        return Vec::new();
    }
    // right-trim trailing spaces and NULs
    let mut end = field.len();
    while end > 0 && (field[end - 1] == b' ' || field[end - 1] == 0) {
        end -= 1;
    }
    // drop embedded quote (0x22) bytes from the trimmed content
    field[..end].iter().copied().filter(|&b| b != 0x22).collect()
}

// ======================================================================================================
// RELATIVE organization (`GNURUST.FILEIO.RELATIVE.1`)
// ======================================================================================================

/// The on-disk width of a RELATIVE record's length header — `sizeof(f->record->size)` (`sizeof(size_t)`,
/// 8 bytes on a 64-bit platform), stored **native-endian** (little-endian on x86-64). A header `> 0`
/// marks an active record; `0` marks an empty or deleted slot (verified against the oracle).
pub const REL_SIZE_HEADER: usize = 8;

/// The on-disk width of one RELATIVE slot: the size header plus the fixed `record_max` data area.
pub fn relsize(record_max: usize) -> usize {
    record_max + REL_SIZE_HEADER
}

fn read_rel_header(file: &[u8], off: usize) -> u64 {
    let mut b = [0u8; 8];
    let end = (off + REL_SIZE_HEADER).min(file.len());
    if off < file.len() {
        b[..end - off].copy_from_slice(&file[off..end]);
    }
    u64::from_le_bytes(b)
}

fn write_rel_header(file: &mut [u8], off: usize, size: u64) {
    file[off..off + REL_SIZE_HEADER].copy_from_slice(&size.to_le_bytes());
}

/// The outcome of a keyed RELATIVE operation: the (possibly mutated) file bytes and the FILE STATUS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelWrite {
    pub file: Vec<u8>,
    /// `"00"` success · `"22"` key already exists · `"23"` key not present · `"24"` key boundary (< 1).
    pub status: &'static str,
}

/// Port of `fileio.c:relative_write` for keyed (RANDOM/DYNAMIC) access: write `record` (the `record_max`
/// data area, logical length `size`) at relative key `key`. The slot at `(key-1)*relsize` must be empty
/// (header `0`); an occupied slot is status `"22"`, a key `< 1` is `"24"`. Gaps before the slot are
/// zero-filled. The 8-byte native-endian size header precedes the `record_max` data bytes.
pub fn relative_write(file: &[u8], record: &[u8], size: usize, record_max: usize, key: i64) -> RelWrite {
    let kindex = key - 1;
    if kindex < 0 {
        return RelWrite { file: file.to_vec(), status: "24" };
    }
    let rs = relsize(record_max);
    let off = kindex as usize * rs;
    let mut out = file.to_vec();
    if off + REL_SIZE_HEADER <= out.len() && read_rel_header(&out, off) > 0 {
        return RelWrite { file: out, status: "22" };
    }
    if off + rs > out.len() {
        out.resize(off + rs, 0);
    }
    write_rel_header(&mut out, off, size as u64);
    let n = record_max.min(record.len());
    out[off + REL_SIZE_HEADER..off + REL_SIZE_HEADER + n].copy_from_slice(&record[..n]);
    RelWrite { file: out, status: "00" }
}

/// The outcome of a RELATIVE read: the `record_max`-wide record data, its logical size, and FILE STATUS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelRead {
    /// The `record_max` data bytes (empty when the read failed).
    pub data: Vec<u8>,
    pub size: usize,
    /// `"00"` success · `"23"` key not present / empty slot · `"10"` end of file (READ NEXT).
    pub status: &'static str,
}

/// Port of `fileio.c:relative_read` for keyed (RANDOM) access: read the record at relative `key`. A key
/// `< 1`, a slot beyond the file, or an empty slot (header `0`) is status `"23"`; otherwise the
/// `record_max` data bytes are returned with the slot's logical size and status `"00"`.
pub fn relative_read(file: &[u8], record_max: usize, key: i64) -> RelRead {
    let relnum = key - 1;
    if relnum < 0 {
        return RelRead { data: Vec::new(), size: 0, status: "23" };
    }
    let rs = relsize(record_max);
    let off = relnum as usize * rs;
    if off + REL_SIZE_HEADER > file.len() {
        return RelRead { data: Vec::new(), size: 0, status: "23" };
    }
    let size = read_rel_header(file, off) as usize;
    if size == 0 {
        return RelRead { data: Vec::new(), size: 0, status: "23" };
    }
    let dstart = off + REL_SIZE_HEADER;
    if dstart + record_max > file.len() {
        return RelRead { data: Vec::new(), size: 0, status: "30" };
    }
    RelRead { data: file[dstart..dstart + record_max].to_vec(), size, status: "00" }
}

/// Port of `fileio.c:relative_read_next` for sequential `READ NEXT`: starting at slot `*slot` (advancing
/// it), return the next **active** record (skipping empty/deleted slots), or status `"10"` at end of
/// file. (The `READ FIRST`/`LAST`/`PREVIOUS` direction options are a declared non-claim.)
pub fn relative_read_next(file: &[u8], slot: &mut usize, record_max: usize) -> RelRead {
    let rs = relsize(record_max);
    loop {
        let off = *slot * rs;
        if off + REL_SIZE_HEADER > file.len() {
            return RelRead { data: Vec::new(), size: 0, status: "10" };
        }
        let size = read_rel_header(file, off) as usize;
        *slot += 1;
        if size > 0 {
            let dstart = off + REL_SIZE_HEADER;
            let end = (dstart + record_max).min(file.len());
            return RelRead { data: file[dstart..end].to_vec(), size, status: "00" };
        }
    }
}

/// Port of `fileio.c:relative_rewrite` for keyed (RANDOM) access: overwrite the `record_max` data of an
/// existing record at relative `key`. A key `< 1` is status `"24"`; an empty slot is `"23"`.
pub fn relative_rewrite(file: &[u8], record: &[u8], record_max: usize, key: i64) -> RelWrite {
    let relnum = key - 1;
    if relnum < 0 {
        return RelWrite { file: file.to_vec(), status: "24" };
    }
    let rs = relsize(record_max);
    let off = relnum as usize * rs;
    let mut out = file.to_vec();
    if off + REL_SIZE_HEADER > out.len() || read_rel_header(&out, off) == 0 {
        return RelWrite { file: out, status: "23" };
    }
    let dstart = off + REL_SIZE_HEADER;
    let n = record_max.min(record.len());
    if dstart + n <= out.len() {
        out[dstart..dstart + n].copy_from_slice(&record[..n]);
    }
    RelWrite { file: out, status: "00" }
}

/// Port of `fileio.c:relative_delete`: tombstone the record at relative `key` by zeroing its size
/// header (the data bytes are left intact). A key `< 1` is status `"24"`; an empty slot is `"23"`.
pub fn relative_delete(file: &[u8], record_max: usize, key: i64) -> RelWrite {
    let relnum = key - 1;
    if relnum < 0 {
        return RelWrite { file: file.to_vec(), status: "24" };
    }
    let rs = relsize(record_max);
    let off = relnum as usize * rs;
    let mut out = file.to_vec();
    if off + REL_SIZE_HEADER > out.len() || read_rel_header(&out, off) == 0 {
        return RelWrite { file: out, status: "23" };
    }
    write_rel_header(&mut out, off, 0);
    RelWrite { file: out, status: "00" }
}

/// Port of `fileio.c:relative_start` positioning: find the slot index satisfying `cond` relative to
/// `key` over the file, returning the 0-based slot (for a following `READ NEXT`) or status `"23"` if no
/// active record qualifies. `cond` is one of [`RelCond`].
pub fn relative_start(file: &[u8], record_max: usize, cond: RelCond, key: i64) -> Result<usize, &'static str> {
    let rs = relsize(record_max);
    if file.is_empty() {
        return Err("23");
    }
    let nslots = file.len() / rs;
    let active = |idx: i64| -> bool {
        if idx < 0 || idx as usize >= nslots {
            return false;
        }
        read_rel_header(file, idx as usize * rs) > 0
    };
    let kindex = (key - 1) as i64;
    // scan in the direction implied by the condition, from the starting index.
    let (mut idx, step): (i64, i64) = match cond {
        RelCond::First | RelCond::Ge => (if matches!(cond, RelCond::First) { 0 } else { kindex.max(0) }, 1),
        RelCond::Gt => (kindex + 1, 1),
        RelCond::Last => (nslots as i64 - 1, -1),
        RelCond::Le => (kindex.min(nslots as i64 - 1), -1),
        RelCond::Lt => (kindex - 1, -1),
        RelCond::Eq => {
            return if active(kindex) { Ok(kindex as usize) } else { Err("23") };
        }
    };
    while idx >= 0 && (idx as usize) < nslots {
        if active(idx) {
            return Ok(idx as usize);
        }
        idx += step;
    }
    Err("23")
}

/// The `START` comparison condition for [`relative_start`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelCond {
    Eq,
    Lt,
    Le,
    Gt,
    Ge,
    First,
    Last,
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
    fn write_opt_advancing() {
        assert_eq!(cob_seq_write_opt(COB_WRITE_LINES | 2), b"\n\n"); // ADVANCING 2 LINES
        assert_eq!(cob_seq_write_opt(COB_WRITE_LINES), b"\r"); // ADVANCING 0
        assert_eq!(cob_seq_write_opt(COB_WRITE_PAGE), b"\x0c"); // ADVANCING PAGE
        assert!(cob_seq_write_opt(0).is_empty());
        assert_eq!(cob_file_write_opt(COB_WRITE_LINES | 3), b"\n\n\n");
    }

    #[test]
    fn copy_check_and_linage_check() {
        // wider target -> copy + space-pad
        assert_eq!(cob_copy_check(b"AB", 5), b"AB   ");
        // narrower target -> truncate
        assert_eq!(cob_copy_check(b"ABCDEF", 4), b"ABCD");
        // linage validation
        assert_eq!(file_linage_check(10, Some(8), Some(2), Some(2)), Ok((10, 8, 2, 2)));
        assert_eq!(file_linage_check(10, None, None, None), Ok((10, 0, 0, 0)));
        assert_eq!(file_linage_check(0, None, None, None), Err(())); // lines < 1
        assert_eq!(file_linage_check(10, Some(11), None, None), Err(())); // footing > lines
        assert_eq!(file_linage_check(10, None, Some(-1), None), Err(())); // top < 0
    }

    #[test]
    fn linage_and_suppress() {
        // a 5-line page, top 1, bot 1, currently at line 2; ADVANCING PAGE finishes the page
        let lin = Linage { lin_lines: 5, lin_top: 1, lin_bot: 1, counter: 2 };
        let (bytes, ctr, sts) = cob_linage_write_opt(lin, COB_WRITE_PAGE);
        // counter 2..5 -> 3 newlines, +1 bottom, +1 top = 5 newlines, counter resets to 1
        assert_eq!((bytes.len(), ctr, sts), (5, 1, "00"));
        // counter 0 -> linage error
        assert_eq!(cob_linage_write_opt(Linage { counter: 0, ..lin }, COB_WRITE_PAGE).2, "57");
        // ADVANCING 2 LINES within the page -> n-1 = 1 newline
        let (b2, c2, _) = cob_linage_write_opt(lin, COB_WRITE_LINES | 2);
        assert_eq!((b2.len(), c2), (1, 4));
        // suppress: all-suppress-char key is suppressed
        assert_eq!(is_suppressed_key_value(b"\x00\x00\x00", 0, true), 1);
        assert_eq!(is_suppressed_key_value(b"\x00A\x00", 0, true), 0);
        assert_eq!(is_suppressed_key_value(b"\x00\x00", 0, false), 0); // not a suppress key
    }

    #[test]
    fn lineseq_rewrite_in_place() {
        // file "OLDLINE\nNEXT\n", rewrite the first line (slotlen 7) with "HI"
        let file = b"OLDLINE\nNEXT\n";
        let plain = LineSeqConfig { ls_fixed: false, ls_nulls: false, ls_validate: false, ls_split: true };
        let r = lineseq_rewrite(file, 0, 7, b"HI", 2, &plain);
        assert_eq!(r.status, "00");
        assert_eq!(r.file, b"HI     \nNEXT\n"); // "HI" + 5 spaces, newline + rest intact
        // a record longer than the slot -> overflow 44
        assert_eq!(lineseq_rewrite(file, 0, 7, b"WAYTOOLONG", 10, &plain).status, "44");
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

    // ---- RELATIVE ----
    #[test]
    fn relative_write_read_delete_matches_oracle() {
        // reproduce the oracle: write key1=AAAA, key3=CCCC, delete key1, write key5=EEEE
        let w1 = relative_write(&[], b"AAAA", 4, 4, 1);
        assert_eq!(w1.status, "00");
        let w3 = relative_write(&w1.file, b"CCCC", 4, 4, 3);
        let d1 = relative_delete(&w3.file, 4, 1);
        assert_eq!(d1.status, "00");
        let w5 = relative_write(&d1.file, b"EEEE", 4, 4, 5);
        // final file bytes == the oracle (8-byte LE header + 4 data per slot; deleted slot keeps data)
        let oracle: Vec<u8> = [
            &[0u8, 0, 0, 0, 0, 0, 0, 0][..], b"AAAA", // slot0: deleted (size 0), data AAAA
            &[0, 0, 0, 0, 0, 0, 0, 0], b"\x00\x00\x00\x00", // slot1: empty
            &[4, 0, 0, 0, 0, 0, 0, 0], b"CCCC", // slot2: key3
            &[0, 0, 0, 0, 0, 0, 0, 0], b"\x00\x00\x00\x00", // slot3: empty
            &[4, 0, 0, 0, 0, 0, 0, 0], b"EEEE", // slot4: key5
        ]
        .concat();
        assert_eq!(w5.file, oracle);
        // read key3 -> CCCC/00; key2 (empty) -> 23; key1 (deleted) -> 23
        assert_eq!(relative_read(&w5.file, 4, 3), RelRead { data: b"CCCC".to_vec(), size: 4, status: "00" });
        assert_eq!(relative_read(&w5.file, 4, 2).status, "23");
        assert_eq!(relative_read(&w5.file, 4, 1).status, "23");
    }

    #[test]
    fn relative_write_existing_is_22_and_low_key_24() {
        let w = relative_write(&[], b"AAAA", 4, 4, 1);
        assert_eq!(relative_write(&w.file, b"ZZZZ", 4, 4, 1).status, "22"); // key exists
        assert_eq!(relative_write(&[], b"AAAA", 4, 4, 0).status, "24"); // key < 1
    }

    #[test]
    fn relative_read_next_skips_deleted() {
        let w1 = relative_write(&[], b"AAAA", 4, 4, 1);
        let w3 = relative_write(&w1.file, b"CCCC", 4, 4, 3);
        let d1 = relative_delete(&w3.file, 4, 1);
        let mut slot = 0usize;
        let r1 = relative_read_next(&d1.file, &mut slot, 4); // skips deleted slot0 + empty slot1
        assert_eq!((r1.data, r1.status), (b"CCCC".to_vec(), "00"));
        assert_eq!(relative_read_next(&d1.file, &mut slot, 4).status, "10"); // end of file
    }

    // ---- CBL_* system routines ----
    #[test]
    fn cbl_file_dir_routines() {
        let base = std::env::temp_dir().join("gnucobol_rs_sys_test_a");
        let _ = std::fs::remove_dir_all(&base);
        // create_dir -> 0, again -> 128 (exists)
        assert_eq!(cob_sys_create_dir(base.to_str().unwrap().as_bytes()), 0);
        assert_eq!(cob_sys_create_dir(base.to_str().unwrap().as_bytes()), 128);
        // write a file, copy it, rename it, delete it
        let f1 = base.join("a.txt");
        std::fs::write(&f1, b"hello").unwrap();
        let f2 = base.join("b.txt");
        assert_eq!(cob_sys_copy_file(f1.to_str().unwrap().as_bytes(), f2.to_str().unwrap().as_bytes()), 0);
        assert_eq!(std::fs::read(&f2).unwrap(), b"hello");
        // copy a missing source -> 35
        assert_eq!(cob_sys_copy_file(base.join("none").to_str().unwrap().as_bytes(), f2.to_str().unwrap().as_bytes()), 35);
        let f3 = base.join("c.txt");
        assert_eq!(cob_sys_rename_file(f2.to_str().unwrap().as_bytes(), f3.to_str().unwrap().as_bytes()), 0);
        assert!(f3.exists() && !f2.exists());
        assert_eq!(cob_sys_delete_file(f3.to_str().unwrap().as_bytes()), 0);
        assert_eq!(cob_sys_delete_file(f3.to_str().unwrap().as_bytes()), 128); // already gone
        std::fs::remove_file(&f1).ok();
        // delete_dir -> 0, again -> 128
        assert_eq!(cob_sys_delete_dir(base.to_str().unwrap().as_bytes()), 0);
        assert_eq!(cob_sys_delete_dir(base.to_str().unwrap().as_bytes()), 128);
        // change_dir to a missing path -> 128 (does not change cwd)
        assert_eq!(cob_sys_change_dir(b"/gnucobol_rs_nonexistent_xyz"), 128);
        // get_current_dir: flags != 0 -> 129; length 0 -> 128; a sane length succeeds
        assert_eq!(cob_sys_get_current_dir(1, 100).0, 129);
        assert_eq!(cob_sys_get_current_dir(0, 0).0, 128);
        assert_eq!(cob_sys_get_current_dir(0, 4096).0, 0);
    }

    // ---- SORT comparison ----
    #[test]
    fn sort_records_ascending_descending_stable() {
        // K1 = X(3) at 0 ASCENDING; K2 = X(2) at 3 DESCENDING
        let mut keys = Vec::new();
        cob_file_sort_init_key(&mut keys, 0, 3, true);
        cob_file_sort_init_key(&mut keys, 3, 2, false);
        let recs: Vec<&[u8]> = vec![b"BBB10xyz", b"AAA20xyz", b"BBB05xyz", b"AAA20abc", b"CCC00xyz"];
        // AAA before BBB before CCC; within AAA the key2 ties (20==20) -> stable insertion order (1,3);
        // within BBB key2 desc -> 10 before 05 (0,2)
        assert_eq!(sort_records(&recs, &keys, None), vec![1, 3, 0, 2, 4]);
    }

    #[test]
    fn sort_cmps_and_collation() {
        assert!(sort_cmps(b"ABC", b"ABD", None) < 0);
        assert_eq!(sort_cmps(b"ABC", b"ABC", None), 0);
        assert!(sort_cmps(b"ABD", b"ABC", None) > 0);
        // a collating table that inverts A and Z ordering swaps the verdict
        let mut col = [0u8; 256];
        for (i, c) in col.iter_mut().enumerate() {
            *c = i as u8;
        }
        col[b'A' as usize] = 255;
        assert!(sort_cmps(b"A", b"B", Some(&col)) > 0); // A now sorts after B
    }

    // ---- verb preconditions ----
    #[test]
    fn verb_preconditions() {
        use AccessMode::*;
        use OpenMode::*;
        // WRITE: sequential access needs OUTPUT/EXTEND; INPUT -> 48; OUTPUT -> proceed
        assert_eq!(cob_write(Input, Sequential, 4, 4, 4), Some("48"));
        assert_eq!(cob_write(Output, Sequential, 4, 4, 4), None);
        assert_eq!(cob_write(Io, Random, 4, 4, 4), None); // random WRITE allows I-O
        assert_eq!(cob_write(Output, Sequential, 9, 4, 4), Some("44")); // too long
        // READ on OUTPUT -> 47; on INPUT -> proceed
        assert_eq!(cob_read(Output, false, false, false, false, false, false), Some("47"));
        assert_eq!(cob_read(Input, false, false, false, false, false, false), None);
        assert_eq!(cob_read(Input, false, false, false, true, false, false), Some("46")); // seq read at EOF
        // REWRITE/DELETE need I-O; INPUT -> 49; I-O seq without read -> 43
        assert_eq!(cob_rewrite(Input, Random, Organization::Relative, true, 4, 4), Some("49"));
        assert_eq!(cob_rewrite(Io, Sequential, Organization::Relative, false, 4, 4), Some("43"));
        assert_eq!(cob_rewrite(Io, Sequential, Organization::Sequential, true, 5, 4), Some("44")); // size change
        assert_eq!(cob_delete(Input, Random, false), Some("49"));
        assert_eq!(cob_delete(Io, Sequential, false), Some("43"));
        assert_eq!(cob_delete(Io, Random, false), None); // random delete needs no prior read
        // START: not INPUT/I-O -> 47; RANDOM -> 47; nonexistent -> 23
        assert_eq!(cob_start(Output, Sequential, false, true), Some("47"));
        assert_eq!(cob_start(Input, Random, false, true), Some("47"));
        assert_eq!(cob_start(Input, Sequential, true, true), Some("23"));
        assert_eq!(cob_start(Io, Dynamic, false, true), None);
        // read_next nonexistent non-first -> 46 (vs cob_read's 23)
        assert_eq!(cob_read_next(Input, true, false, false, false, false), Some("46"));
        assert_eq!(cob_read_next(Input, true, true, false, false, false), Some("10"));
    }

    // ---- indexed key descriptor ----
    #[test]
    fn indexed_key_ops() {
        // a 2-part key: bytes [2..5) and [8..10) of the record
        let kd = KeyDesc { parts: vec![KeyPart { start: 2, leng: 3 }, KeyPart { start: 8, leng: 2 }], duplicates: false };
        assert_eq!(indexed_keylen(&kd), 5);
        let rec = b"XXABCXXX12X";
        let key = indexed_savekey(rec, &kd);
        assert_eq!(key, b"ABC12"); // parts concatenated in order
        // restore into a fresh record
        let mut rec2 = vec![b'.'; 11];
        indexed_restorekey(&mut rec2, &key, &kd);
        assert_eq!(&rec2[2..5], b"ABC");
        assert_eq!(&rec2[8..10], b"12");
        // compare: equal, then greater, then partial
        assert_eq!(indexed_cmpkey(rec, &key, &kd, 0), 0);
        assert_eq!(indexed_cmpkey(b"XXABDXXX12X", &key, &kd, 0), 1); // 'D' > 'C'
        assert_eq!(indexed_cmpkey(b"XXAAAXXX99X", &key, &kd, 0), -1); // 'A' < 'B'
        // partial compare of only the first 3 bytes ignores the second part
        assert_eq!(indexed_cmpkey(b"XXABCXXX99X", &key, &kd, 3), 0);
    }

    #[test]
    fn indexed_keydesc_and_cob_savekey() {
        // single-field key: offset 2, size 4
        let k1 = CobFileKey { duplicates: false, offset: 2, field_size: 4, components: vec![] };
        assert_eq!(indexed_keydesc(&k1).parts, vec![KeyPart { start: 2, leng: 4 }]);
        assert_eq!(cob_savekey(b"XXABCDXX", &k1), b"ABCD");
        // composite key: two components
        let k2 = CobFileKey { duplicates: true, offset: 0, field_size: 0, components: vec![(0, 2), (5, 3)] };
        let kd = indexed_keydesc(&k2);
        assert!(kd.duplicates && kd.parts.len() == 2);
        assert_eq!(cob_savekey(b"AB...XYZ..", &k2), b"ABXYZ");
    }

    #[test]
    fn save_status_and_str_from_fld() {
        assert_eq!(&save_status(0), b"00");
        assert_eq!(&save_status(23), b"23");
        assert_eq!(&save_status(10), b"10");
        assert_eq!(&save_status(7), b"07");
        assert_eq!(cob_str_from_fld(b"HELLO   "), b"HELLO");
        assert_eq!(cob_str_from_fld(b"AB\x00\x00"), b"AB"); // trailing NULs dropped
        assert_eq!(cob_str_from_fld(b"        "), b""); // all spaces -> empty
        assert_eq!(cob_str_from_fld(b"a\x22b\x22c "), b"abc"); // embedded quotes removed
    }

    #[test]
    fn indexed_keycmp_equality() {
        let a = KeyDesc { parts: vec![KeyPart { start: 0, leng: 4 }], duplicates: false };
        let b = KeyDesc { parts: vec![KeyPart { start: 0, leng: 4 }], duplicates: false };
        let c = KeyDesc { parts: vec![KeyPart { start: 0, leng: 5 }], duplicates: false };
        let d = KeyDesc { parts: vec![KeyPart { start: 0, leng: 4 }], duplicates: true };
        assert_eq!(indexed_keycmp(&a, &b), 0);
        assert_eq!(indexed_keycmp(&a, &c), 1); // different length
        assert_eq!(indexed_keycmp(&a, &d), 1); // different dup flag
    }

    // ---- path / status helpers ----
    #[test]
    fn path_helpers() {
        assert!(has_directory_separator(b"a/b"));
        assert!(has_directory_separator(b"a\\b"));
        assert!(!has_directory_separator(b"abc"));
        assert!(looks_absolute(b"/etc/x"));
        assert!(looks_absolute(b"\"/quoted/abs"));
        assert!(!looks_absolute(b"rel/path"));
        assert!(is_absolute(b"/abs"));
        assert!(!is_absolute(b"rel"));
        assert!(has_acu_hyphen(b"-F file"));
        assert!(has_acu_hyphen(b"-d\tdev"));
        assert!(!has_acu_hyphen(b"-Xfile"));
        assert_eq!(do_acu_hyphen_translation(b"-F   myfile"), b"myfile");
        assert_eq!(do_acu_hyphen_translation(b"-D \"a b\""), b"a b"); // quotes dropped
    }

    #[test]
    fn errno_and_dummy_status() {
        assert_eq!(errno_cob_sts(FileErrno::NoSpaceOrQuota, "30"), "34");
        assert_eq!(errno_cob_sts(FileErrno::PermissionOrIsDir, "30"), "37");
        assert_eq!(errno_cob_sts(FileErrno::NotExist, "30"), "35");
        assert_eq!(errno_cob_sts(FileErrno::Other, "30"), "30");
        assert_eq!((dummy_delete(), dummy_read(), dummy_start()), ("91", "91", "91"));
    }

    #[test]
    fn relative_rewrite_and_start() {
        let w1 = relative_write(&[], b"AAAA", 4, 4, 1);
        let w3 = relative_write(&w1.file, b"CCCC", 4, 4, 3);
        // rewrite key3 -> ZZZZ
        let rw = relative_rewrite(&w3.file, b"ZZZZ", 4, 3);
        assert_eq!(rw.status, "00");
        assert_eq!(relative_read(&rw.file, 4, 3).data, b"ZZZZ");
        // rewrite an empty slot -> 23
        assert_eq!(relative_rewrite(&w3.file, b"ZZZZ", 4, 2).status, "23");
        // START >= 2 finds the next active record (slot2 = key3)
        assert_eq!(relative_start(&w3.file, 4, RelCond::Ge, 2), Ok(2));
        assert_eq!(relative_start(&w3.file, 4, RelCond::Eq, 2), Err("23")); // empty slot
        assert_eq!(relative_start(&w3.file, 4, RelCond::First, 0), Ok(0)); // key1 active
        assert_eq!(relative_start(&w3.file, 4, RelCond::Last, 0), Ok(2)); // key3 active
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
    // KANIFOR: GNURUST.FILEIO.RELATIVE.1
    /// A keyed RELATIVE write-then-read round-trip at the same key returns the written record; a deleted
    /// slot reads back as not-present (status 23). Never panics.
    #[kani::proof]
    #[kani::unwind(4)]
    fn relative_write_read_roundtrip() {
        let rec: [u8; 2] = kani::any();
        let key: i64 = kani::any();
        kani::assume(key >= 1 && key <= 3);
        let w = relative_write(&[], &rec, 2, 2, key);
        assert_eq!(w.status, "00");
        let r = relative_read(&w.file, 2, key);
        assert_eq!((r.status, &r.data[..]), ("00", &rec[..]));
        let d = relative_delete(&w.file, 2, key);
        assert_eq!(relative_read(&d.file, 2, key).status, "23");
    }
    // KANIFOR: GNURUST.FILEIO.VERB.1
    /// A verb precondition either denies with a known FILE STATUS or permits (None); a WRITE outside the
    /// allowed open modes is always denied. Never panics.
    #[kani::proof]
    fn verb_precondition_total() {
        let opens = [OpenMode::Closed, OpenMode::Input, OpenMode::Output, OpenMode::Io, OpenMode::Extend];
        let oi: usize = kani::any();
        kani::assume(oi < 5);
        let open = opens[oi];
        let r = cob_write(open, AccessMode::Random, 4, 4, 4);
        match r {
            Some(s) => assert!(s == "48" || s == "44"),
            None => assert!(matches!(open, OpenMode::Output | OpenMode::Io)),
        }
    }
    // KANIFOR: GNURUST.FILEIO.SORT.1
    /// The sort comparison is consistent (a record compares Equal to itself) and antisymmetric on the
    /// key bytes; never panics.
    #[kani::proof]
    #[kani::unwind(5)]
    fn sort_compare_consistent() {
        let a: [u8; 3] = kani::any();
        let b: [u8; 3] = kani::any();
        let keys = [SortKey { offset: 0, size: 3, ascending: true }];
        assert_eq!(cob_file_sort_compare(&a, 0, &a, 0, &keys, None), Ordering::Equal);
        let ab = cob_file_sort_compare(&a, 0, &b, 1, &keys, None);
        let ba = cob_file_sort_compare(&b, 1, &a, 0, &keys, None);
        assert_eq!(ab, ba.reverse());
    }
    // KANIFOR: GNURUST.FILEIO.SYS.1
    /// The non-I/O precondition paths of `cob_sys_get_current_dir` are total: `flags != 0` → 129, a
    /// zero-length field → 128, regardless of value. (The `getcwd` path itself is the OS boundary.)
    #[kani::proof]
    fn cob_sys_get_current_dir_preconditions() {
        let flags: i32 = kani::any();
        kani::assume(flags != 0);
        assert_eq!(cob_sys_get_current_dir(flags, 100).0, 129);
        assert_eq!(cob_sys_get_current_dir(0, 0).0, 128);
    }
}
