//! Port of the **pure** COMMAND-LINE / ARGUMENT / CHAIN surface and the remaining
//! epoch/calendar helpers of `libcob/common.c`.
//!
//! libcob services `ACCEPT ... FROM COMMAND-LINE`, `DISPLAY UPON COMMAND-LINE`,
//! `ACCEPT ... FROM ARGUMENT-NUMBER / ARGUMENT-VALUE`, `DISPLAY UPON ARGUMENT-NUMBER`,
//! the CHAINING parameter setup and `CONTINUE AFTER <n> SECONDS` through a family of
//! functions that, in C, read the program-global argument vector (`cob_argc` /
//! `cob_argv`), a saved COMMAND-LINE string (`commlnptr` / `commlncnt`) and a cursor
//! (`current_arg`). Those are process-global statics in C; here they are modelled
//! **explicitly** as inputs / a small [`CmdlineState`] value -- nothing reads a static.
//!
//! Each ported function is a pure function of its inputs: the argument vector is an
//! injected `&[Vec<u8>]` (mirroring `cob_argv[0..cob_argc]`), the saved COMMAND-LINE is
//! an injected slice, and the receiving-field copy/truncation/space-padding is performed
//! on an owned `Vec<u8>` returned to the caller. The impure boundary -- the actual
//! `cob_move` into a live `cob_field`, the `internal_nanosleep`, the exception register
//! writes -- is the caller's concern; where the C code raises an exception or sleeps we
//! return a typed result instead.
//!
//! The cited C source is `libcob/common.c` (GnuCOBOL 3.2).

#![forbid(unsafe_code)]

/// Port of libcob's `struct cob_time` (common.h), redeclared locally so this module has
/// no dependency on the sibling `common_datetime` port. See that module for the field
/// range documentation; here it is only the receiver of [`cob_set_date_from_epoch`] and
/// [`cob_get_current_date_and_time`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CobTime {
    pub year: i32,
    pub month: i32,
    pub day_of_month: i32,
    pub day_of_week: i32,
    pub day_of_year: i32,
    pub hour: i32,
    pub minute: i32,
    pub second: i32,
    pub nanosecond: i32,
    pub offset_known: i32,
    pub utc_offset: i32,
    pub is_daylight_saving_time: i32,
}

