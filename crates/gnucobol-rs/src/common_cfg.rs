//! Port of the **pure** runtime-config parsing + a few `C$`/`CBL` routines of `libcob/common.c`.
//!
//! `common.c`'s runtime is global-state heavy (`cobsetptr`, `cobglobptr`, the `gc_conf[]` table, the
//! `COB_MODULE_PTR` chain). This module isolates the parts whose result is a **pure function of the inputs**:
//! the numeric / byte-size / boolean parsing of a config string value (the body of `set_config_val`, modelled
//! as a string -> parsed value/error), the `C$JUSTIFY` byte op, the `C$NARG` / `C$PARAMSIZE` / `C$CALLEDBY`
//! routines (with the global module/param state passed in explicitly), the `repeatWord` hexdump helper, and
//! the runtime CHECK routines whose diagnostic message is decided purely by their name/size inputs.
//!
//! No statics; global state is modelled as parameters. EXACT C function names. ASCII-only.
//! The OS/ABI/process-bound routines (`set_value`/`get_value` -- raw width-dependent memory stores;
//! `cob_sys_getpid`; the file/path config branches; `cob_hard_failure`) are NOT ported here -- see the report.

#![forbid(unsafe_code)]

// ======================================================================================================
// Runtime-config value parsing (common.c set_config_val, the ENV_BOOL / ENV_UINT|SINT|SIZE branches).
//
// `set_config_val` reads one config string `value` for the setting at `gc_conf[pos]` whose `data_type` is a
// bitset of the `ENV_*` flags below. This port models the *parse* -- string -> typed value or a typed error
// -- separately from the store (`set_value`, a width-dependent raw memory write, NOT ported) and from the
// stderr/`conf_runtime_error` side effect (the error is returned as a value here). `translate_boolean_to_int`
// is ported elsewhere; the boolean branch takes its already-computed result as input.
// ======================================================================================================

/// `coblocal.h` `ENV_*` data-type flags (`common.c` stores a bitset of these per setting). Only the flags
/// the parse depends on are named.
pub mod env {
    /// Negate the True/False value setting.
    pub const ENV_NOT: u32 = 1 << 1;
    /// An `unsigned int`.
    pub const ENV_UINT: u32 = 1 << 2;
    /// A `signed int`.
    pub const ENV_SINT: u32 = 1 << 3;
    /// A size: a number with optional `K`/`M`/`G` suffix.
    pub const ENV_SIZE: u32 = 1 << 4;
    /// An int boolean (Yes/True/1, No/False/0, ...).
    pub const ENV_BOOL: u32 = 1 << 5;
    /// Value must be in the `enum` list as a `match`.
    pub const ENV_ENUM: u32 = 1 << 9;
    /// Value must be in the `enum` list as a `match` or a `value`.
    pub const ENV_ENUMVAL: u32 = 1 << 10;
}

/// Why a config string value failed to parse (the `conf_runtime_error` the C would print, modelled as a
/// verdict rather than a stderr write).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigParseError {
    /// `should be one of the following values: ...` (enum / bool out of range).
    NotInEnumList,
    /// `should be unsigned` (a sign on a non-`ENV_SINT` setting).
    ShouldBeUnsigned,
    /// `should be numeric` (a non-digit where a digit was required, or trailing junk).
    ShouldBeNumeric,
    /// `minimum value: <min>` -- the parsed value is below `min_value`.
    BelowMinimum(i64),
    /// `maximum value: <max>` -- the parsed value is above `max_value`.
    AboveMaximum(i64),
}

/// The numeric value `set_config_val` would hand to `set_value` after parsing, for the boolean / integer /
/// size branches. (`set_value` itself -- the width-dependent raw store -- is the side effect, not ported.)
pub type ConfigValue = i64;

/// Port of the **`ENV_BOOL` branch** of `common.c:set_config_val`. `boolean_as_int` is the result of the
/// already-ported `translate_boolean_to_int` over the (possibly enum-translated) value: `1`/`0`/`-1` are
/// the accepted settings, anything else (`2`) is the out-of-range error. With `ENV_NOT` set the logic is
/// negated (`numval = !numval`, i.e. `0 -> 1`, non-zero -> `0`). Returns the value `set_value` receives.
pub fn set_config_val_bool(boolean_as_int: i32, data_type: u32) -> Result<ConfigValue, ConfigParseError> {
    if boolean_as_int != -1 && boolean_as_int != 1 && boolean_as_int != 0 {
        return Err(ConfigParseError::NotInEnumList);
    }
    let mut numval = boolean_as_int as i64;
    if (data_type & env::ENV_NOT) != 0 {
        // C `!numval`: 0 -> 1, any non-zero (incl. -1) -> 0.
        numval = if numval == 0 { 1 } else { 0 };
    }
    Ok(numval)
}

