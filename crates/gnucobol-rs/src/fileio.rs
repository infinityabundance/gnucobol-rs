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
}

impl LineSeqConfig {
    /// libcob's out-of-the-box settings: `COB_LS_VALIDATE=1`, `COB_LS_FIXED=0`, `COB_LS_NULLS=0`.
    pub const DEFAULT: LineSeqConfig = LineSeqConfig { ls_fixed: false, ls_nulls: false, ls_validate: true };
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

#[cfg(test)]
mod tests {
    use super::*;

    const PLAIN: LineSeqConfig = LineSeqConfig { ls_fixed: false, ls_nulls: false, ls_validate: false };
    const NULLS: LineSeqConfig = LineSeqConfig { ls_fixed: false, ls_nulls: true, ls_validate: false };
    const FIXED: LineSeqConfig = LineSeqConfig { ls_fixed: true, ls_nulls: false, ls_validate: false };
    const FIXED_NULLS: LineSeqConfig = LineSeqConfig { ls_fixed: true, ls_nulls: true, ls_validate: false };

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
}
