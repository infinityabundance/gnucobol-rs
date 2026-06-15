//! Port of the **DUMP / debug-log formatting surface** of `libcob/common.c`.
//!
//! common.c emits, on an abnormal end (abend) or when a module was compiled with `-fdump`, a textual
//! dump of the active module chain and its fields to a dump file, and -- when built with `COB_DEBUG_LOG`
//! -- a developer trace log. Those functions are I/O leaves: they `fprintf`/`fputs` into a `FILE *`
//! resolved from the runtime settings. A file-write leaf is a documented boundary; the faithful 1:1
//! port here separates the **pure byte formatting** (the dump-line / log-line bytes for given inputs,
//! and the `od`-style hex+ASCII dump of a buffer) from the actual `FILE *` write.
//!
//! Functions ported (each cites its C origin):
//! - [`cob_debug_dump`] -- the hex+ASCII memory dump (`cob_debug_dump`), the most oracle-verifiable one;
//!   exact column layout, hex groups, ASCII gutter, and the `repeatWord`-based run-compression.
//! - [`cob_debug_logit`] -- decide whether a debug-log line is allowed (`cob_debug_logit`).
//! - [`cob_debug_logger`] -- format a debug-log line header + body (`cob_debug_logger`).
//! - [`cob_debug_open`] -- parse `COB_DEBUG_LOG` into module list / level / time-flag / log filename
//!   (`cob_debug_open`, pure parse; the file-open is the boundary).
//! - [`cob_get_dump_file`] -- resolve which destination a dump goes to (`cob_get_dump_file`).
//! - [`create_dumpfile`] -- build the `gcore` command line (`create_dumpfile`, pure name resolution;
//!   the `system()` call is the boundary).
//! - [`cob_dump_module_header`] -- the per-module "Dump Program-Id ..." banner line (`cob_dump_module`).
//!
//! The dump-file / debug-log runtime state is modelled as the explicit [`DebugLogState`] and
//! [`DumpFileState`] structs (never statics).

#![forbid(unsafe_code)]

/// `COB_SMALL_BUFF` from `common.h` -- the small string buffer size used for the logfile name.
pub const COB_SMALL_BUFF: usize = 1024;
/// `COB_SMALL_MAX` from `common.h`.
pub const COB_SMALL_MAX: usize = COB_SMALL_BUFF - 1;
/// `COB_NORMAL_BUFF` from `common.h` -- the buffer used for the `gcore` command line.
pub const COB_NORMAL_BUFF: usize = 2048;

// ---- cob_debug_dump (hex+ASCII memory dump) ------------------------------------------------------

/// `#define dMaxPerLine 24` from `cob_debug_dump`: bytes shown per dump line.
const D_MAX_PER_LINE: usize = 24;
/// `#define dMaxHex ((dMaxPerLine*2)+(dMaxPerLine/4-1))` -- field width of the hex column.
/// For `dMaxPerLine == 24` this is `48 + 5 = 53`.
const D_MAX_HEX: usize = (D_MAX_PER_LINE * 2) + (D_MAX_PER_LINE / 4 - 1);

/// Port of `common.c:repeatWord` -- TRUE if the 4-byte `pattern` is repeated 16 times across `mem`
/// (i.e. `mem[0..16]` is four copies of `pattern`). Redefined locally per the module brief (no
/// dependency on other `common_*` modules). `mem` must be at least 16 bytes.
fn repeat_word(pattern: &[u8; 4], mem: &[u8]) -> bool {
    &mem[0..4] == pattern
        && &mem[4..8] == pattern
        && &mem[8..12] == pattern
        && &mem[12..16] == pattern
}