// `common.c` digit macros (coblocal.h COB_D2I, common.c IS_VALID_DIGIT_DATA / IS_INVALID_DIGIT_DATA): a byte
// is a valid digit iff `(unsigned char)(c - '0') <= 9`; its value is `c & 0x0F`.
#[inline]
fn is_valid_digit_data(c: u8) -> bool {
    c.wrapping_sub(b'0') <= 9
}
#[inline]
fn cob_d2i(c: u8) -> i64 {
    (c & 0x0F) as i64
}

/// Port of the **`ENV_UINT` / `ENV_SINT` / `ENV_SIZE` branch** of `common.c:set_config_val`: parse the string
/// `value` into a signed integer. Faithful to the C scan:
/// 1. skip leading spaces;
/// 2. optional leading `+`/`-` (a sign on a non-`ENV_SINT` setting is [`ConfigParseError::ShouldBeUnsigned`]);
/// 3. the first non-space, non-sign byte must be a digit ([`ConfigParseError::ShouldBeNumeric`]);
/// 4. accumulate `numval = numval*10 + (c & 0x0F)` over digits;
/// 5. an optional *trailing* sign (only with `ENV_SINT`) -- the C re-reads `sign` here;
/// 6. for `ENV_SIZE`, an optional `K`/`M`/`G` suffix (`K`=*1024; `M`=*1024^2 if <4001 else `4294967295`;
///    `G`=*1024^3 if <4 else `4294967295`);
/// 7. skip trailing spaces; any remaining byte is [`ConfigParseError::ShouldBeNumeric`];
/// 8. apply a `-` sign;
/// 9. range-check against `min_value`/`max_value` (each checked only when `> 0`, as the C does).
///
/// `min_value`/`max_value` are the `gc_conf[pos]` bounds (`0` = unchecked). Overflow of the i64 accumulator
/// wraps (the C uses `cob_s64_t` arithmetic identically).
pub fn set_config_val_numeric(
    value: &[u8],
    data_type: u32,
    min_value: i64,
    max_value: i64,
) -> Result<ConfigValue, ConfigParseError> {
    let is_signed = (data_type & env::ENV_SINT) != 0;
    let is_size = (data_type & env::ENV_SIZE) != 0;

    let mut i = 0usize;
    let mut numval: i64 = 0;
    let mut sign: u8 = 0;

    // 1. skip leading spaces
    while i < value.len() && value[i] == b' ' {
        i += 1;
    }
    // 2. optional leading sign
    if i < value.len() && (value[i] == b'-' || value[i] == b'+') {
        if !is_signed {
            return Err(ConfigParseError::ShouldBeUnsigned);
        }
        sign = value[i];
        i += 1;
    }
    // 3. first byte must be a digit (C: IS_INVALID_DIGIT_DATA(*ptr))
    let cur = value.get(i).copied().unwrap_or(0);
    if !is_valid_digit_data(cur) {
        return Err(ConfigParseError::ShouldBeNumeric);
    }
    // 4. accumulate digits
    while i < value.len() && is_valid_digit_data(value[i]) {
        numval = numval.wrapping_mul(10).wrapping_add(cob_d2i(value[i]));
        i += 1;
    }
    // 5. optional trailing sign (C re-reads it, but only when a sign was already seen)
    if sign != 0 && i < value.len() && (value[i] == b'-' || value[i] == b'+') {
        if !is_signed {
            return Err(ConfigParseError::ShouldBeUnsigned);
        }
        sign = value[i];
        i += 1;
    }
    // 6. ENV_SIZE suffix
    if is_size && i < value.len() && value[i] != 0 {
        match value[i].to_ascii_uppercase() {
            b'K' => {
                numval = numval.wrapping_mul(1024);
                i += 1;
            }
            b'M' => {
                if numval < 4001 {
                    numval = numval.wrapping_mul(1024 * 1024);
                } else {
                    numval = 4294967295;
                }
                i += 1;
            }
            b'G' => {
                if numval < 4 {
                    numval = numval.wrapping_mul(1024 * 1024 * 1024);
                } else {
                    numval = 4294967295;
                }
                i += 1;
            }
            _ => {}
        }
    }
    // 7. skip trailing spaces; any remaining byte is an error
    while i < value.len() && value[i] == b' ' {
        i += 1;
    }
    if i < value.len() && value[i] != 0 {
        return Err(ConfigParseError::ShouldBeNumeric);
    }
    // 8. apply a '-' sign
    if sign == b'-' {
        numval = numval.wrapping_neg();
    }
    // 9. range check (each bound checked only when > 0, as the C does)
    if min_value > 0 && numval < min_value {
        return Err(ConfigParseError::BelowMinimum(min_value));
    }
    if max_value > 0 && numval > max_value {
        return Err(ConfigParseError::AboveMaximum(max_value));
    }
    Ok(numval)
}