impl CobTime {
    /// A zeroed `cob_time`, used as the starting point before epoch decomposition.
    fn zeroed() -> CobTime {
        CobTime {
            year: 0,
            month: 0,
            day_of_month: 0,
            day_of_week: 0,
            day_of_year: 0,
            hour: 0,
            minute: 0,
            second: 0,
            nanosecond: 0,
            offset_known: 0,
            utc_offset: 0,
            is_daylight_saving_time: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Explicitly-modelled command-line state (the C statics)
// ---------------------------------------------------------------------------

/// The command-line / argument state that libcob holds in process-global statics
/// (`current_arg`, `commlnptr`, `commlncnt`). Modelled here as an explicit value so the
/// ported accessors stay pure.
///
/// - `current_arg` -- the cursor advanced by `ACCEPT FROM ARGUMENT-VALUE`
///   (`cob_accept_arg_value`) and set by `DISPLAY UPON ARGUMENT-NUMBER`
///   (`cob_display_arg_number`). C initialises it to `0`.
/// - `command_line` -- the saved COMMAND-LINE bytes (`commlnptr` of length `commlncnt`),
///   set by `DISPLAY UPON COMMAND-LINE` (`cob_display_command_line`). `None` /
///   length-0 means "not set", which makes `ACCEPT FROM COMMAND-LINE` fall back to
///   reconstructing the string from the argument vector.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CmdlineState {
    pub current_arg: i32,
    pub command_line: Option<Vec<u8>>,
}

impl CmdlineState {
    /// A fresh state matching libcob's static initialisers (`current_arg = 0`,
    /// `commlncnt = 0`).
    pub fn new() -> CmdlineState {
        CmdlineState {
            current_arg: 0,
            command_line: None,
        }
    }
}

/// Port of `common.c:cob_command_line` -- in C this records the host argument vector
/// (`cob_argc = *pargc; cob_argv = *pargv;`) into the libcob statics and returns `NULL`
/// (the `#if 0` envp/name handling is dead code; the body is otherwise a guarded copy).
///
/// As a pure function the "recording" is just handing back the argument vector the caller
/// supplied (the bytes mirror `cob_argv[0..cob_argc]`, including `argv[0]` = the program
/// name). The `flags`, `envp` and `name` parameters are unused in the C body
/// (`COB_UNUSED`), and the C return value is always `NULL`.
pub fn cob_command_line(argv: &[Vec<u8>]) -> Vec<Vec<u8>> {
    argv.to_vec()
}

/// Port of `common.c:cob_display_command_line` -- `DISPLAY ... UPON COMMAND-LINE`. C
/// copies the receiving field's `f->size` bytes into the freshly (re)allocated
/// `commlnptr` and records `commlncnt = f->size`.
///
/// Here we return the new [`CmdlineState`]`.command_line` value: a copy of `data`
/// (length `f->size`). The single trailing NUL byte the C `cob_malloc (f->size + 1U)`
/// reserves is not part of `commlncnt` and is not represented.
pub fn cob_display_command_line(state: &mut CmdlineState, data: &[u8]) {
    state.command_line = Some(data.to_vec());
}

/// Port of `common.c:cob_accept_command_line` -- `ACCEPT ... FROM COMMAND-LINE`.
///
/// C logic, given the receiving field width `f_size`:
/// 1. If a COMMAND-LINE string was previously set (`commlncnt != 0`), move those
///    `commlncnt` bytes into `f`.
/// 2. Else if `cob_argc <= 1` (no real arguments), move a single space.
/// 3. Else reconstruct " arg1 arg2 ..." (a leading space then the arguments joined by a
///    single space) into a buffer, stopping once the accumulated size exceeds `f->size`,
///    and move that.
///
/// This returns exactly the bytes C passes to `cob_move_intermediate` (before the move's
/// own field-width padding/truncation, which is the caller's concern). `argv` mirrors
/// `cob_argv[0..cob_argc]`; index 0 is the program name and is skipped, matching the C
/// loops that start at `i = 1`.
pub fn cob_accept_command_line(state: &CmdlineState, argv: &[Vec<u8>], f_size: usize) -> Vec<u8> {
    // 1. previously-saved COMMAND-LINE string
    if let Some(cmd) = &state.command_line {
        if !cmd.is_empty() {
            return cmd.clone();
        }
    }

    // 2. no real arguments
    if argv.len() <= 1 {
        return vec![0x20]; // " "
    }

    // 3. reconstruct " arg1 arg2 ..."; the C buffer's leading byte is a space, but the
    //    write cursor starts at 0 so the first argument overwrites it. We therefore build
    //    the joined string directly (arg1 SP arg2 SP ... argN), matching the bytes the C
    //    loop actually emits, and stop once `size > f_size`.
    let mut buff: Vec<u8> = Vec::new();
    let last = argv.len() - 1;
    for (i, arg) in argv.iter().enumerate().skip(1) {
        buff.extend_from_slice(arg);
        if i != last {
            buff.push(0x20); // ' '
        }
        if buff.len() > f_size {
            break;
        }
    }
    buff
}

// ---------------------------------------------------------------------------
// Argument number / value
// ---------------------------------------------------------------------------

/// The outcome of `DISPLAY ... UPON ARGUMENT-NUMBER`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayArgNumber {
    /// The supplied number was in range; `current_arg` was set to it.
    Set(i32),
    /// `n < 0 || n >= cob_argc`: C raises `COB_EC_IMP_DISPLAY` and leaves the cursor
    /// unchanged.
    OutOfRange,
}

/// Port of `common.c:cob_display_arg_number` -- `DISPLAY ... UPON ARGUMENT-NUMBER`. C
/// moves the numeric field `f` into a 9-digit binary `n`, then, if `n < 0 || n >= cob_argc`,
/// raises `COB_EC_IMP_DISPLAY` and returns; otherwise sets `current_arg = n`.
///
/// Here the already-decoded integer `n` and the argument count are passed in; on success
/// the function writes `state.current_arg = n` and returns [`DisplayArgNumber::Set`],
/// otherwise leaves the state untouched and returns [`DisplayArgNumber::OutOfRange`]
/// (the caller maps that to the exception). `argc` mirrors `cob_argc` (the full count
/// including `argv[0]`).
pub fn cob_display_arg_number(state: &mut CmdlineState, n: i32, argc: i32) -> DisplayArgNumber {
    if n < 0 || n >= argc {
        return DisplayArgNumber::OutOfRange;
    }
    state.current_arg = n;
    DisplayArgNumber::Set(n)
}

/// Port of `common.c:cob_accept_arg_number` -- `ACCEPT ... FROM ARGUMENT-NUMBER`. C
/// computes `n = cob_argc - 1` (the number of real arguments, excluding the program name)
/// as a 9-digit unsigned binary and moves it into `f`.
///
/// Returns that count. `argc` mirrors `cob_argc`. (In C `n` is a `cob_u32_t`; with the
/// usual `argc >= 1` this is simply `argc - 1`.)
pub fn cob_accept_arg_number(argc: i32) -> u32 {
    (argc - 1) as u32
}

/// Port of `common.c:cob_accept_arg_value` -- `ACCEPT ... FROM ARGUMENT-VALUE`. C, if
/// `current_arg >= cob_argc`, raises `COB_EC_IMP_ACCEPT` and returns; otherwise moves
/// `cob_argv[current_arg]` (its `strlen` bytes) into `f` and advances `current_arg`.
///
/// Here, on success, `state.current_arg` is advanced and the selected argument's bytes
/// are returned (`Some`); when the cursor is past the end the state is left unchanged and
/// `None` is returned (the caller maps that to `COB_EC_IMP_ACCEPT`). `argv` mirrors
/// `cob_argv[0..cob_argc]`.
pub fn cob_accept_arg_value(state: &mut CmdlineState, argv: &[Vec<u8>]) -> Option<Vec<u8>> {
    let idx = state.current_arg;
    if idx < 0 || idx as usize >= argv.len() {
        return None;
    }
    let value = argv[idx as usize].clone();
    state.current_arg = idx + 1;
    Some(value)
}

// ---------------------------------------------------------------------------
// CHAINING
// ---------------------------------------------------------------------------

/// Port of `common.c:cob_chain_setup` -- copy a CHAINING command-line argument into a
/// fixed-width receiving area with space-padding / truncation.
///
/// C, given the receiver `data`, the 1-based parameter index `parm` and the receiver
/// `size`:
/// ```text
/// if (parm <= (size_t)cob_argc - 1) {
///     len = strlen (cob_argv[parm]);
///     memset (data, ' ', size);
///     if (len <= size) memcpy (data, cob_argv[parm], len);
///     else             memcpy (data, cob_argv[parm], size);
/// }
/// ```
/// i.e. only when the argument was actually given on the command line is the receiver
/// touched; it is first filled with spaces, then the argument bytes are copied left-
/// aligned (truncated to `size` if longer). When no such argument exists the receiver is
/// left as-is (the program's normal internal initialisation stands).
///
/// Here `argv` mirrors `cob_argv[0..cob_argc]`. The function returns `Some(buf)` with the
/// `size`-byte, space-padded receiver bytes when the argument exists, or `None` when it
/// does not (caller leaves its receiver untouched).
pub fn cob_chain_setup(argv: &[Vec<u8>], parm: usize, size: usize) -> Option<Vec<u8>> {
    // C guard: parm <= cob_argc - 1, i.e. parm is a valid index into argv.
    if parm == 0 || parm >= argv.len() {
        return None;
    }
    let arg = &argv[parm];
    let mut buf = vec![0x20u8; size]; // memset ' '
    let copy = arg.len().min(size);
    buf[..copy].copy_from_slice(&arg[..copy]);
    Some(buf)
}

// ---------------------------------------------------------------------------
// CONTINUE AFTER
// ---------------------------------------------------------------------------

/// `3600 * 24 * 7` -- the libcob `MAX_SLEEP_TIME` (one week) cap.
pub const MAX_SLEEP_TIME: i64 = 3600 * 24 * 7;

/// The outcome of parsing a `CONTINUE AFTER <n> SECONDS` value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContinueAfter {
    /// The number of nanoseconds to sleep (already clamped to one week).
    Nanoseconds(i64),
    /// The seconds value was negative: C raises `COB_EC_CONTINUE_LESS_THAN_ZERO` and
    /// does not sleep.
    LessThanZero,
}

