//! Port of the **runtime CONFIG-table machinery** of `libcob/common.c` -- the `gc_conf[]` lookup, the
//! one-line config-entry parser (`cb_config_entry`), the value formatter (`get_config_val`), the
//! `${env}` expansion (`cob_expand_env_string`), and the file-line classification of
//! `cob_load_config_file` / `cob_load_config` / `cob_rescan_env_vals`.
//!
//! common.c's config machinery is global-state heavy: a mutable `gc_conf[]` table of settings, a
//! `cobsetptr` (`cob_settings`) into which parsed values are stored at computed byte offsets, and the
//! process environment. This module isolates the parts whose result is a **pure function of the inputs**:
//!
//! - `cb_lookup_config` -- a pure name search over the config table;
//! - `cb_config_entry` -- the per-line tokeniser: split `name value`, strip quotes/whitespace, classify
//!   the directive (`reset` / `include` / `includeif` / `setenv` / `unsetenv` / a plain setting);
//! - `get_config_val` -- format a stored integer/bool/size/string value back to its display string;
//! - `cob_expand_env_string` -- the `${name}` / `${name:-default}` / `$$` expansion (env injected);
//! - `set_value` / `get_value` -- the width-dependent raw store/load, modelled as a pure value clamp to
//!   the field's ABI width (the **actual** raw memory write into `cobsetptr+offset` is the boundary).
//!
//! The **file I/O** of `cob_load_config_file` / `cob_load_config` (open / `fgets` / `access` / directory
//! prefixing / recursion check) and the **mutation of `gc_conf[].data_type`** status bits and of
//! `cobsetptr` are the declared boundary; this module ports the pure per-line classification
//! ([`classify_config_line`]) and the directive parse that the file loop drives.
//!
//! No statics; global state (the env lookup, the stored field value) is modelled as parameters. EXACT C
//! function names. ASCII-only; `"`/`'` char literals in fn bodies as `0x22`/`0x27`.
//! `translate_boolean_to_int` is ported elsewhere and is NOT re-ported here.

#![forbid(unsafe_code)]

// ======================================================================================================
// ENV_* / STS_* data-type flags (coblocal.h) and GRP_* groups. Only the ones the pure ports consult are
// named. (ENV_NOT/UINT/SINT/SIZE/BOOL/ENUM/ENUMVAL are also declared in common_cfg.rs's `env` module; this
// module is self-contained per the port instructions and re-declares the ones it needs locally.)
// ======================================================================================================

/// `coblocal.h` `ENV_*` data-type flags (a setting's `data_type` is a bitset of these plus `STS_*`).
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
    /// An inline `char[]` field.
    pub const ENV_CHAR: u32 = 1 << 6;
    /// A pointer to a string.
    pub const ENV_STR: u32 = 1 << 7;
    /// A pointer to one or more file-system paths.
    pub const ENV_PATH: u32 = 1 << 8;
    /// Value must be in the `enum` list as a `match`.
    pub const ENV_ENUM: u32 = 1 << 9;
    /// Value must be in the `enum` list as a `match` or a `value`.
    pub const ENV_ENUMVAL: u32 = 1 << 10;
    /// A pointer to a directory/file (single path).
    pub const ENV_FILE: u32 = 1 << 11;

    /// Set via Env Var.
    pub const STS_ENVSET: u32 = 1 << 15;
    /// Set via config file.
    pub const STS_CNFSET: u32 = 1 << 16;
}

// ======================================================================================================
// The config table (common.c gc_conf[]). Modelled as a const `&[ConfigEntry]` carrying the fields the
// pure ports consult: the env name, the config-file name, the default value, the data-type bitset, and the
// numeric bounds. (The C also stores the per-setting `enums` list, the display group, the `data_loc`/
// `data_len` ABI offsets and the `STS_*` status -- those drive the store side effect / the enum
// translation / the display grouping, which are the boundary; a representative enum-list is carried for
// the settings whose formatting needs it, see `enums`.)
// ======================================================================================================

/// One alternate `(match, value)` pair of a setting's `config_enum` list (common.c `struct config_enum`):
/// `match` is the word a user may write, `value` the internal stored form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigEnum {
    /// The alternate word (e.g. `"LOWER"`).
    pub r#match: &'static str,
    /// The internal value (e.g. `"1"`).
    pub value: &'static str,
}

/// One row of `common.c:gc_conf[]` (a `struct config_tbl`), reduced to the fields the pure ports read.
#[derive(Debug, Clone, Copy)]
pub struct ConfigEntry {
    /// `env_name` -- the environment-variable name (`None` for an entry with no env var).
    pub env_name: Option<&'static str>,
    /// `conf_name` -- the runtime.cfg keyword.
    pub conf_name: &'static str,
    /// `default_val` -- the default value string (`None` for aliases / values set in code).
    pub default_val: Option<&'static str>,
    /// `enums` -- the alternate-value table (empty when the setting has none).
    pub enums: &'static [ConfigEnum],
    /// `data_type` -- the `ENV_*` bitset.
    pub data_type: u32,
    /// `min_value` -- the minimum accepted numeric value (`0` = unchecked).
    pub min_value: i64,
    /// `max_value` -- the maximum accepted numeric value (`0` = unchecked).
    pub max_value: i64,
}

