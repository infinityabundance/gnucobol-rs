//! Port of the **source-location / stack-trace / signal-name** surface of `libcob/common.c`.
//!
//! This module ports the parts of common.c that compute and format the runtime's diagnostic
//! location and backtrace output, plus the OS-signal name/description table. Everything here is
//! modeled as a **pure function of its inputs**: the C originals walk live global runtime state
//! (`cobglobptr`, `COB_MODULE_PTR`, `cob_source_file`/`cob_source_line`) and write directly to
//! `stderr`/a `FILE *` via `write(2)`. We capture that state as explicit inputs and return the
//! formatted bytes; **the actual stderr/FILE write is the runtime boundary** and is not performed
//! here. This keeps the port panic-free and side-effect-free while remaining 1:1 with the C
//! formatting logic (every `write_or_return_*` becomes a byte append).
//!
//! Genuinely OS-bound signal *registration* (`cob_set_signal`, `cob_reg_sighnd`,
//! `cob_sig_handler`) is **not** ported -- it installs `sigaction(2)` handlers and has no pure
//! value semantics. Only the signal *name/description* table and its lookups are ported.

#![forbid(unsafe_code)]

// ---------------------------------------------------------------------------
// Statement-word encoding macros (common.h)
// ---------------------------------------------------------------------------

/// Port of `common.h:COB_GET_LINE_NUM(n)` -- `( n & 0xFFFFF )`. The low 20 bits of a packed
/// `module_stmt` word hold the source line number.
pub fn cob_get_line_num(n: u32) -> u32 {
    n & 0x000F_FFFF
}

/// Port of `common.h:COB_GET_FILE_NUM(n)` -- `( (n >> 20) & 0xFFF )`. Bits 20..32 of a packed
/// `module_stmt` word hold the index into the module's `module_sources` array.
pub fn cob_get_file_num(n: u32) -> u32 {
    (n >> 20) & 0x0000_0FFF
}

/// Port of `common.c:MAX_MODULE_ITERS` (`const int MAX_MODULE_ITERS = 10240;`). Guards the
/// module-chain walks against an endless loop on a broken `->next` chain.
pub const MAX_MODULE_ITERS: i32 = 10240;

/// Port of `common.h:COB_MODULE_TYPE_FUNCTION` (`= 1`).
pub const COB_MODULE_TYPE_FUNCTION: i32 = 1;

// ---------------------------------------------------------------------------
// ss_itoa_u10 (signal-safe int -> ASCII), used by every write_or_return_int
// ---------------------------------------------------------------------------

/// Local copy of `common.c:ss_itoa_u10` formatting, used internally so the formatters below stay
/// self-contained (no cross-module dependency). Appends the base-10 ASCII of `value` (leading `-`
/// for negatives) onto `out`.
fn ss_itoa_u10(out: &mut Vec<u8>, value: i32) {
    if value < 0 {
        out.push(b'-');
    }
    let mut u = value.unsigned_abs();
    let start = out.len();
    loop {
        out.push(b'0' + (u % 10) as u8);
        u /= 10;
        if u == 0 {
            break;
        }
    }
    out[start..].reverse();
}

// ---------------------------------------------------------------------------
// Source location tracking
// ---------------------------------------------------------------------------

/// Models `common.c`'s current source position: the globals `cob_source_file` / `cob_source_line`
/// (here as an owned pair). A `None`/empty file mirrors the C `NULL`, and a zero line mirrors the C
/// "no line known" sentinel.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SourceLocation {
    /// `cob_source_file` -- `None` corresponds to the C `NULL` pointer.
    pub file: Option<Vec<u8>>,
    /// `cob_source_line` -- `0` corresponds to "no line known".
    pub line: u32,
}