/// Port of `common.c:cob_debug_dump` -- the `od`-style hex+ASCII dump of a memory buffer.
///
/// Reproduces the exact per-line layout of the C `fprintf` calls:
/// ```text
///  XXXXXX : HH HH HH HH  HH HH ... '....ascii....'
/// ```
/// where the address column is `" %6.6X : "` (a leading space, then 6 upper-hex digits zero-padded,
/// then `" : "`), the hex column is `dMaxHex`-wide left-justified (`%-53s`), each byte `%02X` with a
/// single space after every group of 4 bytes (trailing group-space trimmed), and the ASCII gutter is
/// quoted with `'`. A gutter byte is the character itself when `c >= ' ' && c < 0x7f`, else `.`; note
/// the C `char` is **signed**, so input bytes `>= 0x80` test as negative (`< ' '`) and render as `.`.
///
/// The `repeatWord` run-compression: after a full `dMaxPerLine` line, if the trailing 4 bytes
/// (`lastWord`) repeat for at least the next two `dMaxPerLine` blocks, a single
/// `" XXXXXX :  thru XXXXXX same as last word"` line is emitted and the repeated region skipped (in
/// 16-byte strides). `lastWord` starts as four `0xFD` bytes (matching `memset(lastWord,0xFD,4)`).
///
/// The C function writes to `cob_debug_file` (and returns `0`); this pure port returns the produced
/// bytes instead. `adrs` is always `0` in the C call site, so the address column equals the offset.
pub fn cob_debug_dump(mem: &[u8]) -> Vec<u8> {
    let len = mem.len();
    let adrs: usize = 0;
    let mut out: Vec<u8> = Vec::new();
    let mut last_word: [u8; 4] = [0xFD; 4];

    let mut i = 0usize;
    while i < len {
        // Build the hex column.
        let mut hex: Vec<u8> = Vec::new();
        let mut j = 0usize;
        while j < D_MAX_PER_LINE && (i + j) < len {
            push_hex_byte(&mut hex, mem[i + j]);
            if (j % 4) == 3 {
                hex.push(0x20); // ' '
            }
            j += 1;
        }
        // Trim a single trailing group-space (`if (k && hex[k-1] == ' ') hex[k-1] = 0;`).
        if hex.last() == Some(&0x20) {
            hex.pop();
        }

        // Build the ASCII gutter.
        let mut chr: Vec<u8> = Vec::new();
        let mut j2 = 0usize;
        while j2 < D_MAX_PER_LINE && (i + j2) < len {
            let c = mem[i + j2];
            // signed-char semantics: bytes >= 0x80 are negative, hence < ' '
            let printable = c >= 0x20 && c < 0x7f;
            chr.push(if printable { c } else { b'.' });
            j2 += 1;
        }

        // " %6.6X : %-*s '%s'\n" with width dMaxHex for hex.
        push_addr(&mut out, adrs + i);
        out.extend_from_slice(b" : ");
        out.extend_from_slice(&hex);
        // left-justify hex to D_MAX_HEX
        for _ in hex.len()..D_MAX_HEX {
            out.push(0x20);
        }
        out.push(0x20);
        out.push(0x27); // '\''
        out.extend_from_slice(&chr);
        out.push(0x27); // '\''
        out.push(b'\n');

        // Capture lastWord = trailing (up to 4) bytes of this block, when more remains.
        if (i + D_MAX_PER_LINE) < len {
            let n = if j < 4 { j } else { 4 };
            // memcpy(lastWord, &mem[i+dMaxPerLine-4], n) -- copies the low `n` bytes.
            let src = i + D_MAX_PER_LINE - 4;
            for b in 0..n {
                last_word[b] = mem[src + b];
            }
        }
        i += D_MAX_PER_LINE;

        // Run-compression: if lastWord repeats for the next two blocks, collapse.
        if (i + (16 * 2)) < len
            && i + 16 <= len
            && repeat_word(&last_word, &mem[i..])
            && i + D_MAX_PER_LINE + 16 <= len
            && repeat_word(&last_word, &mem[i + D_MAX_PER_LINE..])
        {
            push_addr(&mut out, adrs + i);
            out.extend_from_slice(b" : ");
            while i < len.saturating_sub(16) && i + 16 <= len && repeat_word(&last_word, &mem[i..]) {
                i += 16;
            }
            out.extend_from_slice(b" thru ");
            push_addr_bare(&mut out, adrs + i - 1);
            out.extend_from_slice(b" same as last word\n");
        }
    }

    out
}

/// `sprintf("%02X", b & 0xFF)` -- two upper-hex digits.
fn push_hex_byte(out: &mut Vec<u8>, b: u8) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    out.push(HEX[(b >> 4) as usize]);
    out.push(HEX[(b & 0x0f) as usize]);
}