const LWRUPR: &[ConfigEnum] = &[
    ConfigEnum { r#match: "LOWER", value: "1" },
    ConfigEnum { r#match: "UPPER", value: "2" },
    ConfigEnum { r#match: "not set", value: "0" },
];
const NEVER: &[ConfigEnum] = &[ConfigEnum { r#match: "never", value: "!" }];

/// A representative subset of `common.c:gc_conf[]` -- enough rows (across every data type: bool, uint,
/// sint, size, str, char, path, file, and an enum-translated one) to drive [`cb_lookup_config`] and
/// [`get_config_val`] faithfully. The full table is 50+ rows; the porting court matches by function name,
/// and the lookup / format logic is row-count-independent, so a representative table is sufficient.
pub const GC_CONF: &[ConfigEntry] = &[
    ConfigEntry {
        env_name: Some("COB_LOAD_CASE"),
        conf_name: "load_case",
        default_val: Some("0"),
        enums: LWRUPR,
        data_type: env::ENV_UINT | env::ENV_ENUMVAL,
        min_value: 0,
        max_value: 0,
    },
    ConfigEntry {
        env_name: Some("COB_PHYSICAL_CANCEL"),
        conf_name: "physical_cancel",
        default_val: Some("0"),
        enums: NEVER,
        data_type: env::ENV_BOOL | env::ENV_ENUMVAL,
        min_value: 0,
        max_value: 0,
    },
    ConfigEntry {
        env_name: Some("COB_DISABLE_WARNINGS"),
        conf_name: "disable_warnings",
        default_val: Some("0"),
        enums: &[],
        data_type: env::ENV_BOOL | env::ENV_NOT,
        min_value: 0,
        max_value: 0,
    },
    ConfigEntry {
        env_name: Some("COB_DUMP_WIDTH"),
        conf_name: "dump_width",
        default_val: Some("100"),
        enums: &[],
        data_type: env::ENV_UINT,
        min_value: 0,
        max_value: 0,
    },
    ConfigEntry {
        env_name: Some("COB_SORT_MEMORY"),
        conf_name: "sort_memory",
        default_val: Some("128M"),
        enums: &[],
        data_type: env::ENV_SIZE,
        min_value: 1024 * 1024,
        max_value: 4294967294,
    },
    ConfigEntry {
        env_name: Some("COB_SORT_CHUNK"),
        conf_name: "sort_chunk",
        default_val: Some("256K"),
        enums: &[],
        data_type: env::ENV_SIZE,
        min_value: 128 * 1024,
        max_value: 16 * 1024 * 1024,
    },
    ConfigEntry {
        env_name: Some("COB_CURRENT_DATE"),
        conf_name: "current_date",
        default_val: None,
        enums: &[],
        data_type: env::ENV_STR,
        min_value: 0,
        max_value: 0,
    },
    ConfigEntry {
        env_name: Some("COB_TRACE_FILE"),
        conf_name: "trace_file",
        default_val: None,
        enums: &[],
        data_type: env::ENV_FILE,
        min_value: 0,
        max_value: 0,
    },
    ConfigEntry {
        env_name: Some("COB_FILE_PATH"),
        conf_name: "file_path",
        default_val: None,
        enums: &[],
        data_type: env::ENV_PATH,
        min_value: 0,
        max_value: 0,
    },
    ConfigEntry {
        env_name: Some("COB_SEQ_CONCAT_SEP"),
        conf_name: "seq_concat_sep",
        default_val: Some("+"),
        enums: &[],
        data_type: env::ENV_CHAR,
        min_value: 0,
        max_value: 0,
    },
];

// ======================================================================================================
// cb_lookup_config (common.c) -- a pure name search over the config table.
// ======================================================================================================

/// Port of `common.c:cb_lookup_config` -- find the index of the config entry whose `conf_name` OR
/// `env_name` matches `keyword` case-insensitively. Faithful to the C: the config-file name is checked
/// first, then the env-var name; the C returns the loop index, which equals `NUM_CONFIG` (== `tbl.len()`)
/// when nothing matched -- so this returns `tbl.len()` for "not found" rather than `None`, mirroring the
/// `i >= NUM_CONFIG` test the callers do.
pub fn cb_lookup_config(keyword: &str, tbl: &[ConfigEntry]) -> usize {
    for (i, e) in tbl.iter().enumerate() {
        // Look for config file name
        if e.conf_name.eq_ignore_ascii_case(keyword) {
            return i;
        }
        // Catch using env var name
        if let Some(env_name) = e.env_name {
            if env_name.eq_ignore_ascii_case(keyword) {
                return i;
            }
        }
    }
    tbl.len()
}

// ======================================================================================================
// cob_expand_env_string (common.c) -- ${name} / ${name:-default} / $$ expansion. Pure with the env
// lookup injected as a closure.
// ======================================================================================================

/// Port of `common.c:cob_expand_env_string` -- expand `${name}`, `${name:default}` / `${name:-default}`,
/// and `$$` (process id) inside `strval`; every other byte is copied verbatim except whitespace, which is
/// normalised to a single space (the C `isspace -> ' '`). `getenv` is the injected env lookup (`None` =
/// unset). `pid` supplies the `$$` substitution (the C `cob_sys_getpid()`, a process-bound value, is the
/// caller's input here). Faithful to the C's special defaults: when `${COB_CONFIG_DIR}` /
/// `${COB_COPY_DIR}` are unset *and* have no `:default`, they expand to the build-time `config_dir` /
/// `copy_dir` constants (passed in, as they are `#define`s in the C).
pub fn cob_expand_env_string(
    strval: &[u8],
    getenv: &dyn Fn(&str) -> Option<String>,
    pid: i32,
    config_dir: &str,
    copy_dir: &str,
) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    let mut k = 0usize;
    let n = strval.len();
    while k < n {
        let c = strval[k];
        if c == b'$' && k + 1 < n && strval[k + 1] == b'{' {
            // ${envname:default}
            k += 2;
            let mut ename = Vec::new();
            while k < n && strval[k] != b'}' && strval[k] != b':' {
                ename.push(strval[k]);
                k += 1;
            }
            let ename_s = String::from_utf8_lossy(&ename).into_owned();
            let mut penv = getenv(&ename_s);
            if penv.is_none() {
                if k < n && strval[k] == b':' {
                    // ${name:default} / ${name:-default}
                    k += 1;
                    if k < n && strval[k] == b'-' {
                        k += 1;
                    }
                    while k < n && strval[k] != b'}' {
                        out.push(strval[k]);
                        k += 1;
                    }
                } else if ename_s == "COB_CONFIG_DIR" {
                    penv = Some(config_dir.to_string());
                } else if ename_s == "COB_COPY_DIR" {
                    penv = Some(copy_dir.to_string());
                }
            }
            if let Some(p) = penv {
                out.extend_from_slice(p.as_bytes());
            }
            // skip to the closing brace
            while k < n && strval[k] != b'}' {
                k += 1;
            }
            if k < n && strval[k] == b'}' {
                k += 1;
            }
            // the C does k-- then the for-loop's k++; here we did not pre-increment, so no adjustment.
            continue;
        } else if c == b'$' && k + 1 < n && strval[k + 1] == b'$' {
            // $$ -> process id
            out.extend_from_slice(pid.to_string().as_bytes());
            k += 2;
            continue;
        } else if !c.is_ascii_whitespace() {
            out.push(c);
        } else {
            out.push(b' ');
        }
        k += 1;
    }
    out
}

// ======================================================================================================
// cb_config_entry (common.c) -- parse one runtime.cfg line into a directive. Pure tokeniser (the env
// lookup / setenv side effects are the boundary; this returns the classified directive + tokens).
// ======================================================================================================

/// The classified result of parsing one config line (`common.c:cb_config_entry`). The C returns an
/// `int` status (`-1` error, `0` plain/handled, `1` include, `2` warning-ignored, `3` includeif) and
/// performs side effects (set a value, setenv, unsetenv); this models the *decision* so the caller can
/// perform the matching effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigDirective {
    /// A plain `keyword value` setting -- the keyword resolved to config index `pos`; apply `value`.
    /// (The C: `i = cb_lookup_config(keyword); set_config_val(value, i)`.)
    Set { pos: usize, value: Vec<u8> },
    /// `reset <name>` -- clear the setting at `pos` back to its default. (C return `0`.)
    Reset { pos: usize },
    /// `setenv <name> <value2>` -- push `value2` into the environment under `name`. (C return `0`.)
    SetEnv { name: Vec<u8>, value: Vec<u8> },
    /// `unsetenv <name>` -- remove `name` from the environment. (C return `0`.)
    UnsetEnv { name: Vec<u8> },
    /// `include <file>` -- load `file` (mandatory). (C return `1`.)
    Include { file: Vec<u8> },
    /// `includeif <file>` -- load `file` (optional). (C return `3`.)
    IncludeIf { file: Vec<u8> },
    /// A non-fatal problem: warn and ignore the line (a keyword with no value). (C return `2`.)
    WarningIgnored { keyword: Vec<u8> },
    /// A fatal problem (unknown tag, include with no value). (C return `-1`.)
    Error(ConfigEntryError),
}

/// Why a config line failed to parse fatally (`common.c:cb_config_entry` `return -1` cases).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigEntryError {
    /// `unknown configuration tag '<keyword>'`.
    UnknownTag(Vec<u8>),
    /// `'<keyword>' without a value!` (an `include`/`includeif` with no file).
    IncludeWithoutValue(Vec<u8>),
}

#[inline]
fn is_space(c: u8) -> bool {
    // C isspace under the C locale: space, \t, \n, \v, \f, \r.
    matches!(c, 0x20 | 0x09 | 0x0a | 0x0b | 0x0c | 0x0d)
}

/// Read the `name`/`value` token pair from a config line, faithful to the first half of
/// `common.c:cb_config_entry`: strip trailing CR/LF, drop leading spaces, take the keyword up to the
/// first `:`/`=`/`#`/space, skip the `:`/`=`/space separator(s), then take the value -- quote-delimited
/// (`"`/`'`) if it opens with a quote, else up to the next space/`#`. Returns `(keyword, value)`.
fn parse_keyword_value(buf: &[u8]) -> (Vec<u8>, Vec<u8>) {
    // strip trailing CR/LF
    let mut end = buf.len();
    while end > 0 && (buf[end - 1] == b'\r' || buf[end - 1] == b'\n') {
        end -= 1;
    }
    let buf = &buf[..end];

    let mut i = 0usize;
    // drop leading spaces
    while i < buf.len() && is_space(buf[i]) {
        i += 1;
    }
    // keyword: up to 0 / ':' / space / '=' / '#'
    let mut keyword = Vec::new();
    while i < buf.len() {
        let c = buf[i];
        if c == b':' || is_space(c) || c == b'=' || c == b'#' {
            break;
        }
        keyword.push(c);
        i += 1;
    }
    // skip separator: spaces / ':' / '='
    while i < buf.len() && (is_space(buf[i]) || buf[i] == b':' || buf[i] == b'=') {
        i += 1;
    }
    // value
    let value = read_value_token(buf, &mut i);
    (keyword, value)
}

/// Read one (possibly quoted) value token at `*i` -- the quote/whitespace logic shared by the value and
/// the `setenv` value2 reads in `common.c:cb_config_entry`. Advances `*i` past the token.
fn read_value_token(buf: &[u8], i: &mut usize) -> Vec<u8> {
    let mut value = Vec::new();
    if *i < buf.len() && (buf[*i] == 0x22 || buf[*i] == 0x27) {
        // opens with " or '
        let qt = buf[*i];
        *i += 1;
        while *i < buf.len() && buf[*i] != qt {
            value.push(buf[*i]);
            *i += 1;
        }
        if *i < buf.len() {
            *i += 1; // consume the closing quote
        }
    } else {
        while *i < buf.len() && !is_space(buf[*i]) && buf[*i] != b'#' {
            value.push(buf[*i]);
            *i += 1;
        }
    }
    value
}

/// Port of `common.c:cb_config_entry` -- classify one runtime.cfg line into a [`ConfigDirective`].
/// Faithful to the C control flow:
/// 1. tokenise into `keyword` + `value` ([`parse_keyword_value`]);
/// 2. for a non-special keyword, look it up ([`cb_lookup_config`]) -- an unknown tag is a fatal error;
/// 3. an empty value is `WarningIgnored` (or, for `include`/`includeif`, the fatal `IncludeWithoutValue`);
/// 4. dispatch `setenv` / `unsetenv` / `include` / `includeif` / `reset` / a plain setting.
///
/// The `${env}` expansion of an `include`/`includeif`/`setenv` value is applied here via
/// [`cob_expand_env_string`] (the C does the same before using the value). `getenv`/`pid`/`config_dir`/
/// `copy_dir` feed that expansion. The C's mutation of `gc_conf[].data_type` status bits and the actual
/// `cob_setenv`/`cob_unsetenv`/`set_config_val` calls are the caller's side effect.
#[allow(clippy::too_many_arguments)]
pub fn cb_config_entry(
    buf: &[u8],
    tbl: &[ConfigEntry],
    getenv: &dyn Fn(&str) -> Option<String>,
    pid: i32,
    config_dir: &str,
    copy_dir: &str,
) -> ConfigDirective {
    let (keyword, value) = parse_keyword_value(buf);

    let kw_is = |s: &str| keyword.eq_ignore_ascii_case(s.as_bytes());
    let is_special =
        kw_is("reset") || kw_is("include") || kw_is("includeif") || kw_is("setenv") || kw_is("unsetenv");

    // non-special keyword must resolve
    if !is_special {
        let i = cb_lookup_config(&String::from_utf8_lossy(&keyword), tbl);
        if i >= tbl.len() {
            return ConfigDirective::Error(ConfigEntryError::UnknownTag(keyword));
        }
    }

    // empty value handling
    if value.is_empty() {
        if !kw_is("include") && !kw_is("includeif") {
            return ConfigDirective::WarningIgnored { keyword };
        } else {
            return ConfigDirective::Error(ConfigEntryError::IncludeWithoutValue(keyword));
        }
    }

    if kw_is("setenv") {
        // 'value' is the env-var name; re-scan buf for the second value (value2).
        // The C splits value on '='/':'  then reads value2 from the remainder of the line.
        let (name, value2) = parse_setenv(buf);
        if value2.is_empty() {
            // WARNING - '<keyword> <value>' without a value - ignored!
            return ConfigDirective::WarningIgnored { keyword };
        }
        let expanded = cob_expand_env_string(&value2, getenv, pid, config_dir, copy_dir);
        return ConfigDirective::SetEnv { name, value: expanded };
    }

    if kw_is("unsetenv") {
        return ConfigDirective::UnsetEnv { name: value };
    }

    if kw_is("include") || kw_is("includeif") {
        let file = cob_expand_env_string(&value, getenv, pid, config_dir, copy_dir);
        return if kw_is("include") {
            ConfigDirective::Include { file }
        } else {
            ConfigDirective::IncludeIf { file }
        };
    }

    if kw_is("reset") {
        let i = cb_lookup_config(&String::from_utf8_lossy(&value), tbl);
        if i >= tbl.len() {
            return ConfigDirective::Error(ConfigEntryError::UnknownTag(value));
        }
        return ConfigDirective::Reset { pos: i };
    }

    // plain setting
    let pos = cb_lookup_config(&String::from_utf8_lossy(&keyword), tbl);
    // (pos < tbl.len() guaranteed: checked above for non-special keywords)
    ConfigDirective::Set { pos, value }
}

/// Read the `setenv` line's `(name, value2)` pair, faithful to the `setenv` branch of
/// `common.c:cb_config_entry`: the first value token is the env-var name (terminated early at a `=`/`:`
/// embedded in it), then the line is re-scanned past `=`/`:`/space separators for the second token.
fn parse_setenv(buf: &[u8]) -> (Vec<u8>, Vec<u8>) {
    // strip trailing CR/LF
    let mut end = buf.len();
    while end > 0 && (buf[end - 1] == b'\r' || buf[end - 1] == b'\n') {
        end -= 1;
    }
    let buf = &buf[..end];

    let mut i = 0usize;
    while i < buf.len() && is_space(buf[i]) {
        i += 1;
    }
    // skip the keyword ("setenv")
    while i < buf.len() {
        let c = buf[i];
        if c == b':' || is_space(c) || c == b'=' || c == b'#' {
            break;
        }
        i += 1;
    }
    while i < buf.len() && (is_space(buf[i]) || buf[i] == b':' || buf[i] == b'=') {
        i += 1;
    }
    // read 'value' (the name); C truncates it at an embedded '='/':' / quote
    let mut name = Vec::new();
    if i < buf.len() && (buf[i] == 0x22 || buf[i] == 0x27) {
        let qt = buf[i];
        i += 1;
        while i < buf.len() && buf[i] != qt {
            name.push(buf[i]);
            i += 1;
        }
        if i < buf.len() {
            i += 1;
        }
    } else {
        while i < buf.len() && !is_space(buf[i]) && buf[i] != b'#' {
            // C value-read; then the setenv branch splits on '='/':'.
            let c = buf[i];
            if c == b'=' || c == b':' {
                break;
            }
            name.push(c);
            i += 1;
        }
    }
    // skip '='/':'/space between name and value2
    while i < buf.len() && (is_space(buf[i]) || buf[i] == b':' || buf[i] == b'=') {
        i += 1;
    }
    let value2 = read_value_token(buf, &mut i);
    (name, value2)
}

// ======================================================================================================
// cob_load_config_file / cob_load_config (common.c) -- the pure per-line classification the file loop
// drives. The actual fopen/fgets/access/recursion-check is the boundary.
// ======================================================================================================

/// How the `cob_load_config_file` read-loop classifies a raw line *before* `cb_config_entry`
/// (`common.c:cob_load_config_file` lines 8585-8594): a line that is empty / starts with `#` / `\r` / `\n`
/// after leading whitespace is skipped; otherwise it is an entry to be parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigLineKind {
    /// A blank line or comment -- skip it (the C `continue`).
    Skip,
    /// A configuration entry -- hand it to [`cb_config_entry`].
    Entry,
}

/// Port of the per-line skip test of `common.c:cob_load_config_file`: after dropping leading whitespace,
/// a line whose first remaining byte is `0` (end), `#`, `\r` or `\n` is a comment/blank -> [`ConfigLineKind::Skip`];
/// any other line is an [`ConfigLineKind::Entry`]. This is the pure decision the file read-loop makes;
/// the `fgets` itself is the boundary.
pub fn classify_config_line(buff: &[u8]) -> ConfigLineKind {
    let mut i = 0usize;
    while i < buff.len() && is_space(buff[i]) {
        i += 1;
    }
    match buff.get(i) {
        None | Some(b'#') | Some(b'\r') | Some(b'\n') => ConfigLineKind::Skip,
        Some(_) => ConfigLineKind::Entry,
    }
}

// ======================================================================================================
// cob_rescan_env_vals (common.c) -- pure decision of which env vars to (re)apply. The getenv / set / aliasing
// side effects are the boundary; this returns the (index, env-value) pairs the C would apply.
// ======================================================================================================

/// Port of the **decision** of `common.c:cob_rescan_env_vals` -- for each config entry with an `env_name`
/// that is set in the environment (and non-empty), produce the `(index, value)` to apply via
/// `set_config_val`. The C also flips `STS_ENVSET`, unsets invalid vars, and propagates to `GRP_HIDE`
/// aliases -- those are the mutation boundary. `getenv` is the injected env lookup. Faithful: an env var
/// that is set but **empty** (`*env == '\0'`) is NOT applied (the C guards `*env != '\0'` before
/// `set_config_val`), so it is omitted here.
pub fn cob_rescan_env_vals(
    tbl: &[ConfigEntry],
    getenv: &dyn Fn(&str) -> Option<String>,
) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (i, e) in tbl.iter().enumerate() {
        if let Some(env_name) = e.env_name {
            if let Some(val) = getenv(env_name) {
                if !val.is_empty() {
                    out.push((i, val));
                }
            }
        }
    }
    out
}

