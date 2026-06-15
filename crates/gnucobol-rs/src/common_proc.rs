//! Port of the **process / OS-leaf** built-in routines of `libcob/common.c` -- the CALL "SYSTEM",
//! hosted-environment, process-control (fork/waitpid/getpid), sleep/nanosleep, CBL_X91 switch
//! multiplexer, the error/exit handler registry, the getopt wrapper, and ACCEPT FROM USER-NAME.
//!
//! These are the documented OS-boundary leaves: `system()`, `fork()`, `waitpid()`, `getpid()`,
//! `nanosleep()`/`sleep()`, `getlogin()`, the platform precise-clock selection, and the `getopt`
//! engine are not reproduced here -- they are the runtime boundary (the same BDB/EXTFH boundary
//! pattern used elsewhere in the port). What **is** ported, faithfully and pure, is every
//! decision/parse/marshal that surrounds those leaves: the SYSTEM command-string trim + size guard
//! + return-code mapping, the hosted-name dispatch, the sleep-duration clamp to `MAX_SLEEP_TIME`,
//! the nanosleep timespec split and the non-`HAVE_NANO_SLEEP` millisecond rounding, the X91 switch
//! cases, the handler add/remove/priority logic, the getopt return-value byte mapping, and the
//! username copy/pad. Each `pub fn` carries the exact C name so the port index can match it.
//!
//! `cob_get_llint`, `cob_get_int`, and field reads are modelled by passing the already-decoded
//! integer (the field decoder is ported in the numeric/field modules); the global runtime state
//! (`cob_switch`, `cob_process_id`, handler lists, `cob_argc/argv`) is passed in as explicit
//! structs/params rather than referenced as statics.

#![forbid(unsafe_code)]

// ---------------------------------------------------------------------------
// CALL "SYSTEM" -- cob_sys_system
// ---------------------------------------------------------------------------

/// `COB_MEDIUM_MAX` from `common.h` -- the size guard SYSTEM applies to its command line.
pub const COB_MEDIUM_MAX: usize = 8192 - 1;

/// The modelled outcome of `common.c:cob_sys_system` once its *pure* part has run: either the
/// command line has been resolved (trimmed of trailing SPACE/NUL, optionally re-quoted on WIN32)
/// and is ready to be handed to the `system()` boundary, or an early `return` value has been
/// decided without ever calling `system()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemOutcome {
    /// The pure part produced a command; `system(command)` is the boundary, whose status the C
    /// function returns verbatim.
    Run { command: Vec<u8> },
    /// The pure part decided a return value without invoking `system()` (empty/blank command, or
    /// the over-length warning path which returns `1`).
    Return(i32),
}

/// Port of the **pure** part of `common.c:cob_sys_system` (CALL "SYSTEM").
///
/// Models `COB_CHK_PARMS(SYSTEM, 1)` via the presence of the command field: `cmdline` is
/// `cob_procedure_params[0]` (its `size` is `cmdline.len()`). The C code walks back from the last
/// byte skipping `' '` and `0`, finds the meaningful length `i`, guards `i > COB_MEDIUM_MAX`, then
/// (on WIN32) optionally wraps a leading/trailing `"` pair, and builds the NUL-terminated
/// `command`. The actual `system(command)` call -- and the WIFSIGNALED warning + returned status --
/// is the boundary, represented by [`SystemOutcome::Run`].
///
/// `win32` selects the `_WIN32` re-quoting branch so both compile-time variants are testable.
pub fn cob_sys_system(cmdline: Option<&[u8]>, win32: bool) -> SystemOutcome {
    let cmd = match cmdline {
        None => return SystemOutcome::Return(1),
        Some(c) => c,
    };
    if cmd.is_empty() {
        return SystemOutcome::Return(1);
    }
    // size i = field size; i--; then skip trailing ' ' / 0 down to (but not past) index 0.
    let mut i = cmd.len() - 1;
    loop {
        if cmd[i] != b' ' && cmd[i] != 0 {
            break;
        }
        if i == 0 {
            break;
        }
        i -= 1;
    }
    // C: the `if (i > 0)` guard -- when i decremented to 0 the command is treated as empty.
    if i == 0 {
        // Re-check index 0 itself: if it is a real byte the loop above broke on it with i==0,
        // but C only enters the build block when i > 0, so a single meaningful byte at index 0
        // still falls through to `return 1`.
        return SystemOutcome::Return(1);
    }
    if i > COB_MEDIUM_MAX {
        // cob_runtime_warning(...); return 1;
        return SystemOutcome::Return(1);
    }
    // Build NUL-terminated command of meaningful bytes cmd[0..=i].
    let meaningful = &cmd[..=i];
    if win32 && i > 2 && cmd[0] == 0x22 && cmd[i] == 0x22 && (cmd[1] != 0x22 || cmd[i - 1] != 0x22) {
        // command = '"' + cmd[0..=i] + '"' + NUL
        let mut command = Vec::with_capacity(i + 4);
        command.push(0x22);
        command.extend_from_slice(meaningful);
        command.push(0x22);
        command.push(0);
        SystemOutcome::Run { command }
    } else {
        let mut command = Vec::with_capacity(i + 2);
        command.extend_from_slice(meaningful);
        command.push(0);
        SystemOutcome::Run { command }
    }
}

// ---------------------------------------------------------------------------
// CBL_GC_HOSTED -- cob_sys_hosted
// ---------------------------------------------------------------------------

/// The hosted C variable selected by `common.c:cob_sys_hosted` (CBL_GC_HOSTED). The *value*/pointer
/// it writes (`argc`, `argv`, the `FILE*`s, `&errno`, the TZ globals) is the runtime boundary; the
/// pure part is the name dispatch, modelled by this enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostedVar {
    Argc,
    Argv,
    Stdin,
    Stdout,
    Stderr,
    Errno,
    Tzname,
    Timezone,
    Daylight,
}

