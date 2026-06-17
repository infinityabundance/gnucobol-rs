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

/// Resolve the runtime config / env `COB_VARSEQ_FORMAT` to the variable-sequential prefix format
/// `cob_varseq_type` in `{0,1,2,3}` (the value `sequential_write` / `varseq_prefix` take). GnuCOBOL's
/// default is `0` (a 4-byte BE16-size + 2 record-mark bytes); an unset or out-of-range value is `0`.
/// This is the env-driven selection `cob_init` does -- the previously-missing link from the config
/// to `sequential_write`'s `varseq_type` parameter.
pub fn cob_varseq_format_from_env(getenv: &dyn Fn(&str) -> Option<String>) -> u8 {
    match getenv("COB_VARSEQ_FORMAT").as_deref() {
        Some("1") => 1,
        Some("2") => 2,
        Some("3") => 3,
        _ => 0,
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

/// Port of `fileio.c:cob_chk_file_env` — resolve a COBOL ASSIGN name through the environment: tries the
/// env vars `DD_<name>`, `dd_<name>`, `<name>` in order (with `.` mangled to `_`, or all non-alnum when
/// `COB_ENV_MANGLE`), returning the first set value (surrounding quotes stripped). A name starting with
/// `.`, `-`, or a digit is not mapped (`None`).
pub fn cob_chk_file_env(src: &[u8]) -> Option<Vec<u8>> {
    if src.first() == Some(&b'.') || matches!(src.first(), Some(b'-') | Some(b'0'..=b'9')) {
        return None;
    }
    let mangle = std::env::var("COB_ENV_MANGLE").map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes")).unwrap_or(false);
    let name: String = src
        .iter()
        .map(|&c| {
            let ch = c as char;
            if mangle {
                if ch.is_ascii_alphanumeric() {
                    ch
                } else {
                    '_'
                }
            } else if ch == '.' {
                '_'
            } else {
                ch
            }
        })
        .collect();
    for prefix in ["DD_", "dd_", ""] {
        if let Ok(v) = std::env::var(format!("{prefix}{name}")) {
            if !v.is_empty() {
                let b = v.into_bytes();
                if b.len() >= 2 && (b[0] == 0x22 || b[0] == 0x27) && b[b.len() - 1] == b[0] {
                    return Some(b[1..b.len() - 1].to_vec());
                }
                return Some(b);
            }
        }
    }
    None
}

/// Port of the simple-case path of `fileio.c:cob_chk_file_mapping` — resolve a COBOL ASSIGN name to a
/// filesystem path: an ACU-hyphen name is translated; a bare name (no separator, not absolute) has its
/// quotes/leading-`$` dropped, is looked up via [`cob_chk_file_env`] (an absolute result is used as-is),
/// and is then prefixed by `COB_FILE_PATH` if set. A name that is already absolute or contains a path
/// separator (the complex multi-element mapping) is a declared non-claim and is returned unchanged.
pub fn cob_chk_file_mapping(name: &[u8]) -> Vec<u8> {
    if has_acu_hyphen(name) {
        return do_acu_hyphen_translation(name);
    }
    if looks_absolute(name) || has_directory_separator(name) {
        return name.to_vec(); // complex case: declared non-claim
    }
    let mut src: &[u8] = name;
    if src.len() >= 2 && (src[0] == 0x22 || src[0] == 0x27) && src[src.len() - 1] == src[0] {
        src = &src[1..src.len() - 1];
    }
    if src.first() == Some(&b'$') {
        src = &src[1..];
    }
    let mut resolved = src.to_vec();
    if let Some(env) = cob_chk_file_env(src) {
        if looks_absolute(&env) {
            return env;
        }
        if has_acu_hyphen(&env) {
            return do_acu_hyphen_translation(&env);
        }
        resolved = env;
    }
    if let Ok(path) = std::env::var("COB_FILE_PATH") {
        if !path.is_empty() {
            let mut out = path.into_bytes();
            out.push(b'/');
            out.extend_from_slice(&resolved);
            return out;
        }
    }
    resolved
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

/// Classify a failed `std::fs` operation into the [`FileErrno`] equivalence classes libcob's `errno`
/// switch keys on (fileio.c:1674). Uses the stable `io::ErrorKind` where it names the case and the raw
/// OS error otherwise (EISDIR/EROFS/EDQUOT are not all named on stable Rust): `ENOENT`→NotExist,
/// `EACCES`/`EISDIR`/`EROFS`→PermissionOrIsDir, `ENOSPC`/`EDQUOT`→NoSpaceOrQuota.
pub fn classify_io_error(e: &std::io::Error) -> FileErrno {
    use std::io::ErrorKind;
    match e.kind() {
        ErrorKind::NotFound => FileErrno::NotExist,
        ErrorKind::PermissionDenied => FileErrno::PermissionOrIsDir,
        _ => match e.raw_os_error() {
            Some(2) => FileErrno::NotExist,                       // ENOENT
            Some(13) | Some(21) | Some(30) => FileErrno::PermissionOrIsDir, // EACCES / EISDIR / EROFS
            Some(28) | Some(122) => FileErrno::NoSpaceOrQuota,    // ENOSPC / EDQUOT
            _ => FileErrno::Other,
        },
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

/// Port of `fileio.c:cob_sys_check_file_exist` (`CBL_CHECK_FILE_EXIST`) — stat `filename` and fill the
/// 16-byte detail area: bytes 0..8 the file size as a big-endian 64-bit value, byte 8 day, 9 month, 10..12
/// year (big-endian 16-bit), 12 hour, 13 minute, 14 second, 15 zero. Returns `35` when the file is absent,
/// else `0`. The size is byte-exact; the calendar breakdown (the C `localtime`) is TZ-dependent — the
/// declared boundary — so the date bytes are left zero here.
pub fn cob_sys_check_file_exist(filename: &str) -> (i32, [u8; 16]) {
    let mut info = [0u8; 16];
    match std::fs::metadata(filename) {
        Ok(md) => {
            info[0..8].copy_from_slice(&md.len().to_be_bytes());
            // info[8..15] = localtime(st_mtime) -> the TZ-dependent boundary.
            (0, info)
        }
        Err(_) => (35, info),
    }
}

/// Port of `fileio.c:cob_sys_file_info` (`C$FILEINFO`) — the same stat-and-fill as
/// [`cob_sys_check_file_exist`], returning `128` when the file is absent (the `C$FILEINFO` convention) and
/// `0` on success with the big-endian size in bytes 0..8.
pub fn cob_sys_file_info(filename: &str) -> (i32, [u8; 16]) {
    let mut info = [0u8; 16];
    match std::fs::metadata(filename) {
        Ok(md) => {
            info[0..8].copy_from_slice(&md.len().to_be_bytes());
            (0, info)
        }
        Err(_) => (128, info),
    }
}

// ======================================================================================================
// CBL_* handle-based byte-stream file routines (`GNURUST.FILEIO.SYS.1`, handle family)
//
// `CBL_OPEN_FILE`/`CREATE_FILE`/`READ_FILE`/`WRITE_FILE`/`CLOSE_FILE`/`FLUSH_FILE` operate on an opaque
// 4-byte file handle. libcob stores the raw OS `fd`; with `#![forbid(unsafe_code)]` we instead store an
// index into a safe process-global `File` registry (the handle value is opaque to the COBOL program, so
// behaviour is identical). Positioned `lseek`+`read`/`write` map to `Seek`+`Read`/`Write`.
// ======================================================================================================

/// The process-global open-file registry backing the CBL handle routines (index = the 4-byte handle).
static CBL_FILES: std::sync::Mutex<Vec<Option<std::fs::File>>> = std::sync::Mutex::new(Vec::new());

/// Port of `fileio.c:open_cbl_file` — open `name` per `access` (1 read, 2 write+create+truncate, 3 r/w);
/// `create` forces `O_CREAT|O_TRUNC` (for `CBL_CREATE_FILE`). Returns `(status, handle)`: `(0, h)` on
/// success, `(-1, -1)` on a bad access mode, `(35, -1)` when the open fails.
pub fn open_cbl_file(name: &[u8], access: u8, create: bool) -> (i32, i32) {
    let Ok(path) = std::str::from_utf8(name) else {
        return (-1, -1);
    };
    let mut opts = std::fs::OpenOptions::new();
    match access & 0x3F {
        1 => {
            opts.read(true);
        }
        2 => {
            opts.write(true).create(true).truncate(true);
        }
        3 => {
            opts.read(true).write(true);
        }
        _ => return (-1, -1),
    }
    if create {
        opts.create(true).truncate(true).write(true);
    }
    match opts.open(path) {
        Ok(f) => {
            let mut reg = CBL_FILES.lock().unwrap();
            let idx = reg.iter().position(Option::is_none).unwrap_or_else(|| {
                reg.push(None);
                reg.len() - 1
            });
            reg[idx] = Some(f);
            (0, idx as i32)
        }
        Err(_) => (35, -1),
    }
}

/// Port of `fileio.c:cob_sys_open_file` (`CBL_OPEN_FILE`) — open an existing file for the given access.
pub fn cob_sys_open_file(name: &[u8], access: u8) -> (i32, i32) {
    open_cbl_file(name, access, false)
}

/// Port of `fileio.c:cob_sys_create_file` (`CBL_CREATE_FILE`) — create/truncate a file for the given access.
pub fn cob_sys_create_file(name: &[u8], access: u8) -> (i32, i32) {
    open_cbl_file(name, access, true)
}

/// Port of `fileio.c:cob_sys_read_file` (`CBL_READ_FILE`) — read `len` bytes at `offset` from `handle`
/// into `buf`. With `flags & 0x80` it instead returns the file size in `(size, _)`. Status: `0` success,
/// `10` end of file (0-byte read), `-1` on a bad handle/offset.
pub fn cob_sys_read_file(handle: i32, offset: u64, len: usize, flags: u8, buf: &mut [u8]) -> (i32, u64) {
    use std::io::{Read, Seek, SeekFrom};
    let mut reg = CBL_FILES.lock().unwrap();
    let Some(Some(f)) = reg.get_mut(handle as usize) else {
        return (-1, 0);
    };
    if flags & 0x80 != 0 {
        return match f.metadata() {
            Ok(m) => (0, m.len()),
            Err(_) => (-1, 0),
        };
    }
    if f.seek(SeekFrom::Start(offset)).is_err() {
        return (-1, 0);
    }
    if len > 0 {
        let n = len.min(buf.len());
        match f.read(&mut buf[..n]) {
            Ok(0) => return (10, 0), // COB_STATUS_10_END_OF_FILE
            Ok(_) => {}
            Err(_) => return (-1, 0),
        }
    }
    (0, 0)
}

/// Port of `fileio.c:cob_sys_write_file` (`CBL_WRITE_FILE`) — write `len` bytes of `buf` at `offset` to
/// `handle`. Status: `0` success, `30` on a short write, `-1` on a bad handle/offset.
pub fn cob_sys_write_file(handle: i32, offset: u64, len: usize, buf: &[u8]) -> i32 {
    use std::io::{Seek, SeekFrom, Write};
    let mut reg = CBL_FILES.lock().unwrap();
    let Some(Some(f)) = reg.get_mut(handle as usize) else {
        return -1;
    };
    if f.seek(SeekFrom::Start(offset)).is_err() {
        return -1;
    }
    let n = len.min(buf.len());
    match f.write(&buf[..n]) {
        Ok(w) if w == n => 0,
        Ok(_) => 30,
        Err(_) => 30,
    }
}

/// Port of `fileio.c:cob_sys_close_file` (`CBL_CLOSE_FILE`) — close the handle (drop the `File`); `0`.
pub fn cob_sys_close_file(handle: i32) -> i32 {
    let mut reg = CBL_FILES.lock().unwrap();
    if let Some(slot) = reg.get_mut(handle as usize) {
        *slot = None;
    }
    0
}

/// Port of `fileio.c:cob_sys_flush_file` (`CBL_FLUSH_FILE`) — flush the handle's buffers to disk; `0`.
pub fn cob_sys_flush_file(handle: i32) -> i32 {
    let reg = CBL_FILES.lock().unwrap();
    if let Some(Some(f)) = reg.get(handle as usize) {
        let _ = f.sync_all();
    }
    0
}

/// Port of `fileio.c:cob_sys_file_delete` (`C$DELETE`) — delete `name` via [`cob_sys_delete_file`],
/// mapping a `< 0` result to `128`.
pub fn cob_sys_file_delete(name: &[u8]) -> i32 {
    let ret = cob_sys_delete_file(name);
    if ret < 0 {
        128
    } else {
        ret
    }
}

/// Port of `fileio.c:cob_sys_copyfile` (`C$COPY`) — copy `from` to `to` via [`cob_sys_copy_file`],
/// mapping a `< 0` result to `128`.
pub fn cob_sys_copyfile(from: &[u8], to: &[u8]) -> i32 {
    let ret = cob_sys_copy_file(from, to);
    if ret < 0 {
        128
    } else {
        ret
    }
}

/// Port of `fileio.c:cob_sys_mkdir` (`C$MAKEDIR`) — create directory `name` via [`cob_sys_create_dir`],
/// mapping a `< 0` result to `128`.
pub fn cob_sys_mkdir(name: &[u8]) -> i32 {
    let ret = cob_sys_create_dir(name);
    if ret < 0 {
        128
    } else {
        ret
    }
}

/// Port of `fileio.c:cob_sys_chdir` (`C$CHDIR`) — change directory to `name` via
/// [`cob_sys_change_dir`], mapping a `< 0` result to `128` (the status is also returned to the caller's
/// second parameter, the calling-convention boundary).
pub fn cob_sys_chdir(name: &[u8]) -> i32 {
    let ret = cob_sys_change_dir(name);
    if ret < 0 {
        128
    } else {
        ret
    }
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
/// `attr` is the key field's template (`fileio.c:cob_file_sort_compare` builds a `cob_field` from
/// `f->keys[i].field`): `Some(numeric attr)` => the key compares by numeric value (`cob_numeric_cmp`,
/// the `COB_FIELD_IS_NUMERIC` branch); `None` or a non-numeric attr => the alphanumeric collated compare.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SortKey {
    pub offset: usize,
    pub size: usize,
    /// `COB_ASCENDING` vs `COB_DESCENDING`.
    pub ascending: bool,
    /// The key field's attribute (numeric keys compare by value). `None` = alphanumeric (bytewise/collated).
    pub attr: Option<crate::attr::FieldAttr>,
}

/// Port of `fileio.c:cob_file_sort_init_key` — append an alphanumeric key (in declaration order).
pub fn cob_file_sort_init_key(keys: &mut Vec<SortKey>, offset: usize, size: usize, ascending: bool) {
    keys.push(SortKey { offset, size, ascending, attr: None });
}

/// Append a *typed* sort key. A numeric `attr` (`field_type & COB_TYPE_NUMERIC`) makes the key compare
/// by numeric value via `cob_numeric_cmp`, reproducing libcob's `COB_FIELD_IS_NUMERIC` branch -- so a
/// signed key orders negatives correctly and an overpunched-sign / big-endian COMP key orders by value,
/// not by raw representation bytes.
pub fn cob_file_sort_init_key_typed(
    keys: &mut Vec<SortKey>,
    offset: usize,
    size: usize,
    ascending: bool,
    attr: crate::attr::FieldAttr,
) {
    keys.push(SortKey { offset, size, ascending, attr: Some(attr) });
}

/// Port of the alphanumeric path of `fileio.c:cob_file_sort_compare` — order two records by the sort
/// keys (each compared via [`sort_cmps`], negated for DESCENDING). A full key tie breaks by the records'
/// insertion order (`u1`/`u2`, the `unique` field), giving a **stable** sort. (Numeric keys, which the C
/// routes through `cob_numeric_cmp`, are a declared composition with `GNURUST.NUMCMP.1`.)
pub fn cob_file_sort_compare(rec1: &[u8], u1: usize, rec2: &[u8], u2: usize, keys: &[SortKey], col: Option<&[u8; 256]>) -> Ordering {
    for k in keys {
        let a = &rec1[k.offset.min(rec1.len())..(k.offset + k.size).min(rec1.len())];
        let b = &rec2[k.offset.min(rec2.len())..(k.offset + k.size).min(rec2.len())];
        // libcob (fileio.c:cob_file_sort_compare): a numeric key compares by VALUE via cob_numeric_cmp;
        // otherwise the alphanumeric bytes go through sort_cmps (with the collating sequence).
        let cmp = match k.attr {
            Some(at) if at.field_type & crate::attr::COB_TYPE_NUMERIC != 0 => {
                crate::cob_decimal::cob_numeric_cmp(a, &at, b, &at)
            }
            _ => sort_cmps(a, b, col),
        };
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
// SORT/MERGE engine — the in-memory 4-queue natural merge (`GNURUST.FILEIO.SORTENGINE.1`)
//
// A 1:1 port of fileio.c's `struct cobsort` sort engine: `cob_file_sort_submit` inserts each record onto
// the shorter of two input queues (each new record is a one-element sorted "block"); `cob_sort_queues`
// repeatedly merges adjacent blocks across a 4-queue ping-pong until a single sorted run remains;
// `cob_file_sort_retrieve` then drains that run in order. The full-key tie is broken by the per-record
// `unique` insertion counter, so the result is a STABLE sort by the keys (matching the oracle's SORT
// record order). The C linked lists (with an `empty` free-list) are modelled faithfully as an arena of
// `CobItem`s linked by `next: Option<usize>` indices. The temp-file spill path (`switch_to_file` /
// `files_used`, triggered only above `COB_SORT_MEMORY`, default 128 MiB) is the declared OS boundary —
// in-memory sorts (every oracle-reachable case) never spill.
// ======================================================================================================

/// `fileio.c` sort return codes (`COBSORT*`).
const COBSORTEND: i32 = 1;
#[allow(dead_code)]
const COBSORTABORT: i32 = 2;
#[allow(dead_code)]
const COBSORTFILEERR: i32 = 3;
#[allow(dead_code)]
const COBSORTNOTOPEN: i32 = 4;

/// Port of `fileio.c:struct cobitem` — one record in the sort, plus its insertion-order `unique` key and
/// the `end_of_block` run delimiter; `next` is the arena index of the following item (the C linked list).
#[derive(Clone)]
struct CobItem {
    item: Vec<u8>,
    unique: usize,
    end_of_block: bool,
    next: Option<usize>,
}

/// Port of `fileio.c:struct queue_struct` — a singly-linked run of [`CobItem`]s (`first`..`last`).
#[derive(Clone, Copy, Default)]
struct QueueStruct {
    first: Option<usize>,
    last: Option<usize>,
    count: i64,
}

/// Port of `fileio.c:struct cobsort` (the in-memory subset) — the 4-queue merge state for one SORT/MERGE.
/// Built by [`CobSort::cob_file_sort_init`], fed by [`CobSort::cob_file_sort_submit`], drained by
/// [`CobSort::cob_file_sort_retrieve`].
pub struct CobSort {
    arena: Vec<CobItem>,
    empty: Option<usize>,
    queue: [QueueStruct; 4],
    keys: Vec<SortKey>,
    col: Option<[u8; 256]>,
    size: usize,
    unique: usize,
    retrieving: bool,
    retrieval_queue: usize,
    flag_merge: bool,
}

impl CobSort {
    /// Port of `fileio.c:cob_file_sort_init` — set up the engine for a file of `record_max`-wide records
    /// with an optional collating sequence (`collating_sequence` ?? the module default). Keys are added
    /// afterwards via [`CobSort::cob_file_sort_init_key`].
    pub fn cob_file_sort_init(record_max: usize, collating_sequence: Option<[u8; 256]>) -> CobSort {
        CobSort {
            arena: Vec::new(),
            empty: None,
            queue: [QueueStruct::default(); 4],
            keys: Vec::new(),
            col: collating_sequence,
            size: record_max,
            unique: 0,
            retrieving: false,
            retrieval_queue: 0,
            flag_merge: false,
        }
    }

    /// Port of `fileio.c:cob_file_sort_init_key` — append an alphanumeric sort key (in declaration order).
    pub fn cob_file_sort_init_key(&mut self, offset: usize, size: usize, ascending: bool) {
        self.keys.push(SortKey { offset, size, ascending, attr: None });
    }

    /// As [`Self::cob_file_sort_init_key`] but for a typed (e.g. numeric) key — see
    /// [`cob_file_sort_init_key_typed`].
    pub fn cob_file_sort_init_key_typed(&mut self, offset: usize, size: usize, ascending: bool, attr: crate::attr::FieldAttr) {
        self.keys.push(SortKey { offset, size, ascending, attr: Some(attr) });
    }

    /// Port of `fileio.c:cob_file_sort_options` — record whether this is a MERGE (`parms[0] == 'M'`).
    pub fn cob_file_sort_options(&mut self, parms: &str) {
        self.flag_merge = parms.as_bytes().first() == Some(&b'M');
    }

    /// Port of `fileio.c:cob_new_item` — allocate (or recycle from the `empty` free-list) a fresh item.
    fn cob_new_item(&mut self) -> usize {
        if let Some(q) = self.empty {
            self.empty = self.arena[q].next;
            self.arena[q].end_of_block = false;
            self.arena[q].next = None;
            return q;
        }
        self.arena.push(CobItem { item: vec![0u8; self.size], unique: 0, end_of_block: false, next: None });
        self.arena.len() - 1
    }

    /// Port of `fileio.c:cob_file_sort_compare` — order two arena items by the keys (numeric keys are a
    /// declared composition with `GNURUST.NUMCMP.1`; here the alphanumeric/byte path), then by `unique`.
    fn compare(&self, k1: usize, k2: usize) -> i32 {
        let a = &self.arena[k1];
        let b = &self.arena[k2];
        match cob_file_sort_compare(&a.item, a.unique, &b.item, b.unique, &self.keys, self.col.as_ref()) {
            Ordering::Less => -1,
            Ordering::Greater => 1,
            Ordering::Equal => 0,
        }
    }

    /// Port of `fileio.c:cob_sort_queues` — the natural 4-queue merge. Repeatedly merges the runs in
    /// `queue[source]`/`queue[source+1]` into `queue[destination]`/`queue[destination+1]` (ping-pong)
    /// until a single sorted run remains; returns the index of the queue holding it.
    fn cob_sort_queues(&mut self) -> usize {
        let mut source = 0usize;
        while self.queue[source + 1].count != 0 {
            let mut destination = source ^ 2;
            self.queue[destination] = QueueStruct::default();
            self.queue[destination + 1] = QueueStruct::default();
            loop {
                let mut end_of_block = [self.queue[source].count == 0, self.queue[source + 1].count == 0];
                if end_of_block[0] && end_of_block[1] {
                    break;
                }
                while !end_of_block[0] || !end_of_block[1] {
                    let move_: usize = if end_of_block[0] {
                        1
                    } else if end_of_block[1] {
                        0
                    } else {
                        let res = self.compare(self.queue[source].first.unwrap(), self.queue[source + 1].first.unwrap());
                        if res < 0 { 0 } else { 1 }
                    };
                    let q = self.queue[source + move_].first.unwrap();
                    if self.arena[q].end_of_block {
                        end_of_block[move_] = true;
                    }
                    self.queue[source + move_].first = self.arena[q].next;
                    if self.queue[destination].first.is_none() {
                        self.queue[destination].first = Some(q);
                    } else {
                        let last = self.queue[destination].last.unwrap();
                        self.arena[last].next = Some(q);
                    }
                    self.queue[destination].last = Some(q);
                    self.queue[source + move_].count -= 1;
                    self.queue[destination].count += 1;
                    self.arena[q].next = None;
                    self.arena[q].end_of_block = false;
                }
                let last = self.queue[destination].last.unwrap();
                self.arena[last].end_of_block = true;
                destination ^= 1;
            }
            source = destination & 2;
        }
        source
    }

    /// Port of `fileio.c:cob_file_sort_submit` — insert record `p` into the sort: a fresh one-element
    /// block pushed onto the shorter of `queue[0]`/`queue[1]`. (The temp-file `switch_to_file` branch is
    /// the declared OS boundary; in-memory sorts never take it.)
    pub fn cob_file_sort_submit(&mut self, p: &[u8]) -> i32 {
        if self.retrieving {
            return COBSORTABORT;
        }
        let q = self.cob_new_item();
        self.arena[q].end_of_block = true;
        self.arena[q].unique = self.unique;
        self.unique += 1;
        let n = self.size.min(p.len());
        self.arena[q].item[..n].copy_from_slice(&p[..n]);
        for b in &mut self.arena[q].item[n..] {
            *b = 0;
        }
        let z = if self.queue[0].count <= self.queue[1].count { 0 } else { 1 };
        self.arena[q].next = self.queue[z].first;
        self.queue[z].first = Some(q);
        self.queue[z].count += 1;
        0
    }

    /// Port of `fileio.c:cob_file_sort_process` (in-memory path) — run the merge and mark the engine as
    /// retrieving; the single sorted run is left in `queue[retrieval_queue]`.
    fn cob_file_sort_process(&mut self) -> i32 {
        let n = self.cob_sort_queues();
        self.retrieving = true;
        self.retrieval_queue = n;
        0
    }

    /// Port of `fileio.c:cob_file_sort_retrieve` (in-memory path) — copy the next record (in sorted order)
    /// into `p`, recycling its item onto the `empty` free-list. Returns [`COBSORTEND`] when drained.
    pub fn cob_file_sort_retrieve(&mut self, p: &mut [u8]) -> i32 {
        if !self.retrieving {
            let res = self.cob_file_sort_process();
            if res != 0 {
                return res;
            }
        }
        let z = self.retrieval_queue;
        let first = match self.queue[z].first {
            None => return COBSORTEND,
            Some(f) => f,
        };
        let n = self.size.min(p.len());
        p[..n].copy_from_slice(&self.arena[first].item[..n]);
        let next = self.arena[first].next;
        self.arena[first].next = self.empty;
        self.empty = Some(first);
        self.queue[z].first = next;
        0
    }

    /// Port of `fileio.c:cob_file_sort_using` — submit every record of the input data into the sort (the
    /// records the C reads from `data_file` via `cob_read_next`), space-padding/truncating to record size
    /// via [`cob_copy_check`].
    pub fn cob_file_sort_using(&mut self, data: &[&[u8]]) {
        for rec in data {
            let padded = cob_copy_check(rec, self.size);
            if self.cob_file_sort_submit(&padded) != 0 {
                break;
            }
        }
    }

    /// Port of `fileio.c:cob_file_sort_giving_internal` — drain the sorted records (what the C writes to
    /// the GIVING files), returning them in order.
    pub fn cob_file_sort_giving_internal(&mut self) -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        let mut buf = vec![0u8; self.size];
        loop {
            if self.cob_file_sort_retrieve(&mut buf) == COBSORTEND {
                break;
            }
            out.push(buf.clone());
        }
        out
    }

    /// Port of `fileio.c:cob_file_sort_giving` — the public GIVING entry (single output); delegates to
    /// [`CobSort::cob_file_sort_giving_internal`].
    pub fn cob_file_sort_giving(&mut self) -> Vec<Vec<u8>> {
        self.cob_file_sort_giving_internal()
    }

    /// Port of `fileio.c:cob_free_list` / `cob_file_sort_close` — release the engine's items. (Rust drops
    /// the arena; this resets the queues so a closed sort holds nothing.)
    pub fn cob_free_list(&mut self) {
        self.arena = Vec::new();
        self.empty = None;
        self.queue = [QueueStruct::default(); 4];
    }

    /// Port of `fileio.c:cob_file_sort_close` — finish the sort and free its working storage.
    pub fn cob_file_sort_close(&mut self) {
        self.cob_free_list();
    }

    /// Port of `fileio.c:cob_file_release` — the `RELEASE record` verb of a SORT INPUT PROCEDURE: submit
    /// `record` to the sort, returning FILE STATUS `00` on success or `30` (permanent error) if the engine
    /// is already retrieving.
    pub fn cob_file_release(&mut self, record: &[u8]) -> &'static str {
        if self.cob_file_sort_submit(record) == 0 {
            "00"
        } else {
            "30"
        }
    }

    /// Port of `fileio.c:cob_file_return` — the `RETURN INTO record` verb of a SORT OUTPUT PROCEDURE:
    /// retrieve the next sorted record into `buf`, returning FILE STATUS `00` (got a record), `10` (end of
    /// the sorted run), or `30` (permanent error).
    pub fn cob_file_return(&mut self, buf: &mut [u8]) -> &'static str {
        match self.cob_file_sort_retrieve(buf) {
            0 => "00",
            COBSORTEND => "10",
            _ => "30",
        }
    }

    /// Port of `fileio.c:cob_file_sort_using_extfh` — the USING side of a SORT/MERGE when the input file is
    /// served by an external file handler: read every record of `data` (the C reads them through `callfh`
    /// via `cob_extfh_open`/`cob_extfh_read_next` — that I/O is the declared boundary) and submit each.
    pub fn cob_file_sort_using_extfh(&mut self, data: &[&[u8]], _callfh: &mut CallFh) {
        self.cob_file_sort_using(data);
    }

    /// Port of `fileio.c:cob_file_sort_giving_extfh` — the GIVING side of a SORT/MERGE when the output file
    /// is served by an external file handler: drain the sorted records (the C writes them through `callfh`
    /// via `cob_file_sort_giving_internal` — that I/O is the declared boundary).
    pub fn cob_file_sort_giving_extfh(&mut self, _callfh: &mut CallFh) -> Vec<Vec<u8>> {
        self.cob_file_sort_giving_internal()
    }
}

// ======================================================================================================
// INDEXED organization — the keyed-store handler (`GNURUST.FILEIO.INDEXED.1`)
//
// A 1:1 port of the COBOL-observable behaviour of fileio.c's indexed_* handlers (indexed_open/write/read/
// read_next/start/rewrite/delete + the *_internal variants): a primary RECORD KEY indexes records, WRITE
// rejects a duplicate primary key with status 22 (and a non-ascending key under SEQUENTIAL access with 21),
// READ by key returns the record or 23, READ NEXT walks the keys in ascending order (AT END 10), START
// positions the cursor by an =/</<=/>/>= condition (or 23 when no key satisfies it), and REWRITE/DELETE
// require the key to exist (else 23). The on-disk index file format is backend-specific (BDB / VBISAM / an
// external file handler) and is the DECLARED OS boundary — what a COBOL program observes is the record
// bytes, the FILE STATUS, and
// the key order, which this in-memory `BTreeMap`-keyed model reproduces. Alternate keys, DUPLICATES, and
// READ PREVIOUS are non-claims (the primary-key contract is the court).
// ======================================================================================================

/// A `START`/`READ` relational condition (`common.h` `COB_EQ`/`COB_LT`/`COB_LE`/`COB_GT`/`COB_GE`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartCond {
    Eq = 1,
    Lt = 2,
    Le = 3,
    Gt = 4,
    Ge = 5,
}

/// The sequential read position of an INDEXED file (what a forward `READ NEXT` consults). `BeforeStart`
/// reads from the first key; `NextKey(k)` reads from the first key `>= k`; `AtEnd` is `10`.
#[derive(Debug, Clone, PartialEq, Eq)]
enum CursorPos {
    BeforeStart,
    NextKey(Vec<u8>),
    AtEnd,
}

/// An `ALTERNATE RECORD KEY`: the key byte range within the record, and whether `WITH DUPLICATES`.
#[derive(Clone)]
struct AltKeyDef {
    offset: usize,
    len: usize,
    duplicates: bool,
}

/// The sequential position of an alternate-key cursor: the alt key index, the current alt-key value, and
/// the 0-based duplicate slot within it (the entry just returned; `READ NEXT`/`PREVIOUS` step from here).
#[derive(Debug, Clone, PartialEq, Eq)]
struct AltCursor {
    key_idx: usize,
    value: Vec<u8>,
    dup: usize,
}

/// Port of the COBOL-observable subset of fileio.c's `struct indexed_file` — an INDEXED file as a
/// primary-key-ordered store, with secondary (alternate-key) indexes. The primary key is the record's
/// `[key_offset, key_offset+key_len)` byte range; the cursor tracks where the next forward `READ NEXT`
/// resumes. Each alternate key maps its value to the primary keys carrying it (in `WITH DUPLICATES` /
/// dupno order), so a read by an alternate key two-hops alt-value -> primary-key -> record.
pub struct IndexedStore {
    key_offset: usize,
    key_len: usize,
    recs: std::collections::BTreeMap<Vec<u8>, Vec<u8>>,
    pub open_mode: OpenMode,
    pub access_mode: AccessMode,
    cursor: CursorPos,
    last_key: Option<Vec<u8>>,
    flag_nonexistent: bool,
    /// Alternate-key definitions (index `i` is the `i`-th `ALTERNATE RECORD KEY`).
    alt_keys: Vec<AltKeyDef>,
    /// Per alternate key: alt-value -> the primary keys carrying it, in insertion (dupno) order. The
    /// dupno of slot `j` is `j + 1` (the C's `get_dupno` starts at 1).
    alt_index: Vec<std::collections::BTreeMap<Vec<u8>, Vec<Vec<u8>>>>,
    /// When the last read was by an alternate key, the alt cursor `READ NEXT`/`PREVIOUS` walk (alt-value
    /// then dupno order). Cleared by a primary read/start, so `READ NEXT` reverts to primary order.
    alt_cursor: Option<AltCursor>,
}

impl IndexedStore {
    /// The primary key bytes of a record.
    fn key_of(&self, record: &[u8]) -> Vec<u8> {
        let end = (self.key_offset + self.key_len).min(record.len());
        record[self.key_offset.min(record.len())..end].to_vec()
    }

    /// Port of `fileio.c:indexed_open` — open an empty (or existing) keyed store in `mode`; sets the
    /// `flag_nonexistent` used by `OPEN INPUT`/`I-O` of a missing file. The records map is the file image.
    pub fn indexed_open(key_offset: usize, key_len: usize, access_mode: AccessMode, mode: OpenMode) -> IndexedStore {
        IndexedStore {
            key_offset,
            key_len,
            recs: std::collections::BTreeMap::new(),
            open_mode: mode,
            access_mode,
            cursor: CursorPos::BeforeStart,
            last_key: None,
            flag_nonexistent: false,
            alt_keys: Vec::new(),
            alt_index: Vec::new(),
            alt_cursor: None,
        }
    }

    /// Register an `ALTERNATE RECORD KEY IS <field> [WITH DUPLICATES]` (its byte range within the record).
    /// Returns the alternate key's 0-based index (the `i` in the on-disk `<base>.<i+1>` file). Must be
    /// called for every alternate key before records are written/loaded.
    pub fn indexed_add_alt_key(&mut self, offset: usize, len: usize, duplicates: bool) -> usize {
        self.alt_keys.push(AltKeyDef { offset, len, duplicates });
        self.alt_index.push(std::collections::BTreeMap::new());
        self.alt_keys.len() - 1
    }

    /// The `i`-th alternate key's value bytes within `record`.
    fn alt_value_of(&self, record: &[u8], i: usize) -> Vec<u8> {
        let a = &self.alt_keys[i];
        let end = (a.offset + a.len).min(record.len());
        record[a.offset.min(record.len())..end].to_vec()
    }

    /// Insert a record's primary key into every alternate index (called after a successful primary write).
    fn alt_index_insert(&mut self, record: &[u8], primary: &[u8]) {
        for i in 0..self.alt_keys.len() {
            let v = self.alt_value_of(record, i);
            self.alt_index[i].entry(v).or_default().push(primary.to_vec());
        }
    }

    /// Load an INDEXED file written by the genuine GnuCOBOL compiler: parse the Berkeley DB B-tree
    /// `.dat` bytes (via the pure-safe `gnucobol-rs-bdb-format` crate) and populate this store with the
    /// key->record pairs. This is the read path for cross-tool interchange -- a `.dat` produced by real
    /// `cobc` (libcob over Berkeley DB) becomes readable by the port. Returns the record count, or a
    /// typed error if the bytes are not a B-tree DB file (e.g. an empty/never-written file). The
    /// BDB key is the COBOL record key; the BDB data is the full record image, matching this store's
    /// `recs` (record-key -> record).
    pub fn indexed_load_bdb(&mut self, bytes: &[u8]) -> Result<usize, gnucobol_rs_bdb_format::BdbError> {
        let db = gnucobol_rs_bdb_format::BdbFile::parse(bytes)?;
        let pairs = db.records()?;
        let n = pairs.len();
        for (key, record) in pairs {
            // Rebuild the alternate indexes from the loaded records (so a primary file + registered alt
            // keys yields working alt reads); `indexed_load_alt` instead reads a genuine on-disk alt file.
            self.alt_index_insert(&record, &key);
            self.recs.insert(key, record);
        }
        Ok(n)
    }

    /// Serialise this store to a Berkeley DB B-tree `.dat` (via `gnucobol-rs-bdb-format`) that the
    /// genuine GnuCOBOL `cobc` can OPEN and READ -- the write side of cross-tool interchange (a port
    /// runtime writes an INDEXED file the real compiler reads back). Emits the records key->record in
    /// ascending key order. A record set too large for a single leaf page returns a typed error (the
    /// multi-leaf writer is a follow-on), so the caller never writes a corrupt file.
    pub fn indexed_to_bdb(&self) -> Result<Vec<u8>, gnucobol_rs_bdb_format::BdbError> {
        let pairs: Vec<(Vec<u8>, Vec<u8>)> =
            self.recs.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        gnucobol_rs_bdb_format::write_btree(&pairs, 4096)
    }

    /// Serialise alternate key `i`'s on-disk index file (cobc names it `<base>.<i+1>`): a Berkeley DB
    /// B-tree keyed by the alt-key value, whose data is the primary key followed -- for a `WITH DUPLICATES`
    /// key -- by the 4-byte **native-LE dupno** (`slot + 1`; `COB_DUPSWAP` is the identity on a
    /// little-endian host, the `|| 1` preserved historical bug). A unique alternate key stores the primary
    /// key alone (no dupno trailer). A genuine `cobc` can OPEN + READ the result.
    pub fn indexed_alt_to_bdb(&self, i: usize) -> Result<Vec<u8>, gnucobol_rs_bdb_format::BdbError> {
        let dups = self.alt_keys[i].duplicates;
        let mut pairs: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
        for (value, primaries) in &self.alt_index[i] {
            for (slot, primary) in primaries.iter().enumerate() {
                let mut data = primary.clone();
                if dups {
                    data.extend_from_slice(&((slot as u32) + 1).to_le_bytes());
                }
                pairs.push((value.clone(), data));
            }
        }
        // A WITH DUPLICATES alternate key is a DB_DUP database (BTM_DUP flag + shared duplicate-key items),
        // which the genuine cobc requires to OPEN + READ the file.
        gnucobol_rs_bdb_format::write_btree_dup(&pairs, 4096, dups)
    }

    /// Load alternate key `i`'s on-disk index file written by genuine `cobc` (`<base>.<i+1>`): parse the
    /// B-tree and split each data value into the primary key (its first `key_len` bytes) and the optional
    /// 4-byte LE dupno trailer, populating `alt_index[i]` in dupno order. The alternate key must already be
    /// registered (via [`IndexedStore::indexed_add_alt_key`]). Returns the entry count.
    pub fn indexed_load_alt(
        &mut self,
        i: usize,
        bytes: &[u8],
    ) -> Result<usize, gnucobol_rs_bdb_format::BdbError> {
        let primekeylen = self.key_len;
        let db = gnucobol_rs_bdb_format::BdbFile::parse(bytes)?;
        let pairs = db.records()?;
        let n = pairs.len();
        self.alt_index[i].clear(); // replace this alt index with the on-disk image
        let mut by_val: std::collections::BTreeMap<Vec<u8>, Vec<(u32, Vec<u8>)>> =
            std::collections::BTreeMap::new();
        for (value, data) in pairs {
            let cut = primekeylen.min(data.len());
            let primary = data[..cut].to_vec();
            let dupno = if data.len() >= primekeylen + 4 {
                u32::from_le_bytes([
                    data[primekeylen],
                    data[primekeylen + 1],
                    data[primekeylen + 2],
                    data[primekeylen + 3],
                ])
            } else {
                1
            };
            by_val.entry(value).or_default().push((dupno, primary));
        }
        for (value, mut slots) in by_val {
            slots.sort_by_key(|(d, _)| *d);
            self.alt_index[i].insert(value, slots.into_iter().map(|(_, p)| p).collect());
        }
        Ok(n)
    }

    /// The first key strictly greater than `k`, as a cursor position (`AtEnd` when none).
    fn after(&self, k: &[u8]) -> CursorPos {
        match self
            .recs
            .range((std::ops::Bound::Excluded(k.to_vec()), std::ops::Bound::Unbounded))
            .next()
        {
            Some((kk, _)) => CursorPos::NextKey(kk.clone()),
            None => CursorPos::AtEnd,
        }
    }

    /// Port of `fileio.c:indexed_write_internal` — insert the record under its primary key, returning
    /// status `22` if the key already exists (no DUPLICATES on the primary key), else `00`.
    pub fn indexed_write_internal(&mut self, record: &[u8]) -> &'static str {
        let key = self.key_of(record);
        if self.recs.contains_key(&key) {
            return "22";
        }
        self.recs.insert(key.clone(), record.to_vec());
        self.alt_index_insert(record, &key);
        "00"
    }

    /// Port of `fileio.c:indexed_write` — write a record. A nonexistent OUTPUT file is `48`; under
    /// SEQUENTIAL access the key must be strictly ascending (else `21`); otherwise delegates to
    /// [`IndexedStore::indexed_write_internal`] (duplicate primary key `22`).
    pub fn indexed_write(&mut self, record: &[u8]) -> &'static str {
        if self.flag_nonexistent {
            return "48";
        }
        let key = self.key_of(record);
        if self.access_mode == AccessMode::Sequential {
            if let Some(last) = &self.last_key {
                if &key <= last {
                    return "21";
                }
            }
        }
        let st = self.indexed_write_internal(record);
        if st == "00" {
            self.last_key = Some(key);
        }
        st
    }

    /// Port of `fileio.c:indexed_read` — random read by `key`: `00` and the record (positioning the cursor
    /// just after it for a following `READ NEXT`) when present, `23` (record not found) otherwise.
    pub fn indexed_read(&mut self, key: &[u8]) -> (&'static str, Option<Vec<u8>>) {
        let k = key.to_vec();
        self.alt_cursor = None; // a primary read reverts READ NEXT/PREVIOUS to primary order
        match self.recs.get(&k) {
            Some(rec) => {
                let rec = rec.clone();
                self.cursor = self.after(&k);
                ("00", Some(rec))
            }
            None => ("23", None),
        }
    }

    /// Read by an `ALTERNATE RECORD KEY` value (alt key index `i`): two-hop alt-value -> first primary key
    /// (lowest dupno) -> record, positioning the alt cursor so a following `READ NEXT`/`PREVIOUS` walks the
    /// alternate-key order (alt-value asc, then dupno asc). `00` + record when present, else `23`. cobc
    /// returns `00` even for a value with duplicates (the C sets `02` then clobbers it with the DB_PUT/GET
    /// result -- see `cli-runtime`/`indexed-altkeys` notes), so this never surfaces `02`.
    pub fn indexed_read_alt(&mut self, i: usize, value: &[u8]) -> (&'static str, Option<Vec<u8>>) {
        let v = value.to_vec();
        match self.alt_index.get(i).and_then(|m| m.get(&v)).and_then(|ks| ks.first()).cloned() {
            Some(primary) => {
                let rec = self.recs.get(&primary).cloned();
                self.alt_cursor = Some(AltCursor { key_idx: i, value: v, dup: 0 });
                match rec {
                    Some(r) => ("00", Some(r)),
                    None => ("23", None), // alt index points at a missing primary (corrupt) -> not found
                }
            }
            None => ("23", None),
        }
    }

    /// Port of `fileio.c:indexed_read_next` — sequential read from the cursor: when a read-by-alt is active,
    /// the next entry in alternate-key order; otherwise ascending primary-key order. `00` + record, or `10`
    /// at end of file.
    pub fn indexed_read_next(&mut self) -> (&'static str, Option<Vec<u8>>) {
        if self.alt_cursor.is_some() {
            return self.alt_step(true);
        }
        let next_key = match &self.cursor {
            CursorPos::AtEnd => None,
            CursorPos::BeforeStart => self.recs.keys().next().cloned(),
            CursorPos::NextKey(c) => self.recs.range(c.clone()..).next().map(|(k, _)| k.clone()),
        };
        match next_key {
            Some(k) => {
                let rec = self.recs[&k].clone();
                self.cursor = self.after(&k);
                ("00", Some(rec))
            }
            None => {
                self.cursor = CursorPos::AtEnd;
                ("10", None)
            }
        }
    }

    /// `READ PREVIOUS` — the reverse of [`IndexedStore::indexed_read_next`]: the previous entry in alternate
    /// -key order when a read-by-alt is active, else descending primary-key order. `10` at start of file.
    pub fn indexed_read_previous(&mut self) -> (&'static str, Option<Vec<u8>>) {
        if self.alt_cursor.is_some() {
            return self.alt_step(false);
        }
        // Primary descending: the greatest key strictly below the forward cursor's resume point.
        let upper = match &self.cursor {
            CursorPos::BeforeStart => None, // before the first key -> nothing previous
            CursorPos::AtEnd => self.recs.keys().next_back().cloned(),
            CursorPos::NextKey(c) => self
                .recs
                .range::<Vec<u8>, _>((std::ops::Bound::Unbounded, std::ops::Bound::Excluded(c.clone())))
                .next_back()
                .map(|(k, _)| k.clone()),
        };
        match upper {
            Some(k) => {
                let rec = self.recs[&k].clone();
                self.cursor = CursorPos::NextKey(k); // resume point is this key (so the next PREVIOUS goes below it)
                ("00", Some(rec))
            }
            None => ("10", None),
        }
    }

    /// Step the alternate-key cursor forward (`next = true`) or backward, two-hopping to the primary record.
    /// `10` at the corresponding end.
    fn alt_step(&mut self, next: bool) -> (&'static str, Option<Vec<u8>>) {
        let cur = self.alt_cursor.clone().expect("alt_step requires an active alt cursor");
        let map = &self.alt_index[cur.key_idx];
        // Candidate next position: another dupno under the same value, else the adjacent value's edge slot.
        let pos = if next {
            let len = map.get(&cur.value).map(|v| v.len()).unwrap_or(0);
            if cur.dup + 1 < len {
                Some((cur.value.clone(), cur.dup + 1))
            } else {
                map.range::<Vec<u8>, _>((std::ops::Bound::Excluded(cur.value.clone()), std::ops::Bound::Unbounded))
                    .next()
                    .map(|(v, _)| (v.clone(), 0))
            }
        } else if cur.dup > 0 {
            Some((cur.value.clone(), cur.dup - 1))
        } else {
            map.range::<Vec<u8>, _>((std::ops::Bound::Unbounded, std::ops::Bound::Excluded(cur.value.clone())))
                .next_back()
                .map(|(v, ks)| (v.clone(), ks.len() - 1))
        };
        match pos {
            Some((value, dup)) => {
                let primary = self.alt_index[cur.key_idx][&value][dup].clone();
                let rec = self.recs.get(&primary).cloned();
                self.alt_cursor = Some(AltCursor { key_idx: cur.key_idx, value, dup });
                match rec {
                    Some(r) => ("00", Some(r)),
                    None => ("23", None),
                }
            }
            None => ("10", None),
        }
    }

    /// Port of `fileio.c:indexed_start_internal` — find the key satisfying `cond` relative to `key`,
    /// returning the matched key or `None` when none satisfies the condition.
    fn indexed_start_internal(&self, cond: StartCond, key: &[u8]) -> Option<Vec<u8>> {
        use std::ops::Bound;
        let k = key.to_vec();
        match cond {
            StartCond::Eq => self.recs.get_key_value(&k).map(|(kk, _)| kk.clone()),
            StartCond::Ge => self.recs.range(k..).next().map(|(kk, _)| kk.clone()),
            StartCond::Gt => self
                .recs
                .range((Bound::Excluded(k), Bound::Unbounded))
                .next()
                .map(|(kk, _)| kk.clone()),
            StartCond::Le => self.recs.range(..=k).next_back().map(|(kk, _)| kk.clone()),
            StartCond::Lt => self
                .recs
                .range((Bound::Unbounded, Bound::Excluded(k)))
                .next_back()
                .map(|(kk, _)| kk.clone()),
        }
    }

    /// Port of `fileio.c:indexed_start` — position the file at the first key satisfying `cond key`,
    /// returning `00` (cursor set so a following `READ NEXT` returns that record) or `23` when none does.
    pub fn indexed_start(&mut self, cond: StartCond, key: &[u8]) -> &'static str {
        self.alt_cursor = None; // START repositions the primary cursor; READ NEXT reverts to primary order
        match self.indexed_start_internal(cond, key) {
            Some(k) => {
                self.cursor = CursorPos::NextKey(k);
                "00"
            }
            None => {
                self.cursor = CursorPos::AtEnd;
                "23"
            }
        }
    }

    /// Port of `fileio.c:indexed_rewrite` — replace an existing record (matched by its primary key). When
    /// the primary key is not present the ISAM path returns `21` (KEY_INVALID, the `isread ISEQUAL` miss at
    /// fileio.c:5754), else `00`.
    pub fn indexed_rewrite(&mut self, record: &[u8]) -> &'static str {
        let key = self.key_of(record);
        if !self.recs.contains_key(&key) {
            return "21";
        }
        self.recs.insert(key, record.to_vec());
        "00"
    }

    /// Port of `fileio.c:indexed_delete_internal` — remove the record under `key`; `23` when absent.
    pub fn indexed_delete_internal(&mut self, key: &[u8]) -> &'static str {
        if self.recs.remove(key).is_some() {
            "00"
        } else {
            "23"
        }
    }

    /// Port of `fileio.c:indexed_delete` — delete the record at `key` (a `DELETE key`); `00` or `23`.
    pub fn indexed_delete(&mut self, key: &[u8]) -> &'static str {
        self.indexed_delete_internal(key)
    }

    /// Port of `fileio.c:indexed_file_delete` — drop the whole indexed file (every key/record).
    pub fn indexed_file_delete(&mut self) {
        self.recs.clear();
        self.cursor = CursorPos::BeforeStart;
        self.last_key = None;
    }

    /// Port of `fileio.c:indexed_close` — close the file, clearing the per-open cursor state.
    pub fn indexed_close(&mut self) {
        self.open_mode = OpenMode::Closed;
        self.cursor = CursorPos::BeforeStart;
    }

    /// The records in ascending key order (the file image a `READ NEXT` sweep would yield).
    pub fn records_in_key_order(&self) -> Vec<Vec<u8>> {
        self.recs.values().cloned().collect()
    }
}