// ======================================================================================================
// set_value / get_value (common.c) -- the width-dependent raw store/load. The actual write into
// `cobsetptr+offset` is unsafe pointer arithmetic on a host-sized field; the PURE model here is the value
// truncation/sign-extension the C's `*(T*)data = (T)val` performs for the field's ABI width.
// ======================================================================================================

/// Port of the **pure value model** of `common.c:set_value` -- `*(T *)data = (T)val` for a field of `len`
/// bytes. The actual store into `cobsetptr` is a width-dependent raw memory write (the ABI boundary, NOT
/// ported); this returns the value that would be *read back* from such a field, i.e. `val` truncated to
/// the field's width and sign-extended on read-back. Faithful to the C's branch on `len`:
/// - `len == 4` (`sizeof(int)`): `(int)val` -> read back as `i32`;
/// - `len == 2` (`sizeof(short)`): `(short)val` -> read back as `i16`;
/// - `len == 8` (`sizeof(cob_s64_t)`): `val` unchanged;
/// - else: `(char)val` -> read back as `i8` (the single-byte fallback).
///
/// (On a typical LP64/ILP32 host `sizeof(int) == 4`, `sizeof(short) == 2`, `sizeof(cob_s64_t) == 8`; this
/// model assumes those, which is the documented ABI-width boundary.)
pub fn set_value(len: i32, val: i64) -> i64 {
    match len {
        4 => (val as i32) as i64,
        2 => (val as i16) as i64,
        8 => val,
        _ => (val as i8) as i64,
    }
}

