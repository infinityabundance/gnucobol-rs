//! Port of the **environment-variable** helpers of `libcob/common.c`.
//!
//! These functions implement COBOL's `ACCEPT ... FROM ENVIRONMENT`, `DISPLAY ... UPON ENVIRONMENT`,
//! the `${VAR}` expansion used when parsing runtime-config values, and the temporary-directory
//! discovery used for SORT/MERGE work files.
//!
//! ## The environment-access boundary
//!
//! The C originals call `getenv`/`setenv`/`putenv`/`unsetenv`, which read and mutate global,
//! process-wide OS state -- the genuinely impure part. This port keeps the *scanning / expansion /
//! precedence* logic faithful while routing every actual lookup through an injected closure
//! `lookup: impl Fn(&str) -> Option<Vec<u8>>` (returning the bytes of the variable's value, or
//! `None` when unset). Mutating operations are modelled as functions over an explicit env map
//! (`&mut BTreeMap<Vec<u8>, Vec<u8>>`). This makes the logic pure, panic-free and testable; the
//! caller decides whether the closure / map is backed by `std::env` or a sandbox.
//!
//! Char literals are written as their byte values (`0x24` `$`, `0x7B` `{`, `0x7D` `}`, `0x3A` `:`,
//! `0x2D` `-`, `0x20` space) to keep the source ASCII-only.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

/// Port of `common.c:cob_expand_env_string` -- expand a string containing environment-variable
/// references, returning the expanded bytes.
///
/// Faithful to the C scanner:
/// * `${name}` is replaced by the value of `name` (via `lookup`).
/// * `${name:default}` / `${name:-default}` use `default` when `name` is unset (the leading `-`
///   after `:` is consumed if present).
/// * When `${name}` is unset and there is no `:default`, the two compile-time fallbacks
///   `COB_CONFIG_DIR` and `COB_COPY_DIR` are substituted from `config_dir` / `copy_dir` when the
///   name matches exactly (pass `None` to leave them unset, as on a build without those macros).
/// * `$$` is replaced by the process id (`pid`, the boundary for `cob_sys_getpid`).
/// * Any whitespace byte is normalised to a single space (`0x20`); all other bytes copy verbatim.
///
/// The lookup is the OS boundary: `lookup(name)` returns the variable's value bytes or `None`.
/// The C `ename` buffer is 128 bytes (`char ename[128]`); names are truncated to fit exactly as
/// the C code would (it writes up to index 127, then a terminator), so at most 127 name bytes are
/// collected here to match.
pub fn cob_expand_env_string(
    strval: &[u8],
    lookup: impl Fn(&str) -> Option<Vec<u8>>,
    pid: i32,
    config_dir: Option<&[u8]>,
    copy_dir: Option<&[u8]>,
) -> Vec<u8> {
    const DOLLAR: u8 = 0x24; // '$'
    const LBRACE: u8 = 0x7B; // '{'
    const RBRACE: u8 = 0x7D; // '}'
    const COLON: u8 = 0x3A; // ':'
    const DASH: u8 = 0x2D; // '-'
    const SPACE: u8 = 0x20; // ' '

    let mut env: Vec<u8> = Vec::new();
    // helper: byte at index k of strval, or 0 (the C strings are NUL-terminated, so reads past the
    // end yield the terminator and stop the loops).
    let at = |idx: usize| -> u8 {
        if idx < strval.len() {
            strval[idx]
        } else {
            0
        }
    };

    let mut k: usize = 0;
    while k < strval.len() && strval[k] != 0 {
        if at(k) == DOLLAR && at(k + 1) == LBRACE {
            // ${envname:default}
            k += 2;
            // collect the name (C uses ename[128], writing the terminator at i, so max 127 chars).
            let mut ename: Vec<u8> = Vec::new();
            while at(k) != RBRACE && at(k) != 0 && at(k) != COLON {
                if ename.len() < 127 {
                    ename.push(strval[k]);
                }
                k += 1;
            }
            let name = String::from_utf8_lossy(&ename).into_owned();
            let mut penv: Option<Vec<u8>> = lookup(&name);

            if penv.is_none() {
                if at(k) == COLON {
                    // ${name:default} / ${name:-default}: copy the default value verbatim.
                    k += 1;
                    if at(k) == DASH {
                        k += 1;
                    }
                    while at(k) != RBRACE && at(k) != 0 {
                        env.push(strval[k]);
                        k += 1;
                    }
                } else if ename == b"COB_CONFIG_DIR" {
                    if let Some(cd) = config_dir {
                        penv = Some(cd.to_vec());
                    }
                } else if ename == b"COB_COPY_DIR" {
                    if let Some(cd) = copy_dir {
                        penv = Some(cd.to_vec());
                    }
                }
            }
            if let Some(value) = penv {
                env.extend_from_slice(&value);
            }
            // skip to (and over) the closing brace
            while at(k) != RBRACE && at(k) != 0 {
                k += 1;
            }
            if at(k) == RBRACE {
                k += 1;
            }
            // C does k-- here because the for-loop will k++; we instead `continue` without the
            // trailing k += 1, so adjust by not advancing further. Match by decrementing then
            // re-incrementing at loop end.
            if k > 0 {
                k -= 1;
            }
        } else if at(k) == DOLLAR && at(k + 1) == DOLLAR {
            // Replace $$ with process-id.
            env.extend_from_slice(common_env_itoa(pid).as_slice());
            k += 1;
        } else if !is_c_space(at(k)) {
            env.push(strval[k]);
        } else {
            env.push(SPACE);
        }
        k += 1;
    }

    env
}