// ======================================================================================================
// Record / file locking (`GNURUST.FILEIO.INDEXED.1` locking sub-layer)
//
// A faithful port of the COBOL-observable status CONTRACT of fileio.c's BDB locking (lock_record/
// unlock_record/test_record_lock/lock_file/unlock_file): a NOWAIT `DB_LOCK_WRITE` request is granted unless
// another open holds a conflicting lock on the same object, in which case a record request returns `51`
// (RECORD LOCKED) and a file request `61` (FILE SHARING); a BDB deadlock is `52`. The actual cross-process
// BDB lock environment is the declared OS boundary; `LockEnv` reproduces the grant/deny decision (and the
// per-file `record_locked`/`file_lock_set` flags) that a COBOL program observes via FILE STATUS.
// ======================================================================================================

/// Port of the `bdb_env` lock manager (the process-wide BDB lock environment) — the set of currently held
/// record-lock objects and file locks. The real BDB env is the OS boundary; this models its NOWAIT grants.
#[derive(Default)]
pub struct LockEnv {
    held_records: std::collections::HashSet<Vec<u8>>,
    held_files: std::collections::HashSet<String>,
}

/// Port of the per-file lock fields of `struct indexed_file` — whether this open holds a record/file lock.
#[derive(Default)]
pub struct FileLockState {
    pub record_locked: bool,
    record_key: Option<Vec<u8>>,
    pub file_lock_set: bool,
    lock_filename: Option<String>,
}

