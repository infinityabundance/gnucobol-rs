//! Port of the **signal-handling decision, print drivers, and misc init** surface of
//! `libcob/common.c`.
//!
//! These C functions all sit on top of OS / libc leaves (`sigaction(2)`, `raise(3)`, `printf(3)`,
//! `setlocale(3)`, `stat(2)`, `localtime(3)`). A faithful 1:1 port models the **pure decision** each
//! makes -- which message to emit, terminate-vs-core, install-vs-skip, valid-vs-invalid -- and
//! returns the bytes / decision; the actual syscall is the **documented boundary**. Every function is
//! named exactly as in `common.c` and is panic-free.
//!
//! Helpers (`ss_itoa_u10`, a tiny word-wrap, the signal-name table) are re-implemented locally so this
//! module does not depend on the other `common_*` modules. Signal numbers are the Linux/glibc values
//! (`SIGSEGV == 11`, ...) -- the platform boundary.

#![forbid(unsafe_code)]

// ======================================================================================================
// Buffer-size and field constants (common.h / common.c).
// ======================================================================================================

/// Port of `common.h:COB_NORMAL_BUFF` (`2048`).
pub const COB_NORMAL_BUFF: usize = 2048;
/// Port of `common.h:COB_NORMAL_MAX` (`COB_NORMAL_BUFF - 1`).
pub const COB_NORMAL_MAX: usize = COB_NORMAL_BUFF - 1;
/// Port of `common.h:COB_MINI_BUFF` (`256`).
pub const COB_MINI_BUFF: usize = 256;
/// Port of `common.c:CB_IMSG_SIZE` (`24`) -- the label column width in `var_print`.
pub const CB_IMSG_SIZE: usize = 24;
/// Port of `common.c:CB_IVAL_SIZE` (`80 - CB_IMSG_SIZE - 4` = `52`) -- the value-wrap width.
pub const CB_IVAL_SIZE: usize = 80 - CB_IMSG_SIZE - 4;

/// Port of `coblocal.h:COB_D2I(x)` -- the low nibble of a digit byte (`x & 0x0F`).
fn cob_d2i(x: u8) -> i32 {
    (x & 0x0F) as i32
}

/// Port of `common.c:IS_INVALID_DIGIT_DATA(c)` -- `(unsigned char)(c - '0') > 9`.
fn is_invalid_digit_data(c: u8) -> bool {
    c.wrapping_sub(b'0') > 9
}

/// Local copy of `common.c:ss_itoa_u10` -- append the base-10 ASCII of `value` (leading `-` for
/// negatives) onto `out`. Re-implemented here so this module is self-contained.
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

// ======================================================================================================
// Signal name / description table (a local copy of the common.c signals[] surface so this module does
// not depend on common_stack). The OS-bound registration uses for_set / for_dump; the handler decision
// uses the short name + description.
// ======================================================================================================

/// Local copy of `common.c`'s `struct signal_table` row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SignalEntry {
    /// `signal_table.sig` -- the Linux/glibc signal number.
    pub sig: i32,
    /// `signal_table.for_set` -- 0 = leave alone, 1 = set if not ignored, 2 = take direct control.
    pub for_set: i16,
    /// `signal_table.for_dump` -- registered via `cob_set_dump_signal`.
    pub for_dump: i16,
    /// `signal_table.shortname` -- e.g. `"SIGSEGV"`.
    pub shortname: &'static str,
    /// `signal_table.description` -- the long message.
    pub description: &'static str,
}

/// Local copy of `common.c:signals[]` (Linux/glibc numbers, descriptions baked in). The trailing
/// `{-1, ..., "unknown"}` row is the lookup fallthrough, not a real signal.
pub const fn signals() -> &'static [SignalEntry] {
    &[
        SignalEntry {
            sig: 2,
            for_set: 1,
            for_dump: 0,
            shortname: "SIGINT",
            description: "interrupt from keyboard",
        },
        SignalEntry {
            sig: 1,
            for_set: 1,
            for_dump: 0,
            shortname: "SIGHUP",
            description: "hangup",
        },
        SignalEntry {
            sig: 3,
            for_set: 1,
            for_dump: 0,
            shortname: "SIGQUIT",
            description: "quit",
        },
        SignalEntry {
            sig: 15,
            for_set: 1,
            for_dump: 0,
            shortname: "SIGTERM",
            description: "termination",
        },
        SignalEntry {
            sig: 13,
            for_set: 1,
            for_dump: 0,
            shortname: "SIGPIPE",
            description: "broken pipe",
        },
        SignalEntry {
            sig: 29,
            for_set: 1,
            for_dump: 0,
            shortname: "SIGIO",
            description: "I/O signal",
        },
        SignalEntry {
            sig: 11,
            for_set: 2,
            for_dump: 1,
            shortname: "SIGSEGV",
            description: "attempt to reference invalid memory address",
        },
        SignalEntry {
            sig: 7,
            for_set: 2,
            for_dump: 1,
            shortname: "SIGBUS",
            description: "bus error",
        },
        SignalEntry {
            sig: 8,
            for_set: 1,
            for_dump: 1,
            shortname: "SIGFPE",
            description: "fatal arithmetic error",
        },
        SignalEntry {
            sig: 4,
            for_set: 0,
            for_dump: 0,
            shortname: "SIGILL",
            description: "illegal instruction",
        },
        SignalEntry {
            sig: 6,
            for_set: 0,
            for_dump: 0,
            shortname: "SIGABRT",
            description: "abort",
        },
        SignalEntry {
            sig: 9,
            for_set: 0,
            for_dump: 0,
            shortname: "SIGKILL",
            description: "process killed",
        },
        SignalEntry {
            sig: 14,
            for_set: 0,
            for_dump: 0,
            shortname: "SIGALRM",
            description: "alarm signal",
        },
        SignalEntry {
            sig: 19,
            for_set: 0,
            for_dump: 0,
            shortname: "SIGSTOP",
            description: "stop process",
        },
        SignalEntry {
            sig: 17,
            for_set: 0,
            for_dump: 0,
            shortname: "SIGCHLD",
            description: "child process stopped",
        },
        SignalEntry {
            sig: -1,
            for_set: 0,
            for_dump: 0,
            shortname: "unknown",
            description: "unknown",
        },
    ]
}

/// Number of real rows in [`signals`] (excludes the `{-1, ..., "unknown"}` sentinel).
const NUM_SIGNALS: usize = signals().len() - 1;