/// `isspace`-equivalent for the C locale used by `cob_expand_env_string` (space, `\t`, `\n`,
/// `\v`, `\f`, `\r`).
fn is_c_space(b: u8) -> bool {
    matches!(b, 0x20 | 0x09 | 0x0A | 0x0B | 0x0C | 0x0D)
}

/// Port of `common.c:cob_getenv_direct` -- return the environment value for `name`, or `None`.
///
/// The C `getenv` returns a pointer into the live environment; here the lookup is the boundary.
pub fn cob_getenv_direct(name: &str, lookup: impl Fn(&str) -> Option<Vec<u8>>) -> Option<Vec<u8>> {
    lookup(name)
}

/// Port of `common.c:cob_getenv` -- return an owned copy of the environment value for `name`.
///
/// The C version returns `NULL` for a `NULL` name or an unset variable; otherwise a `cob_strdup`
/// of the value. Here `name` is always present, so this is `cob_getenv_direct` (the returned
/// `Vec<u8>` is already an owned copy, matching the `cob_strdup`).
pub fn cob_getenv(name: &str, lookup: impl Fn(&str) -> Option<Vec<u8>>) -> Option<Vec<u8>> {
    lookup(name)
}

/// Port of `common.c:cob_putenv` -- install a `name=value` string into the environment.
///
/// The C version requires an `'='` in the string and returns `-1` otherwise; on success it calls
/// `putenv` (returning its result) and rescans the cached env values. Modelled here over an
/// explicit env map: the string is split at the **first** `'='` (matching `putenv` semantics) and
/// stored. Returns `0` on success (mirroring a successful `putenv`) or `-1` when no `'='` is
/// present. The env-cache rescan (`cob_rescan_env_vals`) is a global-state side effect outside
/// this pure port.
pub fn cob_putenv(name: &[u8], env: &mut BTreeMap<Vec<u8>, Vec<u8>>) -> i32 {
    if let Some(eq) = name.iter().position(|&b| b == 0x3D) {
        // '='
        let key = name[..eq].to_vec();
        let value = name[eq + 1..].to_vec();
        env.insert(key, value);
        0
    } else {
        -1
    }
}

/// Port of `common.c:cob_setenv` -- set `name` to `value` in the environment.
///
/// On a `HAVE_SETENV` build this wraps `setenv(name, value, overwrite)`; otherwise it builds a
/// `"name=value"` string and calls `putenv`. Both paths are modelled here over an explicit env
/// map. `overwrite == 0` leaves an existing entry untouched (the POSIX `setenv` contract);
/// `overwrite != 0` always stores. Returns `0` on success (the C success value).
pub fn cob_setenv(
    name: &[u8],
    value: &[u8],
    overwrite: i32,
    env: &mut BTreeMap<Vec<u8>, Vec<u8>>,
) -> i32 {
    if overwrite == 0 && env.contains_key(name) {
        return 0;
    }
    env.insert(name.to_vec(), value.to_vec());
    0
}