// ======================================================================================================
// C$JUSTIFY (common.c cob_sys_justify) -- a pure in-place byte op over a caller-sized buffer.
// ======================================================================================================

/// The `C$JUSTIFY` direction (`common.c` reads the 2nd parameter's first byte): `L` -> left, `C` -> centre,
/// anything else -> the default right justify.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyDir {
    /// `'L'` -- move content to the left, pad the right with spaces.
    Left,
    /// `'C'` -- centre the content, pad both sides with spaces.
    Centre,
    /// default (no direction param / any other byte) -- move content right, pad the left with spaces.
    Right,
}

impl JustifyDir {
    /// Map the C's first-byte read of the direction parameter to a [`JustifyDir`] (`L`/`C`, else `Right`).
    pub fn from_byte(b: u8) -> Self {
        match b {
            b'L' => JustifyDir::Left,
            b'C' => JustifyDir::Centre,
            _ => JustifyDir::Right,
        }
    }
}

/// Port of `common.c:cob_sys_justify` (`C$JUSTIFY`) -- in-place left/centre/right justify of `data` by moving
/// the non-space run to one side / the centre and space-filling the rest. Faithful to the C: a no-op (returns
/// `0`) when `data.len() < 2`, or when neither the first nor the last byte is a space, or when the field is
/// all spaces. `dir` is the parsed 2nd parameter ([`JustifyDir::Right`] is the default with no parameter).
/// Always returns `0` (the routine status).
pub fn cob_sys_justify(data: &mut [u8], dir: JustifyDir) -> i32 {
    let datalen = data.len();
    if datalen < 2 {
        return 0;
    }
    if data[0] != b' ' && data[datalen - 1] != b' ' {
        return 0;
    }
    // left = count of leading spaces
    let mut left = 0usize;
    while left < datalen && data[left] == b' ' {
        left += 1;
    }
    if left == datalen {
        // all spaces
        return 0;
    }
    // right = count of trailing spaces
    let mut right = 0usize;
    {
        let mut n = datalen - 1;
        loop {
            if data[n] != b' ' {
                break;
            }
            right += 1;
            if n == 0 {
                break;
            }
            n -= 1;
        }
    }
    let movelen = datalen - left - right;
    match dir {
        JustifyDir::Left => {
            // memmove(data, &data[left], movelen); memset(&data[movelen], ' ', datalen - movelen)
            data.copy_within(left..left + movelen, 0);
            for b in &mut data[movelen..] {
                *b = b' ';
            }
        }
        JustifyDir::Centre => {
            let centrelen = (left + right) / 2;
            // memmove(&data[centrelen], &data[left], movelen)
            data.copy_within(left..left + movelen, centrelen);
            // memset(data, ' ', centrelen)
            for b in &mut data[..centrelen] {
                *b = b' ';
            }
            // tail fill: centrelen (+1 when (left+right) is odd) starting at centrelen+movelen
            let tail_start = centrelen + movelen;
            let tail_len = if (left + right) % 2 != 0 { centrelen + 1 } else { centrelen };
            let tail_end = (tail_start + tail_len).min(datalen);
            for b in &mut data[tail_start.min(datalen)..tail_end] {
                *b = b' ';
            }
        }
        JustifyDir::Right => {
            // memmove(&data[left+right], &data[left], movelen); memset(data, ' ', datalen - movelen)
            data.copy_within(left..left + movelen, left + right);
            for b in &mut data[..datalen - movelen] {
                *b = b' ';
            }
        }
    }
    0
}

// ======================================================================================================
// C$NARG / C$PARAMSIZE / C$CALLEDBY (common.c cob_sys_return_args / cob_sys_parameter_size /
// cob_sys_calledby). The C reads the COB_MODULE_PTR chain (caller module + its parameter list); modelled
// here as explicit inputs.
// ======================================================================================================