/// A single resolved module frame: the inputs `cob_get_source_line` / `set_source_location` read
/// out of a live `cob_module`. Modeling the chain as a list of these lets the walks be pure.
///
/// Fields mirror the `cob_module` members the C functions touch:
/// - `module_stmt` -- the packed file/line word (0 = "no statement"),
/// - `module_sources` -- the source-file name table indexed by `COB_GET_FILE_NUM`,
/// - `module_name`, `section_name`, `paragraph_name`, `module_type`, `statement`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StackFrame {
    /// `mod->module_stmt`: packed file/line word, `0` meaning "no statement recorded".
    pub module_stmt: u32,
    /// `mod->module_sources`: source-file names indexed by [`cob_get_file_num`].
    pub module_sources: Vec<Vec<u8>>,
    /// `mod->module_name`.
    pub module_name: Option<Vec<u8>>,
    /// `mod->section_name`.
    pub section_name: Option<Vec<u8>>,
    /// `mod->paragraph_name`.
    pub paragraph_name: Option<Vec<u8>>,
    /// `mod->module_type` (compare against [`COB_MODULE_TYPE_FUNCTION`]).
    pub module_type: i32,
    /// `mod->statement` index into the statement-name table; `None` models `STMT_UNKNOWN`.
    pub statement: Option<Vec<u8>>,
}

/// Port of `common.c:cob_get_source_line` -- resolve the current source file+line from the module
/// chain and store it into [`SourceLocation`].
///
/// The C original walks `COB_MODULE_PTR` forward skipping frames with `module_stmt == 0`, then (if
/// the found frame has a non-zero `module_stmt` and a `module_sources` table) sets
/// `cob_source_file = module_sources[COB_GET_FILE_NUM(module_stmt)]` and
/// `cob_source_line = COB_GET_LINE_NUM(module_stmt)`. With no usable frame the location is left
/// unchanged (the C `#if 0` reset branch is disabled). `loc` is the in/out global pair.
pub fn cob_get_source_line(chain: &[StackFrame], loc: &mut SourceLocation) {
    // mirror: while (mod && mod->module_stmt == 0) mod = mod->next;
    let found = chain.iter().find(|m| m.module_stmt != 0);
    if let Some(m) = found {
        if m.module_stmt != 0 && !m.module_sources.is_empty() {
            let idx = cob_get_file_num(m.module_stmt) as usize;
            if let Some(src) = m.module_sources.get(idx) {
                loc.file = Some(src.clone());
                loc.line = cob_get_line_num(m.module_stmt);
            }
        }
    }
}

/// Port of `common.c:set_source_location` -- compute the source file+line to report.
///
/// Starts from the current globals (`cob_source_file`/`cob_source_line`, passed as `current`),
/// then, if the head module (`COB_MODULE_PTR`, modeled as `head`) has a non-zero `module_stmt` and
/// a `module_sources` table, overrides with `module_sources[COB_GET_FILE_NUM]` /
/// `COB_GET_LINE_NUM`. Returns the resolved [`SourceLocation`] (the C version writes through
/// `*file`/`*line` out-params).
pub fn set_source_location(head: Option<&StackFrame>, current: &SourceLocation) -> SourceLocation {
    let mut out = current.clone();
    if let Some(m) = head {
        if m.module_stmt != 0 && !m.module_sources.is_empty() {
            let idx = cob_get_file_num(m.module_stmt) as usize;
            if let Some(src) = m.module_sources.get(idx) {
                out.file = Some(src.clone());
                out.line = cob_get_line_num(m.module_stmt);
            }
        }
    }
    out
}

/// Port of `common.c:output_source_location` -- format the `"<file>:<line>: "` (or `"<file>: "`)
/// prefix written to stderr before a diagnostic.
///
/// Resolves the location via [`set_source_location`], then -- if a file is known -- emits
/// `<file>`, optionally `":" <line>` when the line is non-zero, then `": "`. Returns the bytes;
/// **the stderr write is the runtime boundary**. With no file known, returns an empty `Vec`
/// (the C function writes nothing).
pub fn output_source_location(head: Option<&StackFrame>, current: &SourceLocation) -> Vec<u8> {
    let loc = set_source_location(head, current);
    let mut out = Vec::new();
    if let Some(file) = &loc.file {
        out.extend_from_slice(file);
        if loc.line != 0 {
            out.push(b':');
            ss_itoa_u10(&mut out, loc.line as i32);
        }
        out.extend_from_slice(b": ");
    }
    out
}