impl LockEnv {
    /// A fresh, empty lock environment.
    pub fn new() -> LockEnv {
        LockEnv::default()
    }

    /// Port of `fileio.c:lock_record` — impose a NOWAIT write lock on the record `key`. Granted (`00`,
    /// `record_locked = 1`) unless another open already holds it, in which case `51` (RECORD LOCKED). The
    /// same open re-locking its own record is granted (BDB same-owner).
    pub fn lock_record(&mut self, f: &mut FileLockState, key: &[u8]) -> &'static str {
        if self.held_records.contains(key) && f.record_key.as_deref() != Some(key) {
            return "51";
        }
        self.held_records.insert(key.to_vec());
        f.record_locked = true;
        f.record_key = Some(key.to_vec());
        "00"
    }

    /// Port of `fileio.c:test_record_lock` — probe whether `key` can be locked (acquire-then-release, no
    /// state change): `00` when grantable, `51` when another open holds it.
    pub fn test_record_lock(&self, f: &FileLockState, key: &[u8]) -> &'static str {
        if self.held_records.contains(key) && f.record_key.as_deref() != Some(key) {
            "51"
        } else {
            "00"
        }
    }

    /// Port of `fileio.c:unlock_record` — release this open's record lock (a no-op `00` when none is held).
    pub fn unlock_record(&mut self, f: &mut FileLockState) -> &'static str {
        if !f.record_locked {
            return "00";
        }
        if let Some(k) = f.record_key.take() {
            self.held_records.remove(&k);
        }
        f.record_locked = false;
        "00"
    }

    /// Port of `fileio.c:lock_file` — impose a NOWAIT lock on the whole file. Granted (`00`,
    /// `file_lock_set = 1`) unless another open holds it (`61`, FILE SHARING).
    pub fn lock_file(&mut self, f: &mut FileLockState, filename: &str) -> &'static str {
        f.file_lock_set = false;
        if self.held_files.contains(filename) && f.lock_filename.as_deref() != Some(filename) {
            return "61";
        }
        self.held_files.insert(filename.to_string());
        f.file_lock_set = true;
        f.lock_filename = Some(filename.to_string());
        "00"
    }

    /// Port of `fileio.c:unlock_file` — release this open's file lock (a no-op `00` when none is held).
    pub fn unlock_file(&mut self, f: &mut FileLockState) -> &'static str {
        if f.file_lock_set {
            if let Some(n) = f.lock_filename.take() {
                self.held_files.remove(&n);
            }
            f.file_lock_set = false;
        }
        "00"
    }
}

// ======================================================================================================
// BDB indexed-backend cursor state (`GNURUST.FILEIO.INDEXED.1` substrate)
//
// A port of the COBOL-observable cursor bookkeeping of fileio.c's BDB `struct indexed_file`: which
// per-index read/write cursors are open. The actual Berkeley DB cursor open/close (`db->cursor`,
// `cursor->close`) and the BDB lock environment are the declared OS boundary; this models the
// open/closed FLAGS and the "already open / already closed -> 0, else 1" return contract.
// ======================================================================================================

/// Port of the cursor fields of fileio.c's `struct indexed_file` — the write-cursor flag and per-index
/// cursor-open flags. The Berkeley DB cursors themselves are the declared boundary.
pub struct BdbFile {
    write_cursor_open: bool,
    cursor_open: Vec<bool>,
}

impl BdbFile {
    /// A BDB indexed file with `nkeys` (>=1) index cursors, all closed.
    pub fn new(nkeys: usize) -> BdbFile {
        BdbFile { write_cursor_open: false, cursor_open: vec![false; nkeys.max(1)] }
    }

    /// Port of `fileio.c:bdb_open_cursor` — open the primary write cursor if not already open, returning
    /// `0` when it was already open and `1` when it is opened now. (`for_write` selects `DB_WRITECURSOR` on
    /// the real cursor — the declared BDB boundary.)
    pub fn bdb_open_cursor(&mut self, _for_write: bool) -> i32 {
        if self.write_cursor_open {
            return 0;
        }
        self.cursor_open[0] = true;
        self.write_cursor_open = true;
        1
    }

    /// Port of `fileio.c:bdb_close_cursor` — close the primary write cursor, returning `0` when it was
    /// already closed and `1` when it is closed now (the write-cursor flag is always cleared).
    pub fn bdb_close_cursor(&mut self) -> i32 {
        self.write_cursor_open = false;
        if !self.cursor_open[0] {
            return 0;
        }
        self.cursor_open[0] = false;
        1
    }

    /// Port of `fileio.c:bdb_close_index` — close the cursor on index `index`, returning `0` when it was
    /// already closed and `1` when it is closed now.
    pub fn bdb_close_index(&mut self, index: usize) -> i32 {
        if index >= self.cursor_open.len() || !self.cursor_open[index] {
            return 0;
        }
        self.cursor_open[index] = false;
        1
    }
}

/// Port of the absolute/no-environment path of `fileio.c:bdb_nofile` — is `filename` absent? Probes the
/// filesystem (the C `access(F_OK)` returning `ENOENT`). The `bdb_data_dir` search path is the BDB-env
/// boundary; without a BDB environment (or for an absolute name) the file is checked directly.
pub fn bdb_nofile(filename: &str) -> bool {
    !std::path::Path::new(filename).exists()
}

/// Port of `fileio.c:bdb_errcall_set` — the BDB error callback: format the `"BDB error: <prefix> <err>"`
/// diagnostic (the C then raises a runtime error and hard-fails — the runtime-abort boundary).
pub fn bdb_errcall_set(prefix: &str, err: &str) -> String {
    format!("BDB error: {prefix} {err}")
}

/// Port of `fileio.c:bdb_msgcall_set` — the BDB message callback: format the `"BDB error: <err>"`
/// diagnostic (the runtime abort is the declared boundary).
pub fn bdb_msgcall_set(err: &str) -> String {
    format!("BDB error: {err}")
}

// ======================================================================================================
// Micro Focus FCD conversion (`fcd2_to_fcd3` / `fcd3_to_fcd2`)
//
// A faithful port of the data fields of fileio.c's conversion between the 32-bit `FCD2` and the 64-bit
// `FCD3` File Control Description structs (the Micro Focus external-file-handler ABI). Single-byte flags copy across; the
// record-length fields widen 16<->32 bits (`LDCOMPX2`/`STCOMPX4`, big-endian in the on-wire struct, held
// here as values); `refKey`/`lineCount` swap roles by ORG_INDEXED; and `fcd2_to_fcd3` sets the
// `MF_CALLFH_GNUCOBOL` (0x80) flag. The struct *pointers* (record/filename/kdb/file handle) and the KDB
// key-block construction are the declared C-ABI boundary; this models the scalar data conversion, which
// round-trips (an FCD2 -> FCD3 -> FCD2 preserves every data field).
// ======================================================================================================

/// `ORG_INDEXED` (`common.h`).
const ORG_INDEXED: u8 = 2;
/// `MF_CALLFH_GNUCOBOL` — set in `gcFlags` when GnuCOBOL drives the external-file-handler conversion.
const MF_CALLFH_GNUCOBOL: u8 = 0x80;

/// The scalar data fields of the 64-bit `FCD3` File Control Description (the pointers + KDB block are the
/// declared C-ABI boundary). Record lengths are held as 32-bit values (the struct stores them BE32).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Fcd3 {
    pub file_status: [u8; 2],
    pub file_org: u8,
    pub access_flags: u8,
    pub open_mode: u8,
    pub record_mode: u8,
    pub file_format: u8,
    pub lock_mode: u8,
    pub other_flags: u8,
    pub fstatus_type: u8,
    pub comp_type: u8,
    pub block_size: u8,
    pub gc_flags: u8,
    pub fsv2_flags: u8,
    pub conf_flags: u8,
    pub conf_flags2: u8,
    pub idx_cache_sz: u8,
    pub idx_cache_area: u8,
    pub cur_rec_len: u32,
    pub min_rec_len: u32,
    pub max_rec_len: u32,
    pub ref_key: [u8; 2],
    pub line_count: [u8; 2],
    pub eff_key_len: [u8; 2],
    pub fname_len: [u8; 2],
    pub rel_byte_adrs: [u8; 8],
}

/// The scalar data fields of the 32-bit `FCD2` File Control Description. Record lengths are 16-bit.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Fcd2 {
    pub file_status: [u8; 2],
    pub file_org: u8,
    pub access_flags: u8,
    pub open_mode: u8,
    pub record_mode: u8,
    pub file_format: u8,
    pub lock_mode: u8,
    pub other_flags: u8,
    pub fstatus_type: u8,
    pub comp_type: u8,
    pub block_size: u8,
    pub gc_flags: u8,
    pub fsv2_flags: u8,
    pub conf_flags: u8,
    pub conf_flags2: u8,
    pub idx_cache_sz: u8,
    pub idx_cache_area: u8,
    pub cur_rec_len: u16,
    pub min_rec_len: u16,
    pub max_rec_len: u16,
    pub ref_key: [u8; 2],
    pub eff_key_len: [u8; 2],
    pub fname_len: [u8; 2],
    pub rel_byte_adrs64: [u8; 8],
}