/// Port of `common.c:get_sleep_nanoseconds_from_seconds` -- convert a decimal-seconds
/// value to a nanosecond sleep duration.
///
/// C: `seconds = cob_get_llint (decimal_seconds)` (the integer part of the seconds
/// value). If `seconds < 0` return `-1`; if `seconds >= MAX_SLEEP_TIME` return
/// `MAX_SLEEP_TIME * 1e9`; otherwise re-`cob_move` the *full* decimal value into a
/// `const_bin_nano_attr` field (scale 9) -- i.e. `round(seconds_decimal * 1e9)` -- and
/// return that.
///
/// Here both pieces are passed in already decoded: `int_seconds` is the integer part
/// (`cob_get_llint`), and `total_nanoseconds` is the full decimal value scaled to
/// nanoseconds (what the scale-9 `cob_move` yields). The clamp branches use `int_seconds`
/// exactly as C does, so a value `>= MAX_SLEEP_TIME` is capped before the scaled value is
/// consulted.
pub fn get_sleep_nanoseconds_from_seconds(int_seconds: i64, total_nanoseconds: i64) -> i64 {
    if int_seconds < 0 {
        return -1;
    }
    if int_seconds >= MAX_SLEEP_TIME {
        return MAX_SLEEP_TIME * 1_000_000_000;
    }
    total_nanoseconds
}