/// Port of `common.c:cob_unsetenv` -- remove `name` from the environment.
///
/// On a `HAVE_SETENV` build this wraps `unsetenv(name)`; otherwise it `putenv`s `"name="`.
/// Modelled over an explicit env map (removing the key). Returns `0` on success.
pub fn cob_unsetenv(name: &[u8], env: &mut BTreeMap<Vec<u8>, Vec<u8>>) -> i32 {
    env.remove(name);
    0
}

/// Port of the libcob `setenv` fallback (`common.c`, under `#ifdef _MSC_VER`) -- set `name` to
/// `value`, modelled over an explicit env map. The MSVC fallback ignores `overwrite` (its
/// `_putenv_s` always overwrites), so this always stores.
pub fn setenv(name: &[u8], value: &[u8], _overwrite: i32, env: &mut BTreeMap<Vec<u8>, Vec<u8>>) -> i32 {
    env.insert(name.to_vec(), value.to_vec());
    0
}

/// Port of the libcob `unsetenv` fallback (`common.c`, under `#ifdef _MSC_VER`) -- the MSVC
/// fallback is `_putenv_s(name, "")`, i.e. set the variable to an empty value. Modelled over an
/// explicit env map (storing an empty value rather than removing the key, matching `_putenv_s`).
pub fn unsetenv(name: &[u8], env: &mut BTreeMap<Vec<u8>, Vec<u8>>) -> i32 {
    env.insert(name.to_vec(), Vec::new());
    0
}

/// Result of [`cob_display_environment`]: the mangled-or-verbatim environment-variable **name**
/// to be used by a subsequent `cob_display_env_value` / `cob_get_environment`.
///
/// Port of `common.c:cob_display_environment` -- stores the field's string into the module-global
/// `cob_local_env` (here returned explicitly). When `env_mangle` is set, every non-alphanumeric
/// byte is replaced by `'_'` (`0x5F`). The growth of the `cob_local_env` buffer is a global-state
/// detail; the *transform* is what this ports.
pub fn cob_display_environment(field: &[u8], env_mangle: bool) -> Vec<u8> {
    let mut out = field.to_vec();
    if env_mangle {
        for b in out.iter_mut() {
            if !b.is_ascii_alphanumeric() {
                *b = 0x5F; // '_'
            }
        }
    }
    out
}

/// Outcome of [`cob_display_env_value`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayEnvValue {
    /// The variable named `local_env` was set to `value` (caller stores into the env map and
    /// rescans). Mirrors the successful `cob_setenv` path.
    Set { name: Vec<u8>, value: Vec<u8> },
    /// `COB_EC_IMP_DISPLAY` was raised: the local-env name was empty (no preceding
    /// `DISPLAY ... UPON ENVIRONMENT`).
    Exception,
}

/// Port of `common.c:cob_display_env_value` -- `DISPLAY ... UPON ENVIRONMENT VALUE`.
///
/// If no environment name has been stashed (`cob_local_env` empty/unset) it raises
/// `COB_EC_IMP_DISPLAY`. Otherwise it would `cob_setenv(cob_local_env, value, 1)` and rescan.
/// The exception-stack and env mutation are boundaries; this returns the decision as a
/// [`DisplayEnvValue`]. `local_env` is the name produced by [`cob_display_environment`].
pub fn cob_display_env_value(local_env: Option<&[u8]>, value: &[u8]) -> DisplayEnvValue {
    match local_env {
        Some(name) if !name.is_empty() => DisplayEnvValue::Set {
            name: name.to_vec(),
            value: value.to_vec(),
        },
        _ => DisplayEnvValue::Exception,
    }
}

/// Port of `common.c:cob_set_environment` -- the combined `DISPLAY UPON ENVIRONMENT` /
/// `... VALUE`. Equivalent to calling [`cob_display_environment`] on `f1` then
/// [`cob_display_env_value`] on `f2`. Returns the resulting [`DisplayEnvValue`].
pub fn cob_set_environment(f1: &[u8], f2: &[u8], env_mangle: bool) -> DisplayEnvValue {
    let name = cob_display_environment(f1, env_mangle);
    cob_display_env_value(Some(&name), f2)
}