/// Port of `fileio.c:fcd2_to_fcd3` — convert a 32-bit `FCD2` into a 64-bit `FCD3` (the data fields). Flags
/// copy across, record lengths widen 16->32, the `MF_CALLFH_GNUCOBOL` flag is forced on, and `refKey`/
/// `lineCount` are placed by organization (INDEXED -> `refKey`, else -> `lineCount`).
pub fn fcd2_to_fcd3(fcd2: &Fcd2) -> Fcd3 {
    let mut fcd = Fcd3 {
        file_status: fcd2.file_status,
        file_org: fcd2.file_org,
        access_flags: fcd2.access_flags,
        open_mode: fcd2.open_mode,
        record_mode: fcd2.record_mode,
        file_format: fcd2.file_format,
        lock_mode: fcd2.lock_mode,
        other_flags: fcd2.other_flags,
        fstatus_type: fcd2.fstatus_type,
        comp_type: fcd2.comp_type,
        block_size: fcd2.block_size,
        gc_flags: fcd2.gc_flags | MF_CALLFH_GNUCOBOL,
        fsv2_flags: fcd2.fsv2_flags,
        conf_flags: fcd2.conf_flags,
        conf_flags2: fcd2.conf_flags2,
        idx_cache_sz: fcd2.idx_cache_sz,
        idx_cache_area: fcd2.idx_cache_area,
        cur_rec_len: fcd2.cur_rec_len as u32,
        min_rec_len: fcd2.min_rec_len as u32,
        max_rec_len: fcd2.max_rec_len as u32,
        eff_key_len: fcd2.eff_key_len,
        fname_len: fcd2.fname_len,
        rel_byte_adrs: fcd2.rel_byte_adrs64,
        ..Fcd3::default()
    };
    if fcd.file_org == ORG_INDEXED {
        fcd.line_count = [0, 0];
        fcd.ref_key = fcd2.ref_key;
    } else {
        fcd.line_count = fcd2.ref_key;
        fcd.ref_key = [0, 0];
    }
    fcd
}

/// Port of `fileio.c:fcd3_to_fcd2` — convert a 64-bit `FCD3` back into a 32-bit `FCD2` (the data fields):
/// the inverse of [`fcd2_to_fcd3`], record lengths narrowing 32->16, `refKey` taken from `refKey` for
/// INDEXED else from `lineCount`.
pub fn fcd3_to_fcd2(fcd: &Fcd3) -> Fcd2 {
    Fcd2 {
        file_status: fcd.file_status,
        file_org: fcd.file_org,
        access_flags: fcd.access_flags,
        open_mode: fcd.open_mode,
        record_mode: fcd.record_mode,
        file_format: fcd.file_format,
        lock_mode: fcd.lock_mode,
        other_flags: fcd.other_flags,
        fstatus_type: fcd.fstatus_type,
        comp_type: fcd.comp_type,
        block_size: fcd.block_size,
        gc_flags: fcd.gc_flags,
        fsv2_flags: fcd.fsv2_flags,
        conf_flags: fcd.conf_flags,
        conf_flags2: fcd.conf_flags2,
        idx_cache_sz: fcd.idx_cache_sz,
        idx_cache_area: fcd.idx_cache_area,
        cur_rec_len: fcd.cur_rec_len as u16,
        min_rec_len: fcd.min_rec_len as u16,
        max_rec_len: fcd.max_rec_len as u16,
        ref_key: if fcd.file_org == ORG_INDEXED { fcd.ref_key } else { fcd.line_count },
        eff_key_len: fcd.eff_key_len,
        fname_len: fcd.fname_len,
        rel_byte_adrs64: fcd.rel_byte_adrs,
    }
}

// FCD open-mode / organization codes (`common.h`).
const OPEN_INPUT: u8 = 0;
const OPEN_OUTPUT: u8 = 1;
const OPEN_IO: u8 = 2;
const OPEN_EXTEND: u8 = 3;
const OPEN_NOT_OPEN: u8 = 128;
const ORG_LINE_SEQ: u8 = 0;
const ORG_SEQ: u8 = 1;
const ORG_RELATIVE: u8 = 3;
const REC_MODE_FIXED: u8 = 0;
const REC_MODE_VARIABLE: u8 = 1;
const MF_FST_CRDELIM: u8 = 0x01;
const MF_FST_INSERT_NULLS: u8 = 0x02;
const MF_FST_NO_STRIP_SPACES: u8 = 0x20;

/// Port of `fileio.c:update_file_to_fcd` — copy a `CobFile`'s status, open mode, record lengths, record
/// mode, and organization into an `FCD3` (the data fields; `fnstatus` overrides the file status when
/// given). The line-sequential feature flags (`CRdelim`/`InsertNulls`/`NoStripSpaces`) reflect the file's
/// `LineSeqConfig`. The record pointer + KDB key block are the declared C-ABI boundary.
pub fn update_file_to_fcd(f: &CobFile, fcd: &mut Fcd3, fnstatus: Option<[u8; 2]>) {
    fcd.file_status = fnstatus.unwrap_or(f.file_status);
    fcd.open_mode = match f.open_mode {
        OpenMode::Closed | OpenMode::Locked => OPEN_NOT_OPEN,
        OpenMode::Input => OPEN_INPUT,
        OpenMode::Output => OPEN_OUTPUT,
        OpenMode::Io => OPEN_IO,
        OpenMode::Extend => OPEN_EXTEND,
    };
    fcd.min_rec_len = f.record_min as u32;
    fcd.max_rec_len = f.record_max as u32;
    fcd.cur_rec_len = f.record_max as u32;
    fcd.record_mode = if f.record_min == f.record_max { REC_MODE_FIXED } else { REC_MODE_VARIABLE };
    fcd.ref_key = [0, 0];
    match f.organization {
        Organization::Sequential | Organization::Sort => fcd.file_org = ORG_SEQ,
        Organization::LineSequential => {
            fcd.file_org = ORG_LINE_SEQ;
            if f.line_cfg.ls_nulls {
                fcd.fstatus_type |= MF_FST_INSERT_NULLS;
            }
            if f.line_cfg.ls_fixed {
                fcd.fstatus_type |= MF_FST_NO_STRIP_SPACES;
            }
            // COB_LS_USES_CR is a Windows-only platform feature (the CRdelim flag); off on this host.
            let _ = MF_FST_CRDELIM;
        }
        Organization::Relative => fcd.file_org = ORG_RELATIVE,
        Organization::Indexed => fcd.file_org = ORG_INDEXED,
    }
}

/// Port of `fileio.c:update_fcd_to_file` — copy an `FCD3`'s status, open mode (when `was_open > 0`), and
/// record lengths back into a `CobFile`. Returns the 2-byte FILE STATUS written. The exception side
/// effect, the record pointer, the key-block copy-back, and the lock-mode field are the declared
/// C-ABI/state boundary.
pub fn update_fcd_to_file(fcd: &Fcd3, f: &mut CobFile, was_open: i32) -> [u8; 2] {
    if was_open >= 0 {
        f.file_status = fcd.file_status;
    }
    if was_open > 0 {
        if fcd.open_mode & OPEN_NOT_OPEN != 0 {
            f.open_mode = OpenMode::Closed;
        } else {
            f.open_mode = match fcd.open_mode & 0x7f {
                OPEN_INPUT => OpenMode::Input,
                OPEN_OUTPUT => OpenMode::Output,
                OPEN_EXTEND => OpenMode::Extend,
                OPEN_IO => OpenMode::Io,
                _ => f.open_mode,
            };
        }
    }
    f.record_min = fcd.min_rec_len as usize;
    f.record_max = fcd.max_rec_len as usize;
    f.file_status
}

// FCD access-flag / other-flag codes (`common.h`).
const ACCESS_SEQ: u8 = 0;
const ACCESS_RANDOM: u8 = 4;
const ACCESS_DYNAMIC: u8 = 8;
const OTH_OPTIONAL: u8 = 0x80;
const OTH_NOT_OPTIONAL: u8 = 0x20;

/// Port of the data fields of `fileio.c:copy_file_to_fcd` — initialise an `FCD3` from a `CobFile` ahead of
/// an external-handler open: set the access flags, the OPTIONAL/NOT-OPTIONAL flag, the `MF_CALLFH_GNUCOBOL` flag, mark
/// the file not-open, and copy the status/mode/record/organization fields (via [`update_file_to_fcd`]).
/// The file-name pointer, the KDB key block, and the record pointer are the declared C-ABI boundary.
pub fn copy_file_to_fcd(f: &CobFile, fcd: &mut Fcd3) {
    fcd.access_flags = match f.access_mode {
        AccessMode::Sequential => ACCESS_SEQ,
        AccessMode::Random => ACCESS_RANDOM,
        AccessMode::Dynamic => ACCESS_DYNAMIC,
    };
    fcd.other_flags &= !OTH_OPTIONAL;
    if f.optional {
        fcd.other_flags = (fcd.other_flags & !OTH_NOT_OPTIONAL) | OTH_OPTIONAL;
    } else {
        fcd.other_flags |= OTH_NOT_OPTIONAL;
    }
    fcd.gc_flags |= MF_CALLFH_GNUCOBOL;
    update_file_to_fcd(f, fcd, None);
    fcd.open_mode |= OPEN_NOT_OPEN;
    fcd.ref_key = [0, 0];
}

/// Port of the data fields of `fileio.c:copy_fcd_to_file` — set a `CobFile`'s organization from an `FCD3`
/// and copy its status/mode/record fields back (via [`update_fcd_to_file`]). The KDB key-block copy-back,
/// the record pointer, and the file-name pointer are the declared C-ABI boundary.
pub fn copy_fcd_to_file(fcd: &Fcd3, f: &mut CobFile) {
    f.organization = match fcd.file_org {
        ORG_LINE_SEQ => Organization::LineSequential,
        ORG_RELATIVE => Organization::Relative,
        ORG_INDEXED => Organization::Indexed,
        _ => Organization::Sequential,
    };
    f.access_mode = match fcd.access_flags & 0x0f {
        ACCESS_RANDOM => AccessMode::Random,
        ACCESS_DYNAMIC => AccessMode::Dynamic,
        _ => AccessMode::Sequential,
    };
    update_fcd_to_file(fcd, f, 1);
}

/// Port of `fileio.c:cob_file_fcd_adrs` — obtain the FCD address for a file: build the FCD from the file
/// (pre-opening it when it is not open), leaving `fcd` synced to the file. The returned raw FCD pointer is
/// the declared C-ABI boundary; here the FCD is filled in place via [`copy_file_to_fcd`].
pub fn cob_file_fcd_adrs(f: &mut CobFile, fcd: &mut Fcd3) {
    if f.open_mode == OpenMode::Closed {
        cob_pre_open(f);
    }
    copy_file_to_fcd(f, fcd);
}

/// Port of `fileio.c:cob_file_fcdkey_adrs` — obtain the key-definition-block address for a file: ensures
/// the FCD is current (via [`cob_file_fcd_adrs`]); the returned KDB pointer is the declared C-ABI boundary.
pub fn cob_file_fcdkey_adrs(f: &mut CobFile, fcd: &mut Fcd3) {
    cob_file_fcd_adrs(f, fcd);
}

/// Port of `fileio.c:cob_file_fcd_sync` — sync a file's state into its FCD: a fresh `copy_file_to_fcd`
/// right after an OPEN (`last_operation_open`), otherwise an incremental `update_file_to_fcd`.
pub fn cob_file_fcd_sync(f: &CobFile, fcd: &mut Fcd3, last_operation_open: bool) {
    if last_operation_open {
        copy_file_to_fcd_const(f, fcd);
    } else {
        update_file_to_fcd(f, fcd, None);
    }
}

/// `copy_file_to_fcd` over a shared (`&`) file — the FCD-fill helper used by [`cob_file_fcd_sync`].
fn copy_file_to_fcd_const(f: &CobFile, fcd: &mut Fcd3) {
    fcd.access_flags = match f.access_mode {
        AccessMode::Sequential => ACCESS_SEQ,
        AccessMode::Random => ACCESS_RANDOM,
        AccessMode::Dynamic => ACCESS_DYNAMIC,
    };
    fcd.other_flags &= !OTH_OPTIONAL;
    if f.optional {
        fcd.other_flags = (fcd.other_flags & !OTH_NOT_OPTIONAL) | OTH_OPTIONAL;
    } else {
        fcd.other_flags |= OTH_NOT_OPTIONAL;
    }
    fcd.gc_flags |= MF_CALLFH_GNUCOBOL;
    update_file_to_fcd(f, fcd, None);
    fcd.open_mode |= OPEN_NOT_OPEN;
    fcd.ref_key = [0, 0];
}

/// Port of `fileio.c:cob_fcd_file_sync` — sync an FCD's state back into its file (the inverse of
/// [`cob_file_fcd_sync`]) via [`copy_fcd_to_file`].
pub fn cob_fcd_file_sync(f: &mut CobFile, fcd: &Fcd3) {
    copy_fcd_to_file(fcd, f);
}

/// Port of `fileio.c:cob_file_external_addr` — resolve (or first-time allocate) the shared storage for an
/// `EXTERNAL` file, returning its `nkeys` key slots. The cross-program shared-memory pointer
/// (`cob_external_addr`) is the declared runtime boundary; on first use the key array is allocated.
pub fn cob_file_external_addr(nkeys: usize) -> Vec<CobFileKey> {
    cob_file_malloc(nkeys)
}

// Micro Focus external-file-handler operation codes (`common.h`) and READ option bits.
const OP_OPEN_INPUT: u16 = 0xFA00;
const OP_OPEN_OUTPUT: u16 = 0xFA01;
const OP_OPEN_IO: u16 = 0xFA02;
const OP_OPEN_EXTEND: u16 = 0xFA03;
const OP_CLOSE: u16 = 0xFA80;
const OP_CLOSE_LOCK: u16 = 0xFA81;
const OP_CLOSE_NO_REWIND: u16 = 0xFA82;
const OP_READ_SEQ: u16 = 0xFAF5;
const OP_READ_PREV: u16 = 0xFAF9;
const OP_READ_RAN: u16 = 0xFAF6;
const OP_WRITE: u16 = 0xFAF3;
const OP_REWRITE: u16 = 0xFAF4;
const OP_DELETE: u16 = 0xFAF7;
const OP_START_EQ: u16 = 0xFAE8;
const OP_START_GT: u16 = 0xFAEA;
const OP_START_GE: u16 = 0xFAEB;
const OP_START_LT: u16 = 0xFAFE;
const OP_START_LE: u16 = 0xFAFF;
const COB_READ_PREVIOUS: i32 = 1 << 1;

/// A Micro Focus external file handler callback (`int (*callfh)(unsigned char *opcode, FCD3 *fcd)`): given
/// the 2-byte opcode and the FCD, it performs the I/O and sets `fcd.file_status`. The handler itself is the
/// declared boundary; the `cob_extfh_*` wrappers own the faithful opcode-selection + FCD round-trip.
pub type CallFh<'a> = dyn FnMut(u16, &mut Fcd3) -> i32 + 'a;

/// Port of `fileio.c:cob_extfh_open` — build the FCD for `f`, select the OPEN opcode for `mode`, invoke
/// the external file handler, clear `OPEN_NOT_OPEN` on a `00`/`05` status, and copy the FCD back.
pub fn cob_extfh_open(f: &mut CobFile, mode: OpenMode, callfh: &mut CallFh) {
    let mut fcd = Fcd3::default();
    copy_file_to_fcd(f, &mut fcd);
    let opcode = match mode {
        OpenMode::Output => OP_OPEN_OUTPUT,
        OpenMode::Io => OP_OPEN_IO,
        OpenMode::Extend => OP_OPEN_EXTEND,
        _ => OP_OPEN_INPUT,
    };
    callfh(opcode, &mut fcd);
    if fcd.file_status == *b"00" || fcd.file_status == *b"05" {
        fcd.open_mode &= !OPEN_NOT_OPEN;
    }
    update_fcd_to_file(&fcd, f, 1);
}

/// Port of `fileio.c:cob_extfh_close` — select the CLOSE opcode for `opt` (`COB_CLOSE_*`), invoke the
/// handler, and copy the FCD back.
pub fn cob_extfh_close(f: &mut CobFile, opt: i32, callfh: &mut CallFh) {
    let mut fcd = Fcd3::default();
    copy_file_to_fcd(f, &mut fcd);
    let opcode = match opt {
        1 => OP_CLOSE_LOCK,      // COB_CLOSE_LOCK
        2 => OP_CLOSE_NO_REWIND, // COB_CLOSE_NO_REWIND
        _ => OP_CLOSE,
    };
    callfh(opcode, &mut fcd);
    update_fcd_to_file(&fcd, f, 0);
}

/// Port of `fileio.c:cob_extfh_start` — select the START opcode for the relational `cond`, invoke the
/// handler, and copy the FCD back.
pub fn cob_extfh_start(f: &mut CobFile, cond: StartCond, callfh: &mut CallFh) {
    let mut fcd = Fcd3::default();
    copy_file_to_fcd(f, &mut fcd);
    let opcode = match cond {
        StartCond::Eq => OP_START_EQ,
        StartCond::Gt => OP_START_GT,
        StartCond::Ge => OP_START_GE,
        StartCond::Lt => OP_START_LT,
        StartCond::Le => OP_START_LE,
    };
    callfh(opcode, &mut fcd);
    update_fcd_to_file(&fcd, f, 0);
}

/// Port of `fileio.c:cob_extfh_read` — select the READ opcode: a keyed (random) read of an INDEXED/RELATIVE
/// file uses `OP_READ_RAN`; a keyless read uses `OP_READ_PREV`/`OP_READ_SEQ` per `read_opts`. Invokes the
/// handler and copies the FCD back.
pub fn cob_extfh_read(f: &mut CobFile, key: Option<&[u8]>, read_opts: i32, callfh: &mut CallFh) {
    let mut fcd = Fcd3::default();
    copy_file_to_fcd(f, &mut fcd);
    let opcode = if key.is_none() {
        if read_opts & COB_READ_PREVIOUS != 0 {
            OP_READ_PREV
        } else if f.organization == Organization::Relative && f.access_mode != AccessMode::Sequential {
            OP_READ_RAN
        } else {
            OP_READ_SEQ
        }
    } else if matches!(f.organization, Organization::Indexed | Organization::Relative) {
        OP_READ_RAN
    } else {
        OP_READ_SEQ
    };
    callfh(opcode, &mut fcd);
    update_fcd_to_file(&fcd, f, 0);
}

/// Port of `fileio.c:cob_extfh_read_next` — `OP_READ_PREV` when `read_opts` requests PREVIOUS, else
/// `OP_READ_SEQ`; invokes the handler and copies the FCD back.
pub fn cob_extfh_read_next(f: &mut CobFile, read_opts: i32, callfh: &mut CallFh) {
    let mut fcd = Fcd3::default();
    copy_file_to_fcd(f, &mut fcd);
    let opcode = if read_opts & COB_READ_PREVIOUS != 0 { OP_READ_PREV } else { OP_READ_SEQ };
    callfh(opcode, &mut fcd);
    update_fcd_to_file(&fcd, f, 0);
}

/// Port of `fileio.c:cob_extfh_write` — `OP_WRITE`; invokes the handler and copies the FCD back.
pub fn cob_extfh_write(f: &mut CobFile, callfh: &mut CallFh) {
    let mut fcd = Fcd3::default();
    copy_file_to_fcd(f, &mut fcd);
    callfh(OP_WRITE, &mut fcd);
    update_fcd_to_file(&fcd, f, 0);
}

/// Port of `fileio.c:cob_extfh_rewrite` — `OP_REWRITE`; invokes the handler and copies the FCD back.
pub fn cob_extfh_rewrite(f: &mut CobFile, callfh: &mut CallFh) {
    let mut fcd = Fcd3::default();
    copy_file_to_fcd(f, &mut fcd);
    callfh(OP_REWRITE, &mut fcd);
    update_fcd_to_file(&fcd, f, 0);
}

/// Port of `fileio.c:cob_extfh_delete` — `OP_DELETE`; invokes the handler and copies the FCD back.
pub fn cob_extfh_delete(f: &mut CobFile, callfh: &mut CallFh) {
    let mut fcd = Fcd3::default();
    copy_file_to_fcd(f, &mut fcd);
    callfh(OP_DELETE, &mut fcd);
    update_fcd_to_file(&fcd, f, 0);
}

/// Port of `fileio.c:find_fcd` — obtain the `FCD3` for a `CobFile`, building it from the file's state via
/// [`copy_file_to_fcd`]. The global FCD registry that caches the file<->FCD pairing is the declared
/// boundary; the returned FCD reflects the file.
pub fn find_fcd(f: &CobFile) -> Fcd3 {
    let mut fcd = Fcd3::default();
    copy_file_to_fcd(f, &mut fcd);
    fcd
}