/// Port of the **pure dispatch** of `common.c:cob_sys_hosted` (CBL_GC_HOSTED).
///
/// `data` present models `p != NULL` (returns `1` if the destination pointer is null). `name` is
/// `cob_procedure_params[1]` and its **length** is the discriminator the C uses alongside the byte
/// compare (`i == 4 && memcmp(name,"argc",4)`, etc.). Returns `Some(var)` with `0` semantics (the
/// boundary then stores the value), or `None` for the `return 1` fall-through. `with_timezone`
/// gates the `HAVE_TIMEZONE`-only names.
pub fn cob_sys_hosted(data: bool, name: &[u8], with_timezone: bool) -> Option<HostedVar> {
    if !data {
        // C returns 1; expressed as "no variable resolved".
        return None;
    }
    let i = name.len();
    let m = |s: &[u8]| i == s.len() && name == s;
    if m(b"argc") {
        return Some(HostedVar::Argc);
    }
    if m(b"argv") {
        return Some(HostedVar::Argv);
    }
    if m(b"stdin") {
        return Some(HostedVar::Stdin);
    }
    if m(b"stdout") {
        return Some(HostedVar::Stdout);
    }
    if m(b"stderr") {
        return Some(HostedVar::Stderr);
    }
    if m(b"errno") {
        return Some(HostedVar::Errno);
    }
    if with_timezone {
        if m(b"tzname") {
            return Some(HostedVar::Tzname);
        }
        if m(b"timezone") {
            return Some(HostedVar::Timezone);
        }
        if m(b"daylight") {
            return Some(HostedVar::Daylight);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Process control -- cob_sys_getpid / cob_sys_fork / cob_sys_waitpid
// ---------------------------------------------------------------------------

/// Cached process id, the explicit-struct model of `common.c`'s `static int cob_process_id`.
#[derive(Debug, Clone, Copy, Default)]
pub struct ProcessIdCache {
    /// `0` means "not yet cached" (matching C's `if (!cob_process_id)`).
    pub cob_process_id: i32,
}

/// Port of `common.c:cob_sys_getpid`.
///
/// `getpid()` is the boundary; the pure part is the cache: if `cob_process_id` is `0` it is filled
/// from the injected `os_getpid` (what `(int)getpid()` would return), then returned.
pub fn cob_sys_getpid(cache: &mut ProcessIdCache, os_getpid: i32) -> i32 {
    if cache.cob_process_id == 0 {
        cache.cob_process_id = os_getpid;
    }
    cache.cob_process_id
}

/// The modelled result of `common.c:cob_sys_fork` (CBL_GC_FORK). The `fork()` syscall is the
/// boundary; this captures the function's return-value logic for each fork outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForkOutcome {
    /// `fork()` returned 0: child process. C resets the cached pid and returns 0.
    Child,
    /// `fork()` returned `pid > 0`: parent. Returns the child's pid.
    Parent(i32),
    /// `fork()` returned `< 0`: error. Warns and returns `-2`.
    Error,
    /// Platform without `fork()` (`!HAVE_UNISTD_H || _WIN32`): warns and returns `-1`.
    Unsupported,
}

/// Port of the **return-value logic** of `common.c:cob_sys_fork`.
///
/// `os_fork` is what the `fork()` boundary returned (`None` selects the unsupported-platform
/// branch). On the child branch the cached pid is reset to 0 (modelled here by mutating `cache`).
/// Returns the C integer result paired with the decision enum.
pub fn cob_sys_fork(cache: &mut ProcessIdCache, os_fork: Option<i32>) -> (i32, ForkOutcome) {
    match os_fork {
        None => (-1, ForkOutcome::Unsupported),
        Some(pid) => {
            if pid == 0 {
                cache.cob_process_id = 0;
                (0, ForkOutcome::Child)
            } else if pid < 0 {
                (-2, ForkOutcome::Error)
            } else {
                (pid, ForkOutcome::Parent(pid))
            }
        }
    }
}

/// Port of the **return-value logic** of `common.c:cob_sys_waitpid` (CBL_GC_WAITPID).
///
/// `p_id` is `cob_get_int(cob_procedure_params[0])` (`None` = the field absent / `else` branch).
/// `self_pid` is `cob_sys_getpid()`. `os_waitpid` models the `waitpid(pid,&status,0)` boundary:
/// `Some((wait_sts, raw_status))` where `wait_sts < 0` is an error (returns `-errno`) and otherwise
/// the function returns `WEXITSTATUS(raw_status)` -- here `wexitstatus` is the already-extracted
/// low byte. `errno` / `EINVAL` are passed in so the negative error codes match the platform.
pub fn cob_sys_waitpid(
    p_id: Option<i32>,
    self_pid: i32,
    einval: i32,
    os_waitpid: Option<(i32, i32)>,
    errno: i32,
) -> i32 {
    match p_id {
        None => -einval,
        Some(pid) => {
            if pid == self_pid {
                return -einval;
            }
            match os_waitpid {
                Some((wait_sts, wexitstatus)) => {
                    if wait_sts < 0 {
                        -errno
                    } else {
                        wexitstatus
                    }
                }
                // boundary not reached in this model -> mirror the EINVAL default
                None => -einval,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Sleep / nanosleep
// ---------------------------------------------------------------------------

/// `MAX_SLEEP_TIME` from `common.c` -- one week, in seconds (`3600*24*7`).
pub const MAX_SLEEP_TIME: i64 = 3600 * 24 * 7;

/// Port of `common.c:get_sleep_nanoseconds` -- read a `cob_field` already decoded to `nanoseconds`
/// (via `cob_get_llint`), reject negatives with `-1`, and clamp at `MAX_SLEEP_TIME * 1e9`.
pub fn get_sleep_nanoseconds(nanoseconds: i64) -> i64 {
    if nanoseconds < 0 {
        return -1;
    }
    if nanoseconds >= MAX_SLEEP_TIME * 1_000_000_000 {
        return MAX_SLEEP_TIME * 1_000_000_000;
    }
    nanoseconds
}

/// Port of `common.c:get_sleep_nanoseconds_from_seconds` -- the `C$SLEEP` variant.
///
/// `seconds` is `cob_get_llint(decimal_seconds)` (the integral seconds, used for the sign/clamp
/// guards); `decimal_nanoseconds` is the value the C derives by `cob_move`ing the *decimal* field
/// through `const_bin_nano_attr` (i.e. the seconds field scaled to nanoseconds) -- that field move
/// is the numeric-module boundary, supplied here so the non-clamped path is faithful. Negative
/// `seconds` returns `-1`; `seconds >= MAX_SLEEP_TIME` clamps to `MAX_SLEEP_TIME * 1e9`; otherwise
/// the scaled nanoseconds are returned.
pub fn get_sleep_nanoseconds_from_seconds(seconds: i64, decimal_nanoseconds: i64) -> i64 {
    if seconds < 0 {
        return -1;
    }
    if seconds >= MAX_SLEEP_TIME {
        return MAX_SLEEP_TIME * 1_000_000_000;
    }
    decimal_nanoseconds
}

/// The timespec / millisecond request `common.c:internal_nanosleep` hands to the OS sleep boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SleepRequest {
    /// `HAVE_NANO_SLEEP`: `nanosleep(&{tv_sec, tv_nsec}, NULL)`.
    Nanosleep { tv_sec: i64, tv_nsec: i64 },
    /// `_WIN32`: `Sleep(msecs)` (skipped when `msecs == 0`).
    WinSleep { msecs: u32 },
    /// other platforms: `sleep(msecs)` of *deci-seconds* rounded to seconds (skipped when 0).
    PosixSleep { secs: u32 },
}

/// Port of the **pure split** of `common.c:internal_nanosleep`.
///
/// The actual `nanosleep`/`Sleep`/`sleep` call is the boundary; this computes the argument it would
/// receive for each compile-time variant. `variant` selects which `#if` branch: `"nano"` splits
/// `nsecs` into whole seconds + remainder nanoseconds; `"win"` divides by 1e6 to milliseconds;
/// otherwise the POSIX fallback divides by 1e8 (deci-seconds) and rounds to seconds (`> 4` rounds
/// up).
pub fn internal_nanosleep(nsecs: i64, variant: &str) -> SleepRequest {
    match variant {
        "nano" => SleepRequest::Nanosleep {
            tv_sec: nsecs / 1_000_000_000,
            tv_nsec: nsecs % 1_000_000_000,
        },
        "win" => SleepRequest::WinSleep {
            msecs: (nsecs / 1_000_000) as u32,
        },
        _ => {
            // msecs here is really deci-seconds: nsecs / 100000000
            let mut msecs = (nsecs / 100_000_000) as u32;
            if msecs % 10 > 4 {
                msecs = (msecs / 10) + 1;
            } else {
                msecs /= 10;
            }
            SleepRequest::PosixSleep { secs: msecs }
        }
    }
}

/// Port of the **decision logic** of `common.c:cob_sys_oc_nanosleep` (CBL_GC_NANOSLEEP).
///
/// `nano_seconds` is `cob_procedure_params[0]` already decoded (`None` = field absent -> `-1`).
/// Returns `(rc, Some(nsecs))` where `nsecs` (from [`get_sleep_nanoseconds`]) is to be slept via the
/// `internal_nanosleep` boundary only when `> 0`; `(0, None)` means "return 0, no sleep".
pub fn cob_sys_oc_nanosleep(nano_seconds: Option<i64>) -> (i32, Option<i64>) {
    match nano_seconds {
        None => (-1, None),
        Some(v) => {
            let nsecs = get_sleep_nanoseconds(v);
            if nsecs > 0 {
                (0, Some(nsecs))
            } else {
                (0, None)
            }
        }
    }
}

/// Port of the **decision logic** of `common.c:cob_sys_sleep` (C$SLEEP).
///
/// `seconds`/`decimal_nanoseconds` feed [`get_sleep_nanoseconds_from_seconds`]; `params_present`
/// models `cob_procedure_params[0]`. A negative computed value returns `-1` (ACUCOBOL runtime
/// error); otherwise `(0, Some(nsecs))` schedules the `internal_nanosleep` boundary. No params ->
/// `(0, None)` (the `return 0; /* CHECKME */`).
pub fn cob_sys_sleep(
    params_present: bool,
    seconds: i64,
    decimal_nanoseconds: i64,
) -> (i32, Option<i64>) {
    if !params_present {
        return (0, None);
    }
    let nanoseconds = get_sleep_nanoseconds_from_seconds(seconds, decimal_nanoseconds);
    if nanoseconds < 0 {
        (-1, None)
    } else {
        (0, Some(nanoseconds))
    }
}

// ---------------------------------------------------------------------------
// Precise-time clock selection
// ---------------------------------------------------------------------------

/// The clock function `common.c:get_function_ptr_for_precise_time` selects on WIN32. The function
/// pointer itself is the OS boundary; this names the selection decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreciseTimeFunc {
    /// `GetSystemTimePreciseAsFileTime` was found in `kernel32.dll`.
    Precise,
    /// fell back to `GetSystemTimeAsFileTime`.
    Fallback,
}

/// Port of the **selection decision** of `common.c:get_function_ptr_for_precise_time`.
///
/// `GetModuleHandle`/`GetProcAddress` are the boundary; their outcome -- whether
/// `GetSystemTimePreciseAsFileTime` resolved -- is `precise_available`. The C falls back to
/// `GetSystemTimeAsFileTime` when the precise symbol is absent.
pub fn get_function_ptr_for_precise_time(precise_available: bool) -> PreciseTimeFunc {
    if precise_available {
        PreciseTimeFunc::Precise
    } else {
        PreciseTimeFunc::Fallback
    }
}

// ---------------------------------------------------------------------------
// CBL_X91 multiplexer -- cob_sys_x91
// ---------------------------------------------------------------------------

/// The runtime switch state CBL_X91 reads/writes -- the explicit-struct model of `common.c`'s
/// `int cob_switch[]` plus the `cob_debugging_mode` / `cob_ls_nulls` settings it touches.
#[derive(Debug, Clone)]
pub struct X91State {
    /// `cob_switch[0..37]` (0-7 numeric, 11-36 = A..Z).
    pub cob_switch: [i32; 37],
    pub cob_debugging_mode: bool,
    pub cob_ls_nulls: bool,
    /// `COB_MODULE_PTR->module_num_params` -- the CALL param count func 16 returns.
    pub module_num_params: u8,
}

impl Default for X91State {
    fn default() -> Self {
        X91State {
            cob_switch: [0; 37],
            cob_debugging_mode: false,
            cob_ls_nulls: false,
            module_num_params: 0,
        }
    }
}

/// Port of `common.c:cob_sys_x91` (CBL_X91 multiplexer).
///
/// `func` is `*func` (the function selector). `parm` is the `p3` byte buffer, mutated in place for
/// the get/set cases. `parm_field_size` is `cob_procedure_params[0]->size`, the discriminator the
/// MF DEBUG-switch extension checks (`>= 9`). Returns `*result` (the 1-byte status written to `p1`):
/// `0` on a handled function, `1` on the unimplemented default. The C return value of the function
/// itself is always `0` (here it is the side effect on `state`/`parm`).
///
/// Ported cases: 11 (set switches 0-7 + optional DEBUG), 12 (get switches 0-7 + optional DEBUG),
/// 13 (set A-Z -> 11-36 with the D/N special-casing), 14 (get A-Z), 16 (num CALL params). The
/// `#if 0` program-lookup / `system()`-backed EXEC (35) / FD (46-49) / dir-scan (69) cases are the
/// boundary or compiled-out and fall to the `default` -> `*result = 1`.
pub fn cob_sys_x91(state: &mut X91State, func: u8, parm: &mut [u8], parm_field_size: usize) -> u8 {
    let result: u8;
    match func {
        // Set switches (0-7) + DEBUG module
        11 => {
            for i in 0..8 {
                if i < parm.len() {
                    if parm[i] == 0 {
                        state.cob_switch[i] = 0;
                    } else if parm[i] == 1 {
                        state.cob_switch[i] = 1;
                    }
                }
            }
            if parm_field_size >= 9 && parm.len() > 8 {
                state.cob_debugging_mode = parm[8] == 1;
            }
            result = 0;
        }
        // Get switches (0-7)
        12 => {
            for i in 0..8 {
                if i < parm.len() {
                    parm[i] = state.cob_switch[i] as u8;
                }
            }
            if parm_field_size >= 9 && parm.len() > 8 {
                parm[8] = state.cob_debugging_mode as u8;
            }
            result = 0;
        }
        // Set switches (A-Z -> 11-36)
        13 => {
            // C: for (i = 11; i < 36; ++i, ++p) with p starting at parm.
            for (off, i) in (11..36).enumerate() {
                if off < parm.len() {
                    if parm[off] == 0 {
                        state.cob_switch[i] = 0;
                    } else if parm[off] == 1 {
                        state.cob_switch[i] = 1;
                    }
                }
                // 'D' - 'A' + 11 == 14, 'N' - 'A' + 11 == 24
                if i == (b'D' - b'A') as usize + 11 {
                    state.cob_debugging_mode = state.cob_switch[i] != 0;
                } else if i == (b'N' - b'A') as usize + 11 {
                    state.cob_ls_nulls = state.cob_switch[i] != 0;
                }
            }
            result = 0;
        }
        // Get switches (A-Z -> 11-36): C loops i = 1..27 over cob_switch.
        14 => {
            for (off, i) in (1..27).enumerate() {
                if off < parm.len() {
                    parm[off] = state.cob_switch[i] as u8;
                }
            }
            result = 0;
        }
        // Return number of call parameters
        16 => {
            if !parm.is_empty() {
                parm[0] = state.module_num_params;
            }
            result = 0;
        }
        // unimplemented / boundary / compiled-out functions
        _ => {
            result = 1;
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Error / exit handler registry
// ---------------------------------------------------------------------------

/// One registered exit handler -- the model of `common.c`'s `struct exit_handlerlist`. `proc` is the
/// resolved entry point identity (the function pointer is the boundary; an opaque id stands in).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExitHandler {
    pub proc: usize,
    pub priority: u8,
}

/// The registry of CBL_EXIT_PROC handlers -- the explicit-struct model of the `exit_hdlrs` list.
#[derive(Debug, Clone, Default)]
pub struct ExitHandlerList {
    /// Stored head-first, matching the C list whose newest entry is the head.
    pub exit_hdlrs: Vec<ExitHandler>,
}

/// Port of `common.c:cob_sys_exit_proc` (CBL_EXIT_PROC).
///
/// Models `COB_CHK_PARMS(CBL_EXIT_PROC, 2)` by requiring both params. `dispo` is `*dispo` (the
/// install flag). `proc` is `*p`, the resolved entry point (`None` models `!p || !*p` -> `-1`).
/// `explicit_priority` is the byte the C reads for `install_flag == 3` (`*((unsigned char*)pptr +
/// sizeof(void*))`). On a query (flag 2) of a present handler the priority is written back: returned
/// via [`ExitProcResult::QueriedPriority`].
///
/// Install flags: 0 = add with default priority 64; 1 = remove; 2 = query priority; 3 = add with
/// explicit priority (clamped to 64 if `> 127`). Returns the C `int` plus, for a query hit, the
/// found priority.
pub fn cob_sys_exit_proc(
    list: &mut ExitHandlerList,
    dispo: u8,
    proc: Option<usize>,
    explicit_priority: u8,
) -> ExitProcResult {
    let p = match proc {
        None => return ExitProcResult::Code(-1),
        Some(p) => p,
    };
    let install_flag = dispo;
    let mut priority: u8 = 0;
    match install_flag {
        0 => priority = 64,
        1 | 2 => {}
        3 => {
            priority = explicit_priority;
            if priority > 127 {
                priority = 64;
            }
        }
        _ => return ExitProcResult::Code(-1),
    }

    // Search for the handler.
    if let Some(idx) = list.exit_hdlrs.iter().position(|h| h.proc == p) {
        let found_priority = list.exit_hdlrs[idx].priority;
        match install_flag {
            2 => return ExitProcResult::QueriedPriority(found_priority),
            0 | 3 => {
                if priority == found_priority {
                    return ExitProcResult::Code(-1);
                }
            }
            _ => {}
        }
        list.exit_hdlrs.remove(idx);
        if install_flag == 1 {
            return ExitProcResult::Code(0);
        }
    }
    if install_flag == 1 || install_flag == 2 {
        return ExitProcResult::Code(-1);
    }
    // Insert at head.
    list.exit_hdlrs.insert(0, ExitHandler { proc: p, priority });
    ExitProcResult::Code(0)
}

/// Result of [`cob_sys_exit_proc`]: either a plain C return code, or (for the `install_flag == 2`
/// query of a present handler) the priority written back to the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitProcResult {
    Code(i32),
    QueriedPriority(u8),
}

/// The registry of CBL_ERROR_PROC handlers -- the model of `common.c`'s `hdlrs` list.
#[derive(Debug, Clone, Default)]
pub struct ErrorHandlerList {
    pub hdlrs: Vec<usize>,
}

/// Port of `common.c:cob_sys_error_proc` (CBL_ERROR_PROC).
///
/// Models `COB_CHK_PARMS(CBL_ERROR_PROC, 2)`. `proc` is `*p` (`None` models `!p || !*p` -> `-1`).
/// `dispo` is `*x`: non-zero removes the handler (if present), zero inserts it at the head -- but
/// the MF variant does *nothing* if it already exists. Always returns `0` on the non-error paths.
pub fn cob_sys_error_proc(list: &mut ErrorHandlerList, dispo: u8, proc: Option<usize>) -> i32 {
    let p = match proc {
        None => return -1,
        Some(p) => p,
    };
    let existing = list.hdlrs.iter().position(|&h| h == p);
    if dispo != 0 {
        // Remove handler
        if let Some(idx) = existing {
            list.hdlrs.remove(idx);
        }
    } else if existing.is_none() {
        // Insert at head
        list.hdlrs.insert(0, p);
    }
    // else: MF variant -- already existing: do nothing.
    0
}

/// The decision `common.c:cob_sys_runtime_error_proc` makes before aborting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeErrorProc {
    /// `cob_runtime_error("%s: %s", "Program abandoned at user request", msg)` then hard failure.
    WithMessage(Vec<u8>),
    /// `cob_runtime_error("%s", "Program abandoned at user request")` then hard failure.
    NoMessage,
}

/// Port of `common.c:cob_sys_runtime_error_proc` (CBL_RUNTIME_ERROR).
///
/// `err_msg` is the message field. The C calls `cob_runtime_error` (with the message when present
/// and non-empty, `msg[0]`) then `cob_hard_failure()` -- the error-print + abort is the runtime
/// boundary, so this returns the decision (which message form). The function does not return in C
/// (it aborts); modelled by returning the chosen branch.
pub fn cob_sys_runtime_error_proc(err_msg: Option<&[u8]>) -> RuntimeErrorProc {
    match err_msg {
        Some(msg) if !msg.is_empty() && msg[0] != 0 => RuntimeErrorProc::WithMessage(msg.to_vec()),
        _ => RuntimeErrorProc::NoMessage,
    }
}

// ---------------------------------------------------------------------------
// getopt wrapper -- cob_sys_getopt_long_long
// ---------------------------------------------------------------------------

/// `sizeof(longoption_def)` is the unit CBL_GC_GETOPT validates the long-options buffer against; the
/// COBOL record layout is: name[..], has_option byte, return_value_pointer (ptr), return_value (4).
/// The exact byte size is platform-dependent; the *check* (`lo_size % unit == 0`) is what is ported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GetoptMarshal {
    /// `lo_size % sizeof(longoption_def) != 0` -> `cob_runtime_error` + hard failure.
    BadLongoptionSize,
    /// `cob_procedure_params[2]` (longind) missing -> `cob_runtime_error` + hard failure.
    MissingLongind,
    /// Marshaling succeeded: `lo_amount` long options were translated; the getopt engine
    /// (`cob_getopt_long_long`, ported as cobgetopt) is the boundary.
    Ready { lo_amount: usize },
}

/// Port of the **param-marshaling guards** of `common.c:cob_sys_getopt_long_long` (CBL_GC_GETOPT).
///
/// Models `COB_CHK_PARMS(CBL_GC_GETOPT, 6)`. `lo_size`/`so_size` are the long/short option field
/// sizes; `longoption_def_size` is `sizeof(longoption_def)`; `longind_present` models
/// `cob_procedure_params[2]`. The actual `cob_getopt_long_long` call and the return-byte mapping are
/// handled by [`getopt_exit_status`] / the cobgetopt boundary. Returns the marshaling decision.
pub fn cob_sys_getopt_long_long(
    lo_size: usize,
    so_size: usize,
    longoption_def_size: usize,
    longind_present: bool,
) -> GetoptMarshal {
    let _ = so_size; // shortoptions buffer is so_size+1, allocation is the boundary
    if longoption_def_size == 0 || lo_size % longoption_def_size != 0 {
        return GetoptMarshal::BadLongoptionSize;
    }
    if !longind_present {
        return GetoptMarshal::MissingLongind;
    }
    let lo_amount = lo_size / longoption_def_size;
    GetoptMarshal::Ready { lo_amount }
}

/// Port of the **little-endian return-value mapping** of `common.c:cob_sys_getopt_long_long`.
///
/// After `cob_getopt_long_long` (the boundary) returns `return_value`, the C inspects its low byte
/// (`temp[0]`): `'?'`, `':'`, `'W'`, `-1`, or `0` pass `return_value` through as the exit status;
/// any other low byte means a matched long option -> exit status `3`. Then bytes `[3..1]` that are
/// `0` are SPACE-filled (the COBOL caller sees a blank-padded int). Returns the exit status
/// (the `cob_optarg`-too-long `2` is layered on by the caller as documented in the C).
pub fn getopt_exit_status(return_value: i32) -> i32 {
    let temp = return_value.to_le_bytes();
    let b0 = temp[0] as i8; // signed, to match C `char` compare against -1
    let c = temp[0];
    if c == 0x3F /* ? */ || c == 0x3A /* : */ || c == 0x57 /* W */ || b0 == -1 || c == 0 {
        return_value
    } else {
        3
    }
}

// ---------------------------------------------------------------------------
// ACCEPT FROM USER-NAME -- cob_accept_user_name
// ---------------------------------------------------------------------------

/// The value `common.c:cob_accept_user_name` moves into the receiving field: either the runtime's
/// cached `cob_user_name` (from `getlogin`, the boundary) or a single space when unset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcceptUserName {
    /// `cob_move_intermediate(f, cob_user_name, strlen(cob_user_name))`.
    Name(Vec<u8>),
    /// `cob_move_intermediate(f, " ", 1)`.
    Space,
}

/// Port of `common.c:cob_accept_user_name` (ACCEPT FROM USER-NAME).
///
/// `getlogin()`/`cob_user_name` resolution is the boundary; `cob_user_name` injects what it
/// resolved to. The C moves the name (length = `strlen`, i.e. up to the first NUL) into the field
/// `f` via `cob_move_intermediate`, or a single `" "` when unset. The `cob_move_intermediate`
/// copy/pad to `f` is itself a field-module primitive; this returns the source the move would use,
/// trimmed at the first NUL like `strlen`.
pub fn cob_accept_user_name(cob_user_name: Option<&[u8]>) -> AcceptUserName {
    match cob_user_name {
        Some(name) => {
            let end = name.iter().position(|&b| b == 0).unwrap_or(name.len());
            AcceptUserName::Name(name[..end].to_vec())
        }
        None => AcceptUserName::Space,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_trims_trailing_blanks_and_nuls() {
        let out = cob_sys_system(Some(b"ls -l   \0\0"), false);
        assert_eq!(out, SystemOutcome::Run { command: b"ls -l\0".to_vec() });
    }

    #[test]
    fn system_empty_or_blank_returns_one() {
        assert_eq!(cob_sys_system(None, false), SystemOutcome::Return(1));
        assert_eq!(cob_sys_system(Some(b""), false), SystemOutcome::Return(1));
        assert_eq!(cob_sys_system(Some(b"   \0"), false), SystemOutcome::Return(1));
        // single meaningful byte at index 0 still falls through to return 1 (i>0 guard)
        assert_eq!(cob_sys_system(Some(b"x"), false), SystemOutcome::Return(1));
    }

    #[test]
    fn system_over_length_returns_one() {
        let big = vec![b'a'; COB_MEDIUM_MAX + 2];
        assert_eq!(cob_sys_system(Some(&big), false), SystemOutcome::Return(1));
    }

    #[test]
    fn system_win32_requotes_quoted_pair() {
        // "someprog" "opt"  -> wrapped in an extra pair of quotes
        let cmd = b"\"someprog\" \"opt\"";
        let out = cob_sys_system(Some(cmd), true);
        match out {
            SystemOutcome::Run { command } => {
                assert_eq!(command[0], 0x22);
                assert_eq!(*command.last().unwrap(), 0); // NUL terminator
                assert_eq!(command[command.len() - 2], 0x22);
            }
            _ => panic!("expected Run"),
        }
        // already double-quoted pair: NOT re-wrapped -> command is cmd verbatim + NUL,
        // so its length is exactly cmd.len()+1 (no extra leading/trailing quote pair added).
        let cmd2 = b"\"\"x\"\"";
        let out2 = cob_sys_system(Some(cmd2), true);
        match out2 {
            SystemOutcome::Run { command } => assert_eq!(command.len(), cmd2.len() + 1),
            _ => panic!("expected Run"),
        }
    }

    #[test]
    fn hosted_dispatch_by_name_and_len() {
        assert_eq!(cob_sys_hosted(true, b"argc", false), Some(HostedVar::Argc));
        assert_eq!(cob_sys_hosted(true, b"stderr", false), Some(HostedVar::Stderr));
        assert_eq!(cob_sys_hosted(true, b"errno", false), Some(HostedVar::Errno));
        // timezone names gated
        assert_eq!(cob_sys_hosted(true, b"timezone", false), None);
        assert_eq!(cob_sys_hosted(true, b"timezone", true), Some(HostedVar::Timezone));
        // null destination
        assert_eq!(cob_sys_hosted(false, b"argc", false), None);
        // wrong length -> no match
        assert_eq!(cob_sys_hosted(true, b"argcx", false), None);
    }

    #[test]
    fn getpid_caches() {
        let mut c = ProcessIdCache::default();
        assert_eq!(cob_sys_getpid(&mut c, 4242), 4242);
        // second call ignores os value, returns cached
        assert_eq!(cob_sys_getpid(&mut c, 9999), 4242);
    }

    #[test]
    fn fork_outcomes() {
        let mut c = ProcessIdCache { cob_process_id: 50 };
        assert_eq!(cob_sys_fork(&mut c, Some(0)), (0, ForkOutcome::Child));
        assert_eq!(c.cob_process_id, 0); // child reset
        let mut c2 = ProcessIdCache { cob_process_id: 50 };
        assert_eq!(cob_sys_fork(&mut c2, Some(123)), (123, ForkOutcome::Parent(123)));
        assert_eq!(cob_sys_fork(&mut c2, Some(-1)), (-2, ForkOutcome::Error));
        assert_eq!(cob_sys_fork(&mut c2, None), (-1, ForkOutcome::Unsupported));
    }

    #[test]
    fn waitpid_logic() {
        // self pid -> -EINVAL
        assert_eq!(cob_sys_waitpid(Some(7), 7, 22, None, 0), -22);
        // no param -> -EINVAL
        assert_eq!(cob_sys_waitpid(None, 7, 22, None, 0), -22);
        // success: WEXITSTATUS
        assert_eq!(cob_sys_waitpid(Some(9), 7, 22, Some((9, 3)), 0), 3);
        // wait error -> -errno
        assert_eq!(cob_sys_waitpid(Some(9), 7, 22, Some((-1, 0)), 5), -5);
    }

    #[test]
    fn sleep_nanoseconds_clamp_and_sign() {
        assert_eq!(get_sleep_nanoseconds(-1), -1);
        assert_eq!(get_sleep_nanoseconds(1_000), 1_000);
        let max = MAX_SLEEP_TIME * 1_000_000_000;
        assert_eq!(get_sleep_nanoseconds(max), max);
        assert_eq!(get_sleep_nanoseconds(max + 1), max);
    }

    #[test]
    fn sleep_from_seconds_clamp_and_sign() {
        assert_eq!(get_sleep_nanoseconds_from_seconds(-1, 999), -1);
        assert_eq!(get_sleep_nanoseconds_from_seconds(5, 5_000_000_000), 5_000_000_000);
        let clamp = MAX_SLEEP_TIME * 1_000_000_000;
        assert_eq!(get_sleep_nanoseconds_from_seconds(MAX_SLEEP_TIME, 1), clamp);
    }

    #[test]
    fn internal_nanosleep_split() {
        assert_eq!(
            internal_nanosleep(2_500_000_000, "nano"),
            SleepRequest::Nanosleep { tv_sec: 2, tv_nsec: 500_000_000 }
        );
        assert_eq!(
            internal_nanosleep(5_000_000, "win"),
            SleepRequest::WinSleep { msecs: 5 }
        );
        // posix: 2.5e9 ns -> 25 deci-seconds -> 25%10==5 >4 -> 3 secs
        assert_eq!(
            internal_nanosleep(2_500_000_000, "posix"),
            SleepRequest::PosixSleep { secs: 3 }
        );
        // 2.4e9 ns -> 24 deci-seconds -> 24%10==4 not>4 -> 2 secs
        assert_eq!(
            internal_nanosleep(2_400_000_000, "posix"),
            SleepRequest::PosixSleep { secs: 2 }
        );
    }

    #[test]
    fn oc_nanosleep_and_sleep() {
        assert_eq!(cob_sys_oc_nanosleep(None), (-1, None));
        assert_eq!(cob_sys_oc_nanosleep(Some(0)), (0, None));
        assert_eq!(cob_sys_oc_nanosleep(Some(500)), (0, Some(500)));
        assert_eq!(cob_sys_oc_nanosleep(Some(-5)), (0, None)); // clamps to -1, not >0

        assert_eq!(cob_sys_sleep(false, 0, 0), (0, None));
        assert_eq!(cob_sys_sleep(true, -1, 0), (-1, None));
        assert_eq!(cob_sys_sleep(true, 2, 2_000_000_000), (0, Some(2_000_000_000)));
    }

    #[test]
    fn precise_time_selection() {
        assert_eq!(get_function_ptr_for_precise_time(true), PreciseTimeFunc::Precise);
        assert_eq!(get_function_ptr_for_precise_time(false), PreciseTimeFunc::Fallback);
    }

    #[test]
    fn x91_set_get_switches_0_7() {
        let mut st = X91State::default();
        let mut set = [1u8, 0, 1, 0, 1, 0, 1, 0];
        assert_eq!(cob_sys_x91(&mut st, 11, &mut set, 8), 0);
        assert_eq!(st.cob_switch[0], 1);
        assert_eq!(st.cob_switch[1], 0);
        assert_eq!(st.cob_switch[6], 1);
        let mut get = [9u8; 8];
        assert_eq!(cob_sys_x91(&mut st, 12, &mut get, 8), 0);
        assert_eq!(get, [1, 0, 1, 0, 1, 0, 1, 0]);
    }

    #[test]
    fn x91_debug_switch_extension() {
        let mut st = X91State::default();
        let mut set = [0u8, 0, 0, 0, 0, 0, 0, 0, 1];
        cob_sys_x91(&mut st, 11, &mut set, 9);
        assert!(st.cob_debugging_mode);
    }

    #[test]
    fn x91_set_az_specials() {
        let mut st = X91State::default();
        // index in parm for 'D' (i==14) is off=3; for 'N' (i==24) off=13
        let mut parm = [0u8; 25];
        parm[3] = 1; // D switch on
        parm[13] = 1; // N switch on
        cob_sys_x91(&mut st, 13, &mut parm, 25);
        assert!(st.cob_debugging_mode);
        assert!(st.cob_ls_nulls);
    }

    #[test]
    fn x91_num_params_and_default() {
        let mut st = X91State { module_num_params: 7, ..Default::default() };
        let mut parm = [0u8; 4];
        assert_eq!(cob_sys_x91(&mut st, 16, &mut parm, 4), 0);
        assert_eq!(parm[0], 7);
        // unknown function -> result 1
        let mut p2 = [0u8; 4];
        assert_eq!(cob_sys_x91(&mut st, 99, &mut p2, 4), 1);
    }

    #[test]
    fn exit_proc_add_query_remove() {
        let mut l = ExitHandlerList::default();
        // add with default priority
        assert_eq!(cob_sys_exit_proc(&mut l, 0, Some(100), 0), ExitProcResult::Code(0));
        assert_eq!(l.exit_hdlrs.len(), 1);
        assert_eq!(l.exit_hdlrs[0].priority, 64);
        // query priority
        assert_eq!(
            cob_sys_exit_proc(&mut l, 2, Some(100), 0),
            ExitProcResult::QueriedPriority(64)
        );
        // null proc -> -1
        assert_eq!(cob_sys_exit_proc(&mut l, 0, None, 0), ExitProcResult::Code(-1));
        // remove
        assert_eq!(cob_sys_exit_proc(&mut l, 1, Some(100), 0), ExitProcResult::Code(0));
        assert!(l.exit_hdlrs.is_empty());
        // remove absent -> -1
        assert_eq!(cob_sys_exit_proc(&mut l, 1, Some(100), 0), ExitProcResult::Code(-1));
    }

    #[test]
    fn exit_proc_explicit_priority_clamps() {
        let mut l = ExitHandlerList::default();
        assert_eq!(cob_sys_exit_proc(&mut l, 3, Some(5), 200), ExitProcResult::Code(0));
        assert_eq!(l.exit_hdlrs[0].priority, 64); // 200 > 127 -> 64
        // same priority on re-add -> -1
        assert_eq!(cob_sys_exit_proc(&mut l, 3, Some(5), 64), ExitProcResult::Code(-1));
    }

    #[test]
    fn error_proc_add_remove_idempotent() {
        let mut l = ErrorHandlerList::default();
        assert_eq!(cob_sys_error_proc(&mut l, 0, Some(11)), 0);
        assert_eq!(l.hdlrs, vec![11]);
        // re-add existing: MF variant does nothing
        assert_eq!(cob_sys_error_proc(&mut l, 0, Some(11)), 0);
        assert_eq!(l.hdlrs.len(), 1);
        // remove
        assert_eq!(cob_sys_error_proc(&mut l, 1, Some(11)), 0);
        assert!(l.hdlrs.is_empty());
        // null proc -> -1
        assert_eq!(cob_sys_error_proc(&mut l, 0, None), -1);
    }

    #[test]
    fn runtime_error_proc_message_choice() {
        assert_eq!(
            cob_sys_runtime_error_proc(Some(b"boom")),
            RuntimeErrorProc::WithMessage(b"boom".to_vec())
        );
        assert_eq!(cob_sys_runtime_error_proc(Some(b"")), RuntimeErrorProc::NoMessage);
        assert_eq!(cob_sys_runtime_error_proc(Some(&[0])), RuntimeErrorProc::NoMessage);
        assert_eq!(cob_sys_runtime_error_proc(None), RuntimeErrorProc::NoMessage);
    }

    #[test]
    fn getopt_marshal_guards() {
        assert_eq!(
            cob_sys_getopt_long_long(8, 4, 4, true),
            GetoptMarshal::Ready { lo_amount: 2 }
        );
        assert_eq!(
            cob_sys_getopt_long_long(10, 4, 3, true),
            GetoptMarshal::BadLongoptionSize
        );
        assert_eq!(
            cob_sys_getopt_long_long(8, 4, 4, false),
            GetoptMarshal::MissingLongind
        );
    }

    #[test]
    fn getopt_exit_status_mapping() {
        // '?' low byte passes through
        let q = i32::from_le_bytes([b'?', b' ', b' ', b' ']);
        assert_eq!(getopt_exit_status(q), q);
        // -1 passes through
        assert_eq!(getopt_exit_status(-1), -1);
        // 0 passes through
        assert_eq!(getopt_exit_status(0), 0);
        // a real option char 'x' -> 3
        let x = i32::from_le_bytes([b'x', 0, 0, 0]);
        assert_eq!(getopt_exit_status(x), 3);
    }

    #[test]
    fn accept_user_name() {
        assert_eq!(
            cob_accept_user_name(Some(b"alice")),
            AcceptUserName::Name(b"alice".to_vec())
        );
        // strlen semantics: stop at NUL
        assert_eq!(
            cob_accept_user_name(Some(b"bob\0extra")),
            AcceptUserName::Name(b"bob".to_vec())
        );
        assert_eq!(cob_accept_user_name(None), AcceptUserName::Space);
    }
}
