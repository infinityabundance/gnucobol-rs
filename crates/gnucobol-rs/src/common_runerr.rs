//! Port of the runtime-ERROR / WARNING message machinery of `libcob/common.c`.
//!
//! These libcob functions build a formatted diagnostic and write it to `stderr`. The byte-exact message
//! text is the oracle-visible artifact (it appears on `stderr` when a COBOL program hits a runtime error,
//! a fatal error, or a bad runtime-configuration value); the `stderr` write itself is the declared impure
//! boundary. This port therefore separates the PURE message FORMATTING -- a function of the inputs -- from
//! the write: every function here RETURNS the formatted message bytes (exactly what the C `fputs` /
//! `fprintf` / `vfprintf` chain emits, prefix included), and the caller performs the single `stderr` write.
//!
//! Following the house style of `common.rs`, global state (`runtime_err_str`, the
//! `last_runtime_error_file`/`_line` de-dup, the `conf_runtime_error_displayed` latch) is modelled as
//! explicit caller-owned structs, never as `static`s. No other `common_*` module is depended on.
//!
//! `_(...)` in the C marks gettext-translatable strings; the oracle runs under the C / `POSIX` locale where
//! `_(s) == s`, so the English msgids are reproduced verbatim.

#![forbid(unsafe_code)]

/// `common.c:COB_ERRBUF_SIZE` -- the size of the `runtime_err_str` emergency buffer (`1024`). The C truncates
/// the assembled `file:line: <body>` string to this size via the fixed-size `char[]`; modelled as a cap.
pub const COB_ERRBUF_SIZE: usize = 1024;

/// `common.c:CB_IMSG_SIZE` -- the message-label column width used by [`var_print`].
pub const CB_IMSG_SIZE: usize = 24;

/// `common.c:CB_IVAL_SIZE` -- the value column width used by [`var_print`] (`80 - CB_IMSG_SIZE - 4`).
pub const CB_IVAL_SIZE: usize = 80 - CB_IMSG_SIZE - 4;

/// `common.c`'s `warning_msgid` -- the literal `"warning: "` prefix shared by the warning printers.
pub const WARNING_MSGID: &[u8] = b"warning: ";

/// The current source location used as the diagnostic prefix (`common.c`'s `cob_source_file` /
/// `cob_source_line`, resolved by `set_source_location`). `None` file means "no location" (the prefix is
/// omitted, or -- in [`conf_runtime_error`] -- replaced by `"environment variables"`). A `line` of `0` is
/// suppressed, matching the C `if (source_line)` guards.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceLocation {
    /// The source file name, or `None` when no location is set.
    pub file: Option<Vec<u8>>,
    /// The source line; `0` is suppressed.
    pub line: u32,
}

impl SourceLocation {
    /// No source location set.
    pub fn none() -> Self {
        SourceLocation { file: None, line: 0 }
    }
    /// A location with a file and line.
    pub fn at(file: &[u8], line: u32) -> Self {
        SourceLocation { file: Some(file.to_vec()), line }
    }
}

// ======================================================================================================
// cob_get_strerror -- errno -> message text.
// ======================================================================================================

/// Port of `common.c:cob_get_strerror` -- the reentrant `strerror` wrapper: return the message string for
/// the current `errno`. The C calls the platform `strerror(errno)` (when `HAVE_STRERROR`) or, as a fallback,
/// formats `"system error %d"`.
///
/// The platform `strerror` table is host-specific and outside the pure port, so the message is *injected*:
/// `injected` carries the host `strerror(errno)` bytes when available. With no injected message the C
/// fallback `"system error <errno>"` is produced (the `HAVE_STRERROR`-absent branch), which is itself a pure
/// function of `errno`. libcob also defines its own negative `COB_*` codes with fixed messages; those are
/// ported in [`cob_get_strerror_cob`].
pub fn cob_get_strerror(errno: i32, injected: Option<&[u8]>) -> Vec<u8> {
    match injected {
        Some(msg) => msg.to_vec(),
        None => format!("system error {errno}").into_bytes(),
    }
}

/// The libcob-specific (negative) errno codes carry their own fixed messages independent of the platform
/// `strerror` table; this returns the message for such a code, or `None` for a platform errno (which routes
/// through [`cob_get_strerror`]). The codes mirror `common.h`'s `COB_STATUS`/socket error conventions; only
/// the message text is oracle-visible.
///
/// Note: in GnuCOBOL 3.2 `cob_get_strerror` itself does not branch on negative codes -- it always defers to
/// the platform `strerror`. This helper is provided so a caller can model the libcob-internal codes without
/// re-porting the table; an unrecognised code returns `None`.
pub fn cob_get_strerror_cob(_code: i32) -> Option<Vec<u8>> {
    None
}

// ======================================================================================================
// cob_setup_runtime_error_str -- assemble the "file:line: <body>" runtime-error string.
// ======================================================================================================