/// Local copy of `common.c:get_signal_entry` -- linear lookup of `sig`; the sentinel on no match.
fn get_signal_entry(sig: i32) -> SignalEntry {
    let tbl = signals();
    for entry in &tbl[..NUM_SIGNALS] {
        if entry.sig == sig {
            return *entry;
        }
    }
    tbl[NUM_SIGNALS]
}

/// Local copy of `common.c:cob_get_sig_name`.
fn cob_get_sig_name(sig: i32) -> &'static str {
    get_signal_entry(sig).shortname
}

/// Local copy of `common.c:cob_get_sig_description`.
fn cob_get_sig_description(sig: i32) -> &'static str {
    let entry = get_signal_entry(sig);
    if entry.description.is_empty() {
        "unknown"
    } else {
        entry.description
    }
}

/// Port of `common.c:signal_msgid` -- the (untranslated) "signal" label.
pub const SIGNAL_MSGID: &[u8] = b"signal";

// Linux/glibc signal numbers used by the cob_sig_handler decision (the platform boundary).
const SIGHUP: i32 = 1;
const SIGINT: i32 = 2;
const SIGABRT: i32 = 6;
const SIGSEGV: i32 = 11;
const SIGPIPE: i32 = 13;
const SIGTERM: i32 = 15;
const SIGBUS: i32 = 7;

// ======================================================================================================
// cob_sig_handler -- the runtime signal handler decision.
// ======================================================================================================

/// What the runtime does at the very end of [`cob_sig_handler`]: either re-raise the original signal,
/// or (when an explicit coredump was requested but could not be made) re-raise `SIGABRT` instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Termination {
    /// `cob_terminate_routines()` ran (the normal path).
    Normal,
    /// `ss_terminate_routines()` ran (a hard error -- SIGSEGV/SIGBUS/SIGABRT -- with core-on-error set).
    HardError,
}

/// The pure decision [`cob_sig_handler`] reaches before re-raising: the emitted stderr text, whether a
/// dump should be skipped (SIGTERM/SIGINT/SIGHUP/SIGPIPE set `DUMP_TRACE_DONE_DUMP`), which terminate
/// routine ran, and the signal that is finally re-raised.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SigHandlerDecision {
    /// The bytes written to stderr (the `write_to_stderr_or_return_*` sequence). The write is the boundary.
    pub stderr_text: Vec<u8>,
    /// True for SIGTERM/SIGINT/SIGHUP/SIGPIPE: the dump is skipped (`DUMP_TRACE_DONE_DUMP`).
    pub skip_dump: bool,
    /// Which terminate routine the switch selects.
    pub termination: Termination,
    /// The signal value re-raised at the end (`SIGABRT` substituted when `cob_core_on_error == 4`).
    pub reraise_sig: i32,
}

/// Port of `common.c:cob_sig_handler` -- the runtime signal handler.
///
/// **Boundary:** the `sigaction`/`signal` reset to `SIG_DFL`, `cob_exit_screen`, the terminate-routine
/// side effects, the external handler call, `longjmp`, and the final `raise`/`kill` are all the OS
/// boundary. This ports the **pure decision**: the assembled stderr message
/// (`"\n" <description> " (" <signal_msgid> " " <shortname> ")\n\n"`, prefixed by the
/// "not handled signal" notice for an unknown `sig`), whether the dump is skipped, which terminate
/// routine the SIGSEGV/SIGBUS/SIGABRT switch picks (given `core_on_error`), and the re-raised signal.
///
/// `source_prefix` models the `cob_get_source_line` + `output_source_location` bytes written between
/// the leading `"\n"` and the description (empty when no location is known). `cob_initialized`,
/// `core_on_error` (the `cobsetptr->cob_core_on_error` value, 0..4) are the live state inputs.
pub fn cob_sig_handler(
    sig: i32,
    source_prefix: &[u8],
    cob_initialized: bool,
    core_on_error: i32,
) -> SigHandlerDecision {
    let mut out = Vec::new();

    let signal_name = cob_get_sig_name(sig);
    // LCOV_EXCL_START: not a handled signal -> the "caught not handled" notice.
    if signal_name == signals()[NUM_SIGNALS].shortname {
        out.extend_from_slice(b"\ncob_sig_handler caught not handled signal: ");
        ss_itoa_u10(&mut out, sig);
        out.extend_from_slice(b"\n");
    }
    // LCOV_EXCL_STOP
    let signal_name = if signal_name == signals()[NUM_SIGNALS].shortname {
        signals()[NUM_SIGNALS].description // "unknown"
    } else {
        signal_name
    };

    // dump-skip decision for "soft" signals.
    let skip_dump = matches!(sig, -1 | SIGTERM | SIGINT | SIGHUP | SIGPIPE);

    // "\n" then the (boundary-resolved) source location prefix, then the description.
    out.extend_from_slice(b"\n");
    out.extend_from_slice(source_prefix);
    let msg = cob_get_sig_description(sig);
    out.extend_from_slice(msg.as_bytes());

    // " (signal <name>)\n\n"
    out.extend_from_slice(b" (");
    out.extend_from_slice(SIGNAL_MSGID);
    out.push(b' ');
    out.extend_from_slice(signal_name.as_bytes());
    out.extend_from_slice(b")\n\n");

    // (cob_initialized + abort_reason copy is a state side effect; only the message above is visible.)
    let _ = cob_initialized;

    // terminate-routine decision: hard errors with core-on-error set use ss_terminate_routines.
    let termination = match sig {
        -1 | SIGSEGV | SIGBUS | SIGABRT if core_on_error != 0 => Termination::HardError,
        _ => Termination::Normal,
    };

    // if an explicit requested coredump could not be created -> re-raise SIGABRT (cob_core_on_error == 4).
    let reraise_sig = if core_on_error == 4 { SIGABRT } else { sig };

    SigHandlerDecision {
        stderr_text: out,
        skip_dump,
        termination,
        reraise_sig,
    }
}

// ======================================================================================================
// cob_raise -- raise a signal (run internal + external handlers).
// ======================================================================================================

/// The pure dispatch [`cob_raise`] performs before the OS `raise`/`kill`. When the platform has
/// `signal.h` the raise goes through the OS (which invokes the registered handlers); without it, only
/// the external handler is called directly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RaiseDispatch {
    /// Re-raise `sig` via the OS so registered handlers run (HAVE_SIGNAL_H path -- our platform).
    ViaOs(i32),
    /// No OS signals: call the registered external handler directly (then clear it).
    ExternalHandlerOnly,
}