/// The `" %6.6X"` field: a leading space then 6 upper-hex digits, zero-padded (C `%6.6X`).
fn push_addr(out: &mut Vec<u8>, v: usize) {
    out.push(0x20);
    push_addr_bare(out, v);
}

/// The `%6.6X` value without the leading space (used after `" thru "`).
fn push_addr_bare(out: &mut Vec<u8>, v: usize) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut digits: Vec<u8> = Vec::new();
    let mut x = v;
    loop {
        digits.push(HEX[x & 0x0f]);
        x >>= 4;
        if x == 0 {
            break;
        }
    }
    // zero-pad to at least 6 digits (precision .6); a value wider than 6 prints fully.
    while digits.len() < 6 {
        digits.push(b'0');
    }
    digits.reverse();
    out.extend_from_slice(&digits);
}

// ---- DebugLogState (COB_DEBUG_LOG runtime state) -------------------------------------------------

/// Explicit model of the `COB_DEBUG_LOG` runtime state that `cob_debug_open`/`cob_debug_logit`/
/// `cob_debug_logger` read & write (in C these are the file-scope statics `cob_debug_modules`,
/// `cob_debug_level`, `cob_debug_log_time`, `cob_debug_mod`, `cob_debug_file_name`, plus the
/// `cob_debug_file` `FILE *`). Modelled here instead of as statics.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DebugLogState {
    /// `cob_debug_modules[12][4]` -- up to 12 selected module 2/3-char codes (empty == unset).
    pub modules: Vec<Vec<u8>>,
    /// `cob_debug_level` -- 0 normal, 2 warnings, 3 trace, 9 all.
    pub level: i32,
    /// `cob_debug_log_time` -- non-zero when timestamps are prefixed (set with `L=T`).
    pub log_time: i32,
    /// `cob_debug_file_name` -- resolved log filename (the file-open itself is the boundary).
    pub file_name: Vec<u8>,
    /// Whether the log `FILE *` is considered open (`cob_debug_file != NULL`). The boundary owner
    /// sets this; the pure formatters consult it.
    pub file_open: bool,
}

/// Port of `common.c:cob_debug_open` (pure parse part) -- parse the `COB_DEBUG_LOG` env string into
/// the module list, level, time-flag and log filename. The C function additionally opens the logfile
/// (`cob_open_logfile`) and wires it to the trace file; that file-open is the documented boundary and
/// is **not** performed here -- this returns the resolved [`DebugLogState`] (with `file_open == false`).
///
/// The grammar (per the C comment): a series of `keyword=value` items separated by `,`/`;`:
/// - `M=yy`  module to debug (the 2/3-char code, tabled into `modules` if a free slot exists)
/// - `L=x`   level: `T`->trace(3, sets log_time), `W`->warnings(2), `N`->normal(0), `A`->all(9)
/// - `O=path` output logfile name
///
/// `pid` supplies the value for the default `cob_debug_log.<pid>` filename when `O=` is absent.
pub fn cob_debug_open(debug_env: &[u8], pid: i32) -> DebugLogState {
    let mut state = DebugLogState::default();
    let mut logfile: Vec<u8> = Vec::new();

    let bytes = debug_env;
    let mut i = 0usize;
    while i < bytes.len() && bytes[i] != 0 {
        // skip separator
        if bytes[i] == b',' || bytes[i] == b';' {
            i += 1;
            continue;
        }
        // a "x=" flag?
        if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
            let log_opt = bytes[i];
            i += 2;
            match log_opt {
                b'M' | b'm' => {
                    // gather up to 4 chars until separator / end
                    let mut module_name: Vec<u8> = Vec::new();
                    let mut j = 0;
                    while j < 4 {
                        if i >= bytes.len()
                            || bytes[i] == b','
                            || bytes[i] == b';'
                            || bytes[i] == 0
                        {
                            break;
                        }
                        module_name.push(bytes[i]);
                        j += 1;
                        i += 1;
                    }
                    // find first free slot among up to 12 (slot[0] <= ' ' means free)
                    if state.modules.len() < 12 {
                        state.modules.push(module_name);
                    }
                    if i < bytes.len() && bytes[i] == 0 {
                        i = i.wrapping_sub(1);
                    }
                }
                b'L' | b'l' => {
                    if i < bytes.len() {
                        let lv = bytes[i];
                        match lv {
                            b'T' | b't' => {
                                state.log_time = 3;
                                state.level = 3;
                            }
                            b'W' | b'w' => state.level = 2,
                            b'N' | b'n' => state.level = 0,
                            b'A' | b'a' => state.level = 9,
                            _ => {
                                // unknown log option, step back (C: i--)
                                i = i.wrapping_sub(1);
                            }
                        }
                    }
                }
                b'O' | b'o' => {
                    let mut j = 0;
                    logfile.clear();
                    while j < COB_SMALL_MAX {
                        if i >= bytes.len()
                            || bytes[i] == b','
                            || bytes[i] == b';'
                            || bytes[i] == 0
                        {
                            break;
                        }
                        logfile.push(bytes[i]);
                        j += 1;
                        i += 1;
                    }
                    if i < bytes.len() && bytes[i] == 0 {
                        i = i.wrapping_sub(1);
                    }
                }
                _ => {
                    // unknown x=, ignored
                }
            }
        } else {
            // invalid character, ignored
        }
        i = i.wrapping_add(1);
    }

    // default logfile if none given
    if logfile.is_empty() {
        logfile = format!("cob_debug_log.{}", pid).into_bytes();
    }
    state.file_name = logfile;
    state
}