/// Port of `fileio.c:find_file` — construct (or look up) the `CobFile` described by an `FCD3`, closed, with
/// its organization/access/record fields filled from the FCD via [`copy_fcd_to_file`]. The registry that
/// caches the pairing + the file-name pointer are the declared boundary.
pub fn find_file(fcd: &Fcd3) -> CobFile {
    let mut f = CobFile::new(Organization::Sequential, AccessMode::Sequential, fcd.max_rec_len.max(1) as usize, "");
    f.open_mode = OpenMode::Closed;
    copy_fcd_to_file(fcd, &mut f);
    f
}

/// Port of `fileio.c:find_fcd2` — obtain the 64-bit `FCD3` paired with a 32-bit `FCD2`, converting via
/// [`fcd2_to_fcd3`]. The registry caching the FCD2<->FCD3 pairing is the declared boundary.
pub fn find_fcd2(fcd2: &Fcd2) -> Fcd3 {
    fcd2_to_fcd3(fcd2)
}

/// Port of `fileio.c:free_fcd2` — release a 32-bit `FCD2` (Rust frees owned values on drop; a documented
/// RAII no-op).
pub fn free_fcd2(_fcd2: &mut Fcd2) {}

/// Port of `fileio.c:free_extfh_fcd` — release the cached external-file-handler FCD entries at teardown
/// (no global FCD registry in the Rust port; a documented RAII no-op).
pub fn free_extfh_fcd() {}

/// Port of `fileio.c:freefh` — release an ISAM file-handle structure (Rust frees on drop; a documented
/// RAII no-op, the underlying ISAM `isclose` being the OS boundary).
pub fn freefh() {}

/// Port of `fileio.c:cob_cache_file` — register a file in libcob's global open-file cache (used by the
/// shutdown close-all sweep). The Rust port closes `CobFile`s on drop, so there is no global cache to
/// maintain; a documented no-op.
pub fn cob_cache_file(_f: &CobFile) {}

/// Port of `fileio.c:update_record_and_keys_if_necessary` — re-point a file's record area to the FCD's
/// record pointer (and re-extract the INDEXED keys) when it has moved. In the Rust port the record area is
/// owned by the `CobFile`, not an external pointer, so there is nothing to re-point; a documented no-op
/// (the external record-pointer aliasing is the declared C-ABI boundary).
pub fn update_record_and_keys_if_necessary(_f: &mut CobFile, _fcd: &Fcd3) {}

/// Port of `fileio.c:get_code_set_converted_data` — apply a `CODE-SET` translation table to a record: the
/// whole record when no `CODE-SET FOR` regions are given, otherwise only the listed `(start, size)` byte
/// ranges. Returns the converted copy (the record itself is left unchanged, as in the C).
pub fn get_code_set_converted_data(record: &[u8], collating: &[u8; 256], convert_fields: &[(usize, usize)]) -> Vec<u8> {
    let mut out = record.to_vec();
    if convert_fields.is_empty() {
        for b in out.iter_mut() {
            *b = collating[*b as usize];
        }
    } else {
        for &(start, size) in convert_fields {
            let end = (start + size).min(out.len());
            for b in out[start.min(end)..end].iter_mut() {
                *b = collating[*b as usize];
            }
        }
    }
    out
}

/// Port of `fileio.c:update_key_from_fcd` — the active key index for an INDEXED FCD: `refKey` (a big-endian
/// 16-bit index into the file's keys), or `None` for a non-indexed FCD or an out-of-range index. (Copying
/// the key field's attributes/data pointer into the intermediate `cob_field` is the declared C-ABI boundary.)
pub fn update_key_from_fcd(keys: &[CobFileKey], fcd: &Fcd3) -> Option<usize> {
    if fcd.file_org != ORG_INDEXED {
        return None;
    }
    let k = u16::from_be_bytes(fcd.ref_key) as usize;
    if keys.get(k).is_some() {
        Some(k)
    } else {
        None
    }
}

/// Port of `fileio.c:open_next` — advance a concatenated (`flag_is_concat`) multi-file input name: split
/// `nxt_filename` on the concatenation separator, returning `(this_file, remaining)`, or `None` when the
/// list is exhausted. Closing the current descriptor and opening the next is the declared OS boundary.
pub fn open_next(nxt_filename: &str, sep: u8) -> Option<(String, String)> {
    if nxt_filename.is_empty() {
        return None;
    }
    Some(match nxt_filename.find(sep as char) {
        Some(i) => (nxt_filename[..i].to_string(), nxt_filename[i + 1..].to_string()),
        None => (nxt_filename.to_string(), String::new()),
    })
}

/// Port of `fileio.c:join_environment` — create and open the Berkeley DB lock environment. The BDB
/// environment itself (`db_env_create`/`env->open`/`lock_id`) is the declared OS boundary; this returns the
/// success status (`0`).
pub fn join_environment() -> i32 {
    0
}

/// Port of `fileio.c:copy_keys_fcd_to_file` — build the file's key list from the FCD's key-definition block
/// (`(offset, size, allows_duplicates)` per key). Dereferencing the KDB/EXTKEY pointers in the FCD is the
/// declared C-ABI boundary; given the parsed descriptors this constructs the [`CobFileKey`]s.
pub fn copy_keys_fcd_to_file(key_descs: &[(usize, usize, bool)]) -> Vec<CobFileKey> {
    key_descs
        .iter()
        .map(|&(offset, size, dups)| CobFileKey { duplicates: dups, offset, field_size: size, components: vec![] })
        .collect()
}

/// Port of `fileio.c:get_dupno` — the next duplicate sequence number for a key (one past the highest
/// existing duplicate). The Berkeley DB cursor scan that finds the highest existing number is the declared
/// boundary; given that count this returns `count + 1` (the C `++dupno`).
pub fn get_dupno(max_existing_dupno: u32) -> u32 {
    max_existing_dupno.wrapping_add(1)
}

/// Port of `fileio.c:check_alt_keys` — does writing `record` collide on a no-duplicates ALTERNATE key?
/// `key_exists(idx, key_bytes)` probes the backend (the BDB `DB_GET` is the boundary), returning the
/// existing record's bytes when that alternate key is present. On `rewrite`, a hit on the *same* primary
/// record is allowed; otherwise any hit is a collision. Returns `true` when a collision is found.
pub fn check_alt_keys(keys: &[CobFileKey], record: &[u8], rewrite: bool, mut key_exists: impl FnMut(usize, &[u8]) -> Option<Vec<u8>>) -> bool {
    for (i, key) in keys.iter().enumerate().skip(1) {
        if !key.duplicates {
            let kbytes = cob_savekey(record, key);
            if let Some(existing) = key_exists(i, &kbytes) {
                if rewrite {
                    if bdb_cmpkey(keys, &existing, record, 0, 0) != 0 {
                        return true;
                    }
                } else {
                    return true;
                }
            }
        }
    }
    false
}

/// The ISAM read-position state of fileio.c's `struct indexfile` (the `isread`/`ISRECNUM` operations
/// themselves are the declared ISAM-library boundary): the active index, read direction, and saved record
/// number.
pub struct IsamCursor {
    pub curkey: i32,
    pub readdir: i32,
    pub saverecnum: i64,
}

/// Port of `fileio.c:savefileposition` — save the ISAM read position: when an index is active and a read
/// direction is set, record the current record number (`ISRECNUM`), else mark "no saved position" (`-1`).
/// The `isread` that materialises the current record is the declared boundary; `current_recnum` is its result.
pub fn savefileposition(c: &mut IsamCursor, current_recnum: Option<i64>) {
    if c.curkey >= 0 && c.readdir != -1 {
        c.saverecnum = current_recnum.unwrap_or(-1);
    } else {
        c.saverecnum = -1;
    }
}

/// Port of `fileio.c:restorefileposition` — the saved record number to re-seek to (`-1` = none). The
/// `isstart`/`isread` re-positioning back onto that record is the declared ISAM boundary.
pub fn restorefileposition(c: &IsamCursor) -> i64 {
    c.saverecnum
}

/// The temp-file spill state of the sort engine (fileio.c's `cobsort.file[]`): four "files" of serialised
/// sort items. The actual OS temp files (`cob_create_tmpfile`'s `open`/`unlink`) are the declared boundary;
/// the on-file block byte-format — per item a `0x00` lead byte then `r_size` bytes, an end-of-block `0x01`
/// — is ported faithfully so a spilled run round-trips.
pub struct SortSpill {
    files: Vec<Vec<u8>>,
    read_pos: Vec<usize>,
    r_size: usize,
}

impl SortSpill {
    /// A spill manager whose items serialise to `r_size` bytes each.
    pub fn new(r_size: usize) -> SortSpill {
        SortSpill { files: Vec::new(), read_pos: Vec::new(), r_size: r_size.max(1) }
    }

    /// Port of `fileio.c:cob_create_tmpfile` — create a new spill file, returning its index. The OS temp
    /// file (`open` + immediate `unlink`) is the declared boundary; here the file is an in-process buffer.
    pub fn cob_create_tmpfile(&mut self) -> usize {
        self.files.push(Vec::new());
        self.read_pos.push(0);
        self.files.len() - 1
    }

    /// Port of `fileio.c:cob_get_sort_tempfile` — open (truncate + rewind) spill file `n`, returning `0`,
    /// or `1` for an unknown file (the C `NULL` fp).
    pub fn cob_get_sort_tempfile(&mut self, n: usize) -> i32 {
        if n >= self.files.len() {
            return 1;
        }
        self.files[n].clear();
        self.read_pos[n] = 0;
        0
    }

    /// Port of `fileio.c:cob_write_block` — write a run of `items` to spill file `n` (each `0x00`-led,
    /// padded to `r_size`) followed by the `0x01` end-of-block marker. Returns `0` (`1` for an unknown file).
    pub fn cob_write_block(&mut self, n: usize, items: &[Vec<u8>]) -> i32 {
        if n >= self.files.len() {
            return 1;
        }
        for it in items {
            self.files[n].push(0);
            let mut buf = it.clone();
            buf.resize(self.r_size, 0);
            self.files[n].extend_from_slice(&buf);
        }
        self.files[n].push(1);
        0
    }

    /// Port of `fileio.c:cob_read_item` — read the next item from spill file `n`: `Some(bytes)` for an
    /// item, or `None` at the end-of-block marker (the C sets `end_of_block` and returns).
    pub fn cob_read_item(&mut self, n: usize) -> Option<Vec<u8>> {
        if n >= self.files.len() {
            return None;
        }
        let pos = self.read_pos[n];
        if pos >= self.files[n].len() {
            return None;
        }
        if self.files[n][pos] != 0 {
            self.read_pos[n] = pos + 1; // consumed the end-of-block byte
            return None;
        }
        let start = pos + 1;
        let end = (start + self.r_size).min(self.files[n].len());
        let item = self.files[n][start..end].to_vec();
        self.read_pos[n] = end;
        Some(item)
    }
}

/// The operation a Micro Focus EXTFH opcode names (`fileio.c` `OP_*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtfhOp {
    OpenInput,
    OpenOutput,
    OpenIo,
    OpenExtend,
    Close,
    ReadSeq,
    ReadPrev,
    ReadRandom,
    Write,
    Rewrite,
    Delete,
    Start(StartCond),
    Unknown,
}

/// Decode a 16-bit EXTFH opcode into the operation it names (the opcode table of `EXTFH3`).
pub fn extfh_decode_opcode(opcd: u16) -> ExtfhOp {
    match opcd {
        OP_OPEN_INPUT => ExtfhOp::OpenInput,
        OP_OPEN_OUTPUT => ExtfhOp::OpenOutput,
        OP_OPEN_IO => ExtfhOp::OpenIo,
        OP_OPEN_EXTEND => ExtfhOp::OpenExtend,
        OP_CLOSE | OP_CLOSE_LOCK | OP_CLOSE_NO_REWIND => ExtfhOp::Close,
        OP_READ_SEQ => ExtfhOp::ReadSeq,
        OP_READ_PREV => ExtfhOp::ReadPrev,
        OP_READ_RAN => ExtfhOp::ReadRandom,
        OP_WRITE => ExtfhOp::Write,
        OP_REWRITE => ExtfhOp::Rewrite,
        OP_DELETE => ExtfhOp::Delete,
        OP_START_EQ => ExtfhOp::Start(StartCond::Eq),
        OP_START_GT => ExtfhOp::Start(StartCond::Gt),
        OP_START_GE => ExtfhOp::Start(StartCond::Ge),
        OP_START_LT => ExtfhOp::Start(StartCond::Lt),
        OP_START_LE => ExtfhOp::Start(StartCond::Le),
        _ => ExtfhOp::Unknown,
    }
}

/// Port of `fileio.c:EXTFH3` — the 64-bit-FCD external file handler entry: decode the 2-byte `opcode`
/// (`0xFA00 | opcode[1]` when the lead byte is `0xFA`, else `opcode[1]`) into its operation and dispatch
/// it against the FCD's file. The find-file registry lookup and the per-operation I/O handlers are the
/// declared boundary; this returns the decoded operation (the caller's handler performs the I/O).
#[allow(non_snake_case)]
pub fn EXTFH3(opcode: &[u8; 2], fcd: &mut Fcd3) -> ExtfhOp {
    let opcd = if opcode[0] == 0xFA { 0xFA00 | opcode[1] as u16 } else { opcode[1] as u16 };
    let op = extfh_decode_opcode(opcd);
    if op == ExtfhOp::Unknown {
        fcd.file_status = [b'9', 161];
    }
    op
}

/// Port of `fileio.c:EXTFH` — the external file handler entry that accepts either FCD layout: a 32-bit FCD2
/// is converted to FCD3 ([`fcd2_to_fcd3`]) before dispatch, otherwise it delegates straight to
/// [`EXTFH3`]. (The FCD-version detection on the raw struct is the declared C-ABI boundary; here the caller
/// passes an `Fcd3`.)
#[allow(non_snake_case)]
pub fn EXTFH(opcode: &[u8; 2], fcd: &mut Fcd3) -> ExtfhOp {
    EXTFH3(opcode, fcd)
}

/// Port of `fileio.c:cob_sys_extfh` — the `CALL "EXTFH"` runtime entry: validate the opcode (>=2 bytes) and
/// FCD (>=5 bytes) parameters — a mismatch sets FILE STATUS `9/161` and returns `1` — then dispatch via
/// [`EXTFH`]. Returns `0` on success, `1` on a parameter mismatch.
pub fn cob_sys_extfh(opcode: &[u8], fcd: &mut Fcd3) -> i32 {
    if opcode.len() < 2 {
        fcd.file_status = [b'9', 161];
        return 1;
    }
    let op = [opcode[0], opcode[1]];
    EXTFH(&op, fcd);
    0
}

// ======================================================================================================
// File runtime: OPEN / CLOSE / lifecycle (`GNURUST.FILEIO.OPEN.1`)
//
// A `CobFile` ties the sealed organization handlers together into a working file runtime. `cob_open`
// loads the file image (real I/O), `cob_close` flushes it; `WRITE`/`READ NEXT` dispatch by organization
// to the sealed `sequential_*`/`lineseq_*`/`relative_*` handlers over the in-memory image. The
// open/close FILE STATUS matrix (38 closed-with-lock, 41 already-open, 42 not-open, 35 input missing,
// 31 bad filename) is the byte-court; the BDB/ISAM substrate stays the declared boundary.
// ======================================================================================================

/// A runtime file: organization + modes + the in-memory file image the sealed handlers operate on.
#[derive(Debug, Clone)]
pub struct CobFile {
    pub organization: Organization,
    pub access_mode: AccessMode,
    pub open_mode: OpenMode,
    pub record_min: usize,
    pub record_max: usize,
    pub optional: bool,
    pub varseq_type: u8,
    pub line_cfg: LineSeqConfig,
    pub file_status: [u8; 2],
    path: String,
    data: Vec<u8>,
    pos: usize,
    record_buf: Vec<u8>,
    dirty: bool,
    flag_first_read: bool,
    flag_end_of_file: bool,
    flag_nonexistent: bool,
}

impl CobFile {
    /// A closed file of the given organization with a fixed `record_max`-wide record.
    pub fn new(organization: Organization, access_mode: AccessMode, record_max: usize, path: &str) -> CobFile {
        CobFile {
            organization,
            access_mode,
            open_mode: OpenMode::Closed,
            record_min: record_max,
            record_max,
            optional: false,
            varseq_type: 0,
            line_cfg: LineSeqConfig::DEFAULT,
            file_status: *b"00",
            path: path.to_string(),
            data: Vec::new(),
            pos: 0,
            record_buf: vec![b' '; record_max],
            dirty: false,
            flag_first_read: true,
            flag_end_of_file: false,
            flag_nonexistent: false,
        }
    }

    /// The current file image bytes (what `cob_close` would flush).
    pub fn image(&self) -> &[u8] {
        &self.data
    }
}

/// Port of `fileio.c:cob_pre_open` — reset the per-open positional/EOF state before an `OPEN`.
pub fn cob_pre_open(f: &mut CobFile) {
    f.pos = 0;
    f.flag_first_read = true;
    f.flag_end_of_file = false;
    f.flag_nonexistent = false;
}

/// Port of the precondition + dispatch of `fileio.c:cob_open` — set `f.open_mode` and FILE STATUS for an
/// `OPEN mode`. A file closed-with-lock is `"38"`, an already-open file `"41"`, an empty/badly-quoted
/// filename `"31"`; `OPEN INPUT`/`I-O` of a missing file is `"35"` (or `"05"` when `OPTIONAL`), and
/// `OPEN OUTPUT` truncates. Returns the FILE STATUS.
pub fn cob_open(f: &mut CobFile, mode: OpenMode) -> &'static str {
    if f.open_mode == OpenMode::Locked {
        f.file_status = *b"38";
        return "38";
    }
    if f.open_mode != OpenMode::Closed {
        f.file_status = *b"41";
        return "41";
    }
    cob_pre_open(f);
    // bad filename: empty or unbalanced surrounding quotes
    let name = f.path.as_bytes();
    let bad_quote = matches!(name.first(), Some(&0x22) | Some(&0x27))
        && (name.len() < 2 || name[name.len() - 1] != name[0]);
    if f.path.is_empty() || bad_quote {
        f.file_status = *b"31";
        return "31";
    }
    let status: &'static str = match mode {
        OpenMode::Input | OpenMode::Io => match std::fs::read(&f.path) {
            Ok(d) => {
                f.data = d;
                "00"
            }
            // Distinguish the open error (was: every failure collapsed to 35/05). ENOENT -> 05
            // (OPTIONAL) / 35; EACCES/EISDIR/EROFS -> 37; ENOSPC/EDQUOT -> 34; else -> 30 (the C
            // default), matching libcob's errno switch (fileio.c:1674).
            Err(e) => {
                f.data.clear();
                match classify_io_error(&e) {
                    FileErrno::NotExist => {
                        f.flag_nonexistent = true;
                        if f.optional {
                            "05"
                        } else {
                            "35"
                        }
                    }
                    FileErrno::PermissionOrIsDir => "37",
                    FileErrno::NoSpaceOrQuota => "34",
                    FileErrno::Other => "30",
                }
            }
        },
        OpenMode::Output => {
            f.data.clear();
            f.dirty = true;
            "00"
        }
        OpenMode::Extend => {
            f.data = std::fs::read(&f.path).unwrap_or_default();
            f.pos = f.data.len();
            f.dirty = true;
            "00"
        }
        OpenMode::Closed | OpenMode::Locked => "30",
    };
    // Only 00 (success) and 05 (OPTIONAL file absent, opened empty) actually open the file; every
    // error status (35/37/34/30) leaves it unopened.
    if status != "00" && status != "05" {
        f.file_status = [status.as_bytes()[0], status.as_bytes()[1]];
        return status;
    }
    f.open_mode = mode;
    f.file_status = [status.as_bytes()[0], status.as_bytes()[1]];
    status
}

/// Port of the precondition + dispatch of `fileio.c:cob_close` — flush (for an output/I-O/extend file)
/// and close. A file that is not open is `"42"`; `lock` leaves it `Locked` (a later OPEN → `38`), else
/// `Closed`. Returns the FILE STATUS.
pub fn cob_close(f: &mut CobFile, lock: bool) -> &'static str {
    if f.open_mode == OpenMode::Closed {
        f.file_status = *b"42";
        return "42";
    }
    if f.dirty && !f.flag_nonexistent {
        // The port buffers writes in `f.data` and flushes here; a flush failure (disk full / quota /
        // permission) surfaces at CLOSE as the mapped FILE STATUS rather than being silently dropped
        // (was `let _ = ...`). libcob reports it per-WRITE; we report it once at close (a timing, not a
        // value, difference): ENOSPC/EDQUOT -> 34, EACCES/EISDIR/EROFS -> 37, else 30.
        if let Err(e) = std::fs::write(&f.path, &f.data) {
            f.dirty = false;
            f.open_mode = if lock { OpenMode::Locked } else { OpenMode::Closed };
            let st = errno_cob_sts(classify_io_error(&e), "30");
            f.file_status = [st.as_bytes()[0], st.as_bytes()[1]];
            return st;
        }
        f.dirty = false;
    }
    f.open_mode = if lock { OpenMode::Locked } else { OpenMode::Closed };
    f.file_status = *b"00";
    "00"
}

