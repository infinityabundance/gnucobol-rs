//! Port of the runtime **TRACE family** of `libcob/common.c` -- the `cob_trace_*` /
//! `do_trace_statement` / `cob_trace_prep` / `cob_trace_print` functions that emit the
//! `READY TRACE` / `-debug` runtime trace lines (e.g. `Source: '...'`, `Program-Id:  ...`,
//! `Paragraph: ...`, `Program-Id  Statement  Line: NN`).
//!
//! ## What the C does, and how this port splits it
//! In libcob each `cob_trace_*` function reads the global `cobsetptr` (the runtime settings:
//! `cob_line_trace`, `cob_trace_format`, the open `cob_trace_file`, `cob_last_sfile`,
//! `cob_last_progid`) and `COB_MODULE_PTR` (the running module: `module_name`, `module_type`,
//! `section_name`, `paragraph_name`, `flag_debug_trace`), then **`fprintf`s the formatted line to
//! `cob_trace_file`** and `fflush`es it. That file write is the declared **IO boundary**.
//!
//! Following the house style ([`crate::common_exception::ExceptionState`] precedent) the global
//! mutable state is modelled **explicitly** as [`TraceState`] (no global statics, no `unsafe`,
//! panic-free). Each `cob_trace_*` is ported as a **pure formatter**: it returns the exact line
//! `Vec<u8>` it would have `fprintf`ed (or [`None`] when tracing is off / the prep gate fails),
//! and the caller performs the single declared `fputs`-then-`fflush` to the trace file. The
//! prefix lines emitted as a side effect of `cob_trace_prep` (the `Source:` / `Program-Id:` lines,
//! emitted only when the source-file or program-id *changes*) are returned alongside, because they
//! are themselves a pure function of the state plus the current location.
//!
//! `cob_ready_trace` / `cob_reset_trace` simply flip [`TraceState::cob_line_trace`].
//! `cob_check_trace_file` / `cob_new_trace_file` / `cob_open_logfile` are pure IO (open/attach a
//! file) with **no formatting**; they are ported as state transitions over an injected
//! [`TraceFile`] handle so the file op is a declared, testable boundary (see [`TraceState::cob_check_trace_file`]).

#![forbid(unsafe_code)]

/// `common.h:COB_MAX_NAMELEN` -- the field width used by the `%F` source-file trace directive.
pub const COB_MAX_NAMELEN: usize = 31;

/// `common.h:COB_MODULE_TRACE` -- `flag_debug_trace` bit enabling section/paragraph/entry tracing.
pub const COB_MODULE_TRACE: u32 = 2;
/// `common.h:COB_MODULE_TRACEALL` -- `flag_debug_trace` bit enabling per-statement tracing.
pub const COB_MODULE_TRACEALL: u32 = 4;

/// `common.h:COB_MODULE_TYPE_FUNCTION` -- `module_type` value marking a user-defined FUNCTION
/// (vs. a PROGRAM); selects the `Function-Id:` vs `Program-Id:` prefix.
pub const COB_MODULE_TYPE_FUNCTION: i32 = 1;

/// The runtime default of `cob_trace_format` (`common.c:493`,
/// `{"COB_TRACE_FORMAT", ... "%P %S Line: %L", ...}`).
pub const DEFAULT_TRACE_FORMAT: &str = "%P %S Line: %L";

/// The declared **IO boundary** for the trace file (the C `cobsetptr->cob_trace_file`).
///
/// In libcob this is a `FILE *` that is either an opened log file or `stderr`. Here it is reduced
/// to the only distinction the formatting cares about: whether a file is attached at all. Opening
/// a real file is the runtime's job; this port records *which* sink is attached so
/// [`TraceState::cob_check_trace_file`] / [`TraceState::cob_new_trace_file`] are faithful state
/// transitions rather than `fopen` calls.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TraceFile {
    /// No trace file attached yet (`cob_trace_file == NULL`).
    None,
    /// Attached to `stderr` (the fallback when no/failed filename).
    Stderr,
    /// Attached to a named log file (the string is the filename actually opened).
    File(Vec<u8>),
}