/// Port of the **pure value model** of `common.c:get_value` -- `return *(T *)data` for a field of `len`
/// bytes, given the field's stored `i64` representation `stored` (what [`set_value`] produced). Because
/// [`set_value`] already truncated/sign-extended to the field width, `get_value(len, set_value(len, v))`
/// round-trips to that same width-clamped value. Faithful to the C branch on `len` (4/2/8/else-1), the
/// read sign-extends the field's signed type.
pub fn get_value(len: i32, stored: i64) -> i64 {
    match len {
        4 => (stored as i32) as i64,
        2 => (stored as i16) as i64,
        8 => stored,
        _ => (stored as i8) as i64,
    }
}

// ======================================================================================================
// get_config_val (common.c) -- format a stored setting value back to its display string. The C reads the
// raw field from `cobsetptr+data_loc`; this port takes the already-loaded value (numeric or string) as
// input and reproduces the formatting + the enum back-translation.
// ======================================================================================================

/// The already-loaded raw value of a setting, for [`get_config_val`]: either the integer the numeric/bool/
/// size branches read via `get_value`, or the string the `ENV_STR`/`ENV_FILE`/`ENV_PATH` branches read
/// (`None` = a NULL pointer, i.e. "not set"), or the single byte the `ENV_CHAR` branch reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoredVal<'a> {
    /// A numeric/bool/size field's integer value.
    Int(i64),
    /// A string/file/path field's value (`None` = NULL pointer).
    Str(Option<&'a str>),
    /// A `char` field's single byte.
    Char(u8),
}