/// Port of `common.c:cob_debug_logit` -- decide whether a debug-log line at `level` for `module` is
/// allowed. Returns `1` when **not** allowed (suppressed), `0` when allowed (matching the C return
/// convention). Allowed iff the log file is open, `level <= state.level`, and `module` matches one of
/// the selected modules (case-insensitive) or the special `ALL`. On a match the second return value is
/// the matched module text (the C side stores it in `cob_debug_mod` for the next log header).
pub fn cob_debug_logit(state: &DebugLogState, level: i32, module: &[u8]) -> (i32, Option<Vec<u8>>) {
    if !state.file_open {
        return (1, None);
    }
    if level > state.level {
        return (1, None);
    }
    for m in &state.modules {
        if m.is_empty() {
            break;
        }
        if eq_ascii_ci(m, b"ALL") {
            return (0, Some(module.to_vec()));
        }
        if eq_ascii_ci(module, m) {
            return (0, Some(m.clone()));
        }
    }
    (1, None)
}

/// Inputs to a single `cob_debug_logger` header (the file-scope state the C function reads:
/// `cob_debug_log_time`, `cob_debug_mod`, `cob_source_file`, `cob_source_line`, plus the body text).
#[derive(Debug, Clone, Default)]
pub struct DebugLogLine<'a> {
    /// `cob_debug_log_time` -- emit the `HH:MM:SS.cc ` timestamp prefix.
    pub log_time: bool,
    /// `cob_debug_mod` -- the matched module text (from [`cob_debug_logit`]); `None` to omit.
    pub debug_mod: Option<&'a [u8]>,
    /// `cob_source_file` -- the source filename; `None` to omit the `" file :"` segment.
    pub source_file: Option<&'a [u8]>,
    /// `cob_source_line` -- current source line number; `0` to omit the line segment.
    pub source_line: u32,
    /// `time` -- (hour, minute, second, nanosecond) used only when `log_time` is set.
    pub time: (u32, u32, u32, u32),
    /// The already-formatted body (the `vfprintf(fmt, ...)` result).
    pub body: &'a [u8],
}