/// Outcome of [`cob_get_environment`] / [`cob_accept_environment`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcceptEnv {
    /// Variable found: move these bytes into the receiving field.
    Value(Vec<u8>),
    /// `COB_EC_IMP_ACCEPT` raised and a single space moved into the field (the C `else` path).
    ExceptionSpace,
    /// `COB_EC_IMP_ACCEPT` raised with no move (zero-size name/value field, or empty name).
    Exception,
}

/// Port of `common.c:cob_get_environment` -- `ACCEPT envval FROM ENVIRONMENT envname`.
///
/// Faithful logic: zero-size name or value field -> [`AcceptEnv::Exception`]. Otherwise the name
/// is taken from `envname`; when `env_mangle` is set, non-alphanumeric bytes become `'_'`. The
/// variable is looked up; found -> [`AcceptEnv::Value`], not found -> [`AcceptEnv::ExceptionSpace`]
/// (exception + a single space moved). `envval_size` is the declared size of the receiving field
/// (used only for the zero-size guard); `lookup` is the OS boundary.
pub fn cob_get_environment(
    envname: &[u8],
    envval_size: usize,
    env_mangle: bool,
    lookup: impl Fn(&str) -> Option<Vec<u8>>,
) -> AcceptEnv {
    if envname.is_empty() || envval_size == 0 {
        return AcceptEnv::Exception;
    }
    // cob_field_to_string would yield < 1 for an all-trailing-removed empty name.
    let mut name = envname.to_vec();
    if name.is_empty() {
        return AcceptEnv::Exception;
    }
    if env_mangle {
        for b in name.iter_mut() {
            if !b.is_ascii_alphanumeric() {
                *b = 0x5F; // '_'
            }
        }
    }
    let key = String::from_utf8_lossy(&name).into_owned();
    match lookup(&key) {
        Some(value) => AcceptEnv::Value(value),
        None => AcceptEnv::ExceptionSpace,
    }
}

/// Port of `common.c:cob_accept_environment` -- `ACCEPT f FROM ENVIRONMENT-VALUE`, using the name
/// stashed by a previous `DISPLAY ... UPON ENVIRONMENT` (`cob_local_env`).
///
/// If `local_env` is set and the variable resolves -> [`AcceptEnv::Value`]; otherwise the C code
/// raises `COB_EC_IMP_ACCEPT` and moves a single space -> [`AcceptEnv::ExceptionSpace`].
pub fn cob_accept_environment(
    local_env: Option<&[u8]>,
    lookup: impl Fn(&str) -> Option<Vec<u8>>,
) -> AcceptEnv {
    if let Some(name) = local_env {
        let key = String::from_utf8_lossy(name).into_owned();
        if let Some(value) = lookup(&key) {
            return AcceptEnv::Value(value);
        }
    }
    AcceptEnv::ExceptionSpace
}

/// Port of `common.c:check_valid_env_tmpdir` -- return the value of env var `envname` if it names
/// a valid, writable directory.
///
/// The C version reads `getenv(envname)`; an unset or empty value yields `None`. If set but the
/// directory is invalid it warns, unsets the variable, and yields `None`. The directory `stat`
/// is the OS boundary: `dir_valid(dir)` returns `true` when the directory is usable (the C
/// `check_valid_dir` returns `0` for valid, so this is its negation). On the invalid path the
/// caller should `cob_unsetenv(envname)`; this function reports it via the returned
/// [`TmpdirCandidate`].
pub fn check_valid_env_tmpdir(
    envname: &str,
    lookup: impl Fn(&str) -> Option<Vec<u8>>,
    dir_valid: impl Fn(&[u8]) -> bool,
) -> TmpdirCandidate {
    match lookup(envname) {
        None => TmpdirCandidate::Unset,
        Some(dir) if dir.is_empty() || dir[0] == 0 => TmpdirCandidate::Unset,
        Some(dir) => {
            if dir_valid(&dir) {
                TmpdirCandidate::Valid(dir)
            } else {
                // C warns and unsets the variable, then returns NULL.
                TmpdirCandidate::Invalid
            }
        }
    }
}