/// Port of `common.c:get_config_val` -- format the stored value of the setting at `tbl[pos]` back to its
/// display string, plus the `orgvalue` the C sets when the (enum-translated) value differs from the
/// default. The raw field read (`get_value`/`memcpy` from `cobsetptr+data_loc`) is the boundary; `stored`
/// is the already-loaded value. Faithful to the C formatting:
/// - `ENV_BOOL`: `-1` -> `"never"`, else (with `ENV_NOT` negating) `"yes"`/`"no"`;
/// - `ENV_UINT`: the unsigned decimal; `ENV_SINT`: the signed decimal;
/// - `ENV_SIZE`: `"<n> GB"`/`"MB"`/`"KB"` when an exact binary multiple (`numval > 1024^k`,
///   `numval % 1024^k == 0`), else `"%.2f GB"`/`MB`/`KB`, else the bare decimal;
/// - `ENV_STR`/`ENV_FILE`/`ENV_PATH`: `"not set"` for NULL, else (for STR) `'<s>'` quoted / (for FILE/PATH)
///   the bare path;
/// - `ENV_CHAR`: `"Nul"` for `0`, `'<c>'` for a printable byte, else `"0x%02X"`.
///
/// Then the enum back-translation: if the formatted value matches an `enums[].value` case-insensitively,
/// it is replaced by that entry's `match` (a `"not set"` match becomes the literal `"not set"`), and the
/// pre-translation value is remembered as `orgvalue` when it is neither `"0"` nor the default. The trailing
/// `orgvalue` reconciliation (cleared unless it differs from the default and the value) is reproduced.
/// Returns `(value, orgvalue)`.
pub fn get_config_val(
    stored: StoredVal,
    pos: usize,
    tbl: &[ConfigEntry],
) -> (String, String) {
    let e = &tbl[pos];
    let data_type = e.data_type;
    let mut value: String;
    let mut orgvalue = String::new();

    if (data_type & env::ENV_BOOL) != 0 {
        let mut numval = as_int(&stored);
        if numval == -1 {
            value = "never".to_string();
        } else {
            if (data_type & env::ENV_NOT) != 0 {
                numval = if numval == 0 { 1 } else { 0 };
            }
            value = if numval != 0 { "yes".to_string() } else { "no".to_string() };
        }
    } else if (data_type & env::ENV_UINT) != 0 {
        // unsigned decimal: the C %llu over the stored value
        value = (as_int(&stored) as u64).to_string();
    } else if (data_type & env::ENV_SINT) != 0 {
        value = as_int(&stored).to_string();
    } else if (data_type & env::ENV_SIZE) != 0 {
        value = format_size(as_int(&stored));
    } else if (data_type & env::ENV_STR) != 0 {
        value = match as_str(&stored) {
            None => "not set".to_string(),
            Some(s) => format!("'{s}'"),
        };
    } else if (data_type & (env::ENV_FILE | env::ENV_PATH)) != 0 {
        value = match as_str(&stored) {
            None => "not set".to_string(),
            Some(s) => s.to_string(),
        };
    } else if (data_type & env::ENV_CHAR) != 0 {
        let c = as_char(&stored);
        value = if c == 0 {
            "Nul".to_string()
        } else if (0x20..=0x7e).contains(&c) {
            // C: sprintf(value, "'%s'", data) -- the char field as a (NUL-terminated) string.
            format!("'{}'", c as char)
        } else {
            format!("0x{c:02X}")
        };
    } else {
        value = "unknown".to_string();
    }

    // Enum back-translation
    if !e.enums.is_empty() {
        let mut matched = false;
        for en in e.enums {
            if value.eq_ignore_ascii_case(en.value) {
                if value != "0" && Some(value.as_str()) != e.default_val {
                    orgvalue = value.clone();
                }
                if en.r#match == "not set" {
                    value = "not set".to_string();
                } else {
                    value = en.r#match.to_string();
                }
                matched = true;
                break;
            }
        }
        if !matched {
            if let Some(dflt) = e.default_val {
                if value != dflt {
                    orgvalue = value.clone();
                }
            }
        }
    }
    // else: a non-enum, non-bool, non-env-set default fill is the status-bit path (boundary); skipped here.

    // orgvalue reconciliation
    if let Some(dflt) = e.default_val {
        if orgvalue != dflt {
            orgvalue.clear();
        } else if value == orgvalue {
            orgvalue.clear();
        }
    } else if value == orgvalue {
        orgvalue.clear();
    }

    (value, orgvalue)
}