/// Port of `common.c:cob_raise` -- raise a signal, running both internal and external handlers.
///
/// **Boundary:** the actual `raise(3)` / `kill(2)` (and the external-handler call) are the OS boundary.
/// On Linux (`HAVE_SIGNAL_H`) the decision is simply "re-raise `sig` via the OS"; this returns that
/// dispatch so the caller can perform it. `has_signal_h` selects the platform branch.
pub fn cob_raise(sig: i32, has_signal_h: bool) -> RaiseDispatch {
    if has_signal_h {
        RaiseDispatch::ViaOs(sig)
    } else {
        RaiseDispatch::ExternalHandlerOnly
    }
}

// ======================================================================================================
// cob_set_signal -- install the handlers.
// ======================================================================================================

/// One handler-installation decision from [`cob_set_signal`]: the signal, and how it is registered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SignalSetAction {
    /// The signal number to register `cob_sig_handler` for.
    pub sig: i32,
    /// `true` when `for_set == 2` (direct, unconditional control of a hard error); `false` when
    /// `for_set == 1` (install only if the OS does not already ignore the signal).
    pub direct: bool,
}

/// Port of `common.c:cob_set_signal` -- install the runtime signal handlers.
///
/// **Boundary:** the `sigaction(2)`/`signal(2)` calls (and the `SIG_IGN` probe for the conditional
/// rows) are the OS boundary. This ports the **set-decision**: the list of signals that get a handler
/// installed and whether each is taken directly (`for_set == 2`) or only-if-not-ignored
/// (`for_set == 1`) -- exactly the `for (k=0; k<NUM_SIGNALS; k++) if (signals[k].for_set)` loop.
pub fn cob_set_signal() -> Vec<SignalSetAction> {
    let mut out = Vec::new();
    for entry in &signals()[..NUM_SIGNALS] {
        if entry.for_set != 0 {
            out.push(SignalSetAction {
                sig: entry.sig,
                direct: entry.for_set == 2,
            });
        }
    }
    out
}

// ======================================================================================================
// cob_reg_sighnd -- register the external signal handler.
// ======================================================================================================

/// The decision [`cob_reg_sighnd`] makes when registering an external signal handler: whether the
/// handlers must first be installed (`cob_set_signal`, when the runtime is not yet initialized).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegSighndDecision {
    /// `true` when `!cob_initialized` -> `cob_set_signal()` must run first.
    pub must_set_signal: bool,
}

/// Port of `common.c:cob_reg_sighnd` -- register the external signal handler `sighnd`.
///
/// **Boundary:** storing the handler pointer into `cob_ext_sighdl` (and the `cob_set_signal` syscalls)
/// is the boundary. The C body is `if (!cob_initialized) cob_set_signal(); cob_ext_sighdl = sighnd;` --
/// the pure part is "do we need to install handlers first?", returned here.
pub fn cob_reg_sighnd(cob_initialized: bool) -> RegSighndDecision {
    RegSighndDecision {
        must_set_signal: !cob_initialized,
    }
}

// ======================================================================================================
// raise_arg_mismatch -- the CALL argument-mismatch diagnostics.
// ======================================================================================================

/// The CALL argument-mismatch reason, selecting which `cob_runtime_error` message
/// [`cob_check_linkage_size`] would print after [`raise_arg_mismatch`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgMismatch {
    /// `cob_call_params < ordinal_pos`, or the parameter / its data is NULL: "not passed by caller".
    NotPassed,
    /// `parameter->size < size`: "too small in the caller".
    TooSmall {
        /// The size the callee requires.
        callee_size: u64,
        /// The size the caller actually passed.
        caller_size: u64,
    },
}

/// Port of `common.c:raise_arg_mismatch` -- the pure pre-handler dispatch.
///
/// **Boundary:** building the temporary `cob_module`, pushing it onto `COB_MODULE_PTR`, and
/// `cob_set_exception` are state side effects. The pure decision the C returns is: when the CALL has
/// `ON EXCEPTION` (`cob_stmt_exception`), return to the caller (`0`); otherwise the diagnostic must be
/// raised (`1`). `cob_stmt_exception` is the live-state input.
pub fn raise_arg_mismatch(cob_stmt_exception: bool) -> i32 {
    if cob_stmt_exception {
        0
    } else {
        1
    }
}

/// Port of the `cob_runtime_error` message text `common.c:cob_check_linkage_size` emits once
/// [`raise_arg_mismatch`] returns `1` -- the oracle-visible CALL argument-mismatch diagnostic. `name`
/// already includes its quoting (the C passes `'x' of 'y'`-style names verbatim).
pub fn raise_arg_mismatch_message(name: &str, reason: ArgMismatch) -> Vec<u8> {
    match reason {
        ArgMismatch::NotPassed => format!("LINKAGE item {name} not passed by caller").into_bytes(),
        ArgMismatch::TooSmall {
            callee_size,
            caller_size,
        } => format!(
            "LINKAGE item {name} (size {callee_size}) too small in the caller (size {caller_size})"
        )
        .into_bytes(),
    }
}

// ======================================================================================================
// check_current_date -- parse a COB_CURRENT_DATE override into date components.
// ======================================================================================================

/// The parsed result of `common.c:check_current_date`'s pure parse loop: each component is `-1` when
/// absent (the C sentinel), `ret != 0` flags an invalid string, and `iso_timezone` holds the
/// normalized `Z`/`+hhmm`/`-hhmm` form (empty when none / invalid).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct CurrentDate {
    /// `yr` -- full year (already +2000 for 2-digit input), or `-1`.
    pub year: i32,
    /// `mm` -- month 1..12, or `-1`.
    pub month: i32,
    /// `dd` -- day 1..31, or `-1`.
    pub day: i32,
    /// `hh` -- hour 0..23, or `-1`.
    pub hour: i32,
    /// `mi` -- minute 0..59, or `-1`.
    pub minute: i32,
    /// `ss` -- second 0..60, or `-1`.
    pub second: i32,
    /// `ns` -- nanoseconds, or `-1`.
    pub nanosecond: i32,
    /// `offset` -- UTC offset in minutes (`9999` = none, `0` for `Z`).
    pub offset: i32,
    /// The normalized timezone token: empty, `"Z"`, or `+/-hhmm`.
    pub iso_timezone: Vec<u8>,
    /// `ret != 0` -- the string was invalid (the C emits a `cob_runtime_warning` and bails).
    pub invalid: bool,
    /// `true` for the `@sssssssss` epoch form (the seconds-since-epoch branch).
    pub is_epoch: bool,
    /// For `is_epoch`, the raw digits after `@` (the epoch payload; the parse is the boundary).
    pub epoch_payload: Vec<u8>,
}