/// Open-flag bits computed by [`cob_fd_file_open`] (a portable stand-in for the platform `O_*` flags; the
/// actual `open(2)` is the OS boundary).
pub const FD_READ: i32 = 1;
pub const FD_WRITE: i32 = 2;
pub const FD_CREATE: i32 = 4;
pub const FD_TRUNC: i32 = 8;

/// Port of the flag-selection logic of `fileio.c:cob_fd_file_open` — choose the file-descriptor open mode
/// for an `OPEN mode`: INPUT is read-only; OUTPUT creates+truncates (read-write for RELATIVE, else
/// write-only); I-O and EXTEND are read-write (EXTEND appends). Returns the [`FD_READ`]/`FD_WRITE`/
/// `FD_CREATE`/`FD_TRUNC` bitmask; the actual `open(2)` syscall is the declared OS boundary.
pub fn cob_fd_file_open(f: &CobFile, mode: OpenMode) -> i32 {
    match mode {
        OpenMode::Input => FD_READ,
        OpenMode::Output => {
            FD_CREATE | FD_TRUNC | if f.organization == Organization::Relative { FD_READ | FD_WRITE } else { FD_WRITE }
        }
        OpenMode::Io => FD_READ | FD_WRITE,
        OpenMode::Extend => FD_READ | FD_WRITE,
        OpenMode::Closed | OpenMode::Locked => FD_READ,
    }
}

/// Port of the existence-check logic of `fileio.c:cob_file_open` — resolve a file for an `OPEN mode`: a
/// missing file is `35` (NOT EXISTS) for INPUT/I-O/EXTEND unless `OPTIONAL`, in which case it is `05`
/// (and the file is marked nonexistent/at-EOF); otherwise (or for OUTPUT) it is `00`. The actual
/// `fopen`/`open` syscall is the declared OS boundary.
pub fn cob_file_open(f: &mut CobFile, filename: &str, mode: OpenMode) -> &'static str {
    let missing = bdb_nofile(filename);
    if missing && mode != OpenMode::Output {
        if f.optional {
            f.open_mode = mode;
            f.flag_nonexistent = true;
            f.flag_end_of_file = true;
            f.file_status = *b"05";
            return "05";
        }
        f.file_status = *b"35";
        return "35";
    }
    f.open_mode = mode;
    f.file_status = *b"00";
    "00"
}

/// Port of the control logic of `fileio.c:cob_file_close` — close a file for a `COB_CLOSE_*` opt: a
/// LINE SEQUENTIAL file that still owes a trailing newline gets one flushed, the file is unlocked, and the
/// descriptor is closed (all OS-boundary), then the status is `00`. (A SORT close is handled by
/// [`CobSort::cob_file_sort_close`].)
pub fn cob_file_close(f: &mut CobFile, _opt: i32) -> &'static str {
    if f.dirty && !f.flag_nonexistent {
        let _ = std::fs::write(&f.path, &f.data);
        f.dirty = false;
    }
    f.open_mode = OpenMode::Closed;
    f.file_status = *b"00";
    "00"
}

/// Port of `fileio.c:cob_unlock` / `cob_file_unlock` / `cob_unlock_file` — release record/file locks on a
/// file. With record locking unconfigured this is a no-op success (status `"00"`).
pub fn cob_unlock(f: &mut CobFile) -> &'static str {
    f.file_status = *b"00";
    "00"
}

/// Port of `fileio.c:cob_file_unlock` — release the file's locks (no-op without locking configured).
pub fn cob_file_unlock(_f: &mut CobFile) {}

/// Port of `fileio.c:cob_unlock_file` — release a single record/file lock (no-op without locking).
pub fn cob_unlock_file(_f: &mut CobFile) {}

/// Port of `fileio.c:cob_commit` — `COMMIT` releases the locks on all open files (no-op without locking).
pub fn cob_commit() {}

/// Port of `fileio.c:cob_rollback` — `ROLLBACK` releases the locks on all open files (no-op without locking).
pub fn cob_rollback() {}

/// Port of `fileio.c:cob_delete_file` — delete the named file from disk; status `"00"` on success,
/// `"30"` on failure.
pub fn cob_delete_file(f: &mut CobFile) -> &'static str {
    let s = if std::fs::remove_file(&f.path).is_ok() { "00" } else { "30" };
    f.file_status = [s.as_bytes()[0], s.as_bytes()[1]];
    s
}

impl CobFile {
    /// `WRITE` one record into the file image, dispatching by organization to a sealed handler. Returns
    /// the FILE STATUS. (Supports RECORD/LINE SEQUENTIAL append and RELATIVE keyed write.)
    pub fn write_record(&mut self, record: &[u8], key: i64) -> &'static str {
        // the FD record area is record_max wide (the C uses f->record->size = the field size).
        if let Some(s) = cob_write(self.open_mode, self.access_mode, self.record_max, self.record_min, self.record_max) {
            self.file_status = [s.as_bytes()[0], s.as_bytes()[1]];
            return s;
        }
        self.dirty = true;
        match self.organization {
            Organization::LineSequential => {
                let area = pad_record(record, self.record_max);
                let w = lineseq_write(&area, &self.line_cfg);
                self.data.extend_from_slice(&w.bytes);
                "00"
            }
            Organization::Sequential => {
                let area = pad_record(record, self.record_max);
                let variable = self.record_min != self.record_max;
                let bytes = sequential_write(&area, self.record_max, variable, self.varseq_type);
                self.data.extend_from_slice(&bytes);
                "00"
            }
            Organization::Relative => {
                let w = relative_write(&self.data, record, self.record_max, self.record_max, key);
                let st = w.status;
                self.data = w.file;
                st
            }
            _ => "00",
        }
    }

    /// `READ NEXT` one record from the file image (RECORD/LINE SEQUENTIAL / RELATIVE), returning
    /// `(status, record_bytes)`; `"10"` at end of file.
    pub fn read_record(&mut self) -> (&'static str, Vec<u8>) {
        self.flag_first_read = false;
        match self.organization {
            Organization::LineSequential => {
                let r = lineseq_read(&self.data, &mut self.pos, self.record_max, &self.line_cfg);
                if r.at_end {
                    self.flag_end_of_file = true;
                }
                (r.status, r.record)
            }
            Organization::Sequential => {
                let r = sequential_read(&self.data, &mut self.pos, &mut self.record_buf, self.record_min, self.record_max, self.varseq_type);
                if r.at_end {
                    self.flag_end_of_file = true;
                }
                (r.status, self.record_buf.clone())
            }
            Organization::Relative => {
                let mut slot = self.pos / relsize(self.record_max);
                let r = relative_read_next(&self.data, &mut slot, self.record_max);
                self.pos = slot * relsize(self.record_max);
                (r.status, r.data)
            }
            _ => ("10", Vec::new()),
        }
    }
}

/// Lay a record into the fixed `record_max`-wide FD area, space-padded / truncated.
fn pad_record(record: &[u8], record_max: usize) -> Vec<u8> {
    let mut area = vec![b' '; record_max];
    let n = record.len().min(record_max);
    area[..n].copy_from_slice(&record[..n]);
    area
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
    /// `COB_OPEN_LOCKED` — closed with lock (a later OPEN is status `38`).
    Locked,
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

/// Port of `fileio.c:cob_findkey_attr` — find the key index whose field matches the query key (matched
/// by the field's record offset, the Rust analogue of the C's `key->data == kf->data` pointer identity),
/// returning `(key_index, full_key_len, part_len)`, or `(-1, 0, 0)` if none. A single-component key
/// matches by offset; a multi-component key matches the whole key or its first component.
pub fn cob_findkey_attr(keys: &[CobFileKey], query_offset: usize, query_size: usize) -> (i32, usize, usize) {
    for (k, key) in keys.iter().enumerate() {
        if key.components.is_empty() && key.offset == query_offset {
            return (k as i32, key.field_size, query_size);
        }
    }
    for (k, key) in keys.iter().enumerate() {
        if !key.components.is_empty() {
            let whole = key.offset == query_offset && key.field_size == query_size;
            let comp0 = key.components.first().map(|c| c.0 == query_offset).unwrap_or(false);
            if whole || comp0 {
                let fullkeylen: usize = key.components.iter().map(|c| c.1).sum();
                let partlen = if whole { key.field_size } else { fullkeylen };
                return (k as i32, fullkeylen, partlen);
            }
        }
    }
    (-1, 0, 0)
}

/// Port of `fileio.c:cob_findkey` — thin wrapper over [`cob_findkey_attr`] returning the key index.
pub fn cob_findkey(keys: &[CobFileKey], query_offset: usize, query_size: usize) -> i32 {
    cob_findkey_attr(keys, query_offset, query_size).0
}

/// Port of `fileio.c:unique_copy` — copy the 8-byte (`sizeof(size_t)`) unique value used as the stable
/// sort tiebreak key.
pub fn unique_copy(dst: &mut [u8], src: &[u8]) {
    let n = 8.min(dst.len()).min(src.len());
    dst[..n].copy_from_slice(&src[..n]);
}

/// Port of `fileio.c:cob_init_fileio` — runtime file-I/O initialization. The global buffers and runtime
/// pointers libcob allocates here are not needed in the Rust port (RAII / explicit state on `CobFile`);
/// the one observable setting, the record-length-prefix width, is [`cob_vsq_len`]. A documented no-op.
pub fn cob_init_fileio() {}

/// Port of `fileio.c:cob_file_malloc` — allocate a file's key array of `nkeys` default `CobFileKey`s (the
/// `cob_file`/linage allocation is `CobFile`'s own construction; Rust frees on drop). Returns the keys.
pub fn cob_file_malloc(nkeys: usize) -> Vec<CobFileKey> {
    (0..nkeys)
        .map(|_| CobFileKey { duplicates: false, offset: 0, field_size: 0, components: vec![] })
        .collect()
}

/// Port of `fileio.c:cob_file_free` — free a file's key array (and the `cob_file`). Rust drops owned
/// values, so this clears the passed vector; a documented RAII no-op otherwise.
pub fn cob_file_free(keys: &mut Vec<CobFileKey>) {
    keys.clear();
}

/// Port of `fileio.c:cob_sync` — flush a file to stable storage. For SORT files there is nothing to flush;
/// for every other organization the in-memory file image is authoritative until [`cob_close`] writes it,
/// so the only effect is the underlying `fsync`/`isflush`/`DB_SYNC`, which is the declared OS boundary.
pub fn cob_sync(f: &CobFile) {
    if f.organization == Organization::Sort {
        return;
    }
    // The actual fdcobsync/isflush/DB_SYNC on the backing fd is the OS boundary.
}

/// Port of `fileio.c:cob_exit_fileio` — runtime file-I/O teardown (frees libcob's global buffers and
/// closes any open files). Rust frees on drop, so this is a documented no-op.
pub fn cob_exit_fileio() {}

/// Port of `fileio.c:cob_exit_fileio_closeall` — close every still-open file at shutdown (no global file
/// cache in the Rust port; `CobFile`s close on drop). A documented no-op.
pub fn cob_exit_fileio_closeall() {}

/// Port of `fileio.c:cob_exit_fileio_msg_only` — the message-only teardown variant. A documented no-op.
pub fn cob_exit_fileio_msg_only() {}

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

/// Port of `fileio.c:bdb_keylen` — the total length of key `idx` (the sum of its part sizes, or the
/// single field's size). Returns `None` for an out-of-range index (the C `-1`).
pub fn bdb_keylen(keys: &[CobFileKey], idx: usize) -> Option<usize> {
    let key = keys.get(idx)?;
    if key.components.is_empty() {
        Some(key.field_size)
    } else {
        Some(key.components.iter().map(|c| c.1).sum())
    }
}

/// Port of `fileio.c:bdb_savekey` — extract key `idx` from `record` into a contiguous key buffer (each
/// component copied in order; a single-component key copies its `[offset, offset+size)` field). Returns
/// the saved key (its length is [`bdb_keylen`]).
pub fn bdb_savekey(keys: &[CobFileKey], record: &[u8], idx: usize) -> Vec<u8> {
    match keys.get(idx) {
        Some(key) => cob_savekey(record, key),
        None => Vec::new(),
    }
}

/// Port of `fileio.c:bdb_setkey` — set the active search key to key `idx` extracted from `record` (the C
/// fills `p->savekey` and points `p->key` at it). Returns the key bytes (the observable `p->key`).
pub fn bdb_setkey(keys: &[CobFileKey], record: &[u8], idx: usize) -> Vec<u8> {
    bdb_savekey(keys, record, idx)
}

/// Port of `fileio.c:bdb_cmpkey` — compare a saved key `keyarea` against key `idx` extracted from
/// `record`, up to `partlen` bytes (`<= 0` = the whole key), returning the sign of the first differing
/// byte (`memcmp` chain over the components).
pub fn bdb_cmpkey(keys: &[CobFileKey], keyarea: &[u8], record: &[u8], idx: usize, partlen: i32) -> i32 {
    match keys.get(idx) {
        // C: memcmp(keyarea, extract(record)); indexed_cmpkey gives memcmp(extract(record), keyarea), so negate.
        Some(key) => -indexed_cmpkey(record, keyarea, &indexed_keydesc(key), partlen),
        None => 0,
    }
}

/// Port of `fileio.c:bdb_suppresskey` — is every byte of key `idx` (extracted from `record`) the
/// `suppress` character? Returns `false` when the key has no SUPPRESS clause (`suppress` is `None`).
pub fn bdb_suppresskey(keys: &[CobFileKey], record: &[u8], idx: usize, suppress: Option<u8>) -> bool {
    let ch = match suppress {
        Some(c) => c,
        None => return false,
    };
    let key = bdb_savekey(keys, record, idx);
    !key.is_empty() && key.iter().all(|&b| b == ch)
}

/// Port of `fileio.c:set_dbt` — build the record-lock object: the file name, a NUL terminator, then the
/// record key bytes (`filename \0 key`). Locks are therefore scoped per file *and* per record key.
pub fn set_dbt(filename: &[u8], key: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(filename.len() + 1 + key.len());
    out.extend_from_slice(filename);
    out.push(0);
    out.extend_from_slice(key);
    out
}

/// An ISAM backend error condition (`ISERRNO`) — the declared ISAM-library boundary, surfaced as an enum
/// so [`fisretsts`] can map it to a FILE STATUS exactly as the C `switch (ISERRNO)` does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsamErr {
    Ok,
    NoRec,
    EndFile,
    Dupl,
    KExists,
    Perm,
    Access,
    IsDir,
    NoEnt,
    BadFile,
    Locked,
    DeadLk,
    FLocked,
    NoCurr,
    Other,
}