/// Result of [`check_valid_env_tmpdir`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TmpdirCandidate {
    /// Variable set to a valid directory (these bytes).
    Valid(Vec<u8>),
    /// Variable unset or empty.
    Unset,
    /// Variable set but the directory is invalid (caller should unset it).
    Invalid,
}

/// Port of `common.c:cob_gettmpdir` -- determine the temporary directory, without a trailing
/// slash.
///
/// Faithful precedence: try `TMPDIR` first, then the OS-specific list. On a non-Windows build the
/// order is `TMPDIR`, `TMP`, `TEMP`, then `/tmp` (if valid), then `.`. On Windows it is `TMPDIR`,
/// `TEMP`, `TMP`, `USERPROFILE`, then `.`. Each candidate is checked via
/// [`check_valid_env_tmpdir`]; the directory `stat` and the final `cob_setenv("TMPDIR", ...)` are
/// boundaries. A trailing `slash` byte is stripped from the chosen path. Pass `windows = true`
/// to follow the `_WIN32` precedence. Returns the chosen directory bytes.
pub fn cob_gettmpdir(
    windows: bool,
    slash: u8,
    lookup: impl Fn(&str) -> Option<Vec<u8>>,
    dir_valid: impl Fn(&[u8]) -> bool,
) -> Vec<u8> {
    let names: &[&str] = if windows {
        &["TMPDIR", "TEMP", "TMP", "USERPROFILE"]
    } else {
        &["TMPDIR", "TMP", "TEMP"]
    };

    let mut tmpdir: Option<Vec<u8>> = None;
    for name in names {
        match check_valid_env_tmpdir(name, &lookup, &dir_valid) {
            TmpdirCandidate::Valid(dir) => {
                tmpdir = Some(dir);
                break;
            }
            TmpdirCandidate::Unset | TmpdirCandidate::Invalid => {}
        }
    }

    // non-Windows: after the env list, try "/tmp".
    if tmpdir.is_none() && !windows && dir_valid(b"/tmp") {
        tmpdir = Some(b"/tmp".to_vec());
    }

    // fallback: current directory ".".
    let mut chosen = match tmpdir {
        Some(dir) => dir,
        None => b".".to_vec(),
    };

    // strip a single trailing slash (C: if tmpdir[len-1] == SLASH_CHAR, drop it).
    if let Some(&last) = chosen.last() {
        if last == slash {
            chosen.pop();
        }
    }
    chosen
}