/// Parse up to `maxdigits` digit bytes starting at `p[i]`, returning `(value, digits_consumed,
/// next_index)`. Stops at the first non-digit or after `maxdigits`.
fn parse_digits(p: &[u8], mut i: usize, maxdigits: usize) -> (i32, usize, usize) {
    let mut v = 0i32;
    let mut n = 0usize;
    while i < p.len() {
        if is_invalid_digit_data(p[i]) {
            break;
        }
        v = v.saturating_mul(10).saturating_add(cob_d2i(p[i]));
        i += 1;
        n += 1;
        if n == maxdigits {
            // C does p++ after hitting the max
            break;
        }
    }
    (v, n, i)
}

/// Port of `common.c:check_current_date` -- validate/parse a `COB_CURRENT_DATE` override string into
/// date components.
///
/// **Boundary:** the trailing `localtime`/`mktime` normalization that fills `cob_time_constant`
/// (day-of-week, day-of-year, DST) is the OS clock boundary. This ports the **pure parse**: skipping
/// leading quotes/space, the `@epoch` branch, the year/month/day and hour/minute/second/nanosecond
/// extraction with their range checks, the `Y`/`M`/`D`/`H`/`S` template-skip fallbacks, and the
/// `Z`/`+hhmm`/`-hhmm` timezone normalization -- producing the same `(components, ret)` the C reaches.
pub fn check_current_date(date: &[u8]) -> CurrentDate {
    let mut out = CurrentDate {
        year: -1,
        month: -1,
        day: -1,
        hour: -1,
        minute: -1,
        second: -1,
        nanosecond: -1,
        offset: 9999,
        ..Default::default()
    };
    let p = date;
    let mut i = 0usize;

    // skip quotes and space-characters
    while i < p.len() && (p[i] == 0x27 || p[i] == 0x22 || p[i].is_ascii_whitespace()) {
        i += 1;
    }

    // @sssssssss seconds since epoch
    if i < p.len() && p[i] == b'@' {
        out.is_epoch = true;
        out.epoch_payload = p[i + 1..].to_vec();
        return out;
    }

    let mut ret = false;

    // ----- date: year -----
    if i < p.len() {
        let (yr, n, ni) = parse_digits(p, i, 4);
        i = ni;
        if n != 2 && n != 4 {
            if i < p.len() && p[i] == b'Y' {
                while i < p.len() && p[i] == b'Y' {
                    i += 1;
                }
            } else {
                ret = true;
            }
            out.year = -1;
        } else {
            out.year = if yr < 100 { yr + 2000 } else { yr };
        }
        if i < p.len() && (p[i] == b'/' || p[i] == b'-') {
            i += 1;
        }
    }
    // ----- month -----
    if i < p.len() {
        let (mm, n, ni) = parse_digits(p, i, 2);
        i = ni;
        if n != 2 {
            if i < p.len() && p[i] == b'M' {
                while i < p.len() && p[i] == b'M' {
                    i += 1;
                }
            } else {
                ret = true;
            }
            out.month = -1;
        } else if !(1..=12).contains(&mm) {
            ret = true;
            out.month = mm;
        } else {
            out.month = mm;
        }
        if i < p.len() && (p[i] == b'/' || p[i] == b'-') {
            i += 1;
        }
    }
    // ----- day -----
    if i < p.len() {
        let (dd, n, ni) = parse_digits(p, i, 2);
        i = ni;
        if n != 2 {
            if i < p.len() && p[i] == b'D' {
                while i < p.len() && p[i] == b'D' {
                    i += 1;
                }
            } else {
                ret = true;
            }
            out.day = -1;
        } else if !(1..=31).contains(&dd) {
            ret = true;
            out.day = dd;
        } else {
            out.day = dd;
        }
    }

    // ----- time: hour -----
    if i < p.len() {
        while i < p.len() && p[i].is_ascii_whitespace() {
            i += 1;
        }
        let (hh, n, ni) = parse_digits(p, i, 2);
        i = ni;
        if n != 2 {
            if i < p.len() && p[i] == b'H' {
                while i < p.len() && p[i] == b'H' {
                    i += 1;
                }
            } else {
                ret = true;
            }
            out.hour = -1;
        } else if hh > 23 {
            ret = true;
            out.hour = hh;
        } else {
            out.hour = hh;
        }
        if i < p.len() && (p[i] == b':' || p[i] == b'-') {
            i += 1;
        }
    }
    // ----- minute -----
    if i < p.len() {
        let (mi, n, ni) = parse_digits(p, i, 2);
        i = ni;
        if n != 2 {
            if i < p.len() && p[i] == b'M' {
                while i < p.len() && p[i] == b'M' {
                    i += 1;
                }
            } else {
                ret = true;
            }
            out.minute = -1;
        } else if mi > 59 {
            ret = true;
            out.minute = mi;
        } else {
            out.minute = mi;
        }
        if i < p.len() && (p[i] == b':' || p[i] == b'-') {
            i += 1;
        }
    }
    // ----- second -----
    if i < p.len() && p[i] != b'Z' && p[i] != b'+' && p[i] != b'-' {
        let (ss, n, ni) = parse_digits(p, i, 2);
        i = ni;
        if n != 2 {
            if i < p.len() && p[i] == b'S' {
                while i < p.len() && p[i] == b'S' {
                    i += 1;
                }
            } else {
                ret = true;
            }
            out.second = -1;
        } else if ss > 60 {
            ret = true;
            out.second = ss;
        } else {
            out.second = ss;
        }
    }
    // ----- nanoseconds -----
    if i < p.len() && p[i] != b'Z' && p[i] != b'+' && p[i] != b'-' {
        if i < p.len() && (p[i] == b'.' || p[i] == b':') {
            i += 1;
        }
        let (ns, _n, ni) = parse_digits(p, i, 9);
        i = ni;
        out.nanosecond = ns;
    }

    // ----- UTC offset -----
    if i < p.len() && p[i] == b'Z' {
        out.offset = 0;
        out.iso_timezone = vec![b'Z'];
    } else if i < p.len() && (p[i] == b'+' || p[i] == b'-') {
        // operate on a buffer to drop an included ":" (snprintf into iso_timezone[7])
        let mut tz: Vec<u8> = Vec::new();
        for &b in &p[i..] {
            if tz.len() == 6 {
                break;
            }
            tz.push(b);
        }
        let mut len = tz.len() as i32;
        if len == 3 {
            // memcpy(iso_timezone + 3, "00", 3) -> "+hh" becomes "+hh00"
            tz.extend_from_slice(b"00");
        } else if len >= 5 && tz.len() >= 4 && tz[3] == b':' {
            // snprintf(&iso_timezone[3], 3, "%s", p+4): copy up to 2 bytes from p[i+4] over the ':'
            let mut repl: Vec<u8> = Vec::new();
            for &b in p.get(i + 4..).unwrap_or(&[]) {
                if repl.len() == 2 {
                    break;
                }
                repl.push(b);
            }
            tz.truncate(3);
            tz.extend_from_slice(&repl);
            len -= 1;
        }
        if len > 5 {
            ret = true;
        }
        // count leading digits of tz[1..5]
        let mut digs = 0usize;
        for k in 1..5 {
            match tz.get(k) {
                Some(&c) if c != 0 && !is_invalid_digit_data(c) => digs += 1,
                _ => break,
            }
        }
        if digs == 4 {
            let mut offset = cob_d2i(tz[1]) * 60 * 10
                + cob_d2i(tz[2]) * 60
                + cob_d2i(tz[3]) * 10
                + cob_d2i(tz[4]);
            if tz[0] == b'-' {
                offset = -offset;
            }
            out.offset = offset;
            out.iso_timezone = tz;
        } else {
            ret = true;
            out.iso_timezone.clear();
        }
    }

    out.invalid = ret;
    out
}