/// Port of `common.c:cob_continue_after` -- `CONTINUE AFTER <n> SECONDS`. C computes
/// `nanoseconds = get_sleep_nanoseconds_from_seconds (decimal_seconds)`; if that is `< 0`
/// it raises `COB_EC_CONTINUE_LESS_THAN_ZERO` and returns, otherwise it
/// `internal_nanosleep (nanoseconds)`.
///
/// Here the decoded seconds (`int_seconds` = integer part, `total_nanoseconds` = the full
/// decimal value scaled to ns; see [`get_sleep_nanoseconds_from_seconds`]) are passed in.
/// A negative result yields [`ContinueAfter::LessThanZero`] (caller raises the exception);
/// otherwise [`ContinueAfter::Nanoseconds`] carries the duration the caller would sleep.
pub fn cob_continue_after(int_seconds: i64, total_nanoseconds: i64) -> ContinueAfter {
    let nanoseconds = get_sleep_nanoseconds_from_seconds(int_seconds, total_nanoseconds);
    if nanoseconds < 0 {
        return ContinueAfter::LessThanZero;
    }
    ContinueAfter::Nanoseconds(nanoseconds)
}

// ---------------------------------------------------------------------------
// Date/time leftovers
// ---------------------------------------------------------------------------

/// Port of `common.c:set_cob_time_ns_from_filetime` (the Windows-only `_WIN32` helper) --
/// derive `cob_time.nanosecond` from a Win32 `FILETIME`.
///
/// C:
/// ```text
/// filetime_int = ((ULONGLONG)dwHighDateTime << 32) + dwLowDateTime;
/// /* FILETIMEs are accurate to 100 nanosecond intervals */
/// cb_time->nanosecond = (filetime_int % 10000000) * 100;
/// ```
/// A FILETIME counts 100-ns ticks; the sub-second part is `filetime_int % 10_000_000`
/// ticks (10e6 ticks = 1 s), each tick being 100 ns. This is pure arithmetic; the
/// `FILETIME` is modelled as its two 32-bit halves. The result fits a normal nanosecond
/// field (`< 1_000_000_000`).
pub fn set_cob_time_ns_from_filetime(dw_high_date_time: u32, dw_low_date_time: u32) -> i32 {
    let filetime_int: u64 = ((dw_high_date_time as u64) << 32) + dw_low_date_time as u64;
    ((filetime_int % 10_000_000) * 100) as i32
}