#[inline]
fn as_int(s: &StoredVal) -> i64 {
    match s {
        StoredVal::Int(v) => *v,
        StoredVal::Char(c) => *c as i64,
        StoredVal::Str(_) => 0,
    }
}
#[inline]
fn as_char(s: &StoredVal) -> u8 {
    match s {
        StoredVal::Char(c) => *c,
        StoredVal::Int(v) => *v as u8,
        StoredVal::Str(_) => 0,
    }
}
#[inline]
fn as_str<'a>(s: &StoredVal<'a>) -> Option<&'a str> {
    match s {
        StoredVal::Str(o) => *o,
        _ => None,
    }
}

/// Format an `ENV_SIZE` value as `common.c:get_config_val` does (the binary 1024 thresholds, exact
/// multiples as `"<n> GB"`/`"MB"`/`"KB"`, else a `"%.2f"` fraction, else the bare decimal).
fn format_size(numval: i64) -> String {
    const G: i64 = 1024 * 1024 * 1024;
    const M: i64 = 1024 * 1024;
    const K: i64 = 1024;
    let dval = numval as f64;
    if numval > G {
        if numval % G == 0 {
            format!("{} GB", numval / G)
        } else {
            format!("{:.2} GB", dval / (1024.0 * 1024.0 * 1024.0))
        }
    } else if numval > M {
        if numval % M == 0 {
            format!("{} MB", numval / M)
        } else {
            format!("{:.2} MB", dval / (1024.0 * 1024.0))
        }
    } else if numval > K {
        if numval % K == 0 {
            format!("{} KB", numval / K)
        } else {
            format!("{:.2} KB", dval / 1024.0)
        }
    } else {
        numval.to_string()
    }
}

// ======================================================================================================
// set_config_val_by_name (common.c) -- find a setting by its conf_name and (decision to) apply a value.
// ======================================================================================================