// ======================================================================================================
// check_valid_dir -- the pure length/char check on a directory name.
// ======================================================================================================

/// Port of `common.c:check_valid_dir` -- check a directory-name string is valid.
///
/// **Boundary:** the `stat(2)` + `S_ISDIR` test is the filesystem boundary. This ports the **pure
/// pre-check**: `strlen(dir) > COB_NORMAL_MAX` returns `1` (invalid) before any syscall. Returns `0`
/// when the length is acceptable (the caller then performs the `stat`), `1` when too long.
pub fn check_valid_dir(dir: &[u8]) -> i32 {
    if dir.len() > COB_NORMAL_MAX {
        1
    } else {
        0
    }
}

// ======================================================================================================
// cob_common_init -- the common-module init.
// ======================================================================================================

/// The pure decision [`cob_common_init`] reaches on the `_WIN32` `COB_UNIX_LF` branch: whether the
/// three standard streams should be put into binary mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct CommonInitDecision {
    /// `use_unix_lf` -- set the std streams to `_O_BINARY` (Windows only; always `false` elsewhere).
    pub use_unix_lf: bool,
}

/// Port of `common.c:cob_common_init` -- the common-module init.
///
/// **Boundary:** the `ENABLE_NLS` `bindtextdomain`/`textdomain` setup and the `_WIN32`
/// `_setmode(_O_BINARY)` calls are the OS boundary. The only pure decision is the `COB_UNIX_LF`
/// parse on Windows: when `setptr` is non-null the value is fed through `set_config_val_by_name` (a
/// config side effect); otherwise the leading char selects binary mode (`Y/y/O/o/T/t/1`). `cob_unix_lf`
/// supplies the already-parsed config flag for the `setptr` path. On non-Windows this is always
/// `use_unix_lf == false`.
pub fn cob_common_init(
    unix_lf_env: Option<&[u8]>,
    setptr: bool,
    cob_unix_lf: bool,
) -> CommonInitDecision {
    match unix_lf_env {
        None => CommonInitDecision { use_unix_lf: false },
        Some(s) => {
            if setptr {
                CommonInitDecision {
                    use_unix_lf: cob_unix_lf,
                }
            } else if let Some(&c) = s.first() {
                let use_unix_lf = matches!(c, b'Y' | b'y' | b'O' | b'o' | b'T' | b't' | b'1');
                CommonInitDecision { use_unix_lf }
            } else {
                CommonInitDecision { use_unix_lf: false }
            }
        }
    }
}

// ======================================================================================================
// var_print -- the build/config line formatter used by print_info / print_info_detailed.
// ======================================================================================================

/// Port of `common.c:var_print` -- format one `"<label> : <value>"` info line (with word-wrap).
///
/// **Boundary:** the C `printf`s to stdout; here the assembled bytes are returned and the write is the
/// boundary. Mirrors the live `#else` branch of the C (`format 0` = `"<label,24> : "`, any other =
/// `"  env: <label,18> : "`). With no value the line is just the label and a newline. When the value
/// fits in `CB_IVAL_SIZE` it is printed as-is; otherwise it is word-wrapped on spaces, continuation
/// lines indented to the value column (`CB_IMSG_SIZE + 3`). The `(default)` substitution is only
/// reached for `format == 1` with a matching value -- modeled via `default_val`.
pub fn var_print(
    msg: &[u8],
    val: Option<&[u8]>,
    default_val: Option<&[u8]>,
    format: u32,
) -> Vec<u8> {
    let mut out = Vec::new();

    // label column
    if format == 0 {
        push_padded(&mut out, msg, CB_IMSG_SIZE);
        out.extend_from_slice(b" : ");
    } else {
        out.extend_from_slice(b"  env: ");
        // lablen = CB_IMSG_SIZE - 2 - strlen("env") - 2 = 24 - 2 - 3 - 2 = 17
        let lablen = CB_IMSG_SIZE - 2 - 3 - 2;
        push_padded(&mut out, msg, lablen);
        out.extend_from_slice(b" : ");
    }

    // resolve the effective value (with the (default) substitution / fallback).
    let dflt_subst;
    let effective: Option<Vec<u8>> = match (val, default_val) {
        (None, dv) if dv.is_none() || dv == Some(&b""[..]) => {
            out.push(b'\n');
            return out;
        }
        (Some(v), Some(dv)) if format == 1 && (v.first() == Some(&b'0') || v == dv) => {
            // val = default_val " (default)"
            let mut s = dv.to_vec();
            s.extend_from_slice(b" (default)");
            dflt_subst = s;
            Some(dflt_subst.clone())
        }
        (None, Some(dv)) => Some(dv.to_vec()),
        (Some(v), _) => Some(v.to_vec()),
        (None, None) => {
            out.push(b'\n');
            return out;
        }
    };

    let value = match effective {
        Some(v) => v,
        None => {
            out.push(b'\n');
            return out;
        }
    };

    // short value: print as-is.
    if value.len() <= CB_IVAL_SIZE {
        out.extend_from_slice(&value);
        out.push(b'\n');
        return out;
    }

    // long value: word-wrap on spaces, continuation indented to CB_IMSG_SIZE + 3.
    let mut n = 0usize;
    for token in value.split(|&b| b == b' ') {
        if token.is_empty() {
            continue;
        }
        let toklen = token.len() + 1;
        if n + toklen > CB_IVAL_SIZE {
            if n != 0 {
                out.push(b'\n');
                for _ in 0..(CB_IMSG_SIZE + 3) {
                    out.push(b' ');
                }
            }
            n = 0;
        }
        if n != 0 {
            out.push(b' ');
        }
        out.extend_from_slice(token);
        n += toklen;
    }
    out.push(b'\n');
    out
}