/// Port of `common.c:cob_set_location` (pre-3.0 backward-compat entry).
///
/// The C original mutated module + global state (`mod->section_name`, `mod->paragraph_name`,
/// `cob_source_file`, `cob_source_line`, `mod->statement`) and then, **only when line-tracing is
/// enabled**, wrote a `Source:`/`Program-Id:` trace line to the trace file. We model the pure
/// state update: returns the updated [`SourceLocation`] for the given `sfile`/`sline`. The head
/// module's `section_name`/`paragraph_name`/`statement` updates and the optional trace-file
/// `fprintf` are the runtime boundary (the trace formatting lives with the trace module).
pub fn cob_set_location(sfile: &[u8], sline: u32) -> SourceLocation {
    SourceLocation {
        file: Some(sfile.to_vec()),
        line: sline,
    }
}

// ---------------------------------------------------------------------------
// Stack trace / backtrace formatting
// ---------------------------------------------------------------------------

/// Port of `common.c:flush_target` -- selects which streams to flush before writing a trace.
///
/// Pure here: returns the side-effect description rather than performing it. `true` means the C
/// code flushed both `stdout` and `stderr` (target was one of them); `false` means it flushed only
/// the single `target`. The actual `fflush` is the runtime boundary.
pub fn flush_target(target_is_std: bool) -> bool {
    target_is_std
}

/// Port of `common.c:output_procedure_stack_entry` -- format one `"\n\t<para> OF <section> at
/// <file>:<line>"` procedure-stack line.
///
/// Mirrors the C control flow exactly: returns an empty `Vec` when both `section` and `paragraph`
/// are absent; with both present emits `<paragraph> " OF " <section>`; otherwise the single
/// present name; then ` at <file>:<line>`. Returns the bytes; the `write` is the boundary.
pub fn output_procedure_stack_entry(
    section: Option<&[u8]>,
    paragraph: Option<&[u8]>,
    source_file: &[u8],
    source_line: u32,
) -> Vec<u8> {
    let mut out = Vec::new();
    if section.is_none() && paragraph.is_none() {
        return out;
    }
    out.extend_from_slice(b"\n\t");
    match (section, paragraph) {
        (Some(s), Some(p)) => {
            out.extend_from_slice(p);
            out.extend_from_slice(b" OF ");
            out.extend_from_slice(s);
        }
        (Some(s), None) => out.extend_from_slice(s),
        (None, Some(p)) => out.extend_from_slice(p),
        (None, None) => unreachable!(),
    }
    out.extend_from_slice(b" at ");
    out.extend_from_slice(source_file);
    out.push(b':');
    ss_itoa_u10(&mut out, source_line as i32);
    out
}