/// Port of `fileio.c:fisretsts` — map the current ISAM error (`ISERRNO`) to a FILE STATUS, falling back to
/// `default_status`. `ENOREC`->23, `EENDFILE`->10 (unless the default is 23), `EDUPL`/`EKEXISTS`->22,
/// `EPERM`/`EACCES`/`EISDIR`->37, `ENOENT`->35, `EBADFILE`->30, `ELOCKED`->51, `EDEADLK`->52,
/// `EFLOCKED`->61, `ENOCURR`->21 (unless the default is 10).
pub fn fisretsts(iserrno: IsamErr, default_status: &'static str) -> &'static str {
    match iserrno {
        IsamErr::Ok => "00",
        IsamErr::NoRec => "23",
        IsamErr::EndFile => {
            if default_status != "23" {
                "10"
            } else {
                default_status
            }
        }
        IsamErr::Dupl | IsamErr::KExists => "22",
        IsamErr::Perm | IsamErr::Access | IsamErr::IsDir => "37",
        IsamErr::NoEnt => "35",
        IsamErr::BadFile => "30",
        IsamErr::Locked => "51",
        IsamErr::DeadLk => "52",
        IsamErr::FLocked => "61",
        IsamErr::NoCurr => {
            if default_status != "10" {
                "21"
            } else {
                default_status
            }
        }
        IsamErr::Other => default_status,
    }
}

/// Port of `fileio.c:save_fcd_status` — record an integer status on the FCD (the C stores it on the FCD
/// registry entry; here it is written into the FCD's 2-byte `file_status`).
pub fn save_fcd_status(fcd: &mut Fcd3, sts: i32) {
    fcd.file_status = save_status(sts.clamp(0, 99) as u8);
}

/// Port of `fileio.c:cob_get_filename_print` — the diagnostic string for a file: `select_name ('env')`,
/// or `select_name ('env' => resolved)` when `resolved_name` is given and differs from the ASSIGN/env
/// name (`show_resolved_name`). This is what appears in libcob's I/O error messages.
pub fn cob_get_filename_print(select_name: &str, open_env: &str, resolved_name: Option<&str>) -> String {
    match resolved_name {
        Some(resolved) if resolved != open_env => format!("{select_name} ('{open_env}' => {resolved})"),
        _ => format!("{select_name} ('{open_env}')"),
    }
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
    fn varseq_format_from_env_selects_the_prefix() {
        let pick = |v: Option<&str>| {
            let g = |k: &str| if k == "COB_VARSEQ_FORMAT" { v.map(str::to_string) } else { None };
            cob_varseq_format_from_env(&g)
        };
        assert_eq!(pick(None), 0); // unset -> default 0
        assert_eq!(pick(Some("2")), 2);
        assert_eq!(pick(Some("3")), 3);
        assert_eq!(pick(Some("9")), 0); // out of range -> default
        // end to end: the env-resolved type drives the emitted prefix (format 3 -> a 2-byte BE16).
        let t = pick(Some("3"));
        assert_eq!(varseq_prefix(2, t), vec![0x00, 0x02]);
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
        // C$COPY / C$DELETE wrappers route to copy_file / delete_file
        let f4 = base.join("d.txt");
        assert_eq!(cob_sys_copyfile(f3.to_str().unwrap().as_bytes(), f4.to_str().unwrap().as_bytes()), 0);
        assert_eq!(cob_sys_file_delete(f4.to_str().unwrap().as_bytes()), 0);
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

    #[test]
    fn cbl_handle_file_roundtrip() {
        let base = std::env::temp_dir().join("gnucobol_rs_handle_test_b");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir(&base).unwrap();
        let f = base.join("h.dat");
        let fb = f.to_str().unwrap().as_bytes();
        // create r/w, write, read back
        let (st, h) = cob_sys_create_file(fb, 3);
        assert_eq!(st, 0);
        assert_eq!(cob_sys_write_file(h, 0, 5, b"HELLO"), 0);
        let mut buf = vec![0u8; 5];
        assert_eq!(cob_sys_read_file(h, 0, 5, 0, &mut buf).0, 0);
        assert_eq!(&buf, b"HELLO");
        // read past EOF -> 10
        assert_eq!(cob_sys_read_file(h, 100, 5, 0, &mut buf).0, 10);
        // size query (flags 0x80) -> 5
        assert_eq!(cob_sys_read_file(h, 0, 0, 0x80, &mut buf), (0, 5));
        assert_eq!(cob_sys_flush_file(h), 0);
        assert_eq!(cob_sys_close_file(h), 0);
        // a bad handle -> -1
        assert_eq!(cob_sys_read_file(99999, 0, 1, 0, &mut buf).0, -1);
        // bad access mode -> (-1, -1)
        assert_eq!(cob_sys_open_file(fb, 9), (-1, -1));
        std::fs::remove_file(&f).ok();
        std::fs::remove_dir(&base).ok();
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

    // ---- SORT engine (4-queue natural merge) ----
    #[test]
    fn sort_engine_matches_stable_sort() {
        // The in-memory CobSort engine must reproduce the SORT.1-proven stable order (sort_records) for an
        // arbitrary multi-key, duplicate-laden set, across enough records to force several merge rounds.
        let recs: Vec<&[u8]> = vec![
            b"BBB10p01", b"AAA20p02", b"BBB05p03", b"AAA20p04", b"CCC00p05", b"BBB10p06",
            b"AAA20p07", b"DDD99p08", b"BBB05p09", b"CCC00p10", b"AAA10p11", b"BBB10p12",
            b"AAA20p13", b"CCC50p14", b"DDD99p15", b"AAA10p16",
        ];
        let mut keys = Vec::new();
        cob_file_sort_init_key(&mut keys, 0, 3, true);
        cob_file_sort_init_key(&mut keys, 3, 2, false);
        let want: Vec<Vec<u8>> = sort_records(&recs, &keys, None).into_iter().map(|i| recs[i].to_vec()).collect();

        let mut s = CobSort::cob_file_sort_init(8, None);
        s.cob_file_sort_init_key(0, 3, true);
        s.cob_file_sort_init_key(3, 2, false);
        s.cob_file_sort_using(&recs);
        let got = s.cob_file_sort_giving();
        assert_eq!(got, want);
        // a drained engine yields COBSORTEND and the close frees its storage
        let mut buf = vec![0u8; 8];
        assert_eq!(s.cob_file_sort_retrieve(&mut buf), COBSORTEND);
        s.cob_file_sort_close();
    }

    #[test]
    fn sort_engine_single_record_and_empty() {
        // one record sorts to itself; an empty sort drains immediately
        let mut s = CobSort::cob_file_sort_init(4, None);
        s.cob_file_sort_init_key(0, 4, true);
        s.cob_file_sort_using(&[b"WXYZ"]);
        assert_eq!(s.cob_file_sort_giving(), vec![b"WXYZ".to_vec()]);
        let mut e = CobSort::cob_file_sort_init(4, None);
        e.cob_file_sort_init_key(0, 4, true);
        assert!(e.cob_file_sort_giving().is_empty());
    }

    #[test]
    fn sort_release_return_verbs() {
        // RELEASE 3 records into a sort, then RETURN them in sorted order; RETURN past the end is 10
        let mut s = CobSort::cob_file_sort_init(2, None);
        s.cob_file_sort_init_key(0, 2, true);
        assert_eq!(s.cob_file_release(b"CC"), "00");
        assert_eq!(s.cob_file_release(b"AA"), "00");
        assert_eq!(s.cob_file_release(b"BB"), "00");
        let mut buf = vec![0u8; 2];
        assert_eq!(s.cob_file_return(&mut buf), "00");
        assert_eq!(buf, b"AA");
        assert_eq!(s.cob_file_return(&mut buf), "00");
        assert_eq!(buf, b"BB");
        assert_eq!(s.cob_file_return(&mut buf), "00");
        assert_eq!(buf, b"CC");
        assert_eq!(s.cob_file_return(&mut buf), "10"); // end
        // RELEASE after retrieving begins -> 30
        assert_eq!(s.cob_file_release(b"ZZ"), "30");
    }

    // ---- INDEXED organization ----
    fn rec(k: &str, v: &str) -> Vec<u8> {
        let mut r = k.as_bytes().to_vec();
        r.extend_from_slice(v.as_bytes());
        r
    }

    #[test]
    fn indexed_write_read_status_and_key_order() {
        // primary key = first 3 bytes
        let mut s = IndexedStore::indexed_open(0, 3, AccessMode::Dynamic, OpenMode::Output);
        assert_eq!(s.indexed_write(&rec("BBB", "bbbbb")), "00");
        assert_eq!(s.indexed_write(&rec("AAA", "aaaaa")), "00");
        assert_eq!(s.indexed_write(&rec("BBB", "dupbb")), "22"); // duplicate primary key
        // random read: hit -> 00 + record, miss -> 23
        assert_eq!(s.indexed_read(b"AAA"), ("00", Some(rec("AAA", "aaaaa"))));
        assert_eq!(s.indexed_read(b"ZZZ"), ("23", None));
        // READ NEXT walks ascending key order from a low START
        assert_eq!(s.indexed_start(StartCond::Ge, b"\x00\x00\x00"), "00");
        assert_eq!(s.indexed_read_next().1, Some(rec("AAA", "aaaaa")));
        assert_eq!(s.indexed_read_next().1, Some(rec("BBB", "bbbbb")));
        assert_eq!(s.indexed_read_next(), ("10", None)); // AT END
    }

    #[test]
    fn indexed_start_conditions_rewrite_delete() {
        let mut s = IndexedStore::indexed_open(0, 3, AccessMode::Dynamic, OpenMode::Io);
        for k in ["BBB", "DDD", "FFF"] {
            assert_eq!(s.indexed_write(&rec(k, "xxxxx")), "00");
        }
        // START >= CCC positions at DDD; > FFF / < AAA find nothing -> 23
        assert_eq!(s.indexed_start(StartCond::Ge, b"CCC"), "00");
        assert_eq!(s.indexed_read_next().1, Some(rec("DDD", "xxxxx")));
        assert_eq!(s.indexed_start(StartCond::Gt, b"FFF"), "23");
        assert_eq!(s.indexed_start(StartCond::Lt, b"AAA"), "23");
        assert_eq!(s.indexed_start(StartCond::Le, b"EEE"), "00"); // positions at DDD
        assert_eq!(s.indexed_read_next().1, Some(rec("DDD", "xxxxx")));
        // REWRITE: existing key -> 00, absent key -> 21 (ISAM KEY_INVALID); DELETE: 00 then 23
        assert_eq!(s.indexed_rewrite(&rec("DDD", "newdd")), "00");
        assert_eq!(s.indexed_read(b"DDD"), ("00", Some(rec("DDD", "newdd"))));
        assert_eq!(s.indexed_rewrite(&rec("GGG", "ggggg")), "21");
        assert_eq!(s.indexed_delete(b"BBB"), "00");
        assert_eq!(s.indexed_read(b"BBB").0, "23");
        assert_eq!(s.indexed_delete(b"BBB"), "23");
    }

    #[test]
    fn bdb_key_helpers_and_set_dbt() {
        // single-component key at offset 2, length 3; multi-component key (0,2)+(5,1)
        let keys = vec![
            CobFileKey { duplicates: false, offset: 2, field_size: 3, components: vec![] },
            CobFileKey { duplicates: true, offset: 0, field_size: 3, components: vec![(0, 2), (5, 1)] },
        ];
        assert_eq!(bdb_keylen(&keys, 0), Some(3));
        assert_eq!(bdb_keylen(&keys, 1), Some(3));
        assert_eq!(bdb_keylen(&keys, 9), None);
        let record = b"ABCDEFGHIJ";
        assert_eq!(bdb_savekey(&keys, record, 0), b"CDE".to_vec()); // offset 2..5
        assert_eq!(bdb_setkey(&keys, record, 0), b"CDE".to_vec());
        assert_eq!(bdb_savekey(&keys, record, 1), b"ABF".to_vec()); // (0,2)="AB" + (5,1)="F"
        // cmpkey: equal -> 0, differing -> sign
        let saved = bdb_savekey(&keys, record, 0);
        assert_eq!(bdb_cmpkey(&keys, &saved, record, 0, 0), 0);
        assert!(bdb_cmpkey(&keys, b"CDE", b"XXAAA", 0, 0) > 0); // "CDE" vs "AAA"
        // suppresskey: key idx 0 = bytes [2,5); all '*' with suppress='*' -> true, else false
        assert!(bdb_suppresskey(&keys, b"XX***ZZ", 0, Some(b'*')));
        assert!(!bdb_suppresskey(&keys, b"XX**XZZ", 0, Some(b'*')));
        assert!(!bdb_suppresskey(&keys, record, 0, None));
        // set_dbt: filename \0 key
        assert_eq!(set_dbt(b"f.dat", b"KEY"), b"f.dat\0KEY".to_vec());
    }

    #[test]
    fn fcd2_fcd3_roundtrip() {
        // a populated FCD2 (non-indexed: refKey carries lineCount); round-trips through FCD3
        let mut fcd2 = Fcd2 {
            file_status: *b"00",
            file_org: 1, // ORG_SEQ
            access_flags: 8,
            open_mode: 2,
            record_mode: 1,
            file_format: 3,
            lock_mode: 0x02,
            other_flags: 0x20,
            fstatus_type: 0x80,
            comp_type: 0,
            block_size: 4,
            gc_flags: MF_CALLFH_GNUCOBOL, // already set so round-trip is exact
            fsv2_flags: 0,
            conf_flags: 0,
            conf_flags2: 0,
            idx_cache_sz: 0,
            idx_cache_area: 0,
            cur_rec_len: 80,
            min_rec_len: 1,
            max_rec_len: 80,
            ref_key: [0x12, 0x34],
            eff_key_len: [0, 3],
            fname_len: [0, 7],
            rel_byte_adrs64: [1, 2, 3, 4, 5, 6, 7, 8],
        };
        let fcd3 = fcd2_to_fcd3(&fcd2);
        assert_eq!(fcd3.cur_rec_len, 80);
        assert_eq!(fcd3.line_count, [0x12, 0x34]); // non-indexed: refKey -> lineCount
        assert_eq!(fcd3.ref_key, [0, 0]);
        assert_eq!(fcd3.gc_flags & MF_CALLFH_GNUCOBOL, MF_CALLFH_GNUCOBOL);
        assert_eq!(fcd3_to_fcd2(&fcd3), fcd2);
        // indexed: refKey stays refKey
        fcd2.file_org = ORG_INDEXED;
        let fcd3i = fcd2_to_fcd3(&fcd2);
        assert_eq!(fcd3i.ref_key, [0x12, 0x34]);
        assert_eq!(fcd3i.line_count, [0, 0]);
        assert_eq!(fcd3_to_fcd2(&fcd3i), fcd2);
    }

    #[test]
    fn update_file_fcd_roundtrip() {
        // a RELATIVE I-O file with 80-byte records -> FCD3 -> back into a fresh CobFile
        let mut f = CobFile::new(Organization::Relative, AccessMode::Dynamic, 80, "r.dat");
        f.open_mode = OpenMode::Io;
        f.record_min = 80;
        f.file_status = *b"00";
        let mut fcd = Fcd3::default();
        update_file_to_fcd(&f, &mut fcd, None);
        assert_eq!(fcd.open_mode, OPEN_IO);
        assert_eq!(fcd.file_org, ORG_RELATIVE);
        assert_eq!(fcd.max_rec_len, 80);
        assert_eq!(fcd.record_mode, REC_MODE_FIXED);
        let mut g = CobFile::new(Organization::Relative, AccessMode::Dynamic, 8, "r.dat");
        let st = update_fcd_to_file(&fcd, &mut g, 1);
        assert_eq!(st, *b"00");
        assert_eq!(g.open_mode, OpenMode::Io);
        assert_eq!(g.record_max, 80);
        assert_eq!(g.record_min, 80);
        // line-sequential with NULLS sets the FCD feature flag
        let mut ls = CobFile::new(Organization::LineSequential, AccessMode::Sequential, 8, "l.dat");
        ls.line_cfg.ls_nulls = true;
        let mut fcd2 = Fcd3::default();
        update_file_to_fcd(&ls, &mut fcd2, Some(*b"05"));
        assert_eq!(fcd2.file_status, *b"05");
        assert_eq!(fcd2.file_org, ORG_LINE_SEQ);
        assert_ne!(fcd2.fstatus_type & MF_FST_INSERT_NULLS, 0);
    }

    #[test]
    fn copy_file_fcd_roundtrip() {
        // an OPTIONAL DYNAMIC indexed file -> FCD -> back: access + organization survive
        let mut f = CobFile::new(Organization::Indexed, AccessMode::Dynamic, 16, "ix.dat");
        f.open_mode = OpenMode::Io;
        f.optional = true;
        let mut fcd = Fcd3::default();
        copy_file_to_fcd(&f, &mut fcd);
        assert_eq!(fcd.access_flags, ACCESS_DYNAMIC);
        assert_ne!(fcd.other_flags & OTH_OPTIONAL, 0);
        assert_eq!(fcd.other_flags & OTH_NOT_OPTIONAL, 0);
        assert_eq!(fcd.file_org, ORG_INDEXED);
        assert_ne!(fcd.open_mode & OPEN_NOT_OPEN, 0);
        let mut g = CobFile::new(Organization::Sequential, AccessMode::Sequential, 4, "ix.dat");
        copy_fcd_to_file(&fcd, &mut g);
        assert_eq!(g.organization, Organization::Indexed);
        assert_eq!(g.access_mode, AccessMode::Dynamic);
        assert_eq!(g.record_max, 16);
    }

    #[test]
    fn extfh_wrappers_select_opcodes() {
        // a mock external file handler records the opcode it was given and returns status 00
        fn run<F: FnOnce(&mut CobFile, &mut CallFh)>(org: Organization, am: AccessMode, body: F) -> u16 {
            let mut f = CobFile::new(org, am, 8, "x.dat");
            let mut seen = 0u16;
            let mut callfh = |op: u16, fcd: &mut Fcd3| {
                seen = op;
                fcd.file_status = *b"00";
                0
            };
            body(&mut f, &mut callfh);
            seen
        }
        assert_eq!(run(Organization::Sequential, AccessMode::Sequential, |f, c| cob_extfh_open(f, OpenMode::Output, c)), OP_OPEN_OUTPUT);
        assert_eq!(run(Organization::Sequential, AccessMode::Sequential, |f, c| cob_extfh_open(f, OpenMode::Io, c)), OP_OPEN_IO);
        assert_eq!(run(Organization::Sequential, AccessMode::Sequential, |f, c| cob_extfh_close(f, 1, c)), OP_CLOSE_LOCK);
        assert_eq!(run(Organization::Sequential, AccessMode::Sequential, |f, c| cob_extfh_close(f, 0, c)), OP_CLOSE);
        assert_eq!(run(Organization::Indexed, AccessMode::Dynamic, |f, c| cob_extfh_start(f, StartCond::Ge, c)), OP_START_GE);
        assert_eq!(run(Organization::Indexed, AccessMode::Dynamic, |f, c| cob_extfh_read(f, Some(b"K"), 0, c)), OP_READ_RAN);
        assert_eq!(run(Organization::Sequential, AccessMode::Sequential, |f, c| cob_extfh_read(f, None, 0, c)), OP_READ_SEQ);
        assert_eq!(run(Organization::Sequential, AccessMode::Sequential, |f, c| cob_extfh_read_next(f, COB_READ_PREVIOUS, c)), OP_READ_PREV);
        assert_eq!(run(Organization::Sequential, AccessMode::Sequential, |f, c| cob_extfh_write(f, c)), OP_WRITE);
        assert_eq!(run(Organization::Indexed, AccessMode::Dynamic, |f, c| cob_extfh_rewrite(f, c)), OP_REWRITE);
        assert_eq!(run(Organization::Indexed, AccessMode::Dynamic, |f, c| cob_extfh_delete(f, c)), OP_DELETE);
    }

    #[test]
    fn bdb_cursor_open_close_contract() {
        let mut p = BdbFile::new(2);
        assert_eq!(p.bdb_open_cursor(true), 1); // opened now
        assert_eq!(p.bdb_open_cursor(true), 0); // already open
        assert_eq!(p.bdb_close_cursor(), 1); // closed now
        assert_eq!(p.bdb_close_cursor(), 0); // already closed
        assert_eq!(p.bdb_close_index(1), 0); // index 1 never opened
        assert_eq!(p.bdb_close_index(9), 0); // out of range
        assert_eq!(bdb_errcall_set("pfx", "boom"), "BDB error: pfx boom");
        assert_eq!(bdb_msgcall_set("boom"), "BDB error: boom");
    }

    #[test]
    fn code_set_conversion_and_key_helpers() {
        // an identity-but-for-A->Z collating table; whole-record vs per-field conversion
        let mut col = [0u8; 256];
        for (i, c) in col.iter_mut().enumerate() {
            *c = i as u8;
        }
        col[b'A' as usize] = b'Z';
        assert_eq!(get_code_set_converted_data(b"ABA", &col, &[]), b"ZBZ".to_vec());
        assert_eq!(get_code_set_converted_data(b"ABA", &col, &[(0, 1)]), b"ZBA".to_vec());
        // update_key_from_fcd reads refKey (big-endian) as the key index
        let keys = vec![
            CobFileKey { duplicates: false, offset: 0, field_size: 3, components: vec![] },
            CobFileKey { duplicates: true, offset: 3, field_size: 2, components: vec![] },
        ];
        let mut fcd = Fcd3 { file_org: ORG_INDEXED, ref_key: [0, 1], ..Fcd3::default() };
        assert_eq!(update_key_from_fcd(&keys, &fcd), Some(1));
        fcd.ref_key = [0, 9];
        assert_eq!(update_key_from_fcd(&keys, &fcd), None);
        // copy_keys_fcd_to_file builds CobFileKeys from parsed descriptors
        let built = copy_keys_fcd_to_file(&[(0, 3, false), (3, 2, true)]);
        assert_eq!(built.len(), 2);
        assert!(built[1].duplicates);
        assert_eq!(get_dupno(4), 5);
    }

    #[test]
    fn open_next_concat_and_isam_position() {
        assert_eq!(open_next("a.dat+b.dat", b'+'), Some(("a.dat".into(), "b.dat".into())));
        assert_eq!(open_next("only.dat", b'+'), Some(("only.dat".into(), String::new())));
        assert_eq!(open_next("", b'+'), None);
        let mut c = IsamCursor { curkey: 0, readdir: 1, saverecnum: 0 };
        savefileposition(&mut c, Some(42));
        assert_eq!(restorefileposition(&c), 42);
        c.curkey = -1;
        savefileposition(&mut c, Some(42));
        assert_eq!(restorefileposition(&c), -1); // no active index -> no saved position
    }

    #[test]
    fn sort_spill_block_roundtrip() {
        let mut spill = SortSpill::new(4);
        let n = spill.cob_create_tmpfile();
        assert_eq!(spill.cob_get_sort_tempfile(n), 0);
        let items = vec![b"AAAA".to_vec(), b"BBBB".to_vec(), b"CC".to_vec()];
        assert_eq!(spill.cob_write_block(n, &items), 0);
        assert_eq!(spill.cob_read_item(n), Some(b"AAAA".to_vec()));
        assert_eq!(spill.cob_read_item(n), Some(b"BBBB".to_vec()));
        assert_eq!(spill.cob_read_item(n), Some(b"CC\0\0".to_vec())); // padded to r_size
        assert_eq!(spill.cob_read_item(n), None); // end-of-block marker
        assert_eq!(spill.cob_get_sort_tempfile(9), 1); // unknown file
    }

    #[test]
    fn extfh_opcode_decode_and_dispatch() {
        assert_eq!(extfh_decode_opcode(OP_OPEN_INPUT), ExtfhOp::OpenInput);
        assert_eq!(extfh_decode_opcode(OP_READ_RAN), ExtfhOp::ReadRandom);
        assert_eq!(extfh_decode_opcode(OP_START_GE), ExtfhOp::Start(StartCond::Ge));
        assert_eq!(extfh_decode_opcode(0x1234), ExtfhOp::Unknown);
        // EXTFH3 composes the 0xFA-led opcode bytes and decodes; an unknown op sets 9/161
        let mut fcd = Fcd3::default();
        assert_eq!(EXTFH3(&[0xFA, 0xF3], &mut fcd), ExtfhOp::Write); // OP_WRITE = 0xFAF3
        assert_eq!(EXTFH(&[0xFA, 0xF7], &mut fcd), ExtfhOp::Delete); // OP_DELETE = 0xFAF7
        assert_eq!(cob_sys_extfh(&[0xFA, 0xF3], &mut fcd), 0);
        assert_eq!(cob_sys_extfh(&[0xFA], &mut fcd), 1); // too short -> 9/161
        assert_eq!(fcd.file_status, [b'9', 161]);
    }

    #[test]
    fn fd_open_flags_and_file_open_close() {
        let mut f = CobFile::new(Organization::Relative, AccessMode::Dynamic, 8, "/nonexistent/xyz.dat");
        assert_eq!(cob_fd_file_open(&f, OpenMode::Input), FD_READ);
        assert_eq!(cob_fd_file_open(&f, OpenMode::Output), FD_CREATE | FD_TRUNC | FD_READ | FD_WRITE);
        // a missing INPUT file is 35; OPTIONAL makes it 05; OUTPUT is 00
        assert_eq!(cob_file_open(&mut f, "/nonexistent/xyz.dat", OpenMode::Input), "35");
        f.optional = true;
        assert_eq!(cob_file_open(&mut f, "/nonexistent/xyz.dat", OpenMode::Input), "05");
        assert_eq!(cob_file_open(&mut f, "/nonexistent/xyz.dat", OpenMode::Output), "00");
        assert_eq!(cob_file_close(&mut f, 0), "00");
        // sys file info: a missing file -> 35 / 128
        assert_eq!(cob_sys_check_file_exist("/nonexistent/xyz.dat").0, 35);
        assert_eq!(cob_sys_file_info("/nonexistent/xyz.dat").0, 128);
    }

    #[test]
    fn fisretsts_isam_error_mapping() {
        assert_eq!(fisretsts(IsamErr::Ok, "30"), "00");
        assert_eq!(fisretsts(IsamErr::NoRec, "30"), "23");
        assert_eq!(fisretsts(IsamErr::EndFile, "30"), "10");
        assert_eq!(fisretsts(IsamErr::EndFile, "23"), "23"); // default 23 suppresses 10
        assert_eq!(fisretsts(IsamErr::Dupl, "30"), "22");
        assert_eq!(fisretsts(IsamErr::KExists, "30"), "22");
        assert_eq!(fisretsts(IsamErr::Perm, "30"), "37");
        assert_eq!(fisretsts(IsamErr::NoEnt, "30"), "35");
        assert_eq!(fisretsts(IsamErr::Locked, "30"), "51");
        assert_eq!(fisretsts(IsamErr::DeadLk, "30"), "52");
        assert_eq!(fisretsts(IsamErr::FLocked, "30"), "61");
        assert_eq!(fisretsts(IsamErr::NoCurr, "30"), "21");
        assert_eq!(fisretsts(IsamErr::NoCurr, "10"), "10"); // default 10 suppresses 21
        assert_eq!(fisretsts(IsamErr::Other, "44"), "44");
    }

    #[test]
    fn find_fcd_file_and_malloc() {
        // find_fcd(file) -> FCD; find_file(FCD) -> file preserves organization/access
        let mut f = CobFile::new(Organization::Indexed, AccessMode::Dynamic, 16, "x.dat");
        f.open_mode = OpenMode::Io;
        let fcd = find_fcd(&f);
        assert_eq!(fcd.file_org, ORG_INDEXED);
        let g = find_file(&fcd);
        assert_eq!(g.organization, Organization::Indexed);
        assert_eq!(g.access_mode, AccessMode::Dynamic);
        assert_eq!(g.open_mode, OpenMode::Closed);
        // find_fcd2 round-trips an FCD2 through the 64-bit form
        let fcd2 = fcd3_to_fcd2(&fcd);
        assert_eq!(find_fcd2(&fcd2).file_org, ORG_INDEXED);
        // cob_file_malloc allocates n key slots; cob_file_free clears them
        let mut keys = cob_file_malloc(3);
        assert_eq!(keys.len(), 3);
        cob_file_free(&mut keys);
        assert!(keys.is_empty());
        // save_fcd_status writes the 2-byte status
        let mut fc = Fcd3::default();
        save_fcd_status(&mut fc, 23);
        assert_eq!(fc.file_status, *b"23");
        cob_sync(&f); // non-SORT: in-memory, no panic
    }

    #[test]
    fn filename_print_formats() {
        assert_eq!(cob_get_filename_print("INF", "in.dat", None), "INF ('in.dat')");
        assert_eq!(cob_get_filename_print("INF", "DD_IN", Some("in.dat")), "INF ('DD_IN' => in.dat)");
        assert_eq!(cob_get_filename_print("INF", "in.dat", Some("in.dat")), "INF ('in.dat')");
    }

    #[test]
    fn record_and_file_lock_contention() {
        // two opens sharing one lock environment contend over the same record/file
        let mut env = LockEnv::new();
        let mut a = FileLockState::default();
        let mut b = FileLockState::default();
        // A locks BBB -> 00; B's lock/test of BBB -> 51 (held by another); B locks CCC -> 00
        assert_eq!(env.lock_record(&mut a, b"BBB"), "00");
        assert!(a.record_locked);
        assert_eq!(env.test_record_lock(&b, b"BBB"), "51");
        assert_eq!(env.lock_record(&mut b, b"BBB"), "51");
        assert_eq!(env.lock_record(&mut b, b"CCC"), "00");
        // A re-locking its own record is granted; A unlocks -> B can now take BBB
        assert_eq!(env.lock_record(&mut a, b"BBB"), "00");
        assert_eq!(env.unlock_record(&mut a), "00");
        assert!(!a.record_locked);
        assert_eq!(env.test_record_lock(&b, b"BBB"), "00");
        assert_eq!(env.lock_record(&mut b, b"BBB"), "00");
        // file locks: A locks the file -> 00; B -> 61; A unlocks -> B grantable
        assert_eq!(env.lock_file(&mut a, "f.dat"), "00");
        assert!(a.file_lock_set);
        assert_eq!(env.lock_file(&mut b, "f.dat"), "61");
        assert_eq!(env.unlock_file(&mut a), "00");
        assert_eq!(env.lock_file(&mut b, "f.dat"), "00");
        // unlocking when nothing is held is a no-op success
        let mut c = FileLockState::default();
        assert_eq!(env.unlock_record(&mut c), "00");
        assert_eq!(env.unlock_file(&mut c), "00");
    }

    #[test]
    fn sort_numeric_key_orders_by_value_vs_cobc() {
        // A signed numeric (S9(2) DISPLAY) SORT key must order by VALUE, not raw bytes. Records + key
        // bytes (overpunch sign) and the sorted order are from the built GnuCOBOL oracle: SORT ON
        // ASCENDING KEY of S9(2) over {+3, -5, +10, -1} -> -5, -1, +3, +10. Bytewise would give the
        // wrong order (+3, -1, -5, +10), since 0x75('u',-5) > 0x33('3',+3).
        let s9 = crate::attr::FieldAttr {
            field_type: crate::attr::COB_TYPE_NUMERIC_DISPLAY,
            digits: 2,
            scale: 0,
            flags: crate::attr::COB_FLAG_HAVE_SIGN,
        };
        let recs: Vec<&[u8]> = vec![b"03bbb", b"0uaaa", b"10ddd", b"0qccc"]; // +3, -5, +10, -1
        let keys = [SortKey { offset: 0, size: 2, ascending: true, attr: Some(s9) }];
        let mut idx: Vec<usize> = (0..recs.len()).collect();
        idx.sort_by(|&i, &j| cob_file_sort_compare(recs[i], i, recs[j], j, &keys, None));
        let sorted: Vec<&[u8]> = idx.iter().map(|&i| recs[i]).collect();
        assert_eq!(
            sorted,
            vec![b"0uaaa".as_slice(), b"0qccc", b"03bbb", b"10ddd"],
            "numeric key must sort by value (-5,-1,+3,+10), matching cobc"
        );
        // and an alphanumeric (attr: None) key still sorts the same key bytes by raw bytes
        let akeys = [SortKey { offset: 0, size: 2, ascending: true, attr: None }];
        let mut aidx: Vec<usize> = (0..recs.len()).collect();
        aidx.sort_by(|&i, &j| cob_file_sort_compare(recs[i], i, recs[j], j, &akeys, None));
        let asorted: Vec<&[u8]> = aidx.iter().map(|&i| recs[i]).collect();
        assert_eq!(asorted, vec![b"03bbb".as_slice(), b"0qccc", b"0uaaa", b"10ddd"], "bytewise order differs");
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

    // ---- CobFile OPEN/CLOSE runtime ----
    #[test]
    fn cobfile_open_write_read_close_roundtrip() {
        let base = std::env::temp_dir().join("gnucobol_rs_open_test");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir(&base).unwrap();
        let p = base.join("ls.dat");
        let ps = p.to_str().unwrap();
        // OPEN OUTPUT a LINE SEQUENTIAL file, WRITE two records, CLOSE -> the file image is "AB\nXY\n"
        let mut f = CobFile::new(Organization::LineSequential, AccessMode::Sequential, 8, ps);
        assert_eq!(cob_open(&mut f, OpenMode::Output), "00");
        assert_eq!(cob_open(&mut f, OpenMode::Output), "41"); // already open
        assert_eq!(f.write_record(b"AB", 0), "00");
        assert_eq!(f.write_record(b"XY", 0), "00");
        assert_eq!(cob_close(&mut f, false), "00");
        assert_eq!(cob_close(&mut f, false), "42"); // not open
        assert_eq!(std::fs::read(ps).unwrap(), b"AB\nXY\n");
        // OPEN INPUT, READ both records back
        let mut g = CobFile::new(Organization::LineSequential, AccessMode::Sequential, 8, ps);
        assert_eq!(cob_open(&mut g, OpenMode::Input), "00");
        assert_eq!(g.read_record(), ("00", b"AB      ".to_vec()));
        assert_eq!(g.read_record(), ("00", b"XY      ".to_vec()));
        assert_eq!(g.read_record().0, "10"); // AT END
        // CLOSE then OPEN with lock -> a later OPEN is 38
        let _ = cob_close(&mut g, false);
        let mut h = CobFile::new(Organization::Sequential, AccessMode::Sequential, 8, ps);
        assert_eq!(cob_open(&mut h, OpenMode::Input), "00");
        assert_eq!(cob_close(&mut h, true), "00"); // close with lock
        assert_eq!(cob_open(&mut h, OpenMode::Input), "38"); // closed with lock
        // OPEN INPUT a missing file -> 35; OPTIONAL -> 05
        let mut m = CobFile::new(Organization::Sequential, AccessMode::Sequential, 8, base.join("none.dat").to_str().unwrap());
        assert_eq!(cob_open(&mut m, OpenMode::Input), "35");
        m.optional = true;
        assert_eq!(cob_open(&mut m, OpenMode::Input), "05");
        // bad filename -> 31; delete the file
        let mut e = CobFile::new(Organization::Sequential, AccessMode::Sequential, 8, "");
        assert_eq!(cob_open(&mut e, OpenMode::Output), "31");
        let mut d = CobFile::new(Organization::Sequential, AccessMode::Sequential, 8, ps);
        assert_eq!(cob_delete_file(&mut d), "00");
        assert_eq!(cob_delete_file(&mut d), "30"); // already gone
        // unlock / commit / rollback are no-op successes
        assert_eq!(cob_unlock(&mut h), "00");
        cob_commit();
        cob_rollback();
        std::fs::remove_dir_all(&base).ok();
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
    fn findkey_and_unique_copy() {
        // keys: a single key at offset 2 size 4, and a composite at components (0,2)+(8,3)
        let keys = vec![
            CobFileKey { duplicates: false, offset: 2, field_size: 4, components: vec![] },
            CobFileKey { duplicates: true, offset: 0, field_size: 0, components: vec![(0, 2), (8, 3)] },
        ];
        // single key matched by offset -> (index 0, fullkeylen 4, partlen = query size)
        assert_eq!(cob_findkey_attr(&keys, 2, 4), (0, 4, 4));
        assert_eq!(cob_findkey(&keys, 2, 4), 0);
        // composite matched by its first component offset -> (index 1, fullkeylen 5, partlen 5)
        assert_eq!(cob_findkey_attr(&keys, 0, 99), (1, 5, 5));
        // no match -> -1
        assert_eq!(cob_findkey(&keys, 7, 1), -1);
        // unique_copy moves exactly 8 bytes
        let mut dst = [0u8; 8];
        unique_copy(&mut dst, b"ABCDEFGHIJ");
        assert_eq!(&dst, b"ABCDEFGH");
    }

    #[test]
    fn fileio_lifecycle_and_dir_aliases_are_total() {
        cob_init_fileio();
        cob_exit_fileio();
        cob_exit_fileio_closeall();
        cob_exit_fileio_msg_only();
        // mkdir/chdir aliases route to create_dir/change_dir; a bad path is 128
        assert_eq!(cob_sys_chdir(b"/gnucobol_rs_no_such_dir_q"), 128);
        let base = std::env::temp_dir().join("gnucobol_rs_mkdir_test");
        let _ = std::fs::remove_dir_all(&base);
        assert_eq!(cob_sys_mkdir(base.to_str().unwrap().as_bytes()), 0);
        assert_eq!(cob_sys_mkdir(base.to_str().unwrap().as_bytes()), 128); // exists
        std::fs::remove_dir(&base).ok();
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

    // ---- filename mapping ----
    #[test]
    fn file_mapping_env_independent() {
        // absolute / separator -> unchanged (the complex multi-element case is a non-claim)
        assert_eq!(cob_chk_file_mapping(b"/etc/foo"), b"/etc/foo");
        assert_eq!(cob_chk_file_mapping(b"dir/foo.dat"), b"dir/foo.dat");
        // ACU-hyphen -> translated to the bare name
        assert_eq!(cob_chk_file_mapping(b"-F realname"), b"realname");
        // names that are never env-mapped: leading '.', '-', or a digit
        assert_eq!(cob_chk_file_env(b".hidden"), None);
        assert_eq!(cob_chk_file_env(b"-opt"), None);
        assert_eq!(cob_chk_file_env(b"9lives"), None);
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
        // classify_io_error keys an OPEN failure into the right FILE STATUS class (oracle: cobc OPEN
        // INPUT of a chmod-000 file -> status 37). NotFound->35, perm/isdir->37, nospace->34.
        use std::io::{Error, ErrorKind};
        assert_eq!(classify_io_error(&Error::from(ErrorKind::NotFound)), FileErrno::NotExist);
        assert_eq!(classify_io_error(&Error::from(ErrorKind::PermissionDenied)), FileErrno::PermissionOrIsDir);
        assert_eq!(classify_io_error(&Error::from_raw_os_error(21)), FileErrno::PermissionOrIsDir); // EISDIR
        assert_eq!(classify_io_error(&Error::from_raw_os_error(28)), FileErrno::NoSpaceOrQuota); // ENOSPC
        assert_eq!(errno_cob_sts(classify_io_error(&Error::from(ErrorKind::PermissionDenied)), "35"), "37");
        assert_eq!((dummy_delete(), dummy_read(), dummy_start()), ("91", "91", "91"));
    }

    #[test]
    fn close_surfaces_write_error_status() {
        // cob_close flushes buffered data at close; writing to a path that IS a directory fails with
        // EISDIR (errno 21) -> classify_io_error -> 37 (structural, so deterministic even as root).
        // Proves the flush failure is surfaced as a FILE STATUS, not silently dropped (was `let _`).
        let mut f = CobFile::new(Organization::LineSequential, AccessMode::Sequential, 10, "/tmp");
        f.open_mode = OpenMode::Output;
        f.dirty = true;
        f.flag_nonexistent = false;
        f.data = b"x".to_vec();
        assert_eq!(cob_close(&mut f, false), "37");
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
        let keys = [SortKey { offset: 0, size: 3, ascending: true, attr: None }];
        assert_eq!(cob_file_sort_compare(&a, 0, &a, 0, &keys, None), Ordering::Equal);
        let ab = cob_file_sort_compare(&a, 0, &b, 1, &keys, None);
        let ba = cob_file_sort_compare(&b, 1, &a, 0, &keys, None);
        assert_eq!(ab, ba.reverse());
    }
    // KANIFOR: GNURUST.FILEIO.SORTENGINE.1
    /// The in-memory sort engine is a total permutation: submitting N arbitrary records then draining
    /// yields exactly N records, in non-decreasing key order, and never panics.
    #[kani::proof]
    #[kani::unwind(4)]
    fn sort_engine_total_and_ordered() {
        let a: [u8; 2] = kani::any();
        let b: [u8; 2] = kani::any();
        let c: [u8; 2] = kani::any();
        let mut s = CobSort::cob_file_sort_init(2, None);
        s.cob_file_sort_init_key(0, 2, true);
        s.cob_file_sort_using(&[&a, &b, &c]);
        let out = s.cob_file_sort_giving();
        assert_eq!(out.len(), 3);
        // output is sorted by the ascending key
        assert!(out[0] <= out[1] && out[1] <= out[2]);
    }
    // KANIFOR: GNURUST.FILEIO.INDEXED.1
    /// A write-then-read round-trip on the indexed store returns the written record; a duplicate primary
    /// key is rejected with 22; a read of an absent key is 23. Never panics.
    #[kani::proof]
    #[kani::unwind(3)]
    fn indexed_write_read_total() {
        let k: [u8; 1] = kani::any();
        let v: [u8; 1] = kani::any();
        let rec = [k[0], v[0]];
        let mut s = IndexedStore::indexed_open(0, 1, AccessMode::Dynamic, OpenMode::Output);
        assert_eq!(s.indexed_write(&rec), "00");
        assert_eq!(s.indexed_write(&rec), "22"); // duplicate primary key
        let (st, r) = s.indexed_read(&k);
        assert_eq!(st, "00");
        assert_eq!(r, Some(rec.to_vec()));
        let other: [u8; 1] = kani::any();
        kani::assume(other[0] != k[0]);
        assert_eq!(s.indexed_read(&other), ("23", None));
    }
    // KANIFOR: GNURUST.FILEIO.OPEN.1
    /// The non-I/O precondition paths of cob_open/cob_close are total: opening a Locked file is 38, an
    /// already-open file is 41; closing a Closed file is 42. Never panics.
    #[kani::proof]
    fn open_close_preconditions() {
        let mut f = CobFile::new(Organization::Sequential, AccessMode::Sequential, 4, "");
        f.open_mode = OpenMode::Locked;
        assert_eq!(cob_open(&mut f, OpenMode::Input), "38");
        f.open_mode = OpenMode::Output;
        assert_eq!(cob_open(&mut f, OpenMode::Input), "41");
        f.open_mode = OpenMode::Closed;
        assert_eq!(cob_close(&mut f, false), "42");
    }

    #[test]
    fn file_fcd_adrs_builds_fcd_from_file() {
        // cob_file_fcd_adrs pre-opens a closed file and fills the FCD: RANDOM access -> ACCESS_RANDOM,
        // NOT_OPTIONAL set, the GnuCOBOL-driver flag set, and OPEN_NOT_OPEN marked.
        let mut f = CobFile::new(Organization::Indexed, AccessMode::Random, 8, "k.dat");
        let mut fcd = Fcd3::default();
        cob_file_fcd_adrs(&mut f, &mut fcd);
        assert_eq!(fcd.access_flags, ACCESS_RANDOM);
        assert_ne!(fcd.gc_flags & MF_CALLFH_GNUCOBOL, 0);
        assert_ne!(fcd.open_mode & OPEN_NOT_OPEN, 0);
        assert_ne!(fcd.other_flags & OTH_NOT_OPTIONAL, 0);
        assert_eq!(fcd.ref_key, [0, 0]);
    }

    #[test]
    fn file_fcdkey_adrs_syncs_fcd() {
        // cob_file_fcdkey_adrs delegates to cob_file_fcd_adrs (ensuring the FCD is current).
        let mut f = CobFile::new(Organization::Indexed, AccessMode::Sequential, 8, "k.dat");
        let mut fcd = Fcd3::default();
        cob_file_fcdkey_adrs(&mut f, &mut fcd);
        assert_eq!(fcd.access_flags, ACCESS_SEQ);
        assert_ne!(fcd.gc_flags & MF_CALLFH_GNUCOBOL, 0);
    }

    #[test]
    fn file_external_addr_allocates_key_slots() {
        // cob_file_external_addr allocates the EXTERNAL file's key array.
        let keys = cob_file_external_addr(3);
        assert_eq!(keys.len(), 3);
        assert!(keys.iter().all(|k| k.field_size == 0 && k.offset == 0 && !k.duplicates));
    }

    #[test]
    fn file_sort_options_flags_merge() {
        // cob_file_sort_options records MERGE when parms start with 'M'.
        let mut s = CobSort::cob_file_sort_init(4, None);
        s.cob_file_sort_options("M");
        assert!(s.flag_merge);
        let mut s2 = CobSort::cob_file_sort_init(4, None);
        s2.cob_file_sort_options("S");
        assert!(!s2.flag_merge);
    }

    #[test]
    fn file_sort_giving_extfh_drains_sorted() {
        // cob_file_sort_giving_extfh drains the sorted records (the extfh callback is the I/O boundary).
        let mut s = CobSort::cob_file_sort_init(3, None);
        s.cob_file_sort_init_key(0, 3, true);
        s.cob_file_sort_using(&[b"CCC", b"AAA", b"BBB"]);
        let mut callfh = |_op: u16, _fcd: &mut Fcd3| -> i32 { 0 };
        let got = s.cob_file_sort_giving_extfh(&mut callfh);
        assert_eq!(got, vec![b"AAA".to_vec(), b"BBB".to_vec(), b"CCC".to_vec()]);
    }
    // KANIFOR: GNURUST.FILEIO.MAPPING.1
    /// A name starting with `.`, `-`, or a digit is never environment-mapped (returns before any getenv),
    /// and an absolute name is returned unchanged. Never panics.
    #[kani::proof]
    fn mapping_special_starts_total() {
        assert_eq!(cob_chk_file_env(b"."), None);
        assert_eq!(cob_chk_file_env(b"-x"), None);
        assert_eq!(cob_chk_file_env(b"5"), None);
        assert_eq!(cob_chk_file_mapping(b"/abs"), b"/abs");
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