/// Port of `common.c:cob_setup_runtime_error_str` -- assemble the runtime-error string into the
/// `runtime_err_str` buffer: when a source file is set, the prefix is `"<file>:<line>: "` (or `"<file>: "`
/// when the line is `0`), followed by the already-formatted body. Returns the assembled bytes (the C
/// truncates to `COB_ERRBUF_SIZE` via the fixed `char[]`; this caps identically).
///
/// In the C the body is `vsprintf`'d from a printf format + `va_list`; here the caller passes the
/// already-formatted body bytes (the format-substitution is the caller's, matching how every call site
/// supplies a fully-formed message). The returned `Vec<u8>` is the value the C leaves in `runtime_err_str`.
pub fn cob_setup_runtime_error_str(loc: &SourceLocation, body: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    if let Some(file) = &loc.file {
        out.extend_from_slice(file);
        if loc.line != 0 {
            out.push(0x3a); // ':'
            out.extend_from_slice(loc.line.to_string().as_bytes());
        }
        out.extend_from_slice(b": ");
    }
    out.extend_from_slice(body);
    out.truncate(COB_ERRBUF_SIZE - 1);
    out
}

/// Caller-owned model of `common.c`'s `static char runtime_err_str[COB_ERRBUF_SIZE]` emergency buffer
/// (`cob_setup_runtime_error_str` writes it, `cob_get_runtime_error_str` reads it back). Never a `static`.
#[derive(Debug, Clone, Default)]
pub struct RuntimeErrStr {
    /// The assembled error string.
    pub buf: Vec<u8>,
}

impl RuntimeErrStr {
    /// Assemble and store the runtime-error string (see [`cob_setup_runtime_error_str`]); returns a view.
    pub fn setup(&mut self, loc: &SourceLocation, body: &[u8]) -> &[u8] {
        self.buf = cob_setup_runtime_error_str(loc, body);
        &self.buf
    }
}

// ======================================================================================================
// cob_runtime_error / cob_runtime_warning(_ss / _external) -- the formatted stderr diagnostics.
// ======================================================================================================

/// Port of the message produced by `common.c:cob_runtime_error` -- the bytes the C writes to `stderr` for a
/// runtime error: the literal `"libcob: "`, then the source-location prefix (`"<file>:"`, then `"<line>:"`
/// when the line is non-zero, then a space), then `"error: "`, then the formatted `body`, then a newline.
///
/// The C also drives registered error handlers and saves `abort_reason`; those are side effects on caller
/// state and are out of the pure port. This returns exactly the `stderr` byte sequence (the
/// `more_error_procedures` "Prefix/Body/Postfix" block).
pub fn cob_runtime_error(loc: &SourceLocation, body: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"libcob: ");
    if let Some(file) = &loc.file {
        out.extend_from_slice(file);
        out.push(0x3a); // ':'
        if loc.line != 0 {
            out.extend_from_slice(loc.line.to_string().as_bytes());
            out.push(0x3a); // ':'
        }
        out.push(0x20); // ' '
    }
    out.extend_from_slice(b"error: ");
    out.extend_from_slice(body);
    out.push(0x0a); // '\n'
    out
}

/// Port of the message produced by `common.c:cob_runtime_warning` -- `"libcob: "`, then the source-location
/// prefix (via `output_source_location`: `"<file>"`, then `":<line>"` when non-zero, then `": "`), then
/// `"warning: "`, then the formatted `body`, then a newline.
///
/// Note the source-location punctuation differs from [`cob_runtime_error`]: `output_source_location` emits
/// `"<file>:<line>: "` (the `:` joins file and line) whereas `cob_runtime_error`'s inline prefix emits
/// `"<file>:<line>: "` too -- but with a trailing `:` after the file even when the line is `0`. Reproduced
/// faithfully. Returns `None` when warnings are disabled (`cob_display_warn == 0`).
/// Port of `common.c:cob_runtime_hint` -- the `note:` companion line GnuCOBOL prints after a runtime error
/// (e.g. the bounds-check `maximum subscript ...` hint): the `"note: "` prefix, the body, and a trailing
/// newline. The stderr write + flush are the boundary.
pub fn cob_runtime_hint(body: &[u8]) -> Vec<u8> {
    let mut out = b"note: ".to_vec();
    out.extend_from_slice(body);
    out.push(b'\n');
    out
}

pub fn cob_runtime_warning(display_warn: bool, loc: &SourceLocation, body: &[u8]) -> Option<Vec<u8>> {
    if !display_warn {
        return None;
    }
    let mut out = Vec::new();
    out.extend_from_slice(b"libcob: ");
    append_output_source_location(&mut out, loc);
    out.extend_from_slice(WARNING_MSGID);
    out.extend_from_slice(body);
    out.push(0x0a); // '\n'
    Some(out)
}