/// Port of `common.c:cob_stack_trace_internal` -- format the COBOL-view stack trace for `chain`
/// (modeled `COB_MODULE_PTR -> next ...`).
///
/// `verbose` selects the verbose `Last statement of "..."`-style output (used by
/// [`cob_stack_trace`]); `verbose == false` is the GDB-like single-line-per-frame form (used by
/// [`cob_backtrace`]). `count` limits the number of frames: `> 0` keeps the first `count`, `< 0`
/// keeps the last `-count` (computed up front by counting the chain). Returns the formatted bytes;
/// the `write`/`fputs` to the target is the runtime boundary. The `frame_ptr` PERFORM-stack walk
/// and the `Started by <argv>` tail depend on live `cob_frame_ext`/`cob_argv` state not modeled in
/// `StackFrame`, so they are documented as the boundary and omitted (callers pass the module
/// chain only).
pub fn cob_stack_trace_internal(chain: &[StackFrame], verbose: bool, count: i32) -> Vec<u8> {
    let mut out = Vec::new();

    // exit early: no module, or single head with no statement
    if chain.is_empty() {
        return out;
    }
    if chain.len() == 1 && chain[0].module_stmt == 0 {
        return out;
    }

    // first_entry computation for negative count (keep last -count entries)
    let mut first_entry: i32 = 0;
    if count < 0 {
        let mut total: i32 = 0;
        let mut k = 0;
        for _ in chain.iter() {
            if k == MAX_MODULE_ITERS {
                break;
            }
            k += 1;
            total += 1;
        }
        first_entry = total + count;
    }

    if verbose {
        out.push(b'\n');
    }

    let mut k = 0;
    let mut truncated_more = false;
    let mut i: i32 = 0;
    for m in chain.iter() {
        if i < first_entry {
            i += 1;
            continue;
        }
        if count > 0 && count == i {
            truncated_more = true;
            break;
        }
        out.push(b' ');
        if m.module_stmt != 0 && !m.module_sources.is_empty() {
            let source_file_num = cob_get_file_num(m.module_stmt) as usize;
            let source_line = cob_get_line_num(m.module_stmt);
            let empty: Vec<u8> = Vec::new();
            let source_file = m.module_sources.get(source_file_num).unwrap_or(&empty);
            let module_name = m.module_name.as_deref().unwrap_or(b"");

            if !verbose {
                out.extend_from_slice(module_name);
                out.extend_from_slice(b" at ");
                out.extend_from_slice(source_file);
                out.push(b':');
                ss_itoa_u10(&mut out, source_line as i32);
            } else if m.statement.is_none() && m.section_name.is_none() && m.paragraph_name.is_none()
            {
                // GC 3.1 output: no source location / no trace
                out.extend_from_slice(b"Last statement of ");
                if m.module_type == COB_MODULE_TYPE_FUNCTION {
                    out.extend_from_slice(b"FUNCTION ");
                }
                out.push(0x22);
                out.extend_from_slice(module_name);
                out.extend_from_slice(b"\" was at line ");
                ss_itoa_u10(&mut out, source_line as i32);
                out.extend_from_slice(b" of ");
                out.extend_from_slice(source_file);
            } else if m.section_name.is_none() && m.paragraph_name.is_none() {
                // data exists but no procedure defined in program
                out.extend_from_slice(b"Last statement of ");
                if m.module_type == COB_MODULE_TYPE_FUNCTION {
                    out.extend_from_slice(b"FUNCTION ");
                }
                out.push(0x22);
                out.extend_from_slice(module_name);
                out.extend_from_slice(b"\" was ");
                out.extend_from_slice(m.statement.as_deref().unwrap_or(b""));
                out.extend_from_slice(b" at line ");
                ss_itoa_u10(&mut out, source_line as i32);
                out.extend_from_slice(b" of ");
                out.extend_from_slice(source_file);
            } else {
                // common case: statement + procedure known
                out.extend_from_slice(b"Last statement of ");
                if m.module_type == COB_MODULE_TYPE_FUNCTION {
                    out.extend_from_slice(b"FUNCTION ");
                }
                out.push(0x22);
                out.extend_from_slice(module_name);
                out.extend_from_slice(b"\" was ");
                out.extend_from_slice(m.statement.as_deref().unwrap_or(b""));
            }
            out.extend_from_slice(&output_procedure_stack_entry(
                m.section_name.as_deref(),
                m.paragraph_name.as_deref(),
                source_file,
                source_line,
            ));
            // NOTE: the mod->frame_ptr PERFORM-stack walk is the runtime boundary (cob_frame_ext).
        } else if verbose {
            out.extend_from_slice(b"Last statement of ");
            if m.module_type == COB_MODULE_TYPE_FUNCTION {
                out.extend_from_slice(b"FUNCTION ");
            }
            out.push(0x22);
            out.extend_from_slice(m.module_name.as_deref().unwrap_or(b""));
            if m.statement.is_some() {
                out.extend_from_slice(b"\" was ");
                out.extend_from_slice(m.statement.as_deref().unwrap_or(b""));
            } else {
                out.extend_from_slice(b"\" unknown");
            }
        } else {
            out.extend_from_slice(m.module_name.as_deref().unwrap_or(b""));
            out.extend_from_slice(b" at unknown");
        }
        out.push(b'\n');

        k += 1;
        if k == MAX_MODULE_ITERS {
            out.extend_from_slice(b"max module iterations exceeded, possible broken chain\n");
            break;
        }
        i += 1;
    }

    // if (mod) -> more frames remained (count-limited)
    if truncated_more {
        out.push(b' ');
        out.extend_from_slice(MORE_STACK_FRAMES_MSGID);
        out.push(b'\n');
    }

    // NOTE: the verbose "Started by <argv[0]> ..." tail depends on cob_argc/cob_argv (boundary).
    out
}