/// Whether `c` is an ASCII decimal digit -- port of the `IS_VALID_DIGIT_DATA` macro
/// `((unsigned char)(c - '0') <= 9)` (common.c:428).
fn is_valid_digit_data(c: u8) -> bool {
    c.wrapping_sub(0x30) <= 9
}

/// Convert a days-since-1970-01-01 count to a proleptic-Gregorian `(year, month, day)`.
///
/// This is Howard Hinnant's well-known branch-free `civil_from_days` algorithm. It is the
/// pure-arithmetic equivalent of what the C `cob_set_date_from_epoch` obtains by stuffing
/// the whole-day count into `tm_mday` of an epoch `struct tm` and calling `mktime` to
/// normalise: a UTC civil date from a day index. Valid across the full supported range.
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468; // shift epoch to 0000-03-01
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    let year = (y + if m <= 2 { 1 } else { 0 }) as i32;
    (year, m, d)
}

/// Day-of-week (0 = Sunday .. 6 = Saturday) for a days-since-1970 count. 1970-01-01 was a
/// Thursday (= 4). Mirrors what `mktime` fills into `tm_wday`.
fn weekday_from_days(z: i64) -> i32 {
    (((z % 7) + 4 + 7) % 7) as i32
}

/// Whether `year` is a leap year in the proleptic Gregorian calendar.
fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Day-of-year (1-based) for a `(year, month, day)`. Mirrors `tm_yday + 1`.
fn day_of_year(year: i32, month: u32, day: u32) -> i32 {
    const CUM: [i32; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let mut doy = CUM[(month - 1) as usize] + day as i32;
    if month > 2 && is_leap(year) {
        doy += 1;
    }
    doy
}

/// Port of `common.c:cob_set_date_from_epoch` -- parse a decimal seconds-since-epoch
/// string and decompose it into a `cob_time`.
///
/// C: scan leading ASCII digits accumulating `seconds` (base-10); if any non-digit
/// remains (the terminator is not NUL) or `seconds > 253402300799` (the latest
/// representable `__DATE__`/`__TIME__`, i.e. 9999-12-31 23:59:59 UTC) it returns `1`
/// (error). Otherwise it loads `tm` from epoch `t = 0`, assigns
/// `tm_sec = seconds % 60; seconds /= 60; tm_min = seconds % 60; seconds /= 60;
///  tm_hour = seconds % 24; seconds /= 24; tm_mday = seconds; tm_isdst = -1;`
/// and `mktime`-normalises to fill `year/month/day/hour/min/sec`, `tm_wday`, `tm_yday`,
/// `tm_isdst`; `nanosecond` is set to `-1`. On success returns `0`.
///
/// This port reproduces the same decomposition with pure calendar arithmetic
/// ([`civil_from_days`] etc.), filling a [`CobTime`]. `day_of_week` is set to
/// `tm_wday + 1` (matching the C assignment, which -- note -- is *not* the COBOL
/// Monday-based 1..7 numbering used elsewhere; it is the raw `tm_wday + 1`, i.e.
/// 1 = Sunday .. 7 = Saturday). `is_daylight_saving_time` is left `0`: DST is a
/// local-timezone property the pure layer cannot determine (`mktime` would consult
/// `TZ`); the caller may overwrite it. `nanosecond` is set to `-1` as in C.
///
/// On the error conditions (trailing non-digit, or out-of-range) it returns `Err(())`,
/// mirroring the C return value `1`; on success `Ok(CobTime)`, mirroring `0`.
pub fn cob_set_date_from_epoch(p: &[u8]) -> Result<CobTime, ()> {
    let mut seconds: i64 = 0;
    let mut i = 0usize;
    while i < p.len() && is_valid_digit_data(p[i]) {
        // COB_D2I (*p) == *p - '0'
        seconds = seconds * 10 + (p[i] - 0x30) as i64;
        i += 1;
    }
    // C tests `*p != 0`: a non-NUL terminator (here: any remaining non-digit byte) is an
    // error. A trailing NUL byte in the slice also terminates the number; treat the rest
    // being all NUL / empty as "clean end of string".
    let clean_end = p[i..].iter().all(|&b| b == 0);
    if !clean_end || seconds > 253_402_300_799 {
        return Err(());
    }

    let mut rem = seconds;
    let sec = (rem % 60) as i32;
    rem /= 60;
    let min = (rem % 60) as i32;
    rem /= 60;
    let hour = (rem % 24) as i32;
    rem /= 24;
    let days = rem; // whole days since 1970-01-01

    let (year, month, day) = civil_from_days(days);
    let wday = weekday_from_days(days); // 0 = Sunday

    let mut t = CobTime::zeroed();
    t.year = year;
    t.month = month as i32;
    t.day_of_month = day as i32;
    t.hour = hour;
    t.minute = min;
    t.second = sec;
    t.nanosecond = -1;
    t.day_of_week = wday + 1; // C: tmptr->tm_wday + 1
    t.day_of_year = day_of_year(year, month, day);
    t.is_daylight_saving_time = 0;
    Ok(t)
}

/// Port of `common.c:cob_get_current_date_and_time` -- in C this is simply
/// `return cob_get_current_datetime (DTR_TIME_NO_NANO);`, i.e. it reads the OS clock
/// (the impure boundary) at "time, no nanoseconds" resolution and then applies any
/// configured constant-time override.
///
/// The OS read and the constant-time override are not pure, so this port takes the
/// already-broken-down clock components and assembles the [`CobTime`] -- it is the
/// pure "given these raw components, produce the cob_time" core. Because the resolution
/// is `DTR_TIME_NO_NANO`, the nanosecond field is forced to `0` (the C path that reads
/// sub-second precision is gated off at that resolution), and the leap-second clamp
/// (`second >= 60 -> 59`) that `cob_get_current_datetime` applies last is reproduced.
///
/// Parameters mirror the broken-down local-time components the OS layer would supply
/// (`day_of_week`/`day_of_year` already in their `cob_time` form), plus the timezone
/// `offset_known` / `utc_offset` pair.
#[allow(clippy::too_many_arguments)]
pub fn cob_get_current_date_and_time(
    year: i32,
    month: i32,
    day_of_month: i32,
    day_of_week: i32,
    day_of_year: i32,
    hour: i32,
    minute: i32,
    second: i32,
    offset_known: i32,
    utc_offset: i32,
    is_daylight_saving_time: i32,
) -> CobTime {
    // Leap seconds ? (cob_get_current_datetime clamps second >= 60 down to 59)
    let second = if second >= 60 { 59 } else { second };
    CobTime {
        year,
        month,
        day_of_month,
        day_of_week,
        day_of_year,
        hour,
        minute,
        second,
        nanosecond: 0, // DTR_TIME_NO_NANO
        offset_known,
        utc_offset,
        is_daylight_saving_time,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(parts: &[&[u8]]) -> Vec<Vec<u8>> {
        parts.iter().map(|p| p.to_vec()).collect()
    }

    // argv[0] = program name; argv[1..] = the real arguments.
    fn sample_argv() -> Vec<Vec<u8>> {
        argv(&[b"prog", b"alpha", b"beta", b"gamma"])
    }

    #[test]
    fn command_line_records_argv() {
        let a = sample_argv();
        assert_eq!(cob_command_line(&a), a);
    }

    #[test]
    fn display_then_accept_command_line() {
        let mut st = CmdlineState::new();
        cob_display_command_line(&mut st, b"USER SET LINE");
        assert_eq!(st.command_line.as_deref(), Some(&b"USER SET LINE"[..]));
        // a set COMMAND-LINE wins over the argv reconstruction
        assert_eq!(
            cob_accept_command_line(&st, &sample_argv(), 80),
            b"USER SET LINE"
        );
    }

    #[test]
    fn accept_command_line_from_argv() {
        let st = CmdlineState::new();
        // " arg1 arg2 ..." -> joined by single spaces, no trailing space
        assert_eq!(
            cob_accept_command_line(&st, &sample_argv(), 80),
            b"alpha beta gamma"
        );
    }

    #[test]
    fn accept_command_line_no_args() {
        let st = CmdlineState::new();
        let a = argv(&[b"prog"]); // only the program name -> argc == 1
        assert_eq!(cob_accept_command_line(&st, &a, 80), b" ");
    }

    #[test]
    fn accept_command_line_truncation_stop() {
        let st = CmdlineState::new();
        // f_size small: loop stops once accumulated size exceeds it.
        // "alpha"(5) -> 5 not > 4, then ' '(6) > 4 -> break after appending the space.
        let out = cob_accept_command_line(&st, &sample_argv(), 4);
        assert_eq!(out, b"alpha ");
    }

    #[test]
    fn display_arg_number_in_range_and_out() {
        let mut st = CmdlineState::new();
        assert_eq!(cob_display_arg_number(&mut st, 2, 4), DisplayArgNumber::Set(2));
        assert_eq!(st.current_arg, 2);
        // out of range leaves cursor unchanged
        assert_eq!(
            cob_display_arg_number(&mut st, 4, 4),
            DisplayArgNumber::OutOfRange
        );
        assert_eq!(st.current_arg, 2);
        assert_eq!(
            cob_display_arg_number(&mut st, -1, 4),
            DisplayArgNumber::OutOfRange
        );
        assert_eq!(st.current_arg, 2);
    }

    #[test]
    fn accept_arg_number_counts_real_args() {
        // argc == 4 -> 3 real arguments
        assert_eq!(cob_accept_arg_number(4), 3);
        assert_eq!(cob_accept_arg_number(1), 0);
    }

    #[test]
    fn accept_arg_value_walks_and_stops() {
        let a = sample_argv();
        let mut st = CmdlineState::new();
        // current_arg starts at 0 -> argv[0] (program name), as the C code would do if
        // it had not been positioned; then advances.
        assert_eq!(cob_accept_arg_value(&mut st, &a), Some(b"prog".to_vec()));
        assert_eq!(st.current_arg, 1);
        assert_eq!(cob_accept_arg_value(&mut st, &a), Some(b"alpha".to_vec()));
        // jump the cursor to the last and exhaust it
        st.current_arg = 3;
        assert_eq!(cob_accept_arg_value(&mut st, &a), Some(b"gamma".to_vec()));
        assert_eq!(st.current_arg, 4);
        // past the end -> None, cursor unchanged
        assert_eq!(cob_accept_arg_value(&mut st, &a), None);
        assert_eq!(st.current_arg, 4);
    }

    #[test]
    fn chain_setup_pad_and_truncate() {
        let a = sample_argv();
        // "alpha" into width 8 -> left-aligned, space-padded
        assert_eq!(cob_chain_setup(&a, 1, 8), Some(b"alpha   ".to_vec()));
        // "gamma" into width 3 -> truncated
        assert_eq!(cob_chain_setup(&a, 3, 3), Some(b"gam".to_vec()));
        // exact fit
        assert_eq!(cob_chain_setup(&a, 2, 4), Some(b"beta".to_vec()));
        // missing argument -> None (receiver left untouched)
        assert_eq!(cob_chain_setup(&a, 9, 8), None);
        // parm 0 is the program name; C guard is parm <= argc-1 with parm 1-based use,
        // but the index 0 case is not a chained parameter -> None.
        assert_eq!(cob_chain_setup(&a, 0, 8), None);
    }

    #[test]
    fn continue_after_clamps_and_rejects() {
        // negative -> LessThanZero
        assert_eq!(cob_continue_after(-1, -1), ContinueAfter::LessThanZero);
        // normal: 2.5 s -> 2_500_000_000 ns
        assert_eq!(
            cob_continue_after(2, 2_500_000_000),
            ContinueAfter::Nanoseconds(2_500_000_000)
        );
        // >= one week -> capped at MAX_SLEEP_TIME * 1e9
        assert_eq!(
            cob_continue_after(MAX_SLEEP_TIME, 0),
            ContinueAfter::Nanoseconds(MAX_SLEEP_TIME * 1_000_000_000)
        );
    }

    #[test]
    fn filetime_to_ns() {
        // 1 FILETIME tick = 100 ns. filetime_int % 10_000_000 ticks is the sub-second.
        // pick filetime_int = 12_345_678 ticks -> %1e7 = 2_345_678 -> *100 = 234_567_800 ns
        let high = (12_345_678u64 >> 32) as u32;
        let low = (12_345_678u64 & 0xFFFF_FFFF) as u32;
        assert_eq!(set_cob_time_ns_from_filetime(high, low), 234_567_800);
        // zero
        assert_eq!(set_cob_time_ns_from_filetime(0, 0), 0);
    }

    #[test]
    fn set_date_from_epoch_known_value() {
        // 0 -> 1970-01-01 00:00:00 UTC, a Thursday.
        let t = cob_set_date_from_epoch(b"0").unwrap();
        assert_eq!((t.year, t.month, t.day_of_month), (1970, 1, 1));
        assert_eq!((t.hour, t.minute, t.second), (0, 0, 0));
        assert_eq!(t.day_of_year, 1);
        assert_eq!(t.nanosecond, -1);
        // 1970-01-01 was Thursday: tm_wday = 4 -> day_of_week = 5
        assert_eq!(t.day_of_week, 5);

        // 1_000_000_000 -> 2001-09-09 01:46:40 UTC (a famous epoch round number), Sunday.
        let t = cob_set_date_from_epoch(b"1000000000").unwrap();
        assert_eq!((t.year, t.month, t.day_of_month), (2001, 9, 9));
        assert_eq!((t.hour, t.minute, t.second), (1, 46, 40));
        // 2001-09-09 was a Sunday: tm_wday = 0 -> day_of_week = 1
        assert_eq!(t.day_of_week, 1);

        // a leap-year date past Feb to exercise day_of_year leap adjustment:
        // 1_078_099_200 -> 2004-03-01 00:00:00 UTC. day_of_year: Jan(31)+Feb(29)+1 = 61.
        let t = cob_set_date_from_epoch(b"1078099200").unwrap();
        assert_eq!((t.year, t.month, t.day_of_month), (2004, 3, 1));
        assert_eq!(t.day_of_year, 61);
    }

    #[test]
    fn set_date_from_epoch_max_and_errors() {
        // exactly the max -> 9999-12-31 23:59:59
        let t = cob_set_date_from_epoch(b"253402300799").unwrap();
        assert_eq!((t.year, t.month, t.day_of_month), (9999, 12, 31));
        assert_eq!((t.hour, t.minute, t.second), (23, 59, 59));
        // one past the max -> error
        assert!(cob_set_date_from_epoch(b"253402300800").is_err());
        // trailing non-digit -> error
        assert!(cob_set_date_from_epoch(b"123x").is_err());
        // trailing NUL is a clean terminator (C tests *p == 0)
        assert!(cob_set_date_from_epoch(b"123\0").is_ok());
    }

    #[test]
    fn get_current_date_and_time_assembles_and_clamps() {
        let t = cob_get_current_date_and_time(2026, 6, 15, 1, 166, 14, 30, 60, 1, 0, 0);
        assert_eq!(t.year, 2026);
        assert_eq!(t.month, 6);
        assert_eq!(t.day_of_month, 15);
        assert_eq!(t.second, 59); // 60 clamped to 59
        assert_eq!(t.nanosecond, 0); // DTR_TIME_NO_NANO
        assert_eq!(t.offset_known, 1);
    }
}