/// Port of `common.c:cob_sys_return_args` (`C$NARG`) -- the number of CALL USING parameters of the current
/// module (`COB_MODULE_PTR->module_num_params`). The C stores it into the routine's output field via
/// `cob_set_int`; this port returns the value (the store is the caller's side effect). `module_num_params`
/// is the input.
pub fn cob_sys_return_args(module_num_params: i32) -> i32 {
    module_num_params
}

/// Port of `common.c:cob_sys_parameter_size` (`C$PARAMSIZE`) -- the byte size of the `n`-th (1-based) CALL
/// USING parameter of the **calling** module. The C reads `n` from the routine's input field, validates
/// `1 <= n <= module_num_params`, then returns `caller_param_sizes[n-1]` (or `0` when out of range / the
/// parameter was OMITTED, i.e. its entry is `None`). Faithful: returns `0` for any out-of-range or omitted
/// case. `caller_param_sizes` models `COB_MODULE_PTR->next->cob_procedure_params[]->size` (`None` = omitted).
pub fn cob_sys_parameter_size(n: i32, module_num_params: i32, caller_param_sizes: &[Option<i32>]) -> i32 {
    if n > 0 && n <= module_num_params {
        let idx = (n - 1) as usize;
        if let Some(Some(size)) = caller_param_sizes.get(idx) {
            return *size;
        }
    }
    0
}

/// The result of `C$CALLEDBY`: the routine writes the caller name into the output field and returns a status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalledByResult {
    /// The output field's new contents (caller name, space-padded / truncated to the field size).
    pub data: Vec<u8>,
    /// The routine status: `-1` (no output field), `0` (top-level / no caller), `1` (caller name written).
    pub status: i32,
}

/// Port of `common.c:cob_sys_calledby` (`C$CALLEDBY`) -- write the name of the **calling** program into the
/// `field_size`-byte output field, space-padded, truncated to the field. Returns:
/// `-1` when there is no output field (modelled as `field_size == None`);
/// `0` and an all-spaces field when there is no caller (`caller_name == None`, the top-level program);
/// `1` and the (truncated) caller name otherwise. Faithful to the C's `memset(' ')` then `memcpy(min(len))`.
pub fn cob_sys_calledby(field_size: Option<usize>, caller_name: Option<&[u8]>) -> CalledByResult {
    let size = match field_size {
        None => return CalledByResult { data: Vec::new(), status: -1 },
        Some(s) => s,
    };
    let mut data = vec![b' '; size];
    match caller_name {
        None => CalledByResult { data, status: 0 },
        Some(name) => {
            let msize = name.len().min(size);
            data[..msize].copy_from_slice(&name[..msize]);
            CalledByResult { data, status: 1 }
        }
    }
}

// ======================================================================================================
// repeatWord (common.c) -- a pure 4-byte-pattern match helper for the hexdump (cob_debug_dump).
// ======================================================================================================

/// Port of `common.c:repeatWord` -- true iff the 4-byte `pattern` is repeated 4 times (16 bytes) at the start
/// of `mem` (the `cob_debug_dump` run-compression check). Faithful: `mem` shorter than 16 bytes cannot match
/// (the C reads `mem[0..16]`; this port returns `false` rather than reading out of bounds).
#[allow(non_snake_case)]
pub fn repeatWord(pattern: &[u8; 4], mem: &[u8]) -> bool {
    if mem.len() < 16 {
        return false;
    }
    mem[0..4] == *pattern && mem[4..8] == *pattern && mem[8..12] == *pattern && mem[12..16] == *pattern
}

// ======================================================================================================
// Runtime CHECK routines (common.c cob_check_beyond_exit / cob_check_based / cob_check_fence /
// cob_check_linkage_size / cob_check_linkage). Each prints a `cob_runtime_error` line then aborts via
// `cob_hard_failure`. This port separates the PURE decision + the exact message text (the oracle-visible
// artifact) from the abort side effect: each returns the message it would raise, or `None` when in bounds.
// (Pattern parallels common.rs's cob_check_subscript; a local result type is defined here -- no import.)
// ======================================================================================================

/// A runtime CHECK diagnostic: the `cob_runtime_error` message (the text after
/// `libcob: <file>:<line>: error: `) and whether the runtime then aborts (`cob_hard_failure`). For these
/// routines the message is decided purely by the name/size inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckViolation {
    /// The runtime-error message text.
    pub message: Vec<u8>,
    /// Whether this violation aborts the run (`cob_hard_failure`).
    pub fatal: bool,
}