/// Port of `common.c:cob_stack_trace` -- the verbose entry point. Early-out (empty output) when
/// there is no module chain; otherwise delegates to [`cob_stack_trace_internal`] with
/// `verbose = true, count = 0`. The C version writes to `target` (`FILE *`); here the bytes are
/// returned and the write is the boundary.
pub fn cob_stack_trace(chain: &[StackFrame]) -> Vec<u8> {
    if chain.is_empty() {
        return Vec::new();
    }
    cob_stack_trace_internal(chain, true, 0)
}

/// Port of `common.c:cob_backtrace` -- the GDB-like entry point with a frame `count` limit.
///
/// When the chain is empty the C version writes
/// `" No COBOL runtime elements on stack.\n"`; we return exactly those bytes. Otherwise delegates
/// to [`cob_stack_trace_internal`] with `verbose = false`. The write to `target` is the boundary.
pub fn cob_backtrace(chain: &[StackFrame], count: i32) -> Vec<u8> {
    if chain.is_empty() {
        let mut out = Vec::new();
        out.push(b' ');
        out.extend_from_slice(b"No COBOL runtime elements on stack.");
        out.push(b'\n');
        return out;
    }
    cob_stack_trace_internal(chain, false, count)
}

// ---------------------------------------------------------------------------
// Signal name / description table
// ---------------------------------------------------------------------------

/// Port of `common.c:more_stack_frames_msgid` -- the (untranslated) marker shown when a
/// `count`-limited stack trace omitted further frames.
pub const MORE_STACK_FRAMES_MSGID: &[u8] = b"(more COBOL runtime elements follow...)";

/// Port of `common.c:signal_msgid` -- the (untranslated) "signal" label.
pub const SIGNAL_MSGID: &[u8] = b"signal";

/// Port of `common.c:warning_msgid` -- the (untranslated) "warning: " prefix.
pub const WARNING_MSGID: &[u8] = b"warning: ";

/// Port of `common.c`'s `struct signal_table` row -- maps a signal number to its short name and
/// long description.
///
/// In C the table also carries `for_set`/`for_dump`/`unused` flags used only by the OS-bound
/// `cob_set_signal`/`cob_set_dump_signal` registration (not ported). We keep them for fidelity to
/// the source row but they are not consulted by any ported function.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SignalEntry {
    /// `signal_table.sig` -- the signal number (Linux/glibc value; see [`signals`]).
    pub sig: i32,
    /// `signal_table.for_set` -- registration flag (1=set if not SIG_IGN, 2=take direct control).
    pub for_set: i16,
    /// `signal_table.for_dump` -- set via `cob_set_dump_signal`.
    pub for_dump: i16,
    /// `signal_table.shortname` -- e.g. `"SIGSEGV"`.
    pub shortname: &'static str,
    /// `signal_table.description` -- the long message, filled by `cob_init_sig_descriptions`.
    pub description: &'static str,
}