/// Local base-10 signed-integer formatter (the C `sprintf(&env[j], "%d", ...)` for `$$`/pid).
/// Defined here to keep this module free of cross-module dependencies.
pub(crate) fn common_env_itoa(value: i32) -> Vec<u8> {
    let mut out = Vec::new();
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
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn map_lookup<'a>(m: &'a BTreeMap<&'a str, &'a str>) -> impl Fn(&str) -> Option<Vec<u8>> + 'a {
        move |name: &str| m.get(name).map(|v| v.as_bytes().to_vec())
    }

    #[test]
    fn expand_braced_var() {
        let mut m = BTreeMap::new();
        m.insert("FOO", "baz");
        let out = cob_expand_env_string(b"${FOO}bar", map_lookup(&m), 0, None, None);
        assert_eq!(out, b"bazbar");
    }

    #[test]
    fn expand_unset_no_default_drops() {
        let m: BTreeMap<&str, &str> = BTreeMap::new();
        let out = cob_expand_env_string(b"x${NOPE}y", map_lookup(&m), 0, None, None);
        assert_eq!(out, b"xy");
    }

    #[test]
    fn expand_default_when_unset() {
        let m: BTreeMap<&str, &str> = BTreeMap::new();
        let out = cob_expand_env_string(b"${NOPE:def}", map_lookup(&m), 0, None, None);
        assert_eq!(out, b"def");
    }

    #[test]
    fn expand_default_dash_form() {
        let m: BTreeMap<&str, &str> = BTreeMap::new();
        let out = cob_expand_env_string(b"${NOPE:-def}", map_lookup(&m), 0, None, None);
        assert_eq!(out, b"def");
    }

    #[test]
    fn expand_set_overrides_default() {
        let mut m = BTreeMap::new();
        m.insert("FOO", "val");
        let out = cob_expand_env_string(b"${FOO:def}", map_lookup(&m), 0, None, None);
        assert_eq!(out, b"val");
    }

    #[test]
    fn expand_config_dir_fallback() {
        let m: BTreeMap<&str, &str> = BTreeMap::new();
        let out = cob_expand_env_string(
            b"${COB_CONFIG_DIR}",
            map_lookup(&m),
            0,
            Some(b"/etc/cob"),
            None,
        );
        assert_eq!(out, b"/etc/cob");
    }

    #[test]
    fn expand_pid_double_dollar() {
        let m: BTreeMap<&str, &str> = BTreeMap::new();
        let out = cob_expand_env_string(b"pid=$$", map_lookup(&m), 4321, None, None);
        assert_eq!(out, b"pid=4321");
    }

    #[test]
    fn expand_whitespace_normalised() {
        let m: BTreeMap<&str, &str> = BTreeMap::new();
        let out = cob_expand_env_string(b"a\tb\nc", map_lookup(&m), 0, None, None);
        assert_eq!(out, b"a b c");
    }

    #[test]
    fn expand_plain_passthrough() {
        let m: BTreeMap<&str, &str> = BTreeMap::new();
        let out = cob_expand_env_string(b"hello", map_lookup(&m), 0, None, None);
        assert_eq!(out, b"hello");
    }

    #[test]
    fn getenv_direct_and_getenv() {
        let mut m = BTreeMap::new();
        m.insert("A", "1");
        assert_eq!(cob_getenv_direct("A", map_lookup(&m)), Some(b"1".to_vec()));
        assert_eq!(cob_getenv_direct("B", map_lookup(&m)), None);
        assert_eq!(cob_getenv("A", map_lookup(&m)), Some(b"1".to_vec()));
    }

    #[test]
    fn putenv_splits_on_first_eq() {
        let mut env = BTreeMap::new();
        assert_eq!(cob_putenv(b"K=v=w", &mut env), 0);
        assert_eq!(env.get(b"K".as_slice()), Some(&b"v=w".to_vec()));
        assert_eq!(cob_putenv(b"noeq", &mut env), -1);
    }

    #[test]
    fn setenv_respects_overwrite() {
        let mut env = BTreeMap::new();
        cob_setenv(b"K", b"1", 1, &mut env);
        cob_setenv(b"K", b"2", 0, &mut env); // no overwrite
        assert_eq!(env.get(b"K".as_slice()), Some(&b"1".to_vec()));
        cob_setenv(b"K", b"2", 1, &mut env); // overwrite
        assert_eq!(env.get(b"K".as_slice()), Some(&b"2".to_vec()));
    }

    #[test]
    fn unsetenv_removes() {
        let mut env = BTreeMap::new();
        cob_setenv(b"K", b"1", 1, &mut env);
        cob_unsetenv(b"K", &mut env);
        assert!(!env.contains_key(b"K".as_slice()));
    }

    #[test]
    fn msvc_unsetenv_empties() {
        let mut env = BTreeMap::new();
        unsetenv(b"K", &mut env);
        assert_eq!(env.get(b"K".as_slice()), Some(&Vec::new()));
    }

    #[test]
    fn display_environment_mangles() {
        assert_eq!(cob_display_environment(b"A.B-C", true), b"A_B_C");
        assert_eq!(cob_display_environment(b"A.B-C", false), b"A.B-C");
    }

    #[test]
    fn display_env_value_decisions() {
        assert_eq!(
            cob_display_env_value(Some(b"FOO"), b"bar"),
            DisplayEnvValue::Set {
                name: b"FOO".to_vec(),
                value: b"bar".to_vec()
            }
        );
        assert_eq!(cob_display_env_value(None, b"bar"), DisplayEnvValue::Exception);
        assert_eq!(cob_display_env_value(Some(b""), b"bar"), DisplayEnvValue::Exception);
    }

    #[test]
    fn set_environment_combines() {
        assert_eq!(
            cob_set_environment(b"FO.O", b"bar", true),
            DisplayEnvValue::Set {
                name: b"FO_O".to_vec(),
                value: b"bar".to_vec()
            }
        );
    }

    #[test]
    fn get_environment_paths() {
        let mut m = BTreeMap::new();
        m.insert("PATH", "/bin");
        assert_eq!(
            cob_get_environment(b"PATH", 80, false, map_lookup(&m)),
            AcceptEnv::Value(b"/bin".to_vec())
        );
        assert_eq!(
            cob_get_environment(b"NOPE", 80, false, map_lookup(&m)),
            AcceptEnv::ExceptionSpace
        );
        assert_eq!(
            cob_get_environment(b"", 80, false, map_lookup(&m)),
            AcceptEnv::Exception
        );
        assert_eq!(
            cob_get_environment(b"PATH", 0, false, map_lookup(&m)),
            AcceptEnv::Exception
        );
    }

    #[test]
    fn get_environment_mangle() {
        let mut m = BTreeMap::new();
        m.insert("A_B", "x");
        assert_eq!(
            cob_get_environment(b"A.B", 80, true, map_lookup(&m)),
            AcceptEnv::Value(b"x".to_vec())
        );
    }

    #[test]
    fn accept_environment_paths() {
        let mut m = BTreeMap::new();
        m.insert("TERM", "xterm");
        assert_eq!(
            cob_accept_environment(Some(b"TERM"), map_lookup(&m)),
            AcceptEnv::Value(b"xterm".to_vec())
        );
        assert_eq!(
            cob_accept_environment(Some(b"NOPE"), map_lookup(&m)),
            AcceptEnv::ExceptionSpace
        );
        assert_eq!(
            cob_accept_environment(None, map_lookup(&m)),
            AcceptEnv::ExceptionSpace
        );
    }

    #[test]
    fn valid_env_tmpdir_outcomes() {
        let mut m = BTreeMap::new();
        m.insert("TMPDIR", "/var/tmp");
        let valid = |_d: &[u8]| true;
        assert_eq!(
            check_valid_env_tmpdir("TMPDIR", map_lookup(&m), valid),
            TmpdirCandidate::Valid(b"/var/tmp".to_vec())
        );
        assert_eq!(
            check_valid_env_tmpdir("NOPE", map_lookup(&m), valid),
            TmpdirCandidate::Unset
        );
        let invalid = |_d: &[u8]| false;
        assert_eq!(
            check_valid_env_tmpdir("TMPDIR", map_lookup(&m), invalid),
            TmpdirCandidate::Invalid
        );
    }

    #[test]
    fn gettmpdir_precedence_tmpdir_first() {
        let mut m = BTreeMap::new();
        m.insert("TMPDIR", "/a");
        m.insert("TMP", "/b");
        let out = cob_gettmpdir(false, b'/', map_lookup(&m), |_d| true);
        assert_eq!(out, b"/a");
    }

    #[test]
    fn gettmpdir_falls_through_to_tmp() {
        let mut m = BTreeMap::new();
        m.insert("TMP", "/b");
        let out = cob_gettmpdir(false, b'/', map_lookup(&m), |_d| true);
        assert_eq!(out, b"/b");
    }

    #[test]
    fn gettmpdir_default_dot_when_nothing_valid() {
        let m: BTreeMap<&str, &str> = BTreeMap::new();
        let out = cob_gettmpdir(false, b'/', map_lookup(&m), |_d| false);
        assert_eq!(out, b".");
    }

    #[test]
    fn gettmpdir_strips_trailing_slash() {
        let mut m = BTreeMap::new();
        m.insert("TMPDIR", "/a/");
        let out = cob_gettmpdir(false, b'/', map_lookup(&m), |_d| true);
        assert_eq!(out, b"/a");
    }

    #[test]
    fn gettmpdir_tmp_fallback_nonwindows() {
        let m: BTreeMap<&str, &str> = BTreeMap::new();
        // no env vars; /tmp validates -> chosen.
        let out = cob_gettmpdir(false, b'/', map_lookup(&m), |d| d == b"/tmp");
        assert_eq!(out, b"/tmp");
    }

    #[test]
    fn gettmpdir_windows_precedence() {
        let mut m = BTreeMap::new();
        m.insert("USERPROFILE", "C:\\Users\\x");
        let out = cob_gettmpdir(true, b'\\', map_lookup(&m), |_d| true);
        assert_eq!(out, b"C:\\Users\\x");
    }

    #[test]
    fn itoa_negative() {
        assert_eq!(common_env_itoa(-12), b"-12");
        assert_eq!(common_env_itoa(0), b"0");
    }
}