/// The explicit model of the trace-relevant global mutable state of `libcob` -- the union of the
/// fields the `cob_trace_*` family reads from `cobsetptr` and from `COB_MODULE_PTR`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceState {
    // ---- cobsetptr fields ----
    /// `cobsetptr->cob_line_trace` -- master on/off for tracing (flipped by `cob_ready_trace` /
    /// `cob_reset_trace`).
    pub cob_line_trace: bool,
    /// `cobsetptr->cob_trace_format` -- the `%P %S Line: %L`-style template consumed by
    /// [`Self::cob_trace_print`].
    pub cob_trace_format: Vec<u8>,
    /// `cobsetptr->cob_trace_file` -- the open trace sink (the IO boundary).
    pub cob_trace_file: TraceFile,
    /// `cobsetptr->cob_trace_filename` -- the configured trace filename (`None` == NULL).
    pub cob_trace_filename: Option<Vec<u8>>,
    /// `cobsetptr->external_trace_file` -- set when the trace file was supplied externally
    /// (affects `cob_new_trace_file`).
    pub external_trace_file: bool,
    /// `cob_last_sfile` -- the source file named by the most recent trace line (for the
    /// "only print `Source:` when it changes" gate).
    pub cob_last_sfile: Option<Vec<u8>>,
    /// `cob_last_progid` -- the program/function id named by the most recent trace line.
    pub cob_last_progid: Option<Vec<u8>>,
    /// `cob_source_file` -- the current source file (set by `cob_get_source_line`).
    pub cob_source_file: Option<Vec<u8>>,
    /// `cob_source_line` -- the current source line (set by `cob_get_source_line`).
    pub cob_source_line: u32,

    // ---- COB_MODULE_PTR fields ----
    /// `COB_MODULE_PTR->module_name` (`None` -> the `_("unknown")` fallback).
    pub module_name: Option<Vec<u8>>,
    /// `COB_MODULE_PTR->module_type` (compared against [`COB_MODULE_TYPE_FUNCTION`]).
    pub module_type: i32,
    /// `COB_MODULE_PTR->flag_debug_trace` -- the per-module `COB_MODULE_TRACE`/`TRACEALL` bits.
    pub flag_debug_trace: u32,
    /// `COB_MODULE_PTR->section_name` -- updated by [`Self::cob_trace_sect`] (compat write).
    pub section_name: Option<Vec<u8>>,
    /// `COB_MODULE_PTR->paragraph_name` -- updated by [`Self::cob_trace_para`] (compat write).
    pub paragraph_name: Option<Vec<u8>>,
}

impl Default for TraceState {
    fn default() -> Self {
        TraceState {
            cob_line_trace: false,
            cob_trace_format: DEFAULT_TRACE_FORMAT.as_bytes().to_vec(),
            cob_trace_file: TraceFile::None,
            cob_trace_filename: None,
            external_trace_file: false,
            cob_last_sfile: None,
            cob_last_progid: None,
            cob_source_file: None,
            cob_source_line: 0,
            module_name: None,
            module_type: 0,
            flag_debug_trace: 0,
            section_name: None,
            paragraph_name: None,
        }
    }
}

// ---------------------------------------------------------------------------
// printf field-width helpers (the `%-16s`, `%-21.21s`, `%6u`, ... directives)
// ---------------------------------------------------------------------------

/// `printf("%-*.*s")` style: left-justify `s` (truncated to `prec` bytes if `prec` is `Some`) in a
/// field of `width` bytes, space-padded on the right. A `prec` of `None` means "no precision"
/// (no truncation). A `width` of `0` means "no minimum field width".
fn fmt_left(s: &[u8], width: usize, prec: Option<usize>) -> Vec<u8> {
    let body: &[u8] = match prec {
        Some(p) if s.len() > p => &s[..p],
        _ => s,
    };
    let mut out = body.to_vec();
    while out.len() < width {
        out.push(b' ');
    }
    out
}

/// `printf("%6u")` style: right-justify a base-10 unsigned in a field of `width` bytes,
/// space-padded on the left (never truncated -- `%u` ignores precision-less width overflow).
fn fmt_right_u(value: u32, width: usize) -> Vec<u8> {
    let digits = value.to_string().into_bytes();
    let mut out = Vec::new();
    while out.len() + digits.len() < width {
        out.push(b' ');
    }
    out.extend_from_slice(&digits);
    out
}

/// `printf("%d")` / `%u` of a plain integer with no padding.
fn fmt_u(value: u32) -> Vec<u8> {
    value.to_string().into_bytes()
}

impl TraceState {
    /// Port of `common.c:cob_ready_trace` -- `READY TRACE`: turn line tracing on.
    pub fn cob_ready_trace(&mut self) {
        self.cob_line_trace = true;
    }