/// Port of `common.c:signals[]` (with descriptions from `cob_init_sig_descriptions` already
/// applied, since we cannot delay-initialize a `const`).
///
/// **Platform boundary:** the C source guards each row with `#ifdef SIGxxx` and uses the macro
/// `SIGxxx`, whose numeric values are platform constants. This table uses the **Linux/glibc**
/// signal numbers (`SIGINT=2`, `SIGSEGV=11`, etc.) and includes every row that is defined on
/// Linux. `SIGEMT`, `SIGCLD` and friends that are not distinct on Linux/glibc are reflected as on
/// a standard glibc build (`SIGCLD == SIGCHLD == 17`, `SIGIO == SIGPOLL == 29`); `SIGEMT` is not
/// defined on Linux and is therefore omitted, matching the `#ifdef`-out behavior there. The final
/// `{-1, ..., "unknown"}` sentinel row is the lookup fallthrough and is **not** counted in
/// [`NUM_SIGNALS`].
pub const fn signals() -> &'static [SignalEntry] {
    &[
        SignalEntry { sig: 2,  for_set: 1, for_dump: 0, shortname: "SIGINT",  description: "interrupt from keyboard" },
        SignalEntry { sig: 1,  for_set: 1, for_dump: 0, shortname: "SIGHUP",  description: "hangup" },
        SignalEntry { sig: 3,  for_set: 1, for_dump: 0, shortname: "SIGQUIT", description: "quit" },
        SignalEntry { sig: 15, for_set: 1, for_dump: 0, shortname: "SIGTERM", description: "termination" },
        SignalEntry { sig: 13, for_set: 1, for_dump: 0, shortname: "SIGPIPE", description: "broken pipe" },
        SignalEntry { sig: 29, for_set: 1, for_dump: 0, shortname: "SIGIO",   description: "I/O signal" },
        SignalEntry { sig: 11, for_set: 2, for_dump: 1, shortname: "SIGSEGV", description: "attempt to reference invalid memory address" },
        SignalEntry { sig: 7,  for_set: 2, for_dump: 1, shortname: "SIGBUS",  description: "bus error" },
        SignalEntry { sig: 8,  for_set: 1, for_dump: 1, shortname: "SIGFPE",  description: "fatal arithmetic error" },
        SignalEntry { sig: 4,  for_set: 0, for_dump: 0, shortname: "SIGILL",  description: "illegal instruction" },
        SignalEntry { sig: 6,  for_set: 0, for_dump: 0, shortname: "SIGABRT", description: "abort" },
        SignalEntry { sig: 9,  for_set: 0, for_dump: 0, shortname: "SIGKILL", description: "process killed" },
        SignalEntry { sig: 14, for_set: 0, for_dump: 0, shortname: "SIGALRM", description: "alarm signal" },
        SignalEntry { sig: 19, for_set: 0, for_dump: 0, shortname: "SIGSTOP", description: "stop process" },
        SignalEntry { sig: 17, for_set: 0, for_dump: 0, shortname: "SIGCHLD", description: "child process stopped" },
        // sentinel "unknown" row -- NOT counted in NUM_SIGNALS
        SignalEntry { sig: -1, for_set: 0, for_dump: 0, shortname: "unknown", description: "unknown" },
    ]
}

/// Port of `common.c:NUM_SIGNALS` -- `(sizeof(signals)/sizeof(...)) - 1`, i.e. the number of real
/// rows, excluding the trailing `{-1, ..., "unknown"}` sentinel.
pub const fn num_signals() -> usize {
    signals().len() - 1
}

/// Port of `common.c:get_signal_entry` -- linear lookup of `sig` over the first [`num_signals`]
/// rows; on no match returns the trailing `"unknown"` sentinel (the C loop ends at index
/// `NUM_SIGNALS`, which is that sentinel row).
pub fn get_signal_entry(sig: i32) -> SignalEntry {
    let tbl = signals();
    for entry in &tbl[..num_signals()] {
        if entry.sig == sig {
            return *entry;
        }
    }
    // ended at the max: "unknown" sentinel row
    tbl[num_signals()]
}

/// Port of `common.c:cob_get_sig_name` -- the short name (`"SIGSEGV"`, ...) for `sig`, or
/// `"unknown"`.
pub fn cob_get_sig_name(sig: i32) -> &'static str {
    get_signal_entry(sig).shortname
}

/// Port of `common.c:cob_get_sig_description` -- the long description for `sig`. The C
/// LCOV-excluded guard returns `"unknown"` when the description is still NULL (pre-init); our
/// table is initialized so the description is always present, but we keep the empty-string guard
/// for parity.
pub fn cob_get_sig_description(sig: i32) -> &'static str {
    let entry = get_signal_entry(sig);
    if entry.description.is_empty() {
        return "unknown";
    }
    entry.description
}