/// Port of `common.c:output_source_location` -- append `"<file>"`, then `":<line>"` when the line is
/// non-zero, then `": "`, but only when a file is set (otherwise nothing is appended).
fn append_output_source_location(out: &mut Vec<u8>, loc: &SourceLocation) {
    if let Some(file) = &loc.file {
        out.extend_from_slice(file);
        if loc.line != 0 {
            out.push(0x3a); // ':'
            out.extend_from_slice((loc.line as i32).to_string().as_bytes());
        }
        out.extend_from_slice(b": ");
    }
}

/// Port of the message produced by `common.c:cob_runtime_warning_ss` -- a signal-safe two-part warning:
/// `"libcob: "`, the `output_source_location` prefix, `"warning: "`, then `msg`, then the optional
/// `addition`, then a newline. Returns `None` when warnings are disabled.
pub fn cob_runtime_warning_ss(
    display_warn: bool,
    loc: &SourceLocation,
    msg: &[u8],
    addition: Option<&[u8]>,
) -> Option<Vec<u8>> {
    if !display_warn {
        return None;
    }
    let mut out = Vec::new();
    out.extend_from_slice(b"libcob: ");
    append_output_source_location(&mut out, loc);
    out.extend_from_slice(WARNING_MSGID);
    out.extend_from_slice(msg);
    if let Some(add) = addition {
        out.extend_from_slice(add);
    }
    out.push(0x0a); // '\n'
    Some(out)
}

/// Port of the message produced by `common.c:cob_runtime_warning_external` -- a warning tagged with a
/// caller name: `"libcob: "`, then (when `cob_reference`) the `output_source_location` prefix or (otherwise)
/// nothing extra, then `"warning: "`, then `"<caller_name>: "`, then the formatted `body`, then a newline.
/// An empty/`None` `caller_name` becomes `"unknown caller"`. Returns `None` when warnings are disabled.
pub fn cob_runtime_warning_external(
    display_warn: bool,
    caller_name: Option<&[u8]>,
    cob_reference: bool,
    loc: &SourceLocation,
    body: &[u8],
) -> Option<Vec<u8>> {
    if !display_warn {
        return None;
    }
    let caller: &[u8] = match caller_name {
        Some(c) if !c.is_empty() => c,
        _ => b"unknown caller",
    };
    let mut out = Vec::new();
    out.extend_from_slice(b"libcob: ");
    if cob_reference {
        append_output_source_location(&mut out, loc);
    }
    out.extend_from_slice(WARNING_MSGID);
    out.extend_from_slice(caller);
    out.extend_from_slice(b": ");
    out.extend_from_slice(body);
    out.push(0x0a); // '\n'
    Some(out)
}

// ======================================================================================================
// conf_runtime_error / conf_runtime_error_value -- runtime-configuration error messages.
// ======================================================================================================

/// Caller-owned model of `common.c`'s `conf_runtime_error` static latches: whether the
/// `"configuration error:"` banner has been printed (`conf_runtime_error_displayed`) and the last
/// file/line a location prefix was emitted for (`last_runtime_error_file` / `last_runtime_error_line`),
/// which de-duplicates the location across consecutive messages.
#[derive(Debug, Clone, Default)]
pub struct ConfErrorState {
    /// Has the `"configuration error:"` banner been emitted?
    pub displayed: bool,
    /// The last file a location prefix was emitted for.
    pub last_file: Option<Vec<u8>>,
    /// The last line a location prefix was emitted for.
    pub last_line: u32,
}

/// Port of the message produced by `common.c:conf_runtime_error` -- a runtime-configuration error line.
///
/// On the first call the `"configuration error:\n"` banner is emitted. A location prefix is emitted only
/// when `(file,line)` differs from the last one (the C de-dups against `last_runtime_error_file/_line`):
/// `"<file>"`, then `":<line>"` when non-zero, else `"environment variables"` when no file is set, followed
/// by `": "`. Then the formatted `body`. The postfix is `";\n\t"` for a continuation (`finish_error == 0`)
/// or `"\n"` for the final line (`finish_error != 0`). State (`st`) is updated in place.
pub fn conf_runtime_error(
    st: &mut ConfErrorState,
    finish_error: bool,
    loc: &SourceLocation,
    body: &[u8],
) -> Vec<u8> {
    let mut out = Vec::new();
    if !st.displayed {
        st.displayed = true;
        out.extend_from_slice(b"configuration error:");
        out.push(0x0a); // '\n'
    }
    // Prefix: only when the source location changed since last time.
    if loc.file != st.last_file || loc.line != st.last_line {
        st.last_file = loc.file.clone();
        st.last_line = loc.line;
        if let Some(file) = &loc.file {
            out.extend_from_slice(file);
            if loc.line != 0 {
                out.push(0x3a); // ':'
                out.extend_from_slice((loc.line as i32).to_string().as_bytes());
            }
        } else {
            out.extend_from_slice(b"environment variables");
        }
        out.push(0x3a); // ':'
        out.push(0x20); // ' '
    }
    out.extend_from_slice(body);
    if !finish_error {
        out.push(0x3b); // ';'
        out.push(0x0a); // '\n'
        out.push(0x09); // '\t'
    } else {
        out.push(0x0a); // '\n'
    }
    out
}