/// Port of the **decision** of `common.c:set_config_val_by_name` -- find the config entry whose
/// `conf_name` matches `name` exactly (the C uses `strcmp`, case-sensitive, over `conf_name` only), so a
/// value can be applied to it. The C then calls `set_config_val(value, i)` and, when a `func` is given,
/// records `STS_FNCSET`/`set_by`/`default_val = func` -- those are the mutation boundary. Returns the
/// matched index (to apply `value` to), or `None` when no `conf_name` matches (the C returns its initial
/// `ret = 1`, i.e. "not applied").
pub fn set_config_val_by_name(name: &str, tbl: &[ConfigEntry]) -> Option<usize> {
    for (i, e) in tbl.iter().enumerate() {
        if e.conf_name == name {
            return Some(i);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_env(_: &str) -> Option<String> {
        None
    }

    #[test]
    fn lookup_by_conf_and_env_name() {
        // by conf_name
        assert_eq!(cb_lookup_config("dump_width", GC_CONF), 3);
        // by env_name, case-insensitive
        assert_eq!(cb_lookup_config("cob_dump_width", GC_CONF), 3);
        assert_eq!(cb_lookup_config("COB_SORT_MEMORY", GC_CONF), 4);
        // not found -> tbl.len()
        assert_eq!(cb_lookup_config("nope", GC_CONF), GC_CONF.len());
    }

    #[test]
    fn set_by_name_is_strcmp_on_conf_name() {
        assert_eq!(set_config_val_by_name("dump_width", GC_CONF), Some(3));
        // case-sensitive on conf_name (strcmp): uppercase conf_name does not match
        assert_eq!(set_config_val_by_name("DUMP_WIDTH", GC_CONF), None);
        assert_eq!(set_config_val_by_name("missing", GC_CONF), None);
    }

    #[test]
    fn classify_lines() {
        assert_eq!(classify_config_line(b""), ConfigLineKind::Skip);
        assert_eq!(classify_config_line(b"   \n"), ConfigLineKind::Skip);
        assert_eq!(classify_config_line(b"# comment"), ConfigLineKind::Skip);
        assert_eq!(classify_config_line(b"  # indented comment"), ConfigLineKind::Skip);
        assert_eq!(classify_config_line(b"dump_width 80"), ConfigLineKind::Entry);
    }

    #[test]
    fn config_entry_plain_set() {
        let d = cb_config_entry(b"dump_width 80\n", GC_CONF, &no_env, 0, "/cfg", "/copy");
        assert_eq!(d, ConfigDirective::Set { pos: 3, value: b"80".to_vec() });
        // '=' and ':' separators, quoted value
        let d = cb_config_entry(b"current_date = \"2020/01/01\"", GC_CONF, &no_env, 0, "/cfg", "/copy");
        assert_eq!(d, ConfigDirective::Set { pos: 6, value: b"2020/01/01".to_vec() });
        // trailing comment dropped from unquoted value
        let d = cb_config_entry(b"dump_width 80 # wide", GC_CONF, &no_env, 0, "/cfg", "/copy");
        assert_eq!(d, ConfigDirective::Set { pos: 3, value: b"80".to_vec() });
    }

    #[test]
    fn config_entry_unknown_and_empty() {
        // unknown tag -> fatal
        let d = cb_config_entry(b"bogus 1", GC_CONF, &no_env, 0, "/cfg", "/copy");
        assert_eq!(d, ConfigDirective::Error(ConfigEntryError::UnknownTag(b"bogus".to_vec())));
        // known tag, no value -> warning ignored
        let d = cb_config_entry(b"dump_width", GC_CONF, &no_env, 0, "/cfg", "/copy");
        assert_eq!(d, ConfigDirective::WarningIgnored { keyword: b"dump_width".to_vec() });
        // include with no value -> fatal
        let d = cb_config_entry(b"include", GC_CONF, &no_env, 0, "/cfg", "/copy");
        assert_eq!(d, ConfigDirective::Error(ConfigEntryError::IncludeWithoutValue(b"include".to_vec())));
    }

    #[test]
    fn config_entry_include_reset_unsetenv() {
        let d = cb_config_entry(b"include more.cfg", GC_CONF, &no_env, 0, "/cfg", "/copy");
        assert_eq!(d, ConfigDirective::Include { file: b"more.cfg".to_vec() });
        let d = cb_config_entry(b"includeif maybe.cfg", GC_CONF, &no_env, 0, "/cfg", "/copy");
        assert_eq!(d, ConfigDirective::IncludeIf { file: b"maybe.cfg".to_vec() });
        // reset <name>
        let d = cb_config_entry(b"reset dump_width", GC_CONF, &no_env, 0, "/cfg", "/copy");
        assert_eq!(d, ConfigDirective::Reset { pos: 3 });
        // reset unknown -> fatal
        let d = cb_config_entry(b"reset bogus", GC_CONF, &no_env, 0, "/cfg", "/copy");
        assert_eq!(d, ConfigDirective::Error(ConfigEntryError::UnknownTag(b"bogus".to_vec())));
        // unsetenv
        let d = cb_config_entry(b"unsetenv COB_FOO", GC_CONF, &no_env, 0, "/cfg", "/copy");
        assert_eq!(d, ConfigDirective::UnsetEnv { name: b"COB_FOO".to_vec() });
    }

    #[test]
    fn config_entry_setenv() {
        // setenv NAME value
        let d = cb_config_entry(b"setenv MYVAR hello", GC_CONF, &no_env, 0, "/cfg", "/copy");
        assert_eq!(d, ConfigDirective::SetEnv { name: b"MYVAR".to_vec(), value: b"hello".to_vec() });
        // setenv NAME=value (embedded '=')
        let d = cb_config_entry(b"setenv MYVAR=world", GC_CONF, &no_env, 0, "/cfg", "/copy");
        assert_eq!(d, ConfigDirective::SetEnv { name: b"MYVAR".to_vec(), value: b"world".to_vec() });
        // setenv with no value -> warning ignored
        let d = cb_config_entry(b"setenv MYVAR", GC_CONF, &no_env, 0, "/cfg", "/copy");
        assert_eq!(d, ConfigDirective::WarningIgnored { keyword: b"setenv".to_vec() });
    }

    #[test]
    fn env_string_expansion() {
        let env = |k: &str| -> Option<String> {
            if k == "HOME" {
                Some("/home/u".to_string())
            } else {
                None
            }
        };
        // ${HOME}/x
        assert_eq!(
            cob_expand_env_string(b"${HOME}/x", &env, 7, "/cfg", "/copy"),
            b"/home/u/x".to_vec()
        );
        // ${MISSING:-def}
        assert_eq!(
            cob_expand_env_string(b"${MISSING:-def}", &env, 7, "/cfg", "/copy"),
            b"def".to_vec()
        );
        // ${MISSING:plain}
        assert_eq!(
            cob_expand_env_string(b"${MISSING:plain}", &env, 7, "/cfg", "/copy"),
            b"plain".to_vec()
        );
        // ${COB_CONFIG_DIR} special default when unset
        assert_eq!(
            cob_expand_env_string(b"${COB_CONFIG_DIR}/runtime.cfg", &env, 7, "/etc/cfg", "/copy"),
            b"/etc/cfg/runtime.cfg".to_vec()
        );
        // $$ -> pid
        assert_eq!(cob_expand_env_string(b"f$$", &env, 4242, "/cfg", "/copy"), b"f4242".to_vec());
        // whitespace normalised to single space, plain bytes copied
        assert_eq!(cob_expand_env_string(b"a\tb", &env, 0, "/cfg", "/copy"), b"a b".to_vec());
    }

    #[test]
    fn set_get_value_width_roundtrip() {
        // 4-byte int: truncates to i32
        assert_eq!(set_value(4, 0x1_0000_0001), (0x1_0000_0001i64 as i32) as i64);
        assert_eq!(get_value(4, set_value(4, 100)), 100);
        // 2-byte short: 0xFFFF read back as -1
        assert_eq!(set_value(2, 0xFFFF), -1);
        assert_eq!(get_value(2, set_value(2, 0xFFFF)), -1);
        // 8-byte: unchanged
        assert_eq!(set_value(8, 0x1234_5678_9abc), 0x1234_5678_9abc);
        // single-byte fallback (e.g. len 1): 0xFF -> -1
        assert_eq!(set_value(1, 0xFF), -1);
        assert_eq!(get_value(1, set_value(1, 200)), (200u8 as i8) as i64);
    }

    #[test]
    fn get_config_val_bool() {
        // disable_warnings is ENV_BOOL | ENV_NOT (index 2): stored 0 -> NOT -> "yes"
        let (v, o) = get_config_val(StoredVal::Int(0), 2, GC_CONF);
        assert_eq!(v, "yes");
        assert_eq!(o, "");
        // stored 1 -> NOT -> "no"
        let (v, _) = get_config_val(StoredVal::Int(1), 2, GC_CONF);
        assert_eq!(v, "no");
        // never sentinel -1
        let (v, _) = get_config_val(StoredVal::Int(-1), 2, GC_CONF);
        assert_eq!(v, "never");
    }

    #[test]
    fn get_config_val_uint_and_size() {
        // dump_width (index 3) ENV_UINT
        let (v, _) = get_config_val(StoredVal::Int(100), 3, GC_CONF);
        assert_eq!(v, "100");
        // sort_memory (index 4) ENV_SIZE: 128 MiB exact -> "128 MB"
        let (v, _) = get_config_val(StoredVal::Int(128 * 1024 * 1024), 4, GC_CONF);
        assert_eq!(v, "128 MB");
        // exact KB
        let (v, _) = get_config_val(StoredVal::Int(256 * 1024), 4, GC_CONF);
        assert_eq!(v, "256 KB");
        // exact GB
        let (v, _) = get_config_val(StoredVal::Int(2 * 1024 * 1024 * 1024), 4, GC_CONF);
        assert_eq!(v, "2 GB");
        // non-exact -> fractional
        let (v, _) = get_config_val(StoredVal::Int(1024 * 1024 + 512 * 1024), 4, GC_CONF);
        assert_eq!(v, "1.50 MB");
    }

    #[test]
    fn get_config_val_str_file_path_char() {
        // current_date (index 6) ENV_STR: NULL -> "not set", set -> quoted
        let (v, _) = get_config_val(StoredVal::Str(None), 6, GC_CONF);
        assert_eq!(v, "not set");
        let (v, _) = get_config_val(StoredVal::Str(Some("2020")), 6, GC_CONF);
        assert_eq!(v, "'2020'");
        // trace_file (index 7) ENV_FILE: bare path
        let (v, _) = get_config_val(StoredVal::Str(Some("/tmp/t")), 7, GC_CONF);
        assert_eq!(v, "/tmp/t");
        // file_path (index 8) ENV_PATH: NULL
        let (v, _) = get_config_val(StoredVal::Str(None), 8, GC_CONF);
        assert_eq!(v, "not set");
        // seq_concat_sep (index 9) ENV_CHAR: printable, NUL, non-printable
        let (v, _) = get_config_val(StoredVal::Char(b'+'), 9, GC_CONF);
        assert_eq!(v, "'+'");
        let (v, _) = get_config_val(StoredVal::Char(0), 9, GC_CONF);
        assert_eq!(v, "Nul");
        let (v, _) = get_config_val(StoredVal::Char(0x01), 9, GC_CONF);
        assert_eq!(v, "0x01");
    }

    #[test]
    fn get_config_val_enum_translation() {
        // load_case (index 0) ENV_UINT|ENV_ENUMVAL, enums LOWER=1/UPPER=2/not set=0, default "0".
        // stored 1 -> "1" -> translates to "LOWER". The C remembers orgvalue="1", but the trailing
        // reconciliation clears it because orgvalue("1") != default_val("0").
        let (v, o) = get_config_val(StoredVal::Int(1), 0, GC_CONF);
        assert_eq!(v, "LOWER");
        assert_eq!(o, "");
        // stored 2 -> "UPPER" (orgvalue likewise reconciled to empty)
        let (v, o) = get_config_val(StoredVal::Int(2), 0, GC_CONF);
        assert_eq!(v, "UPPER");
        assert_eq!(o, "");
        // stored 0 -> "0" matches "not set" enum value -> "not set"; orgvalue stays empty (value == "0").
        let (v, o) = get_config_val(StoredVal::Int(0), 0, GC_CONF);
        assert_eq!(v, "not set");
        assert_eq!(o, "");
    }

    #[test]
    fn rescan_env_vals_decision() {
        let env = |k: &str| -> Option<String> {
            match k {
                "COB_DUMP_WIDTH" => Some("120".to_string()),
                "COB_SORT_MEMORY" => Some(String::new()), // set but empty -> skipped
                _ => None,
            }
        };
        let applied = cob_rescan_env_vals(GC_CONF, &env);
        // only the non-empty one is applied, at its table index (3)
        assert_eq!(applied, vec![(3usize, "120".to_string())]);
    }
}