/// Port of `common.c:cob_init_sig_descriptions` -- in C this lazily fills each row's
/// `description` (and the `*_msgid` strings) from the message catalog. Here the descriptions are
/// already baked into [`signals`] and the msgids are the [`SIGNAL_MSGID`] / [`WARNING_MSGID`] /
/// [`MORE_STACK_FRAMES_MSGID`] consts, so this is a no-op preserved for port-index parity (it
/// confirms every real row carries a non-empty description and short name).
pub fn cob_init_sig_descriptions() {
    // descriptions are statically initialized in signals(); nothing to fill at runtime.
    debug_assert!(signals()[..num_signals()]
        .iter()
        .all(|e| !e.description.is_empty() && !e.shortname.is_empty()));
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn pack(file: u32, line: u32) -> u32 {
        ((file & 0xFFF) << 20) | (line & 0xF_FFFF)
    }

    #[test]
    fn macros_split_packed_word() {
        let w = pack(3, 1234);
        assert_eq!(cob_get_file_num(w), 3);
        assert_eq!(cob_get_line_num(w), 1234);
        assert_eq!(cob_get_line_num(0), 0);
    }

    #[test]
    fn cob_get_source_line_skips_zero_stmt_frames() {
        let chain = vec![
            StackFrame { module_stmt: 0, ..Default::default() },
            StackFrame {
                module_stmt: pack(1, 42),
                module_sources: vec![b"a.cob".to_vec(), b"b.cob".to_vec()],
                ..Default::default()
            },
        ];
        let mut loc = SourceLocation::default();
        cob_get_source_line(&chain, &mut loc);
        assert_eq!(loc.file.as_deref(), Some(b"b.cob".as_ref()));
        assert_eq!(loc.line, 42);
    }

    #[test]
    fn cob_get_source_line_leaves_loc_when_no_usable_frame() {
        let chain = vec![StackFrame { module_stmt: 0, ..Default::default() }];
        let mut loc = SourceLocation { file: Some(b"keep.cob".to_vec()), line: 7 };
        cob_get_source_line(&chain, &mut loc);
        assert_eq!(loc.file.as_deref(), Some(b"keep.cob".as_ref()));
        assert_eq!(loc.line, 7);
    }

    #[test]
    fn set_source_location_overrides_from_head() {
        let head = StackFrame {
            module_stmt: pack(0, 99),
            module_sources: vec![b"main.cob".to_vec()],
            ..Default::default()
        };
        let current = SourceLocation { file: Some(b"old.cob".to_vec()), line: 1 };
        let loc = set_source_location(Some(&head), &current);
        assert_eq!(loc.file.as_deref(), Some(b"main.cob".as_ref()));
        assert_eq!(loc.line, 99);
    }

    #[test]
    fn output_source_location_formats_file_line() {
        let head = StackFrame {
            module_stmt: pack(0, 88),
            module_sources: vec![b"prog.cob".to_vec()],
            ..Default::default()
        };
        let out = output_source_location(Some(&head), &SourceLocation::default());
        assert_eq!(out, b"prog.cob:88: ");
    }

    #[test]
    fn output_source_location_no_line() {
        let current = SourceLocation { file: Some(b"prog.cob".to_vec()), line: 0 };
        let out = output_source_location(None, &current);
        assert_eq!(out, b"prog.cob: ");
    }

    #[test]
    fn output_source_location_empty_when_no_file() {
        let out = output_source_location(None, &SourceLocation::default());
        assert!(out.is_empty());
    }

    #[test]
    fn cob_set_location_builds_pair() {
        let loc = cob_set_location(b"x.cob", 55);
        assert_eq!(loc.file.as_deref(), Some(b"x.cob".as_ref()));
        assert_eq!(loc.line, 55);
    }

    #[test]
    fn procedure_stack_entry_both_names() {
        let out = output_procedure_stack_entry(Some(b"SEC"), Some(b"PAR"), b"f.cob", 10);
        assert_eq!(out, b"\n\tPAR OF SEC at f.cob:10");
    }

    #[test]
    fn procedure_stack_entry_section_only() {
        let out = output_procedure_stack_entry(Some(b"SEC"), None, b"f.cob", 5);
        assert_eq!(out, b"\n\tSEC at f.cob:5");
    }

    #[test]
    fn procedure_stack_entry_empty_when_no_names() {
        let out = output_procedure_stack_entry(None, None, b"f.cob", 5);
        assert!(out.is_empty());
    }

    #[test]
    fn backtrace_empty_chain_message() {
        let out = cob_backtrace(&[], 0);
        assert_eq!(out, b" No COBOL runtime elements on stack.\n");
    }

    #[test]
    fn backtrace_non_verbose_single_frame() {
        let chain = vec![
            StackFrame {
                module_stmt: pack(0, 12),
                module_sources: vec![b"m.cob".to_vec()],
                module_name: Some(b"MYPROG".to_vec()),
                ..Default::default()
            },
            // second frame so the "single head, no stmt" early-out doesn't fire
            StackFrame { module_stmt: 0, ..Default::default() },
        ];
        let out = cob_backtrace(&chain, 0);
        // first frame, GDB-like: " MYPROG at m.cob:12\n"
        assert!(out.starts_with(b" MYPROG at m.cob:12\n"), "got {:?}", out);
    }

    #[test]
    fn stack_trace_verbose_common_case() {
        let chain = vec![
            StackFrame {
                module_stmt: pack(0, 20),
                module_sources: vec![b"p.cob".to_vec()],
                module_name: Some(b"PROG".to_vec()),
                section_name: Some(b"SEC".to_vec()),
                paragraph_name: Some(b"PAR".to_vec()),
                statement: Some(b"MOVE".to_vec()),
                module_type: 0,
            },
            StackFrame { module_stmt: 0, ..Default::default() },
        ];
        let out = cob_stack_trace(&chain);
        let s = String::from_utf8_lossy(&out);
        assert!(s.contains("Last statement of \"PROG\" was MOVE"), "{s}");
        assert!(s.contains("PAR OF SEC at p.cob:20"), "{s}");
    }

    #[test]
    fn stack_trace_empty_and_single_zero_stmt() {
        assert!(cob_stack_trace(&[]).is_empty());
        let single = vec![StackFrame { module_stmt: 0, ..Default::default() }];
        assert!(cob_stack_trace(&single).is_empty());
    }

    #[test]
    fn signal_lookup_known_and_unknown() {
        assert_eq!(cob_get_sig_name(11), "SIGSEGV");
        assert_eq!(
            cob_get_sig_description(11),
            "attempt to reference invalid memory address"
        );
        assert_eq!(cob_get_sig_name(8), "SIGFPE");
        assert_eq!(cob_get_sig_description(8), "fatal arithmetic error");
        // unknown signal -> sentinel
        assert_eq!(cob_get_sig_name(9999), "unknown");
        assert_eq!(cob_get_sig_description(9999), "unknown");
    }

    #[test]
    fn num_signals_excludes_sentinel() {
        assert_eq!(num_signals(), signals().len() - 1);
        assert_eq!(signals()[num_signals()].sig, -1);
        assert_eq!(signals()[num_signals()].shortname, "unknown");
    }

    #[test]
    fn init_sig_descriptions_noop_ok() {
        cob_init_sig_descriptions();
        // every real row has a non-empty description + name
        for e in &signals()[..num_signals()] {
            assert!(!e.description.is_empty());
            assert!(!e.shortname.is_empty());
        }
    }

    #[test]
    fn flush_target_reports_std() {
        assert!(flush_target(true));
        assert!(!flush_target(false));
    }

    #[test]
    fn get_signal_entry_finds_segv_and_falls_back() {
        let e = get_signal_entry(11);
        assert_eq!(e.sig, 11);
        assert_eq!(e.shortname, "SIGSEGV");
        assert_eq!(e.description, "attempt to reference invalid memory address");
        // unknown signal -> the trailing sentinel row.
        let u = get_signal_entry(9999);
        assert_eq!(u.sig, -1);
        assert_eq!(u.shortname, "unknown");
    }

    #[test]
    fn cob_stack_trace_internal_formats_and_gates() {
        // empty chain / single zero-stmt head -> no output.
        assert!(cob_stack_trace_internal(&[], true, 0).is_empty());
        assert!(cob_stack_trace_internal(
            &[StackFrame { module_stmt: 0, ..Default::default() }],
            true,
            0
        )
        .is_empty());

        // one frame with a known statement + procedure: the verbose "Last statement of" form.
        let chain = vec![StackFrame {
            module_stmt: pack(0, 42),
            module_sources: vec![b"prog.cob".to_vec()],
            module_name: Some(b"MAIN".to_vec()),
            statement: Some(b"MOVE".to_vec()),
            paragraph_name: Some(b"P1".to_vec()),
            ..Default::default()
        }];
        let out = cob_stack_trace_internal(&chain, true, 0);
        let s = String::from_utf8_lossy(&out);
        assert!(out.starts_with(b"\n"), "verbose output leads with a newline");
        assert!(s.contains("Last statement of \"MAIN\" was MOVE"), "{s}");

        // non-verbose (backtrace) form: "<name> at <file>:<line>".
        let out = cob_stack_trace_internal(&chain, false, 0);
        let s = String::from_utf8_lossy(&out);
        assert!(s.contains("MAIN at prog.cob:42"), "{s}");
    }
}