/// Port of `common.c:cob_check_beyond_exit` -- the diagnostic for code execution leaving a paragraph/section
/// past its exit (`code execution leaving <name>`). Always raised + fatal (the C unconditionally errors then
/// `cob_hard_failure`s); the decision is purely the `name`.
pub fn cob_check_beyond_exit(name: &str) -> CheckViolation {
    CheckViolation { message: format!("code execution leaving {name}").into_bytes(), fatal: true }
}

/// Port of `common.c:cob_check_based` -- a BASED/LINKAGE item with a NULL address. `is_null` models `!x`
/// (the C only errors when the data pointer is null). Returns the message `BASED/LINKAGE item <name> has NULL
/// address` (fatal) when null, else `None`. `name` already includes its quoting, as the C notes.
pub fn cob_check_based(is_null: bool, name: &str) -> Option<CheckViolation> {
    if is_null {
        Some(CheckViolation {
            message: format!("BASED/LINKAGE item {name} has NULL address").into_bytes(),
            fatal: true,
        })
    } else {
        None
    }
}

/// The two fence sentinels `cob_check_fence` verifies (the C `memcmp`s 8 bytes each, though only 7 distinct
/// bytes are listed in the literal -- reproduced exactly).
pub const FENCE_PRE: [u8; 8] = [0xFF, 0xFE, 0xFD, 0xFC, 0xFB, 0xFA, 0xFF, 0x00];
/// The post-fence sentinel.
pub const FENCE_POST: [u8; 8] = [0xFA, 0xFB, 0xFC, 0xFD, 0xFE, 0xFF, 0xFA, 0x00];

/// Port of `common.c:cob_check_fence` -- detect a memory violation by checking the pre/post fence bytes around
/// a CALL/UDF working-storage area. The C `memcmp`s the first 8 bytes of each fence against the literals
/// `"\xFF\xFE\xFD\xFC\xFB\xFA\xFF"` / `"\xFA\xFB\xFC\xFD\xFE\xFF\xFA"` (a C string literal is NUL-terminated,
/// so the 8th compared byte is `0x00` -- hence [`FENCE_PRE`]/[`FENCE_POST`]). On a mismatch it raises
/// `memory violation detected for '<name>' after <stmt>` (or, with no `name`, `memory violation detected
/// after <stmt>`) and aborts. `stmt_name` is the resolved `cob_statement_name[stmt]`. Returns the message,
/// or `None` when both fences are intact.
pub fn cob_check_fence(
    fence_pre: &[u8],
    fence_post: &[u8],
    stmt_name: &str,
    name: Option<&str>,
) -> Option<CheckViolation> {
    let pre_ok = fence_pre.len() >= 8 && fence_pre[..8] == FENCE_PRE;
    let post_ok = fence_post.len() >= 8 && fence_post[..8] == FENCE_POST;
    if pre_ok && post_ok {
        return None;
    }
    let message = match name {
        Some(name) => format!("memory violation detected for '{name}' after {stmt_name}"),
        None => format!("memory violation detected after {stmt_name}"),
    };
    Some(CheckViolation { message: message.into_bytes(), fatal: true })
}

/// The verdict of `cob_check_linkage_size`: the C returns `0` (ok), or `-1` after raising / skipping a
/// COB_EC_PROGRAM_ARG_MISMATCH. This port returns the matching status plus the message it would raise (if any).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkageSizeResult {
    /// The C return code: `0` (ok / runtime not initialized), `-1` (mismatch raised/handled).
    pub status: i32,
    /// The `cob_runtime_error` message raised, when one was (and the run then aborts unless ON EXCEPTION).
    pub violation: Option<CheckViolation>,
}