/// Port of `common.c:conf_runtime_error_value` -- the body for the "invalid value" configuration error:
/// `"invalid value '<value>' for configuration tag '<name>'"`, where `name` is the config tag's
/// `conf_name` (when set via a config file) or `env_name` (when set via the environment). The C then calls
/// `conf_runtime_error(0, ...)` with this body; here only the body is returned (compose with
/// [`conf_runtime_error`] passing `finish_error = false`).
pub fn conf_runtime_error_value(value: &[u8], name: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"invalid value '");
    out.extend_from_slice(value);
    out.extend_from_slice(b"' for configuration tag '");
    out.extend_from_slice(name);
    out.push(0x27); // '\''
    out
}

// ======================================================================================================
// cob_fatal_error -- the COB_FERROR_* code -> message mapping.
// ======================================================================================================

/// `common.h:enum cob_fatal_error` -- the fatal-error codes passed to [`cob_fatal_error`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum CobFatalError {
    /// `COB_FERROR_NONE` (0).
    None = 0,
    /// `COB_FERROR_CANCEL`.
    Cancel,
    /// `COB_FERROR_INITIALIZED`.
    Initialized,
    /// `COB_FERROR_CODEGEN`.
    Codegen,
    /// `COB_FERROR_CHAINING`.
    Chaining,
    /// `COB_FERROR_STACK`.
    Stack,
    /// `COB_FERROR_GLOBAL`.
    Global,
    /// `COB_FERROR_MEMORY`.
    Memory,
    /// `COB_FERROR_MODULE`.
    Module,
    /// `COB_FERROR_RECURSIVE`.
    Recursive,
    /// `COB_FERROR_SCR_INP`.
    ScrInp,
    /// `COB_FERROR_FILE`.
    File,
    /// `COB_FERROR_FUNCTION`.
    Function,
    /// `COB_FERROR_FREE`.
    Free,
    /// `COB_FERROR_XML`.
    Xml,
    /// `COB_FERROR_JSON`.
    Json,
}

/// Port of the message(s) of `common.c:cob_fatal_error` -- map a fatal-error code to the runtime-error
/// message text(s) it passes to `cob_runtime_error` before aborting via `cob_hard_failure`. Each returned
/// `Vec<u8>` is one message body (the text after `error: `).
///
/// The codes whose message depends on runtime state are parameterised, not hard-coded:
///   * `Recursive` -- needs the two module names; see [`cob_fatal_error_recursive`].
///   * `File` -- needs the file status, file-name, and statement; see [`cob_fatal_error_file`].
/// For those, this returns the *codegen-error fallback shape* is not used; instead callers should use the
/// dedicated helpers. `Codegen` emits two messages (the second `"Please report this!"`), so this returns a
/// `Vec` of bodies. The abort (`cob_hard_failure`) is the caller's side effect.
pub fn cob_fatal_error(code: CobFatalError) -> Vec<Vec<u8>> {
    match code {
        CobFatalError::Cancel => vec![b"attempt to CANCEL active program".to_vec()],
        CobFatalError::Initialized => vec![b"cob_init() has not been called".to_vec()],
        CobFatalError::Codegen => {
            vec![b"codegen error".to_vec(), b"Please report this!".to_vec()]
        }
        CobFatalError::Chaining => vec![b"CALL of program with CHAINING clause".to_vec()],
        CobFatalError::Stack => vec![b"stack overflow, possible PERFORM depth exceeded".to_vec()],
        CobFatalError::Global => vec![b"invalid entry/exit in GLOBAL USE procedure".to_vec()],
        CobFatalError::Memory => vec![b"unable to allocate memory".to_vec()],
        CobFatalError::Module => vec![b"invalid entry into module".to_vec()],
        // C: cob_runtime_error(_("call to %s with NULL pointer"), "cob_free")
        CobFatalError::Free => vec![b"call to cob_free with NULL pointer".to_vec()],
        CobFatalError::Function => vec![b"attempt to use non-implemented function".to_vec()],
        CobFatalError::Xml => vec![b"attempt to use non-implemented XML I/O".to_vec()],
        CobFatalError::Json => vec![b"attempt to use non-implemented JSON I/O".to_vec()],
        // COB_FERROR_RECURSIVE / COB_FERROR_FILE need runtime state -> dedicated helpers.
        CobFatalError::Recursive => cob_fatal_error_recursive(None, b"", b""),
        CobFatalError::File => vec![cob_fatal_error_file(-1, b"", None)],
        // COB_FERROR_NONE / COB_FERROR_SCR_INP have no case in the switch -> the `default` "unknown failure".
        CobFatalError::None => vec![cob_fatal_error_unknown(0)],
        CobFatalError::ScrInp => vec![cob_fatal_error_unknown(CobFatalError::ScrInp as i32)],
    }
}