/// Port of `common.c:cob_debug_logger` (pure formatting part) -- produce the bytes of one debug-log
/// entry. The C function maintains a "header pending" static (`cob_debug_hdr`) and the previous-line
/// memo (`cob_debug_prv_line`); this pure port takes the resolved state via [`DebugLogLine`] and
/// always renders the header (callers that suppressed it pass the body only). It reproduces the header
/// `fprintf`s:
/// - `"%02d:%02d:%02d.%02d "` time prefix when `log_time`
/// - `"%-3s:"` for the module (left-justified to width 3)
/// - `" %s :"` for the source file
/// - `"%5d : "` for a new source line, else `"%5s : "` with a single space when the line is unchanged
///   (modelled here as: emit the number form when `source_line != 0`, else the blank `"    "` form).
///
/// The returned `header_pending` is `true` when the body ends in `\n` (C sets `cob_debug_hdr = 1`).
pub fn cob_debug_logger(line: &DebugLogLine) -> (Vec<u8>, bool) {
    let mut out: Vec<u8> = Vec::new();

    if line.log_time {
        // "%02d:%02d:%02d.%02d " ; nanosecond/10000000 -> centiseconds
        let (h, m, s, ns) = line.time;
        push_02(&mut out, h);
        out.push(b':');
        push_02(&mut out, m);
        out.push(b':');
        push_02(&mut out, s);
        out.push(b'.');
        push_02(&mut out, ns / 10_000_000);
        out.push(0x20);
    }
    if let Some(md) = line.debug_mod {
        // "%-3s:" -- left-justify to 3
        out.extend_from_slice(md);
        for _ in md.len()..3 {
            out.push(0x20);
        }
        out.push(b':');
    }
    if let Some(sf) = line.source_file {
        out.push(0x20);
        out.extend_from_slice(sf);
        out.extend_from_slice(b" :");
    }
    if line.source_line != 0 {
        // "%5d : "
        push_width(&mut out, format!("{}", line.source_line).into_bytes(), 5);
        out.extend_from_slice(b" : ");
    } else {
        // "%5s : " with a single space argument -> 4 leading spaces + ' '
        push_width(&mut out, vec![0x20], 5);
        out.extend_from_slice(b" : ");
    }

    out.extend_from_slice(line.body);

    let header_pending = line.body.last() == Some(&b'\n');
    (out, header_pending)
}

/// `"%02d"` -- two decimal digits, zero-padded (values >= 100 print fully, like C).
fn push_02(out: &mut Vec<u8>, v: u32) {
    let s = format!("{}", v).into_bytes();
    for _ in s.len()..2 {
        out.push(b'0');
    }
    out.extend_from_slice(&s);
}

/// Right-justify `s` to `width` with leading spaces (C `%Nd` / `%Ns`); longer prints fully.
fn push_width(out: &mut Vec<u8>, s: Vec<u8>, width: usize) {
    for _ in s.len()..width {
        out.push(0x20);
    }
    out.extend_from_slice(&s);
}

/// ASCII case-insensitive byte-slice equality (the `strcasecmp` in `cob_debug_logit`/`cob_debug_open`).
fn eq_ascii_ci(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .all(|(&x, &y)| x.to_ascii_lowercase() == y.to_ascii_lowercase())
}

// ---- dump-file destination resolution ------------------------------------------------------------

/// The destination `cob_get_dump_file` resolves the dump to. The C function returns a `FILE *`; this
/// enum names the resolved sink (the actual `FILE *` open is the boundary).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DumpDest {
    /// `cobsetptr->cob_dump_file` is already open -- use it.
    DumpFile,
    /// Open `cobsetptr->cob_dump_filename` (the contained name) as the dump file.
    OpenFilename(Vec<u8>),
    /// The dump is explicitly disabled (`COB_DUMP_FILE` resolved to a false value) -- no dump.
    Disabled,
    /// Fall through to the trace file (`cobsetptr->cob_trace_file`).
    TraceFile,
    /// Fall through to `stderr`.
    Stderr,
}

/// Explicit model of the settings `cob_get_dump_file` consults (`cobsetptr` fields).
#[derive(Debug, Clone, Default)]
pub struct DumpFileState {
    /// Whether `cob_dump_file` is already open (`cob_dump_file != NULL`).
    pub dump_file_open: bool,
    /// `cob_dump_filename` -- the configured dump filename, if any.
    pub dump_filename: Option<Vec<u8>>,
    /// Whether `cob_trace_file` is open (`cob_trace_file != NULL`).
    pub trace_file_open: bool,
}

/// Returns `true` when `s` is a recognised "false"/disable value (the `cob_check_env_false` check that
/// gates the dump filename). Matches GnuCOBOL's set of disable tokens, case-insensitively.
fn cob_check_env_false(s: &[u8]) -> bool {
    matches!(
        s.to_ascii_uppercase().as_slice(),
        b"0" | b"NO" | b"NONE" | b"OFF" | b"FALSE" | b"N" | b"F"
    )
}