    /// Port of `common.c:cob_reset_trace` -- `RESET TRACE`: turn line tracing off.
    pub fn cob_reset_trace(&mut self) {
        self.cob_line_trace = false;
    }

    /// Port of `common.c:cob_open_logfile` -- pick the `fopen` mode for a trace/log filename.
    ///
    /// IO boundary: the C calls `fopen(filename, mode)`. This pure port returns the
    /// `(stripped_filename, mode)` pair (a leading `+` selects append and is stripped); the caller
    /// performs the open. `cob_unix_lf` selects binary (`"b"`) vs text mode.
    pub fn cob_open_logfile<'a>(filename: &'a [u8], cob_unix_lf: bool) -> (&'a [u8], &'static str) {
        let append = filename.first() == Some(&b'+');
        let name = if append { &filename[1..] } else { filename };
        let mode = match (cob_unix_lf, append) {
            (false, false) => "w",
            (false, true) => "a",
            (true, false) => "wb",
            (true, true) => "ab",
        };
        (name, mode)
    }

    /// Port of `common.c:cob_check_trace_file` -- ensure `cob_trace_file` is attached for writing.
    ///
    /// IO boundary: where the C `fopen`s the configured filename, this state transition records the
    /// resulting [`TraceFile`]. If a file is already attached it is a no-op. With a configured
    /// `cob_trace_filename` and `open_ok == true` it attaches that file; a failed open
    /// (`open_ok == false`) clears the filename and falls back to `stderr`; with no filename it uses
    /// `stderr` directly.
    pub fn cob_check_trace_file(&mut self, open_ok: bool) {
        if self.cob_trace_file != TraceFile::None {
            return;
        }
        if let Some(name) = self.cob_trace_filename.clone() {
            if open_ok {
                self.cob_trace_file = TraceFile::File(name);
            } else {
                self.cob_trace_filename = None;
                self.cob_trace_file = TraceFile::Stderr;
            }
        } else {
            self.cob_trace_file = TraceFile::Stderr;
        }
    }

    /// Port of `common.c:cob_new_trace_file` -- close the current trace file (if a real one) and
    /// re-attach via [`Self::cob_check_trace_file`].
    ///
    /// IO boundary: the C `fclose`s the old `FILE *`; here that is the implicit drop of the old
    /// [`TraceFile`]. When the current file is NULL / external / `stderr` it just (re)opens; a real
    /// file is "closed" (replaced with `None`) before re-opening. The C also re-points any other
    /// sinks (`cob_display_print_file`, `cob_dump_file`, `cob_debug_file`) that aliased the old
    /// file; those sinks live outside this trace module, so that re-pointing is the caller's job.
    pub fn cob_new_trace_file(&mut self, open_ok: bool) {
        if self.cob_trace_file == TraceFile::None
            || self.external_trace_file
            || self.cob_trace_file == TraceFile::Stderr
        {
            self.cob_trace_file = TraceFile::None;
            self.cob_check_trace_file(open_ok);
            return;
        }
        // fclose (real file) -> NULL, then re-open
        self.cob_trace_file = TraceFile::None;
        self.cob_check_trace_file(open_ok);
    }

    /// Port of `common.c:cob_trace_prep` -- the shared prefix-emitter for the new (3.2+) trace
    /// functions.
    ///
    /// Returns `Err(())` mirroring the C `return 1` "gate failed" (no trace file could be attached);
    /// every caller bails on that. On `Ok(prefix)` the `prefix` is the (possibly empty) sequence of
    /// bytes the C `fprintf`ed before the body line: a `Source: '<file>'\n` line iff the source file
    /// changed since the last trace, and a `Program-Id:  <id>\n` / `Function-Id: <id>\n` line iff the
    /// program/function id changed. It also performs the C's side effects of updating
    /// `cob_last_sfile` / `cob_last_progid` and (via the injected `open_ok`) ensuring the trace file.
    pub fn cob_trace_prep(&mut self, open_ok: bool) -> Result<Vec<u8>, ()> {
        if self.cob_trace_file == TraceFile::None {
            self.cob_check_trace_file(open_ok);
            if self.cob_trace_file == TraceFile::None {
                return Err(());
            }
        }
        // cob_get_source_line () -- already reflected in cob_source_file / cob_source_line.
        let mut out = Vec::new();

        if let Some(src) = self.cob_source_file.clone() {
            if self.cob_last_sfile.as_deref() != Some(src.as_slice()) {
                self.cob_last_sfile = Some(src.clone());
                out.extend_from_slice(b"Source: '");
                out.extend_from_slice(&src);
                out.extend_from_slice(b"'\n");
            }
        }

        let s: Vec<u8> = match &self.module_name {
            Some(n) => n.clone(),
            None => b"unknown".to_vec(),
        };
        if self.cob_last_progid.as_deref() != Some(s.as_slice()) {
            self.cob_last_progid = Some(s.clone());
            if self.module_type == COB_MODULE_TYPE_FUNCTION {
                out.extend_from_slice(b"Function-Id: ");
            } else {
                out.extend_from_slice(b"Program-Id:  ");
            }
            out.extend_from_slice(&s);
            out.push(b'\n');
        }
        Ok(out)
    }

    /// Port of `common.c:cob_trace_print` -- render the `cob_trace_format` template for body `val`,
    /// returning the formatted line (the trailing `'\n'` included). This is the pure builder behind
    /// the single `fputs`-then-`fflush` the C performs.
    ///
    /// The directives (`common.c:2800`-`2852`) are, with `i != last_pos` meaning "not the final
    /// char of the format" (which is what enables/disables the field-width padding):
    /// `%P`/`%p` program-or-function id prefix, `%I`/`%i` bare id, `%L`/`%l` line (`%6u`),
    /// `%S`/`%s` the body `val` (`%-42.42s`), `%F`/`%f` source file (`Source: %-31.31s`).
    /// An unknown directive prints `%` then the char verbatim; a `%` at end-of-string falls through.
    pub fn cob_trace_print(&self, val: &[u8]) -> Vec<u8> {
        let fmt = &self.cob_trace_format;
        let progid: &[u8] = match &self.cob_last_progid {
            Some(p) => p,
            None => b"",
        };
        let sfile: &[u8] = match &self.cob_last_sfile {
            Some(f) => f,
            None => b"",
        };
        let last_pos = fmt.len().wrapping_sub(1);
        let mut out = Vec::new();
        let mut i = 0usize;
        while i < fmt.len() {
            if fmt[i] == 0x25 {
                // '%'
                if i + 1 >= fmt.len() {
                    // trailing '%': the C reads fmt[i+1] == '\0', hits default, prints "%\0"
                    // but the loop condition then stops; faithfully we emit just '%' then stop.
                    out.push(0x25);
                    break;
                }
                i += 1;
                let last = i == last_pos;
                match fmt[i] {
                    b'P' | b'p' => {
                        if self.module_type == COB_MODULE_TYPE_FUNCTION {
                            out.extend_from_slice(b"Function-Id: ");
                        } else {
                            out.extend_from_slice(b"Program-Id:  ");
                        }
                        if last {
                            out.extend_from_slice(progid);
                        } else {
                            out.extend_from_slice(&fmt_left(progid, 16, None));
                        }
                    }
                    b'I' | b'i' => out.extend_from_slice(progid),
                    b'L' | b'l' => out.extend_from_slice(&fmt_right_u(self.cob_source_line, 6)),
                    b'S' | b's' => {
                        if last {
                            out.extend_from_slice(val);
                        } else {
                            out.extend_from_slice(&fmt_left(val, 42, Some(42)));
                        }
                    }
                    b'F' | b'f' => {
                        out.extend_from_slice(b"Source: ");
                        if last {
                            out.extend_from_slice(sfile);
                        } else {
                            out.extend_from_slice(&fmt_left(sfile, COB_MAX_NAMELEN, Some(COB_MAX_NAMELEN)));
                        }
                    }
                    other => {
                        out.push(0x25);
                        out.push(other);
                    }
                }
            } else {
                out.push(fmt[i]);
            }
            i += 1;
        }
        out.push(b'\n');
        out
    }

    /// Port of `common.c:cob_trace_section` (pre-4.x backward-compat). Returns the formatted trace
    /// output for a `para`/`source`/`line` location, or [`None`] when `cob_line_trace` is off.
    ///
    /// Emits a `Source:     '<source>'\n` line when `source` changes (updating `cob_last_sfile`),
    /// then a `Program-Id: <name-%-16s> ` prefix and either `<para-%-34.34s>Line: <d>\n` (when a
    /// line number is known) or `<para>\n`. `module_line` supplies `COB_GET_LINE_NUM(module_stmt)`
    /// used when `line == 0`. `open_ok` feeds the trace-file IO boundary.
    pub fn cob_trace_section(
        &mut self,
        para: &[u8],
        source: Option<&[u8]>,
        line: i32,
        module_line: i32,
        open_ok: bool,
    ) -> Option<Vec<u8>> {
        if !self.cob_line_trace {
            return None;
        }
        let mut pline = line;
        if self.cob_trace_file == TraceFile::None {
            self.cob_check_trace_file(open_ok);
        }
        let mut out = Vec::new();
        if let Some(src) = source {
            if self.cob_last_sfile.as_deref() != Some(src) {
                self.cob_last_sfile = Some(src.to_vec());
                out.extend_from_slice(b"Source:     '");
                out.extend_from_slice(src);
                out.extend_from_slice(b"'\n");
            }
        }
        let s: Vec<u8> = match &self.module_name {
            Some(n) => {
                if pline == 0 && module_line != 0 {
                    pline = module_line;
                }
                n.clone()
            }
            None => b"unknown".to_vec(),
        };
        out.extend_from_slice(b"Program-Id: ");
        out.extend_from_slice(&fmt_left(&s, 16, None));
        out.push(b' ');
        if pline != 0 {
            out.extend_from_slice(&fmt_left(para, 34, Some(34)));
            out.extend_from_slice(b"Line: ");
            out.extend_from_slice(&fmt_u(pline as u32));
            out.push(b'\n');
        } else {
            out.extend_from_slice(para);
            out.push(b'\n');
        }
        Some(out)
    }

    /// Port of `common.c:cob_trace_sect` -- trace a SECTION. Always records `name` into
    /// `section_name` (the pre-3.2 compat write). When `cob_line_trace` and the module's
    /// `COB_MODULE_TRACE` bit are both set and prep succeeds and `name` is non-NULL, returns the
    /// prep prefix followed by the `cob_trace_print` of `"  Section: <name>"`.
    pub fn cob_trace_sect(&mut self, name: Option<&[u8]>, open_ok: bool) -> Option<Vec<u8>> {
        self.section_name = name.map(|n| n.to_vec());
        if self.cob_line_trace && (self.flag_debug_trace & COB_MODULE_TRACE) != 0 {
            let prefix = self.cob_trace_prep(open_ok).ok()?;
            let name = name?;
            let mut val = b"  Section: ".to_vec();
            val.extend_from_slice(name);
            let mut out = prefix;
            out.extend_from_slice(&self.cob_trace_print(&val));
            return Some(out);
        }
        None
    }

    /// Port of `common.c:cob_trace_para` -- trace a PARAGRAPH. Records `name` into `paragraph_name`,
    /// then (gated like [`Self::cob_trace_sect`]) returns the prep prefix + the printed
    /// `"Paragraph: <name>"` line.
    pub fn cob_trace_para(&mut self, name: Option<&[u8]>, open_ok: bool) -> Option<Vec<u8>> {
        self.paragraph_name = name.map(|n| n.to_vec());
        if self.cob_line_trace && (self.flag_debug_trace & COB_MODULE_TRACE) != 0 {
            let prefix = self.cob_trace_prep(open_ok).ok()?;
            let name = name?;
            let mut val = b"Paragraph: ".to_vec();
            val.extend_from_slice(name);
            let mut out = prefix;
            out.extend_from_slice(&self.cob_trace_print(&val));
            return Some(out);
        }
        None
    }

    /// Port of `common.c:cob_trace_entry` -- trace a program/function ENTRY. Gated like
    /// [`Self::cob_trace_sect`]; returns the prep prefix + the printed `"    Entry: <name>"` line.
    pub fn cob_trace_entry(&mut self, name: Option<&[u8]>, open_ok: bool) -> Option<Vec<u8>> {
        if self.cob_line_trace && (self.flag_debug_trace & COB_MODULE_TRACE) != 0 {
            let prefix = self.cob_trace_prep(open_ok).ok()?;
            let name = name?;
            let mut val = b"    Entry: ".to_vec();
            val.extend_from_slice(name);
            let mut out = prefix;
            out.extend_from_slice(&self.cob_trace_print(&val));
            return Some(out);
        }
        None
    }

    /// Port of `common.c:cob_trace_exit` -- trace a program/function EXIT. Gated like
    /// [`Self::cob_trace_sect`]; returns the prep prefix + the printed `"     Exit: <name>"` line.
    pub fn cob_trace_exit(&mut self, name: Option<&[u8]>, open_ok: bool) -> Option<Vec<u8>> {
        if self.cob_line_trace && (self.flag_debug_trace & COB_MODULE_TRACE) != 0 {
            let prefix = self.cob_trace_prep(open_ok).ok()?;
            let name = name?;
            let mut val = b"     Exit: ".to_vec();
            val.extend_from_slice(name);
            let mut out = prefix;
            out.extend_from_slice(&self.cob_trace_print(&val));
            return Some(out);
        }
        None
    }

    /// Port of `common.c:do_trace_statement` -- the shared per-statement tracer. `stmt_name` is
    /// `cob_statement_name[stmt]` (the resolved statement text, or `None` for `STMT_UNKNOWN` which
    /// the C renders as `_("unknown")`). Returns the prep prefix + the printed `"           <stmt>"`
    /// line, or [`None`] when prep gates out.
    pub fn do_trace_statement(&mut self, stmt_name: Option<&[u8]>, open_ok: bool) -> Option<Vec<u8>> {
        let prefix = self.cob_trace_prep(open_ok).ok()?;
        let mut val = b"           ".to_vec();
        match stmt_name {
            Some(n) => val.extend_from_slice(n),
            None => val.extend_from_slice(b"unknown"),
        }
        let mut out = prefix;
        out.extend_from_slice(&self.cob_trace_print(&val));
        Some(out)
    }

    /// Port of `common.c:cob_trace_stmt` (pre-3.2 compat). Records the resolved statement (`stmt`)
    /// into the module's current statement -- modelled here by leaving the caller to set it -- then,
    /// when `cob_line_trace` and the `COB_MODULE_TRACEALL` bit are set, defers to
    /// [`Self::do_trace_statement`]. `stmt_name` is the already-resolved statement text.
    ///
    /// (The C's `get_stmt_from_name` / `COB_MODULE_PTR->statement` write is the statement-name
    /// resolution + module-state update, both of which live outside this trace module; the caller
    /// supplies the resolved `stmt_name`.)
    pub fn cob_trace_stmt(&mut self, stmt_name: Option<&[u8]>, open_ok: bool) -> Option<Vec<u8>> {
        if self.cob_line_trace && (self.flag_debug_trace & COB_MODULE_TRACEALL) != 0 {
            return self.do_trace_statement(stmt_name, open_ok);
        }
        None
    }

    /// Port of `common.c:cob_trace_statement` (3.2+) -- trace a single statement by its resolved
    /// `cob_statement` text. When `cob_line_trace` and `COB_MODULE_TRACEALL` are set, defers to
    /// [`Self::do_trace_statement`]; otherwise [`None`].
    pub fn cob_trace_statement(&mut self, stmt_name: Option<&[u8]>, open_ok: bool) -> Option<Vec<u8>> {
        if self.cob_line_trace && (self.flag_debug_trace & COB_MODULE_TRACEALL) != 0 {
            return self.do_trace_statement(stmt_name, open_ok);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fixed, fully-populated state: line trace on, both trace bits set, a known module/source.
    fn ready_state() -> TraceState {
        TraceState {
            cob_line_trace: true,
            cob_trace_format: DEFAULT_TRACE_FORMAT.as_bytes().to_vec(),
            cob_trace_file: TraceFile::Stderr, // already attached -> prep won't gate
            cob_trace_filename: None,
            external_trace_file: false,
            cob_last_sfile: None,
            cob_last_progid: None,
            cob_source_file: Some(b"prog.cob".to_vec()),
            cob_source_line: 42,
            module_name: Some(b"MAIN".to_vec()),
            module_type: 0,
            flag_debug_trace: COB_MODULE_TRACE | COB_MODULE_TRACEALL,
            section_name: None,
            paragraph_name: None,
        }
    }

    #[test]
    fn ready_and_reset_flip_flag() {
        let mut s = TraceState::default();
        assert!(!s.cob_line_trace);
        s.cob_ready_trace();
        assert!(s.cob_line_trace);
        s.cob_reset_trace();
        assert!(!s.cob_line_trace);
    }

    #[test]
    fn open_logfile_modes() {
        assert_eq!(TraceState::cob_open_logfile(b"t.log", false), (&b"t.log"[..], "w"));
        assert_eq!(TraceState::cob_open_logfile(b"+t.log", false), (&b"t.log"[..], "a"));
        assert_eq!(TraceState::cob_open_logfile(b"t.log", true), (&b"t.log"[..], "wb"));
        assert_eq!(TraceState::cob_open_logfile(b"+t.log", true), (&b"t.log"[..], "ab"));
    }

    #[test]
    fn check_trace_file_transitions() {
        // already attached -> no-op
        let mut s = TraceState::default();
        s.cob_trace_file = TraceFile::Stderr;
        s.cob_check_trace_file(true);
        assert_eq!(s.cob_trace_file, TraceFile::Stderr);

        // filename + ok -> File
        let mut s = TraceState::default();
        s.cob_trace_filename = Some(b"t.log".to_vec());
        s.cob_check_trace_file(true);
        assert_eq!(s.cob_trace_file, TraceFile::File(b"t.log".to_vec()));

        // filename + failed open -> filename cleared, stderr
        let mut s = TraceState::default();
        s.cob_trace_filename = Some(b"t.log".to_vec());
        s.cob_check_trace_file(false);
        assert_eq!(s.cob_trace_file, TraceFile::Stderr);
        assert_eq!(s.cob_trace_filename, None);

        // no filename -> stderr
        let mut s = TraceState::default();
        s.cob_check_trace_file(true);
        assert_eq!(s.cob_trace_file, TraceFile::Stderr);
    }

    #[test]
    fn prep_emits_source_and_progid_once() {
        let mut s = ready_state();
        // first prep: both Source and Program-Id lines (progid not function)
        let p = s.cob_trace_prep(true).unwrap();
        assert_eq!(p, b"Source: 'prog.cob'\nProgram-Id:  MAIN\n".to_vec());
        // second prep: nothing changed -> empty prefix
        let p2 = s.cob_trace_prep(true).unwrap();
        assert_eq!(p2, Vec::<u8>::new());
    }

    #[test]
    fn prep_function_prefix() {
        let mut s = ready_state();
        s.module_type = COB_MODULE_TYPE_FUNCTION;
        let p = s.cob_trace_prep(true).unwrap();
        assert_eq!(p, b"Source: 'prog.cob'\nFunction-Id: MAIN\n".to_vec());
    }

    #[test]
    fn prep_gates_when_no_file() {
        let mut s = ready_state();
        s.cob_trace_file = TraceFile::None;
        s.cob_trace_filename = None;
        // open_ok=false with no filename still attaches stderr (check_trace_file), so prep succeeds.
        assert!(s.cob_trace_prep(false).is_ok());
    }

    #[test]
    fn trace_print_default_format() {
        let mut s = ready_state();
        s.cob_last_progid = Some(b"MAIN".to_vec());
        s.cob_last_sfile = Some(b"prog.cob".to_vec());
        // format "%P %S Line: %L": %P not last -> "Program-Id:  " + MAIN padded to 16;
        // %S not last -> val padded/truncated to 42; %L is last char -> "%6u" of 42.
        let line = s.cob_trace_print(b"Paragraph: P1");
        let mut expect = Vec::new();
        expect.extend_from_slice(b"Program-Id:  ");
        expect.extend_from_slice(&fmt_left(b"MAIN", 16, None)); // "MAIN" + 12 spaces
        expect.push(b' ');
        expect.extend_from_slice(&fmt_left(b"Paragraph: P1", 42, Some(42)));
        expect.extend_from_slice(b" Line: ");
        expect.extend_from_slice(b"    42"); // %6u
        expect.push(b'\n');
        assert_eq!(line, expect);
    }

    #[test]
    fn trace_print_directives_f_i() {
        let mut s = ready_state();
        s.cob_trace_format = b"%I@%F".to_vec();
        s.cob_last_progid = Some(b"PID".to_vec());
        s.cob_last_sfile = Some(b"a.cob".to_vec());
        // %I bare progid, '@', %F last -> "Source: a.cob"
        let line = s.cob_trace_print(b"x");
        assert_eq!(line, b"PID@Source: a.cob\n".to_vec());
    }

    #[test]
    fn trace_sect_records_and_formats() {
        let mut s = ready_state();
        let out = s.cob_trace_sect(Some(b"SEC1"), true).unwrap();
        assert_eq!(s.section_name, Some(b"SEC1".to_vec()));
        // prefix (Source + Program-Id) then the printed "  Section: SEC1" body
        let body = {
            let t = s.clone();
            t.cob_trace_print(b"  Section: SEC1")
        };
        assert!(out.ends_with(&body));
        assert!(out.starts_with(b"Source: 'prog.cob'\n"));
    }

    #[test]
    fn trace_sect_off_records_name_only() {
        let mut s = ready_state();
        s.cob_line_trace = false;
        assert_eq!(s.cob_trace_sect(Some(b"SEC1"), true), None);
        assert_eq!(s.section_name, Some(b"SEC1".to_vec()));
    }

    #[test]
    fn trace_section_compat_with_line() {
        let mut s = ready_state();
        let out = s
            .cob_trace_section(b"P1", Some(b"x.cob"), 7, 0, true)
            .unwrap();
        let mut expect = Vec::new();
        expect.extend_from_slice(b"Source:     'x.cob'\n");
        expect.extend_from_slice(b"Program-Id: ");
        expect.extend_from_slice(&fmt_left(b"MAIN", 16, None));
        expect.push(b' ');
        expect.extend_from_slice(&fmt_left(b"P1", 34, Some(34)));
        expect.extend_from_slice(b"Line: 7\n");
        assert_eq!(out, expect);
    }

    #[test]
    fn trace_section_no_line() {
        let mut s = ready_state();
        let out = s.cob_trace_section(b"P1", None, 0, 0, true).unwrap();
        let mut expect = Vec::new();
        expect.extend_from_slice(b"Program-Id: ");
        expect.extend_from_slice(&fmt_left(b"MAIN", 16, None));
        expect.push(b' ');
        expect.extend_from_slice(b"P1\n");
        assert_eq!(out, expect);
    }

    #[test]
    fn statement_trace_gated_by_traceall() {
        let mut s = ready_state();
        s.flag_debug_trace = COB_MODULE_TRACE; // TRACEALL missing
        assert_eq!(s.cob_trace_statement(Some(b"MOVE"), true), None);
        s.flag_debug_trace = COB_MODULE_TRACEALL;
        let out = s.cob_trace_statement(Some(b"MOVE"), true).unwrap();
        assert!(out.contains(&b'\n'));
        // body is "           MOVE"
        let body = {
            let t = s.clone();
            t.cob_trace_print(b"           MOVE")
        };
        assert!(out.ends_with(&body));
    }

    #[test]
    fn statement_unknown_renders_unknown() {
        let mut s = ready_state();
        let out = s.do_trace_statement(None, true).unwrap();
        let body = {
            let t = s.clone();
            t.cob_trace_print(b"           unknown")
        };
        assert!(out.ends_with(&body));
    }

    #[test]
    fn trace_para_records_and_formats() {
        let mut s = ready_state();
        let out = s.cob_trace_para(Some(b"P1"), true).unwrap();
        assert_eq!(s.paragraph_name, Some(b"P1".to_vec()));
        // prep prefix (Source + Program-Id) then the printed "Paragraph: P1" body.
        assert!(out.starts_with(b"Source: 'prog.cob'\n"));
        let body = {
            let t = s.clone();
            t.cob_trace_print(b"Paragraph: P1")
        };
        assert!(out.ends_with(&body));
        // line trace off -> still records the name but returns None.
        let mut s = ready_state();
        s.cob_line_trace = false;
        assert_eq!(s.cob_trace_para(Some(b"P2"), true), None);
        assert_eq!(s.paragraph_name, Some(b"P2".to_vec()));
    }

    #[test]
    fn trace_entry_formats_entry_line() {
        let mut s = ready_state();
        let out = s.cob_trace_entry(Some(b"FUNC1"), true).unwrap();
        assert!(out.starts_with(b"Source: 'prog.cob'\n"));
        let body = {
            let t = s.clone();
            t.cob_trace_print(b"    Entry: FUNC1")
        };
        assert!(out.ends_with(&body));
        // gated off when the TRACE module bit is missing.
        let mut s = ready_state();
        s.flag_debug_trace = 0;
        assert_eq!(s.cob_trace_entry(Some(b"FUNC1"), true), None);
    }

    #[test]
    fn trace_stmt_gated_by_traceall() {
        let mut s = ready_state();
        // TRACEALL present (ready_state sets both bits): defers to do_trace_statement.
        let out = s.cob_trace_stmt(Some(b"MOVE"), true).unwrap();
        let body = {
            let t = s.clone();
            t.cob_trace_print(b"           MOVE")
        };
        assert!(out.ends_with(&body));
        // TRACEALL missing -> None.
        let mut s = ready_state();
        s.flag_debug_trace = COB_MODULE_TRACE;
        assert_eq!(s.cob_trace_stmt(Some(b"MOVE"), true), None);
    }
}