/// Append `s` left-justified in a `width`-byte field (`printf("%-*.*s")`): truncated to `width`, then
/// space-padded up to `width`.
fn push_padded(out: &mut Vec<u8>, s: &[u8], width: usize) {
    let take = s.len().min(width);
    out.extend_from_slice(&s[..take]);
    for _ in take..width {
        out.push(b' ');
    }
}

// ======================================================================================================
// print_info / print_info_detailed -- the build/feature summary.
// ======================================================================================================

/// One feature/build line for [`print_info_detailed`]: a label and its value, in `var_print` form.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InfoLine {
    /// The label (first `var_print` arg).
    pub label: Vec<u8>,
    /// The value (second `var_print` arg); `None` models a NULL value (label-only line).
    pub value: Option<Vec<u8>>,
    /// The `format` arg (0 = normal, 1 = `env:` continuation).
    pub format: u32,
}

/// Port of `common.c:print_info_detailed` -- assemble the build/feature summary.
///
/// **Boundary:** the C resolves every line from compile-time `#ifdef`s, `getenv`, and the screenio /
/// math / library version probes, then `printf`s them. Since which lines appear is entirely
/// build-configuration dependent, this port takes the **already-resolved** `header` (the
/// `print_version` + `"build information"` text the C `puts`/`print_version` emit) and the ordered
/// list of `lines`, and renders each through [`var_print`], returning the full output bytes. This is
/// the pure assembly of the report for given injected inputs; the version probe + stdout write is the
/// boundary.
pub fn print_info_detailed(header: &[u8], lines: &[InfoLine]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(header);
    for line in lines {
        out.extend_from_slice(&var_print(
            &line.label,
            line.value.as_deref(),
            Some(b""),
            line.format,
        ));
    }
    out
}

/// Port of `common.c:print_info` -- `print_info_detailed(0)`. Same assembly with the non-verbose line
/// set; here it simply delegates to [`print_info_detailed`] with the injected `header`/`lines`.
pub fn print_info(header: &[u8], lines: &[InfoLine]) -> Vec<u8> {
    print_info_detailed(header, lines)
}

// ======================================================================================================
// print_runtime_conf -- the runtime configuration table.
// ======================================================================================================

/// One resolved runtime-config row for [`print_runtime_conf`] -- the already-evaluated pieces the C
/// reads out of `gc_conf[i]` + `get_config_val`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfRow {
    /// The settings-group index (`gc_conf[i].env_group`), `1..GRP_MAX`.
    pub group: u32,
    /// The name shown in the label column (`env_name` or `conf_name`, already selected).
    pub name: Vec<u8>,
    /// The stringified current value (`get_config_val` output).
    pub value: Vec<u8>,
    /// The original-value note shown in `(...)` (`orgvalue`); empty for none.
    pub orgvalue: Vec<u8>,
    /// The 4-char status prefix the C prints before `": "` (`"env"`, `"Ovr"`, `"    "`, `"  N "`, ...).
    pub status_prefix: Vec<u8>,
    /// The trailing tag (`"(default)"`, `"(reset)"`, `"(set by X)"`, ...); empty for none.
    pub tag: Vec<u8>,
}

/// Port of `common.c:print_runtime_conf` -- assemble the runtime configuration table.
///
/// **Boundary:** the C walks the live `gc_conf[]`/`cobsetptr` config state, calls `get_config_val`,
/// and `printf`s, then appends the `setlocale(3)` `LC_*` lines. This port takes the **already-resolved**
/// rows (grouped, with value/orgvalue/status/tag computed) plus the group `headings` and the leading
/// `header` line, and renders the grouped table: for each non-empty group a blank line (except the
/// first) and the ` <heading>` line, then each row as `<status>: <name,hdlen> : <value> (<org>) <tag>`.
/// `hdlen` is the label column width (`max(15, longest name)`); the value-wrap at column 71-hdlen is
/// preserved. Returns the assembled bytes; the config walk + stdout write + `LC_*` `setlocale` tail are
/// the boundary.
pub fn print_runtime_conf(header: &[u8], headings: &[(u32, Vec<u8>)], rows: &[ConfRow]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(header);
    out.push(b'\n');

    // hdlen = max(15, longest name)
    let mut hdlen = 15usize;
    for r in rows {
        if r.name.len() > hdlen {
            hdlen = r.name.len();
        }
    }

    // groups in ascending order; the C iterates j = 1 .. GRP_MAX.
    let mut groups: Vec<u32> = rows.iter().map(|r| r.group).collect();
    groups.sort_unstable();
    groups.dedup();

    let mut printed_any_group = false;
    for &g in &groups {
        let group_rows: Vec<&ConfRow> = rows.iter().filter(|r| r.group == g).collect();
        if group_rows.is_empty() {
            continue;
        }
        if printed_any_group {
            out.push(b'\n');
        }
        printed_any_group = true;
        out.push(b' ');
        if let Some((_, h)) = headings.iter().find(|(gi, _)| *gi == g) {
            out.extend_from_slice(h);
        }
        out.push(b'\n');

        for r in group_rows {
            out.extend_from_slice(&r.status_prefix);
            out.extend_from_slice(b": ");
            push_padded(&mut out, &r.name, hdlen);
            out.extend_from_slice(b" : ");

            // value with wrap at plen = 71 - hdlen
            let plen = 71usize.saturating_sub(hdlen);
            let value = &r.value;
            if plen != 0 && value.len() > plen {
                let mut k = 0usize;
                while value.len() - k > plen {
                    out.extend_from_slice(&value[k..k + plen]);
                    out.push(b'\n');
                    out.extend_from_slice(b"      ");
                    push_padded(&mut out, b"", hdlen);
                    out.extend_from_slice(b" : ");
                    k += plen;
                }
                out.extend_from_slice(&value[k..]);
            } else {
                out.extend_from_slice(value);
            }

            if !r.orgvalue.is_empty() {
                out.extend_from_slice(b" (");
                out.extend_from_slice(&r.orgvalue);
                out.push(b')');
            }
            if !r.tag.is_empty() {
                out.push(b' ');
                out.extend_from_slice(&r.tag);
            }
            out.push(b'\n');
        }
    }
    out
}