/// Port of `common.c:cob_get_dump_file` (the active `#if 1` branch) -- resolve the dump destination
/// from the settings: an already-open dump file wins; else a configured dump filename is opened
/// (unless it is a "false" value, which disables the dump, or unless opening fails -- modelled by the
/// caller); else the trace file if open; else `stderr`.
///
/// A `false` filename yields [`DumpDest::Disabled`] (the C `return NULL`). The open-failure
/// fall-through (`cob_open_logfile` returning NULL) is the boundary owner's concern: on failure it
/// should re-resolve with `dump_filename = None`.
pub fn cob_get_dump_file(state: &DumpFileState) -> DumpDest {
    if state.dump_file_open {
        return DumpDest::DumpFile;
    }
    if let Some(name) = &state.dump_filename {
        if cob_check_env_false(name) {
            return DumpDest::Disabled;
        }
        return DumpDest::OpenFilename(name.clone());
    }
    if state.trace_file_open {
        return DumpDest::TraceFile;
    }
    DumpDest::Stderr
}

/// Port of `common.c:create_dumpfile` (pure command-line build) -- build the `gcore` command line used
/// to request a core dump. Resolves the core filename from `core_filename` (the `COB_CORE_FILENAME`
/// setting) falling back to `./core.libcob`, and formats `"gcore -a -o <name> <pid>"`. If the command
/// would exceed `COB_NORMAL_BUFF`, the C code retries with the default name -- reproduced here. The
/// actual `system()` execution is the documented boundary and is **not** performed.
pub fn create_dumpfile(core_filename: Option<&[u8]>, pid: i32) -> Vec<u8> {
    const DEFAULT_NAME: &[u8] = b"./core.libcob";
    let name: &[u8] = core_filename.unwrap_or(DEFAULT_NAME);

    let cmd = build_gcore_cmd(name, pid);
    // snprintf into COB_NORMAL_BUFF: content_size >= COB_NORMAL_BUFF means truncation occurred.
    if cmd.len() >= COB_NORMAL_BUFF {
        return build_gcore_cmd(DEFAULT_NAME, pid);
    }
    cmd
}

fn build_gcore_cmd(name: &[u8], pid: i32) -> Vec<u8> {
    let mut cmd: Vec<u8> = Vec::new();
    cmd.extend_from_slice(b"gcore -a -o ");
    cmd.extend_from_slice(name);
    cmd.push(0x20);
    cmd.extend_from_slice(format!("{}", pid).as_bytes());
    cmd
}

// ---- cob_dump_module banner ----------------------------------------------------------------------

/// Port of the per-module banner line of `common.c:cob_dump_module` -- the
/// `"Dump Program-Id <name> from <source> compiled <date>"` line emitted for each module in the chain
/// that has a cancel/dump entry, followed by a newline. (The surrounding chain walk, locale switch and
/// `cancel_func(-10)` field dump are runtime/FFI side effects -- the documented boundary; this ports
/// the pure formatting of the banner bytes.)
pub fn cob_dump_module_header(
    module_name: &[u8],
    module_source: &[u8],
    module_formatted_date: &[u8],
) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    out.extend_from_slice(b"Dump Program-Id ");
    out.extend_from_slice(module_name);
    out.extend_from_slice(b" from ");
    out.extend_from_slice(module_source);
    out.extend_from_slice(b" compiled ");
    out.extend_from_slice(module_formatted_date);
    out.push(b'\n');
    out
}