/// Port of `common.c:cob_check_linkage_size` -- validate that the `ordinal_pos`-th (1-based) CALL USING
/// parameter was passed by the caller and is at least `size` bytes. Modelled inputs (the C reads them from
/// `cobglobptr`/`COB_MODULE_PTR`):
/// - `runtime_initialized`: false models `!cobglobptr || !COB_MODULE_PTR` -> skip, return `0` (no message);
/// - `cob_call_params`: the count of parameters the caller actually passed;
/// - `caller_param_size`: the passed parameter's size (`None` models a missing/NULL-data parameter);
/// - `optional`: an OPTIONAL linkage item (a missing one is allowed -> `0`, no message);
/// - `stmt_exception`: the CALL has ON EXCEPTION -> the mismatch is handled silently (no message, no abort)
///   but the routine still returns `-1`.
///
/// Decision (faithful): if `cob_call_params < ordinal_pos` OR the passed parameter is missing, that is a
/// missing-arg case -> `LINKAGE item <name> not passed by caller` (fatal, unless `optional`/`stmt_exception`).
/// Otherwise if the passed size `< size` -> `LINKAGE item <name> (size <size>) too small in the caller
/// (size <caller>)` (fatal, unless `stmt_exception`). An equal-or-larger size is ok (`0`). `name` already
/// includes its quoting.
pub fn cob_check_linkage_size(
    runtime_initialized: bool,
    cob_call_params: u32,
    ordinal_pos: u32,
    optional: bool,
    size: u64,
    caller_param_size: Option<u64>,
    stmt_exception: bool,
    name: &str,
) -> LinkageSizeResult {
    if !runtime_initialized {
        return LinkageSizeResult { status: 0, violation: None };
    }
    // missing parameter: not enough params passed, or the entry has no data
    let missing = cob_call_params < ordinal_pos || caller_param_size.is_none();
    if missing {
        if optional {
            return LinkageSizeResult { status: 0, violation: None };
        }
        // raise_arg_mismatch: ON EXCEPTION -> handled silently, else error+abort
        let violation = if stmt_exception {
            None
        } else {
            Some(CheckViolation {
                message: format!("LINKAGE item {name} not passed by caller").into_bytes(),
                fatal: true,
            })
        };
        return LinkageSizeResult { status: -1, violation };
    }
    let caller = caller_param_size.unwrap_or(0);
    if caller < size {
        let violation = if stmt_exception {
            None
        } else {
            Some(CheckViolation {
                message: format!(
                    "LINKAGE item {name} (size {size}) too small in the caller (size {caller})"
                )
                .into_bytes(),
                fatal: true,
            })
        };
        return LinkageSizeResult { status: -1, violation };
    }
    LinkageSizeResult { status: 0, violation: None }
}