// ======================================================================================================
// Tests
// ======================================================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sig_handler_segv_message_and_termination() {
        // SIGSEGV with core-on-error: ss_terminate_routines path, message contains the description.
        let d = cob_sig_handler(SIGSEGV, b"prog.cob:12: ", true, 1);
        assert_eq!(d.termination, Termination::HardError);
        assert!(!d.skip_dump);
        assert_eq!(d.reraise_sig, SIGSEGV);
        let s = String::from_utf8_lossy(&d.stderr_text);
        assert!(
            s.contains("attempt to reference invalid memory address"),
            "{s}"
        );
        assert!(s.contains("(signal SIGSEGV)"), "{s}");
        assert!(s.contains("prog.cob:12: "), "{s}");
    }

    #[test]
    fn sig_handler_term_skips_dump_and_normal_terminate() {
        let d = cob_sig_handler(SIGTERM, b"", true, 0);
        assert!(d.skip_dump);
        assert_eq!(d.termination, Termination::Normal);
        let s = String::from_utf8_lossy(&d.stderr_text);
        assert!(s.contains("(signal SIGTERM)"), "{s}");
    }

    #[test]
    fn signal_regime_and_table_match_current_upstream() {
        // Upstream c53ae5f80 / 3f897122a / a207a4595: signal-handler updates, stack handling and
        // the COB_SIGNAL_REGIME registration strategy (0 = take control, 1 = only if no handler
        // registered, 2 = register nothing; common.c cob_set_signal at head).
        // The candidate registers NO OS signal handlers (it is an in-process interpreter); its
        // model is the pure signal table + cob_sig_handler diagnostic decision -- the OS
        // registration machinery is a typed boundary, and the candidate's effective regime is
        // '2'-like (recorded). This court pins the table identity and the regime row.
        let tbl = signals();
        let by_sig = |s: i32| -> &'static SignalEntry { tbl.iter().find(|e| e.sig == s).unwrap() };
        assert_eq!(by_sig(2).for_set, 1, "SIGINT");
        assert_eq!(by_sig(15).for_set, 1, "SIGTERM");
        assert_eq!(by_sig(11).for_set, 2, "SIGSEGV hard error");
        assert_eq!(by_sig(6).for_set, 0, "SIGABRT not taken");
        // the regime row is part of the runtime-conf surface (verified in common_runtimeconf).
    }

    #[test]
    fn sig_handler_unknown_signal_notice() {
        let d = cob_sig_handler(9999, b"", true, 0);
        let s = String::from_utf8_lossy(&d.stderr_text);
        assert!(
            s.contains("cob_sig_handler caught not handled signal: 9999"),
            "{s}"
        );
        assert!(s.contains("(signal unknown)"), "{s}");
    }

    #[test]
    fn sig_handler_core4_reraises_sigabrt() {
        let d = cob_sig_handler(SIGSEGV, b"", true, 4);
        assert_eq!(d.reraise_sig, SIGABRT);
    }

    #[test]
    fn raise_dispatch_via_os_on_linux() {
        assert_eq!(cob_raise(SIGABRT, true), RaiseDispatch::ViaOs(SIGABRT));
        assert_eq!(
            cob_raise(SIGABRT, false),
            RaiseDispatch::ExternalHandlerOnly
        );
    }

    #[test]
    fn set_signal_picks_for_set_rows() {
        let actions = cob_set_signal();
        // SIGSEGV (11) is for_set==2 -> direct; SIGINT (2) is for_set==1 -> conditional.
        let segv = actions.iter().find(|a| a.sig == SIGSEGV).unwrap();
        assert!(segv.direct);
        let int = actions.iter().find(|a| a.sig == SIGINT).unwrap();
        assert!(!int.direct);
        // SIGKILL (9) has for_set==0 -> not installed.
        assert!(actions.iter().all(|a| a.sig != 9));
    }

    #[test]
    fn reg_sighnd_needs_set_when_not_initialized() {
        assert!(cob_reg_sighnd(false).must_set_signal);
        assert!(!cob_reg_sighnd(true).must_set_signal);
    }

    #[test]
    fn arg_mismatch_dispatch_and_messages() {
        assert_eq!(raise_arg_mismatch(true), 0); // ON EXCEPTION -> return to caller
        assert_eq!(raise_arg_mismatch(false), 1);
        assert_eq!(
            raise_arg_mismatch_message("'X'", ArgMismatch::NotPassed),
            b"LINKAGE item 'X' not passed by caller".to_vec()
        );
        assert_eq!(
            raise_arg_mismatch_message(
                "'X' of 'Y'",
                ArgMismatch::TooSmall {
                    callee_size: 10,
                    caller_size: 4
                }
            ),
            b"LINKAGE item 'X' of 'Y' (size 10) too small in the caller (size 4)".to_vec()
        );
    }

    #[test]
    fn check_current_date_full_iso() {
        let d = check_current_date(b"2021-03-15 12:30:45");
        assert!(!d.invalid);
        assert_eq!(d.year, 2021);
        assert_eq!(d.month, 3);
        assert_eq!(d.day, 15);
        assert_eq!(d.hour, 12);
        assert_eq!(d.minute, 30);
        assert_eq!(d.second, 45);
    }

    #[test]
    fn check_current_date_two_digit_year() {
        let d = check_current_date(b"21/03/15");
        assert_eq!(d.year, 2021);
        assert_eq!(d.month, 3);
        assert_eq!(d.day, 15);
        assert!(!d.invalid);
    }

    #[test]
    fn check_current_date_epoch() {
        let d = check_current_date(b"@1600000000");
        assert!(d.is_epoch);
        assert_eq!(d.epoch_payload, b"1600000000".to_vec());
    }

    #[test]
    fn check_current_date_timezone_z_and_offset() {
        let z = check_current_date(b"2021-03-15 00:00:00Z");
        assert_eq!(z.offset, 0);
        assert_eq!(z.iso_timezone, b"Z".to_vec());

        // +0530 -> offset = 5*600 + 30 = ... per the C: d2i(5)*600 + d2i(5? )...
        let tz = check_current_date(b"2021-03-15 00:00:00+0530");
        // C: 5*60*10 + 3*60 + 0*10 + ... wait: digits are 0,5,3,0 -> tz[1..5] = '0','5','3','0'
        // offset = 0*600 + 5*60 + 3*10 + 0 = 330
        assert_eq!(tz.offset, 330);
        assert_eq!(tz.iso_timezone, b"+0530".to_vec());

        // negative offset with a colon, normalized
        let neg = check_current_date(b"2021-03-15 00:00:00-05:00");
        // tz "-05:00" -> drop colon -> "-0500"; tz[1..5]='0','5','0','0' -> 0*600+5*60+0+0 = 300; negative
        assert_eq!(neg.offset, -300);
        assert_eq!(neg.iso_timezone, b"-0500".to_vec());
    }

    #[test]
    fn check_current_date_invalid_month() {
        let d = check_current_date(b"2021-13-15");
        assert!(d.invalid);
    }

    #[test]
    fn check_valid_dir_length_gate() {
        assert_eq!(check_valid_dir(b"/tmp"), 0);
        let long = vec![b'a'; COB_NORMAL_MAX + 1];
        assert_eq!(check_valid_dir(&long), 1);
        let exact = vec![b'a'; COB_NORMAL_MAX];
        assert_eq!(check_valid_dir(&exact), 0);
    }

    #[test]
    fn common_init_unix_lf() {
        assert!(!cob_common_init(None, false, false).use_unix_lf);
        assert!(cob_common_init(Some(b"Y"), false, false).use_unix_lf);
        assert!(cob_common_init(Some(b"true"), false, false).use_unix_lf); // 't'
        assert!(!cob_common_init(Some(b"no"), false, false).use_unix_lf); // 'n'
                                                                          // setptr path uses the pre-parsed flag
        assert!(cob_common_init(Some(b"whatever"), true, true).use_unix_lf);
        assert!(!cob_common_init(Some(b"whatever"), true, false).use_unix_lf);
    }

    #[test]
    fn var_print_short_and_label() {
        let out = var_print(b"CC", Some(b"gcc"), Some(b""), 0);
        // label padded to 24, " : ", value, newline
        let s = String::from_utf8_lossy(&out);
        assert!(s.starts_with("CC"), "{s}");
        assert!(s.ends_with("gcc\n"), "{s}");
        assert!(s.contains(" : "), "{s}");
        // total label column is 24 wide
        assert_eq!(&out[..24], b"CC                      ");
    }

    #[test]
    fn var_print_no_value_label_only() {
        let out = var_print(b"X", None, Some(b""), 0);
        assert!(out.ends_with(b"\n"));
        // no value text after the " : "
        let s = String::from_utf8_lossy(&out);
        assert!(s.trim_end().ends_with(":"), "{s}");
    }

    #[test]
    fn var_print_env_format() {
        let out = var_print(b"COB_VARSEQ_FORMAT", Some(b"0"), Some(b"0"), 1);
        let s = String::from_utf8_lossy(&out);
        assert!(s.starts_with("  env: "), "{s}");
        assert!(s.contains("(default)"), "{s}");
    }

    #[test]
    fn var_print_long_value_wraps() {
        let long = b"aaaa bbbb cccc dddd eeee ffff gggg hhhh iiii jjjj kkkk llll mmmm nnnn";
        let out = var_print(b"CFLAGS", Some(long), Some(b""), 0);
        // wrapped across multiple lines
        assert!(out.iter().filter(|&&b| b == b'\n').count() >= 2);
    }

    #[test]
    fn print_info_assembles_lines() {
        let lines = vec![
            InfoLine {
                label: b"CC".to_vec(),
                value: Some(b"gcc".to_vec()),
                format: 0,
            },
            InfoLine {
                label: b"64bit-mode".to_vec(),
                value: Some(b"yes".to_vec()),
                format: 0,
            },
        ];
        let out = print_info(b"build information\n", &lines);
        let s = String::from_utf8_lossy(&out);
        assert!(s.starts_with("build information\n"), "{s}");
        assert!(s.contains("gcc"), "{s}");
        assert!(s.contains("yes"), "{s}");
    }

    #[test]
    fn print_runtime_conf_grouped_table() {
        let headings = vec![
            (1u32, b"CALL configuration".to_vec()),
            (4u32, b"Miscellaneous".to_vec()),
        ];
        let rows = vec![
            ConfRow {
                group: 1,
                name: b"COB_LIBRARY_PATH".to_vec(),
                value: b"/usr/lib".to_vec(),
                orgvalue: Vec::new(),
                status_prefix: b"    ".to_vec(),
                tag: b"(default)".to_vec(),
            },
            ConfRow {
                group: 4,
                name: b"COB_DISP_WARN".to_vec(),
                value: b"true".to_vec(),
                orgvalue: b"Y".to_vec(),
                status_prefix: b"env ".to_vec(),
                tag: Vec::new(),
            },
        ];
        let out = print_runtime_conf(b"GnuCOBOL 3.2.0 runtime configuration\n", &headings, &rows);
        let s = String::from_utf8_lossy(&out);
        assert!(s.contains(" CALL configuration\n"), "{s}");
        assert!(s.contains("COB_LIBRARY_PATH"), "{s}");
        assert!(s.contains("(default)"), "{s}");
        assert!(s.contains(" Miscellaneous\n"), "{s}");
        assert!(s.contains("(Y)"), "{s}");
    }

    #[test]
    fn print_info_detailed_assembles_header_and_lines() {
        let lines = vec![
            InfoLine {
                label: b"CC".to_vec(),
                value: Some(b"gcc".to_vec()),
                format: 0,
            },
            InfoLine {
                label: b"version".to_vec(),
                value: Some(b"3.2.0".to_vec()),
                format: 0,
            },
        ];
        let out = print_info_detailed(b"build information\n", &lines);
        let s = String::from_utf8_lossy(&out);
        assert!(s.starts_with("build information\n"), "{s}");
        assert!(s.contains("gcc"), "{s}");
        assert!(s.contains("3.2.0"), "{s}");
        // each InfoLine renders through var_print: the label column is 24 wide, then " : ".
        assert_eq!(
            out[b"build information\n".len()..][..24],
            b"CC                      "[..]
        );
        // header-only with no lines is just the header.
        assert_eq!(print_info_detailed(b"hdr\n", &[]), b"hdr\n".to_vec());
    }

    #[test]
    fn get_signal_entry_local_copy_finds_segv() {
        // the module-local get_signal_entry: real row lookup + sentinel fallback.
        let e = get_signal_entry(11);
        assert_eq!(e.sig, 11);
        assert_eq!(e.shortname, "SIGSEGV");
        assert_eq!(e.description, "attempt to reference invalid memory address");
        let u = get_signal_entry(9999);
        assert_eq!(u.sig, -1);
        assert_eq!(u.shortname, "unknown");
    }
}