/// Port of the "module dump due to" reason banner of `common.c:cob_dump_module` -- the
/// `"\nModule dump due to <reason>\n"` block written before the chain when `reason` is non-empty.
/// An empty reason becomes `"unknown"` (the C `_("unknown")`).
pub fn cob_dump_module_reason(reason: &[u8]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    out.push(b'\n');
    out.extend_from_slice(b"Module dump due to ");
    if reason.is_empty() {
        out.extend_from_slice(b"unknown");
    } else {
        out.extend_from_slice(reason);
    }
    out.push(b'\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dmax_constants() {
        assert_eq!(D_MAX_PER_LINE, 24);
        assert_eq!(D_MAX_HEX, 53);
    }

    #[test]
    fn hexdump_known_buffer_exact_bytes() {
        // "Hello, World!" -- a single short line (13 bytes < dMaxPerLine).
        let buf = b"Hello, World!";
        let out = cob_debug_dump(buf);
        // Expected layout:
        //  000000 : 48656C6C 6F2C2057 6F726C64 21<pad to 53> 'Hello, World!'\n
        // hex column content:
        let hex = "48656C6C 6F2C2057 6F726C64 21";
        let mut expected = String::new();
        expected.push(' ');
        expected.push_str("000000");
        expected.push_str(" : ");
        expected.push_str(hex);
        // pad hex to D_MAX_HEX (53)
        for _ in hex.len()..D_MAX_HEX {
            expected.push(' ');
        }
        expected.push(' ');
        expected.push('\'');
        expected.push_str("Hello, World!");
        expected.push('\'');
        expected.push('\n');
        assert_eq!(out, expected.as_bytes(), "got: {:?}", String::from_utf8_lossy(&out));
    }

    #[test]
    fn hexdump_nonprintable_and_high_bytes_become_dot() {
        // 0x00 (control), 0x41 'A', 0x7f (DEL, excluded: c < 0x7f), 0x80 (high -> signed negative)
        let buf = [0x00u8, 0x41, 0x7f, 0x80];
        let out = cob_debug_dump(&buf);
        let s = String::from_utf8_lossy(&out);
        // gutter must be ".A.."
        assert!(s.contains("'.A..'"), "got: {}", s);
        // hex must contain the bytes
        assert!(s.contains("00417F80"), "got: {}", s);
    }

    #[test]
    fn hexdump_group_spacing_and_address() {
        // exactly 8 bytes -> two 4-byte groups separated by one space, trailing group-space trimmed.
        let buf = [1u8, 2, 3, 4, 5, 6, 7, 8];
        let out = cob_debug_dump(&buf);
        let s = String::from_utf8_lossy(&out);
        assert!(s.starts_with(" 000000 : 01020304 05060708"), "got: {}", s);
        // no double-trailing space before the quote region beyond padding
        assert!(s.contains("'........'"), "got: {}", s);
    }

    #[test]
    fn hexdump_run_compression() {
        // Build a buffer where a 4-byte word repeats far past two blocks so run-compression triggers.
        // First fill one dMaxPerLine block with distinct bytes, then a long run of 0xAB.
        let mut buf: Vec<u8> = (0..D_MAX_PER_LINE as u8).collect();
        buf.extend(std::iter::repeat(0xABu8).take(16 * 20));
        let out = cob_debug_dump(&buf);
        let s = String::from_utf8_lossy(&out);
        assert!(s.contains("same as last word"), "expected run-compression, got:\n{}", s);
        assert!(s.contains(" thru "), "got:\n{}", s);
    }

    #[test]
    fn hexdump_empty_is_empty() {
        assert_eq!(cob_debug_dump(&[]), Vec::<u8>::new());
    }

    #[test]
    fn debug_open_parses_module_level_time_and_file() {
        // M=RW,L=T,O=/tmp/mylog
        let st = cob_debug_open(b"M=RW,L=T,O=/tmp/mylog", 42);
        assert_eq!(st.modules, vec![b"RW".to_vec()]);
        assert_eq!(st.level, 3);
        assert_eq!(st.log_time, 3);
        assert_eq!(st.file_name, b"/tmp/mylog".to_vec());
        assert!(!st.file_open);
    }

    #[test]
    fn debug_open_default_logfile_uses_pid() {
        let st = cob_debug_open(b"L=A", 999);
        assert_eq!(st.level, 9);
        assert_eq!(st.file_name, b"cob_debug_log.999".to_vec());
    }

    #[test]
    fn debug_open_levels() {
        assert_eq!(cob_debug_open(b"L=W", 1).level, 2);
        assert_eq!(cob_debug_open(b"L=N", 1).level, 0);
        assert_eq!(cob_debug_open(b"L=A", 1).level, 9);
        // trace also sets log_time
        let t = cob_debug_open(b"L=T", 1);
        assert_eq!(t.level, 3);
        assert_eq!(t.log_time, 3);
    }

    #[test]
    fn debug_logit_allows_matching_module_and_all() {
        let mut st = DebugLogState {
            modules: vec![b"RW".to_vec()],
            level: 9,
            file_open: true,
            ..Default::default()
        };
        // matching module -> allowed (0), returns matched text
        assert_eq!(cob_debug_logit(&st, 1, b"rw"), (0, Some(b"RW".to_vec())));
        // non-matching -> suppressed (1)
        assert_eq!(cob_debug_logit(&st, 1, b"XX"), (1, None));
        // level above configured -> suppressed
        assert_eq!(cob_debug_logit(&st, 99, b"RW"), (1, None));
        // closed file -> suppressed
        st.file_open = false;
        assert_eq!(cob_debug_logit(&st, 1, b"RW"), (1, None));
        // ALL matches anything
        let all = DebugLogState {
            modules: vec![b"ALL".to_vec()],
            level: 9,
            file_open: true,
            ..Default::default()
        };
        assert_eq!(cob_debug_logit(&all, 1, b"ZZ"), (0, Some(b"ZZ".to_vec())));
    }

    #[test]
    fn debug_logger_header_and_body() {
        let line = DebugLogLine {
            log_time: false,
            debug_mod: Some(b"RW"),
            source_file: Some(b"prog.cob"),
            source_line: 42,
            time: (0, 0, 0, 0),
            body: b"hello\n",
        };
        let (out, pending) = cob_debug_logger(&line);
        // "%-3s:" for RW -> "RW :"  ; " %s :" -> " prog.cob :" ; "%5d : " -> "   42 : "
        assert_eq!(out, b"RW : prog.cob :   42 : hello\n");
        assert!(pending); // body ends with \n
    }

    #[test]
    fn debug_logger_time_prefix() {
        let line = DebugLogLine {
            log_time: true,
            debug_mod: None,
            source_file: None,
            source_line: 0,
            // 33500000 ns / 10000000 == 3 centiseconds
            time: (9, 5, 7, 33_500_000),
            body: b"x",
        };
        let (out, pending) = cob_debug_logger(&line);
        // "09:05:07.03 " (time + trailing space) then "%5s : " with a single-space arg:
        // width-5 of " " => 4 leading spaces + the space arg = "     ", then " : ", then body.
        assert_eq!(out, b"09:05:07.03       : x");
        assert!(!pending);
    }

    #[test]
    fn get_dump_file_resolution() {
        // already open
        let s = DumpFileState { dump_file_open: true, ..Default::default() };
        assert_eq!(cob_get_dump_file(&s), DumpDest::DumpFile);
        // filename configured (and not false)
        let s = DumpFileState {
            dump_filename: Some(b"/tmp/d.dump".to_vec()),
            ..Default::default()
        };
        assert_eq!(cob_get_dump_file(&s), DumpDest::OpenFilename(b"/tmp/d.dump".to_vec()));
        // false filename disables
        let s = DumpFileState { dump_filename: Some(b"NONE".to_vec()), ..Default::default() };
        assert_eq!(cob_get_dump_file(&s), DumpDest::Disabled);
        // trace file fallback
        let s = DumpFileState { trace_file_open: true, ..Default::default() };
        assert_eq!(cob_get_dump_file(&s), DumpDest::TraceFile);
        // stderr fallback
        assert_eq!(cob_get_dump_file(&DumpFileState::default()), DumpDest::Stderr);
    }

    #[test]
    fn create_dumpfile_cmdline() {
        let cmd = create_dumpfile(Some(b"/tmp/mycore"), 1234);
        assert_eq!(cmd, b"gcore -a -o /tmp/mycore 1234".to_vec());
        let def = create_dumpfile(None, 7);
        assert_eq!(def, b"gcore -a -o ./core.libcob 7".to_vec());
    }

    #[test]
    fn dump_module_banners() {
        let h = cob_dump_module_header(b"PROG", b"prog.cob", b"Jan 01 2020 12:00:00");
        assert_eq!(
            h,
            b"Dump Program-Id PROG from prog.cob compiled Jan 01 2020 12:00:00\n".to_vec()
        );
        assert_eq!(cob_dump_module_reason(b""), b"\nModule dump due to unknown\n".to_vec());
        assert_eq!(
            cob_dump_module_reason(b"signal"),
            b"\nModule dump due to signal\n".to_vec()
        );
    }

    #[test]
    fn repeat_word_helper() {
        let pat = [0xAAu8; 4];
        let mem = [0xAAu8; 16];
        assert!(repeat_word(&pat, &mem));
        let mut bad = [0xAAu8; 16];
        bad[10] = 0;
        assert!(!repeat_word(&pat, &bad));
    }
}