/// Port of `common.c:cob_check_linkage` -- a LINKAGE item with NULL data `x`. `is_null` models `!x`.
/// `check_type` selects the exception (`0` = ARG_MISMATCH on module entry, `1` = ARG_OMITTED on item use);
/// both print the same text `LINKAGE item <name> not passed by caller`. `stmt_exception` only applies to
/// `check_type == 0` (a CALL with ON EXCEPTION returns silently -> no message, no abort). Returns the message
/// it would raise, or `None` when `x` is non-null (or the exception was handled). `name` already includes its
/// quoting.
pub fn cob_check_linkage(
    is_null: bool,
    name: &str,
    check_type: i32,
    stmt_exception: bool,
) -> Option<CheckViolation> {
    if !is_null {
        return None;
    }
    if check_type == 0 && stmt_exception {
        // CALL has ON EXCEPTION: return to caller silently
        return None;
    }
    Some(CheckViolation {
        message: format!("LINKAGE item {name} not passed by caller").into_bytes(),
        fatal: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_bool_branch() {
        // accepted booleans pass through
        assert_eq!(set_config_val_bool(1, 0), Ok(1));
        assert_eq!(set_config_val_bool(0, 0), Ok(0));
        assert_eq!(set_config_val_bool(-1, 0), Ok(-1));
        // out of range (translate_boolean_to_int returned 2)
        assert_eq!(set_config_val_bool(2, 0), Err(ConfigParseError::NotInEnumList));
        // ENV_NOT negates: 0 -> 1, 1 -> 0, -1 -> 0
        assert_eq!(set_config_val_bool(0, env::ENV_NOT), Ok(1));
        assert_eq!(set_config_val_bool(1, env::ENV_NOT), Ok(0));
        assert_eq!(set_config_val_bool(-1, env::ENV_NOT), Ok(0));
    }

    #[test]
    fn config_numeric_unsigned() {
        // plain unsigned int
        assert_eq!(set_config_val_numeric(b"100", env::ENV_UINT, 0, 0), Ok(100));
        // leading/trailing spaces ok
        assert_eq!(set_config_val_numeric(b"  42  ", env::ENV_UINT, 0, 0), Ok(42));
        // a sign on an unsigned setting is rejected
        assert_eq!(
            set_config_val_numeric(b"-1", env::ENV_UINT, 0, 0),
            Err(ConfigParseError::ShouldBeUnsigned)
        );
        // non-digit first byte
        assert_eq!(set_config_val_numeric(b"abc", env::ENV_UINT, 0, 0), Err(ConfigParseError::ShouldBeNumeric));
        // trailing junk
        assert_eq!(set_config_val_numeric(b"12x", env::ENV_UINT, 0, 0), Err(ConfigParseError::ShouldBeNumeric));
    }

    #[test]
    fn config_numeric_signed_and_bounds() {
        // signed accepts a leading -
        assert_eq!(set_config_val_numeric(b"-5", env::ENV_SINT, 0, 0), Ok(-5));
        assert_eq!(set_config_val_numeric(b"+7", env::ENV_SINT, 0, 0), Ok(7));
        // min/max (only checked when > 0)
        assert_eq!(
            set_config_val_numeric(b"3", env::ENV_UINT, 5, 0),
            Err(ConfigParseError::BelowMinimum(5))
        );
        assert_eq!(
            set_config_val_numeric(b"99", env::ENV_UINT, 0, 10),
            Err(ConfigParseError::AboveMaximum(10))
        );
        assert_eq!(set_config_val_numeric(b"7", env::ENV_UINT, 5, 10), Ok(7));
    }

    #[test]
    fn config_numeric_size_suffix() {
        // K = *1024
        assert_eq!(set_config_val_numeric(b"256K", env::ENV_SIZE, 0, 0), Ok(256 * 1024));
        // M = *1024^2 when < 4001
        assert_eq!(set_config_val_numeric(b"128M", env::ENV_SIZE, 0, 0), Ok(128 * 1024 * 1024));
        // M >= 4001 clamps to 4294967295
        assert_eq!(set_config_val_numeric(b"5000M", env::ENV_SIZE, 0, 0), Ok(4294967295));
        // G = *1024^3 when < 4
        assert_eq!(set_config_val_numeric(b"2G", env::ENV_SIZE, 0, 0), Ok(2i64 * 1024 * 1024 * 1024));
        // G >= 4 clamps
        assert_eq!(set_config_val_numeric(b"4G", env::ENV_SIZE, 0, 0), Ok(4294967295));
        // lowercase suffix accepted (toupper)
        assert_eq!(set_config_val_numeric(b"1k", env::ENV_SIZE, 0, 0), Ok(1024));
    }

    #[test]
    fn justify_left_centre_right() {
        // right (default): "ab   " -> "   ab"
        let mut d = *b"ab   ";
        assert_eq!(cob_sys_justify(&mut d, JustifyDir::Right), 0);
        assert_eq!(&d, b"   ab");
        // left: "   ab" -> "ab   "
        let mut d = *b"   ab";
        cob_sys_justify(&mut d, JustifyDir::Left);
        assert_eq!(&d, b"ab   ");
        // centre: "ab    " (left=0,right=4) -> centrelen=2 -> "  ab  "
        let mut d = *b"ab    ";
        cob_sys_justify(&mut d, JustifyDir::Centre);
        assert_eq!(&d, b"  ab  ");
        // no-op when neither end is a space
        let mut d = *b"abcde";
        cob_sys_justify(&mut d, JustifyDir::Right);
        assert_eq!(&d, b"abcde");
        // no-op when all spaces
        let mut d = *b"     ";
        cob_sys_justify(&mut d, JustifyDir::Right);
        assert_eq!(&d, b"     ");
        // no-op when len < 2
        let mut d = *b" ";
        cob_sys_justify(&mut d, JustifyDir::Left);
        assert_eq!(&d, b" ");
    }

    #[test]
    fn justify_centre_odd() {
        // " ab " left=1 right=1 -> centrelen=1, odd -> tail 2 from index 1+2=3
        let mut d = *b" ab ";
        cob_sys_justify(&mut d, JustifyDir::Centre);
        // content moved to index 1, length 2: " ab " stays " ab " (centred already)
        assert_eq!(&d, b" ab ");
        // direction byte mapping
        assert_eq!(JustifyDir::from_byte(b'L'), JustifyDir::Left);
        assert_eq!(JustifyDir::from_byte(b'C'), JustifyDir::Centre);
        assert_eq!(JustifyDir::from_byte(b'R'), JustifyDir::Right);
        assert_eq!(JustifyDir::from_byte(b'x'), JustifyDir::Right);
    }

    #[test]
    fn narg_paramsize_calledby() {
        assert_eq!(cob_sys_return_args(3), 3);
        // paramsize: 1-based; param 2 has size 8
        let sizes = [Some(4), Some(8), Some(2)];
        assert_eq!(cob_sys_parameter_size(2, 3, &sizes), 8);
        // out of range -> 0
        assert_eq!(cob_sys_parameter_size(0, 3, &sizes), 0);
        assert_eq!(cob_sys_parameter_size(4, 3, &sizes), 0);
        // omitted param -> 0
        let sizes2 = [Some(4), None];
        assert_eq!(cob_sys_parameter_size(2, 2, &sizes2), 0);
        // calledby
        let r = cob_sys_calledby(Some(6), Some(b"MAIN"));
        assert_eq!(r.data, b"MAIN  ".to_vec());
        assert_eq!(r.status, 1);
        // truncation
        let r = cob_sys_calledby(Some(3), Some(b"MAINPROG"));
        assert_eq!(r.data, b"MAI".to_vec());
        assert_eq!(r.status, 1);
        // no caller (top-level)
        let r = cob_sys_calledby(Some(4), None);
        assert_eq!(r.data, b"    ".to_vec());
        assert_eq!(r.status, 0);
        // no output field
        let r = cob_sys_calledby(None, Some(b"X"));
        assert_eq!(r.status, -1);
    }

    #[test]
    fn repeat_word_match() {
        assert!(repeatWord(b"ABCD", b"ABCDABCDABCDABCD"));
        assert!(!repeatWord(b"ABCD", b"ABCDABCDABCDABCE"));
        assert!(!repeatWord(b"ABCD", b"ABCDABCD")); // too short
    }

    #[test]
    fn check_routines_messages() {
        // beyond exit
        assert_eq!(cob_check_beyond_exit("'P'").message, b"code execution leaving 'P'".to_vec());
        // based: null vs non-null
        assert_eq!(
            cob_check_based(true, "'X'").unwrap().message,
            b"BASED/LINKAGE item 'X' has NULL address".to_vec()
        );
        assert!(cob_check_based(false, "'X'").is_none());
        // fence: intact -> none
        assert!(cob_check_fence(&FENCE_PRE, &FENCE_POST, "MOVE", None).is_none());
        // fence: corrupt pre, with name
        let bad = [0u8; 8];
        let v = cob_check_fence(&bad, &FENCE_POST, "CALL", Some("Y")).unwrap();
        assert_eq!(v.message, b"memory violation detected for 'Y' after CALL".to_vec());
        // fence: corrupt, no name
        let v = cob_check_fence(&FENCE_PRE, &bad, "INIT CALL", None).unwrap();
        assert_eq!(v.message, b"memory violation detected after INIT CALL".to_vec());
    }

    #[test]
    fn linkage_checks() {
        // runtime not initialized -> ok, no message
        let r = cob_check_linkage_size(false, 0, 1, false, 10, None, false, "'A'");
        assert_eq!(r.status, 0);
        assert!(r.violation.is_none());
        // not enough params passed (call_params 0 < ordinal 1) -> not passed
        let r = cob_check_linkage_size(true, 0, 1, false, 10, None, false, "'A'");
        assert_eq!(r.status, -1);
        assert_eq!(r.violation.unwrap().message, b"LINKAGE item 'A' not passed by caller".to_vec());
        // optional missing -> ok
        let r = cob_check_linkage_size(true, 0, 1, true, 10, None, false, "'A'");
        assert_eq!(r.status, 0);
        assert!(r.violation.is_none());
        // too small
        let r = cob_check_linkage_size(true, 1, 1, false, 10, Some(4), false, "'B'");
        assert_eq!(r.status, -1);
        assert_eq!(
            r.violation.unwrap().message,
            b"LINKAGE item 'B' (size 10) too small in the caller (size 4)".to_vec()
        );
        // exactly enough -> ok
        let r = cob_check_linkage_size(true, 1, 1, false, 10, Some(10), false, "'B'");
        assert_eq!(r.status, 0);
        assert!(r.violation.is_none());
        // ON EXCEPTION: status -1 but no message
        let r = cob_check_linkage_size(true, 0, 1, false, 10, None, true, "'A'");
        assert_eq!(r.status, -1);
        assert!(r.violation.is_none());
        // cob_check_linkage
        assert_eq!(
            cob_check_linkage(true, "'Z'", 0, false).unwrap().message,
            b"LINKAGE item 'Z' not passed by caller".to_vec()
        );
        assert!(cob_check_linkage(false, "'Z'", 0, false).is_none());
        // ON EXCEPTION on entry-check (type 0) -> silent
        assert!(cob_check_linkage(true, "'Z'", 0, true).is_none());
        // type 1 ignores stmt_exception
        assert!(cob_check_linkage(true, "'Z'", 1, true).is_some());
    }
}