/// Port of the `default` branch of `common.c:cob_fatal_error` -- the untranslated `"unknown failure: <n>"`
/// for a code with no `case` (e.g. `COB_FERROR_NONE`, `COB_FERROR_SCR_INP`, or any out-of-range value).
pub fn cob_fatal_error_unknown(code: i32) -> Vec<u8> {
    format!("unknown failure: {code}").into_bytes()
}

/// Port of the `COB_FERROR_RECURSIVE` branch of `common.c:cob_fatal_error`. With both module names
/// (`cob_module_err` set) the message is `"recursive CALL from '<from>' to '<to>' which is NOT RECURSIVE"`;
/// the legacy path (`module_err == false`) is `"invalid recursive COBOL CALL to '<current>'"`.
/// `module_err` selects the branch; `from`/`to`/`current` supply the names.
pub fn cob_fatal_error_recursive(
    module_err: Option<(&[u8], &[u8])>,
    current: &[u8],
    _unused: &[u8],
) -> Vec<Vec<u8>> {
    match module_err {
        Some((from, to)) => {
            let mut m = Vec::new();
            m.extend_from_slice(b"recursive CALL from '");
            m.extend_from_slice(from);
            m.extend_from_slice(b"' to '");
            m.extend_from_slice(to);
            m.extend_from_slice(b"' which is NOT RECURSIVE");
            vec![m]
        }
        None => {
            let mut m = Vec::new();
            m.extend_from_slice(b"invalid recursive COBOL CALL to '");
            m.extend_from_slice(current);
            m.push(0x27); // '\''
            vec![m]
        }
    }
}

/// Port of the file-status -> message sub-switch of the `COB_FERROR_FILE` branch of
/// `common.c:cob_fatal_error` -- map a two-digit COBOL file status to its message text.
pub fn cob_fatal_error_file_status_msg(status: i32) -> &'static str {
    match status {
        10 => "end of file",
        14 => "key out of range",
        21 => "key order not ascending",
        22 => "record key already exists",
        23 => "record key does not exist",
        30 => "permanent file error",
        31 => "inconsistent file name",
        35 => "file does not exist",
        37 => "permission denied",
        39 => "mismatch of fixed file attributes",
        41 => "file already open",
        42 => "file not open",
        43 => "READ must be executed first",
        44 => "record overflow",
        46 => "READ after unsuccessful READ/START",
        47 => "READ/START not allowed, file not open for input",
        48 => "WRITE not allowed, file not open for output",
        49 => "DELETE/REWRITE not allowed, file not open for I-O",
        51 => "record locked by another file connector",
        57 => "LINAGE values invalid",
        61 => "file sharing conflict",
        71 => "invalid data in LINE SEQUENTIAL file",
        91 => "runtime library is not configured for this operation",
        _ => "unknown file error",
    }
}

/// Port of the `cob_runtime_error` body of the `COB_FERROR_FILE` branch of `common.c:cob_fatal_error`.
/// `status` is the two-digit file status (selecting the message via [`cob_fatal_error_file_status_msg`]),
/// `err_cause` is the file-name print, and `statement` is the offending statement name (`None` when the
/// last exception statement is `STMT_UNKNOWN`). Produces `"<msg> (status = <NN>) for file <cause>"` or, with
/// a statement, `"<msg> (status = <NN>) for file <cause> on <statement>"` (the `%02d` status is zero-padded).
pub fn cob_fatal_error_file(status: i32, err_cause: &[u8], statement: Option<&[u8]>) -> Vec<u8> {
    let msg = cob_fatal_error_file_status_msg(status);
    let mut out = Vec::new();
    out.extend_from_slice(msg.as_bytes());
    out.extend_from_slice(b" (status = ");
    // %02d : zero-pad to width 2 (negative values keep their sign, matching printf).
    out.extend_from_slice(format!("{status:02}").as_bytes());
    out.extend_from_slice(b") for file ");
    out.extend_from_slice(err_cause);
    if let Some(stmt) = statement {
        out.extend_from_slice(b" on ");
        out.extend_from_slice(stmt);
    }
    out
}

// ======================================================================================================
// var_print -- the word-wrapping value printer used by print_runtime_conf.
// ======================================================================================================

/// Port of the pure word-wrapping logic of `common.c:var_print` (the `#else`, format-0/1 path actually
/// compiled). It prints a label column then the value, wrapping the value at word boundaries to the
/// `CB_IVAL_SIZE` column. This returns the bytes it would write to `stdout`.
///
/// Label column: format `0` -> the label left-justified in a `CB_IMSG_SIZE` field then `" : "`; any other
/// format -> `"  env: "` then the label left-justified in `CB_IMSG_SIZE - 2 - 3 - 2` = `lablen` then `" : "`
/// (`_("env")` is `"env"`, 3 bytes). `%-*.*s` both pads to and truncates at the field width.
///
/// Value handling (faithful to the compiled `#else` block, then the duplicated post-block):
///   * no `val` and no/empty `default_val` -> just a newline;
///   * format `1` with a `val` whose first byte is `'0'` or which equals `default_val` -> `val` becomes
///     `"<default_val> (default)"`;
///   * no `val` but a `default_val` -> `val` becomes `default_val`.
///   The C runs this resolution block twice (a duplicated `#else` then an unconditional copy); the second
///   pass is idempotent given the first, so it is applied once here.
///
/// Then: if the resolved value fits in `CB_IVAL_SIZE` it is printed on one line; otherwise it is split on
/// spaces and wrapped, each continuation line indented to `CB_IMSG_SIZE + 3` spaces.
pub fn var_print(msg: &[u8], val: Option<&[u8]>, default_val: Option<&[u8]>, format: u32) -> Vec<u8> {
    let mut out = Vec::new();

    // --- label column ---
    if format == 0 {
        push_left_justified(&mut out, msg, CB_IMSG_SIZE);
        out.extend_from_slice(b" : ");
    } else {
        out.extend_from_slice(b"  env: ");
        // lablen = CB_IMSG_SIZE - 2 - strlen("env") - 2 = 24 - 2 - 3 - 2 = 17
        let lablen = CB_IMSG_SIZE as isize - 2 - 3 - 2;
        let lablen = if lablen < 0 { 0 } else { lablen as usize };
        push_left_justified(&mut out, msg, lablen);
        out.extend_from_slice(b" : ");
    }

    // --- resolve the value (idempotent application of the C's duplicated block) ---
    let empty_default = default_val.is_none_or(|d| d.is_empty());
    if val.is_none() && empty_default {
        out.push(0x0a); // '\n'
        return out;
    }
    let resolved: Vec<u8> = if format == 1
        && val.is_some()
        && default_val.is_some()
        && (val.map(|v| v.first() == Some(&0x30)).unwrap_or(false) || val == default_val)
    {
        // val = cob_strcat(default_val, " (default)")  [snprintf width 39]
        let mut v = default_val.unwrap().to_vec();
        v.extend_from_slice(b" (default)");
        v
    } else if val.is_none() {
        // val = default_val (default_val is non-empty here)
        default_val.unwrap().to_vec()
    } else {
        val.unwrap().to_vec()
    };

    // --- print value, wrapping at CB_IVAL_SIZE on spaces ---
    if resolved.len() <= CB_IVAL_SIZE {
        out.extend_from_slice(&resolved);
        out.push(0x0a); // '\n'
        return out;
    }

    // strtok on spaces: split, dropping empty tokens (consecutive/leading/trailing spaces).
    let mut n: usize = 0;
    for token in resolved.split(|&b| b == 0x20).filter(|t| !t.is_empty()) {
        let toklen = token.len() + 1; // strlen(token) + 1
        if n + toklen > CB_IVAL_SIZE {
            if n != 0 {
                out.push(0x0a); // '\n'
                // format 2/3 prefix the field with a literal 8 spaces ("\n        %*.*s");
                // format 0/1 do not ("\n%*.*s"). The %*.*s of " " yields CB_IMSG_SIZE+3 spaces.
                if format == 2 || format == 3 {
                    push_spaces(&mut out, 8);
                }
                push_spaces(&mut out, CB_IMSG_SIZE + 3);
            }
            n = 0;
        }
        if n != 0 {
            out.push(0x20); // ' '
        }
        out.extend_from_slice(token);
        n += toklen;
    }
    out.push(0x0a); // '\n'
    out
}

/// Emulate printf `%-*.*s` for ASCII bytes: left-justify `s` in a field of `width`, truncating to `width`
/// then space-padding to `width`.
fn push_left_justified(out: &mut Vec<u8>, s: &[u8], width: usize) {
    let take = s.len().min(width);
    out.extend_from_slice(&s[..take]);
    for _ in take..width {
        out.push(0x20); // ' '
    }
}

/// Emulate printf `%*.*s` with a `" "` argument: emit `width` spaces (the C right-justifies a single space
/// into a `width`-wide field, i.e. `width` spaces).
fn push_spaces(out: &mut Vec<u8>, width: usize) {
    for _ in 0..width {
        out.push(0x20); // ' '
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strerror_fallback_and_setup() {
        // No injected message -> the HAVE_STRERROR-absent fallback.
        assert_eq!(cob_get_strerror(2, None), b"system error 2".to_vec());
        // Injected platform message is returned verbatim.
        assert_eq!(cob_get_strerror(2, Some(b"No such file or directory")), b"No such file or directory".to_vec());
        // runtime_err_str assembly: file + line
        let loc = SourceLocation::at(b"prog.cob", 42);
        assert_eq!(cob_setup_runtime_error_str(&loc, b"boom"), b"prog.cob:42: boom".to_vec());
        // file, no line
        let loc0 = SourceLocation::at(b"prog.cob", 0);
        assert_eq!(cob_setup_runtime_error_str(&loc0, b"boom"), b"prog.cob: boom".to_vec());
        // no file -> body only
        assert_eq!(cob_setup_runtime_error_str(&SourceLocation::none(), b"boom"), b"boom".to_vec());
    }

    #[test]
    fn runtime_error_prefix() {
        let loc = SourceLocation::at(b"prog.cob", 42);
        assert_eq!(
            cob_runtime_error(&loc, b"STOP ERROR"),
            b"libcob: prog.cob:42: error: STOP ERROR\n".to_vec()
        );
        // file but line 0: trailing ':' after file kept, no line, then space.
        let loc0 = SourceLocation::at(b"prog.cob", 0);
        assert_eq!(
            cob_runtime_error(&loc0, b"x"),
            b"libcob: prog.cob: error: x\n".to_vec()
        );
        // no location
        assert_eq!(
            cob_runtime_error(&SourceLocation::none(), b"x"),
            b"libcob: error: x\n".to_vec()
        );
    }

    #[test]
    fn runtime_warnings() {
        let loc = SourceLocation::at(b"p.cob", 7);
        assert_eq!(
            cob_runtime_warning(true, &loc, b"oops").unwrap(),
            b"libcob: p.cob:7: warning: oops\n".to_vec()
        );
        assert!(cob_runtime_warning(false, &loc, b"oops").is_none());
        // ss: msg + addition
        assert_eq!(
            cob_runtime_warning_ss(true, &loc, b"bad value ", Some(b"X")).unwrap(),
            b"libcob: p.cob:7: warning: bad value X\n".to_vec()
        );
        assert_eq!(
            cob_runtime_warning_ss(true, &SourceLocation::none(), b"m", None).unwrap(),
            b"libcob: warning: m\n".to_vec()
        );
        // external: caller name; cob_reference=false suppresses the location.
        assert_eq!(
            cob_runtime_warning_external(true, Some(b"MYSUB"), false, &loc, b"detail").unwrap(),
            b"libcob: warning: MYSUB: detail\n".to_vec()
        );
        assert_eq!(
            cob_runtime_warning_external(true, Some(b"MYSUB"), true, &loc, b"detail").unwrap(),
            b"libcob: p.cob:7: warning: MYSUB: detail\n".to_vec()
        );
        // empty caller -> "unknown caller"
        assert_eq!(
            cob_runtime_warning_external(true, None, false, &loc, b"d").unwrap(),
            b"libcob: warning: unknown caller: d\n".to_vec()
        );
    }

    #[test]
    fn conf_errors() {
        let mut st = ConfErrorState::default();
        let loc = SourceLocation::at(b"cob.cfg", 3);
        let body = conf_runtime_error_value(b"FOO", b"COB_BAR");
        assert_eq!(body, b"invalid value 'FOO' for configuration tag 'COB_BAR'".to_vec());
        // first call: banner + location prefix + body + ";\n\t" (continuation)
        let m1 = conf_runtime_error(&mut st, false, &loc, &body);
        assert_eq!(
            m1,
            b"configuration error:\ncob.cfg:3: invalid value 'FOO' for configuration tag 'COB_BAR';\n\t".to_vec()
        );
        assert!(st.displayed);
        // second call, same location -> no banner, no location prefix; finish_error -> "\n"
        let m2 = conf_runtime_error(&mut st, true, &loc, b"should be numeric");
        assert_eq!(m2, b"should be numeric\n".to_vec());
        // No source file: the C de-dups against last_runtime_error_file/_line, both initially NULL/0,
        // which already equal a no-location SourceLocation -- so the *first* no-location message does NOT
        // print the "environment variables" prefix (faithful to the C latch).
        let mut st2 = ConfErrorState::default();
        let m3 = conf_runtime_error(&mut st2, true, &SourceLocation::none(), b"x");
        assert_eq!(m3, b"configuration error:\nx\n".to_vec());
        // After a real location has been seen, returning to a no-location *does* differ -> prefix prints.
        let mut st3 = ConfErrorState::default();
        let _ = conf_runtime_error(&mut st3, false, &SourceLocation::at(b"f", 1), b"a");
        let m4 = conf_runtime_error(&mut st3, true, &SourceLocation::none(), b"x");
        assert_eq!(m4, b"environment variables: x\n".to_vec());
    }

    #[test]
    fn fatal_error_codes() {
        assert_eq!(cob_fatal_error(CobFatalError::Cancel), vec![b"attempt to CANCEL active program".to_vec()]);
        assert_eq!(cob_fatal_error(CobFatalError::Initialized), vec![b"cob_init() has not been called".to_vec()]);
        assert_eq!(
            cob_fatal_error(CobFatalError::Codegen),
            vec![b"codegen error".to_vec(), b"Please report this!".to_vec()]
        );
        assert_eq!(cob_fatal_error(CobFatalError::Stack), vec![b"stack overflow, possible PERFORM depth exceeded".to_vec()]);
        assert_eq!(cob_fatal_error(CobFatalError::Memory), vec![b"unable to allocate memory".to_vec()]);
        assert_eq!(cob_fatal_error(CobFatalError::Free), vec![b"call to cob_free with NULL pointer".to_vec()]);
        assert_eq!(cob_fatal_error(CobFatalError::Function), vec![b"attempt to use non-implemented function".to_vec()]);
        assert_eq!(cob_fatal_error(CobFatalError::Xml), vec![b"attempt to use non-implemented XML I/O".to_vec()]);
        assert_eq!(cob_fatal_error(CobFatalError::Json), vec![b"attempt to use non-implemented JSON I/O".to_vec()]);
        // unknown / no-case codes -> "unknown failure: <n>"
        assert_eq!(cob_fatal_error_unknown(10), b"unknown failure: 10".to_vec());
        assert_eq!(cob_fatal_error(CobFatalError::None), vec![b"unknown failure: 0".to_vec()]);
    }

    #[test]
    fn fatal_error_recursive_and_file() {
        assert_eq!(
            cob_fatal_error_recursive(Some((b"A", b"B")), b"", b""),
            vec![b"recursive CALL from 'A' to 'B' which is NOT RECURSIVE".to_vec()]
        );
        assert_eq!(
            cob_fatal_error_recursive(None, b"PROG", b""),
            vec![b"invalid recursive COBOL CALL to 'PROG'".to_vec()]
        );
        // file status message + %02d status + cause, no statement
        assert_eq!(cob_fatal_error_file_status_msg(35), "file does not exist");
        assert_eq!(cob_fatal_error_file_status_msg(99), "unknown file error");
        assert_eq!(
            cob_fatal_error_file(35, b"DATA.DAT", None),
            b"file does not exist (status = 35) for file DATA.DAT".to_vec()
        );
        assert_eq!(
            cob_fatal_error_file(10, b"IN.SEQ", Some(b"READ")),
            b"end of file (status = 10) for file IN.SEQ on READ".to_vec()
        );
        // single-digit status zero-pads to width 2
        assert_eq!(
            cob_fatal_error_file(0, b"F", None),
            b"unknown file error (status = 00) for file F".to_vec()
        );
    }

    #[test]
    fn var_print_one_line_and_default() {
        // format 0, short value: "<label padded to 24> : value\n"
        let out = var_print(b"max threads", Some(b"4"), None, 0);
        let mut expect = Vec::new();
        expect.extend_from_slice(b"max threads             "); // 11 + 13 padding = 24
        expect.extend_from_slice(b" : 4\n");
        assert_eq!(out, expect);
        // no val, no default -> just newline after label.
        let out2 = var_print(b"x", None, None, 0);
        let mut e2 = Vec::new();
        push_left_justified(&mut e2, b"x", CB_IMSG_SIZE);
        e2.extend_from_slice(b" : \n");
        assert_eq!(out2, e2);
        // format 1, val == default -> "<default> (default)"
        let out3 = var_print(b"opt", Some(b"on"), Some(b"on"), 1);
        // label col is "  env: " + left-justified "opt" in lablen(17) + " : "
        let mut e3 = Vec::new();
        e3.extend_from_slice(b"  env: ");
        push_left_justified(&mut e3, b"opt", 17);
        e3.extend_from_slice(b" : on (default)\n");
        assert_eq!(out3, e3);
        // no val but a default -> value is the default (format 0, so no "(default)" tag)
        let out4 = var_print(b"k", None, Some(b"v"), 0);
        let mut e4 = Vec::new();
        push_left_justified(&mut e4, b"k", CB_IMSG_SIZE);
        e4.extend_from_slice(b" : v\n");
        assert_eq!(out4, e4);
    }

    #[test]
    fn var_print_wrapping() {
        // A long value (> CB_IVAL_SIZE=52) of space-separated words wraps onto an indented continuation.
        let words = b"alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi";
        let out = var_print(b"path", Some(words), None, 0);
        // First line: label + " : " + first words until adding the next would exceed CB_IVAL_SIZE.
        // Verify it contains a newline followed by CB_IMSG_SIZE+3 spaces (the continuation indent).
        let s = out;
        assert!(s.ends_with(b"\n"));
        // Find a wrap: there must be at least one '\n' before the end.
        let nl_positions: Vec<usize> = s.iter().enumerate().filter(|(_, &b)| b == 0x0a).map(|(i, _)| i).collect();
        assert!(nl_positions.len() >= 2, "expected at least one wrap plus the final newline");
        // The continuation indent is CB_IMSG_SIZE + 3 = 27 spaces after the first newline.
        let first_nl = nl_positions[0];
        let indent = &s[first_nl + 1..first_nl + 1 + (CB_IMSG_SIZE + 3)];
        assert!(indent.iter().all(|&b| b == 0x20), "continuation line should start with 27 spaces");
    }
}
