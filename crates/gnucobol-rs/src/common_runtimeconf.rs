//! Native-Rust port of libcob `print_runtime_conf` (common.c:9762) -- the `cobcrun --runtime-config`
//! report. Reproduces, byte-for-byte, the header, the grouped `gc_conf` settings (CALL / File I/O /
//! Screen I/O / Miscellaneous), and the System-configuration block, exactly as the admitted GnuCOBOL
//! 3.2 oracle prints them under a clean environment.
//!
//! ## What lives where
//! The settings table ([`GC_CONF_DISPLAY`]) and the value renderer ([`render_value`]) are a faithful
//! port of `gc_conf[]` (common.c:469) and `get_config_val` (common.c:8134): the boolean / unsigned /
//! size / string / file / path / char rendering, the enum back-translation, and -- crucially -- the
//! trailing block (common.c:8287) that echoes the *raw default string* for a non-enum, non-boolean
//! field (so `COB_SORT_CHUNK` shows `256K`, not the numeric `256 KB`).
//!
//! The System block (USERNAME / LANG / OSTYPE / TERM, LOCALEDIR, the `LC_*` locales) and the main
//! config-file path are *host* state the `#![forbid(unsafe_code)]` core cannot read itself (they need
//! `getenv` / `setlocale` / `getlogin`). The caller supplies them via [`SystemConf`]; the binary
//! (`cobrun --runtime-config`) fills it from the process environment, reproducing libcob's locale
//! initialisation (`cob_init`, common.c:10096 -- `setlocale(LC_ALL,"")` then force `LC_NUMERIC`/
//! `LC_CTYPE` to `"C"`) and its username resolution (`USERNAME` env, else `LOGNAME`, else `getlogin()`).

use crate::common_configload::{
    cb_lookup_config, cob_expand_env_string, get_config_val, StoredVal, GC_CONF,
};
use crate::common_misc::translate_boolean_to_int;
use std::collections::BTreeMap;

/// `gc_conf[].env_group` -- the section a setting prints under, in `print_runtime_conf` order.
mod grp {
    pub const CALL: u8 = 1;
    pub const FILE: u8 = 2;
    pub const SCREEN: u8 = 3;
    pub const MISC: u8 = 4;
    pub const SYSENV: u8 = 5;
    /// `GRP_MAX` -- the loop runs groups `1 .. MAX`.
    pub const MAX: u8 = 6;
}

/// A field's storage/rendering category (the `ENV_*` type bits of `gc_conf`, reduced to what the
/// `--runtime-config` renderer needs).
#[derive(Clone, Copy, PartialEq)]
enum Kind {
    /// `ENV_BOOL` -- rendered `yes`/`no` from the stored integer sense of the default.
    Bool,
    /// `ENV_UINT` / `ENV_SINT` -- the integer, decimal.
    Uint,
    /// `ENV_SIZE` -- integer with `K`/`M`/`G` (only via the raw-default echo in the clean report).
    Size,
    /// `ENV_STR` -- `'quoted'` when runtime-set, else the raw default / `not set`.
    Str,
    /// `ENV_FILE` / `ENV_PATH` -- the path verbatim, else `not set`.
    Path,
    /// `ENV_CHAR` -- a single character (echoed raw from the default).
    Char,
}
use Kind::*;

/// One displayed `gc_conf[]` row (hidden `GRP_HIDE` aliases and platform-`#ifdef`-excluded rows omitted).
struct ConfRow {
    /// `env_name` -- the environment-variable spelling (the column label).
    env: &'static str,
    /// `conf_name` -- the runtime.cfg keyword (only affects `hdlen`).
    conf: &'static str,
    /// `default_val` -- the compiled default, or `None` for a NULL default.
    default: Option<&'static str>,
    /// `env_group` -- the section.
    group: u8,
    /// the storage/render category.
    kind: Kind,
    /// `enums` -- the value back-translation table (`(match, value)` pairs), or `None`.
    enums: Option<&'static [(&'static str, &'static str)]>,
}

// --- enum tables (common.c:448-457) ---------------------------------------------------------------
const LWRUPR: &[(&str, &str)] = &[("LOWER", "1"), ("UPPER", "2"), ("not set", "0")];
const NEVER: &[(&str, &str)] = &[("never", "!")];
const BEEPOPTS: &[(&str, &str)] = &[("FLASH", "1"), ("SPEAKER", "2"), ("FALSE", "9"), ("BEEP", "0")];
const TIMEOPTS: &[(&str, &str)] = &[("0", "1000"), ("1", "100"), ("2", "10"), ("3", "1")];
const SYNCOPTS: &[(&str, &str)] = &[("P", "1")];
const VARSEQOPTS: &[(&str, &str)] = &[("0", "0"), ("1", "1"), ("2", "2"), ("3", "3")];
const COEOPTS: &[(&str, &str)] = &[("0", "0"), ("1", "1"), ("2", "2"), ("3", "3")];

/// The displayed `gc_conf[]` rows in table order (common.c:469-541), minus `GRP_HIDE` aliases and
/// rows excluded by this build's `#ifdef`s (`_WIN32`-only `COB_UNIX_LF`/`OS`). `COB_MOUSE_INTERVAL`
/// (`HAVE_MOUSEINTERVAL`) and `DB_HOME` (`WITH_DB`) are present, matching the admitted oracle.
const GC_CONF_DISPLAY: &[ConfRow] = &[
    ConfRow { env: "COB_LOAD_CASE", conf: "load_case", default: Some("0"), group: grp::CALL, kind: Uint, enums: Some(LWRUPR) },
    ConfRow { env: "COB_PHYSICAL_CANCEL", conf: "physical_cancel", default: Some("0"), group: grp::CALL, kind: Bool, enums: Some(NEVER) },
    ConfRow { env: "COB_LIBRARY_PATH", conf: "library_path", default: None, group: grp::CALL, kind: Path, enums: None },
    ConfRow { env: "COB_PRE_LOAD", conf: "pre_load", default: None, group: grp::CALL, kind: Str, enums: None },
    ConfRow { env: "COB_BELL", conf: "bell", default: Some("0"), group: grp::SCREEN, kind: Uint, enums: Some(BEEPOPTS) },
    ConfRow { env: "COB_DISABLE_WARNINGS", conf: "disable_warnings", default: Some("0"), group: grp::MISC, kind: Bool, enums: None },
    ConfRow { env: "COB_ENV_MANGLE", conf: "env_mangle", default: Some("0"), group: grp::MISC, kind: Bool, enums: None },
    ConfRow { env: "COB_REDIRECT_DISPLAY", conf: "redirect_display", default: Some("0"), group: grp::SCREEN, kind: Bool, enums: None },
    ConfRow { env: "COB_SCREEN_ESC", conf: "screen_esc", default: Some("0"), group: grp::SCREEN, kind: Bool, enums: None },
    ConfRow { env: "COB_SCREEN_EXCEPTIONS", conf: "screen_exceptions", default: Some("0"), group: grp::SCREEN, kind: Bool, enums: None },
    ConfRow { env: "COB_TIMEOUT_SCALE", conf: "timeout_scale", default: Some("0"), group: grp::SCREEN, kind: Uint, enums: Some(TIMEOPTS) },
    ConfRow { env: "COB_INSERT_MODE", conf: "insert_mode", default: Some("0"), group: grp::SCREEN, kind: Bool, enums: None },
    ConfRow { env: "COB_MOUSE_FLAGS", conf: "mouse_flags", default: Some("1"), group: grp::SCREEN, kind: Uint, enums: None },
    ConfRow { env: "COB_MOUSE_INTERVAL", conf: "mouse_interval", default: Some("100"), group: grp::SCREEN, kind: Uint, enums: None },
    ConfRow { env: "COB_SET_DEBUG", conf: "debugging_mode", default: Some("0"), group: grp::MISC, kind: Bool, enums: None },
    ConfRow { env: "COB_SET_TRACE", conf: "set_trace", default: Some("0"), group: grp::MISC, kind: Bool, enums: None },
    ConfRow { env: "COB_TRACE_FILE", conf: "trace_file", default: None, group: grp::MISC, kind: Path, enums: None },
    ConfRow { env: "COB_TRACE_FORMAT", conf: "trace_format", default: Some("%P %S Line: %L"), group: grp::MISC, kind: Str, enums: None },
    ConfRow { env: "COB_STACKTRACE", conf: "stacktrace", default: Some("1"), group: grp::MISC, kind: Bool, enums: None },
    ConfRow { env: "COB_CORE_ON_ERROR", conf: "core_on_error", default: Some("0"), group: grp::MISC, kind: Uint, enums: Some(COEOPTS) },
    ConfRow { env: "COB_CORE_FILENAME", conf: "core_filename", default: Some("./core.libcob"), group: grp::MISC, kind: Str, enums: None },
    ConfRow { env: "COB_DUMP_FILE", conf: "dump_file", default: None, group: grp::MISC, kind: Path, enums: None },
    ConfRow { env: "COB_DUMP_WIDTH", conf: "dump_width", default: Some("100"), group: grp::MISC, kind: Uint, enums: None },
    // System (env-driven; default NULL -- value supplied by SystemConf)
    ConfRow { env: "USERNAME", conf: "username", default: None, group: grp::SYSENV, kind: Str, enums: None },
    ConfRow { env: "LANG", conf: "lang", default: None, group: grp::SYSENV, kind: Str, enums: None },
    ConfRow { env: "OSTYPE", conf: "ostype", default: None, group: grp::SYSENV, kind: Str, enums: None },
    ConfRow { env: "TERM", conf: "term", default: None, group: grp::SYSENV, kind: Str, enums: None },
    ConfRow { env: "COB_FILE_PATH", conf: "file_path", default: None, group: grp::FILE, kind: Path, enums: None },
    ConfRow { env: "COB_VARSEQ_FORMAT", conf: "varseq_format", default: Some("0"), group: grp::FILE, kind: Uint, enums: Some(VARSEQOPTS) },
    ConfRow { env: "COB_LS_FIXED", conf: "ls_fixed", default: Some("0"), group: grp::FILE, kind: Bool, enums: None },
    ConfRow { env: "COB_LS_VALIDATE", conf: "ls_validate", default: Some("1"), group: grp::FILE, kind: Bool, enums: None },
    ConfRow { env: "COB_LS_NULLS", conf: "ls_nulls", default: Some("0"), group: grp::FILE, kind: Bool, enums: None },
    ConfRow { env: "COB_LS_SPLIT", conf: "ls_split", default: Some("1"), group: grp::FILE, kind: Bool, enums: None },
    ConfRow { env: "COB_SEQ_CONCAT_NAME", conf: "seq_concat_name", default: Some("0"), group: grp::FILE, kind: Bool, enums: None },
    ConfRow { env: "COB_SEQ_CONCAT_SEP", conf: "seq_concat_sep", default: Some("+"), group: grp::FILE, kind: Char, enums: None },
    ConfRow { env: "COB_SORT_CHUNK", conf: "sort_chunk", default: Some("256K"), group: grp::FILE, kind: Size, enums: None },
    ConfRow { env: "COB_SORT_MEMORY", conf: "sort_memory", default: Some("128M"), group: grp::FILE, kind: Size, enums: None },
    ConfRow { env: "COB_SYNC", conf: "sync", default: Some("0"), group: grp::FILE, kind: Bool, enums: Some(SYNCOPTS) },
    ConfRow { env: "DB_HOME", conf: "db_home", default: None, group: grp::FILE, kind: Path, enums: None },
    ConfRow { env: "COB_COL_JUST_LRC", conf: "col_just_lrc", default: Some("true"), group: grp::FILE, kind: Bool, enums: None },
    ConfRow { env: "COB_DISPLAY_PRINT_PIPE", conf: "display_print_pipe", default: None, group: grp::SCREEN, kind: Str, enums: None },
    ConfRow { env: "COB_DISPLAY_PRINT_FILE", conf: "display_print_file", default: None, group: grp::SCREEN, kind: Str, enums: None },
    ConfRow { env: "COB_DISPLAY_PUNCH_FILE", conf: "display_punch_file", default: None, group: grp::SCREEN, kind: Str, enums: None },
    ConfRow { env: "COB_LEGACY", conf: "legacy", default: None, group: grp::SCREEN, kind: Bool, enums: None },
    ConfRow { env: "COB_EXIT_WAIT", conf: "exit_wait", default: Some("1"), group: grp::SCREEN, kind: Bool, enums: None },
    // default set in cob_init_screenio() -- supplied as the runtime default below.
    ConfRow { env: "COB_EXIT_MSG", conf: "exit_msg", default: None, group: grp::SCREEN, kind: Str, enums: None },
    ConfRow { env: "COB_CURRENT_DATE", conf: "current_date", default: None, group: grp::MISC, kind: Str, enums: None },
];

/// `cob_init_screenio()` sets the `COB_EXIT_MSG` default to this (common: the press-a-key prompt).
const EXIT_MSG_DEFAULT: &str = "end of program, please press a key to exit";

/// The section headings (`setting_group[]`, common.c:444), indexed by group number.
const SETTING_GROUP: [&str; 6] = [
    " hidden setting ",
    "CALL configuration",
    "File I/O configuration",
    "Screen I/O configuration",
    "Miscellaneous",
    "System configuration",
];

/// How the username was resolved -- drives the `(set by ...)` suffix (common.c:9908).
pub enum UserOrigin {
    /// `USERNAME` env var: shown `env`, no `(set by)`.
    Username,
    /// `LOGNAME` env var (a hidden alias): `(set by LOGNAME)`.
    Logname,
    /// `getlogin()`: `(set by getlogin())` (the `FUNC_NAME_IN_DEFAULT` path).
    Getlogin,
}

/// Host state the core cannot read under `#![forbid(unsafe_code)]` (env vars + resolved locales),
/// supplied by the binary so [`print_runtime_conf`] can render the full report.
pub struct SystemConf {
    /// The main configuration file path (`cob_config_file[0]`) -- `COB_CONFIG_DIR`/`runtime.cfg`.
    pub config_file: String,
    /// The resolved user name and how it was found, or `None` if unresolved.
    pub username: Option<(String, UserOrigin)>,
    /// `$LANG`, if set.
    pub lang: Option<String>,
    /// `$OSTYPE`, if set.
    pub ostype: Option<String>,
    /// `$TERM`, if set.
    pub term: Option<String>,
    /// `LOCALEDIR` -- `$LOCALEDIR`, else the compiled locale directory.
    pub localedir: String,
    /// `setlocale(LC_CTYPE, NULL)` after init (libcob forces `"C"`).
    pub lc_ctype: String,
    /// `setlocale(LC_NUMERIC, NULL)` after init (libcob forces `"C"`).
    pub lc_numeric: String,
    /// `setlocale(LC_COLLATE, NULL)`.
    pub lc_collate: String,
    /// `setlocale(LC_MESSAGES, NULL)`.
    pub lc_messages: String,
    /// `setlocale(LC_MONETARY, NULL)`.
    pub lc_monetary: String,
    /// `setlocale(LC_TIME, NULL)`.
    pub lc_time: String,
}

/// `not set` -- the sentinel string and the basis for `min_conf_length` (common.c:8149).
const NOT_SET: &str = "not set";

/// Render a row's effective value in the *clean* environment (no `COB_*` overrides), porting the
/// default path of `get_config_val` (common.c:8134): boolean sense, enum back-translation, and the
/// raw-default echo for a non-enum, non-boolean field. Returns the displayed value string.
fn render_value(row: &ConfRow) -> String {
    // ENV_BOOL: yes/no from the stored integer sense of the default (ENV_NOT cancels across set+get,
    // so the default's plain sense is what shows). The enum (e.g. `never`) never matches yes/no here.
    if row.kind == Bool {
        let n = match row.default {
            Some(d) => translate_boolean_to_int(Some(d.as_bytes())),
            None => 0,
        };
        let mut v = if n == 1 { "yes".to_string() } else { "no".to_string() };
        v = enum_translate(row, v);
        return v;
    }
    // Non-boolean with a default present.
    if let Some(def) = row.default {
        if let Some(_enums) = row.enums {
            // ENV_*|ENV_ENUM[VAL]: render the integer first, then back-translate via the enum table.
            let base = match row.kind {
                Uint | Size => def.to_string(), // defaults here are plain integers ("0", ...)
                _ => def.to_string(),
            };
            return enum_translate(row, base);
        }
        // No enum, not boolean -> the trailing block (common.c:8287) echoes the RAW default string
        // (so COB_SORT_CHUNK shows "256K", COB_TRACE_FORMAT shows "%P %S Line: %L" unquoted, etc.).
        return def.to_string();
    }
    // NULL default -> "not set" (all NULL-default non-boolean rows here are Str/Path/Char).
    NOT_SET.to_string()
}

/// Apply the enum back-translation (common.c:8270): if the rendered `value` case-insensitively equals
/// an enum entry's `value`, replace it with that entry's `match` (`"not set"` translated as such).
fn enum_translate(row: &ConfRow, value: String) -> String {
    if let Some(enums) = row.enums {
        for (m, v) in enums {
            if value.eq_ignore_ascii_case(v) {
                return if *m == "not set" { NOT_SET.to_string() } else { (*m).to_string() };
            }
        }
    }
    value
}

/// Append one settings line: prefix (`    : ` / ` env: `) + padded name + value + padding + origin.
/// Ports the per-row body of `print_runtime_conf` (common.c:9862-9929). The two trailing decorations
/// are independent, exactly as in the C: the `(set by X)` block (`set_by != 0`) and the default block
/// (only when the row is *not* env-/config-/function-set). `env_prefix` selects ` env` vs `    `; the
/// function-set (`getlogin()`) prefix is byte-identical to the default `    `, so it passes `false`.
fn push_line(out: &mut Vec<u8>, hdlen: usize, min_conf: usize, env_name: &str, value: &str, env_prefix: bool, set_by: Option<&str>, run_default: bool) {
    // Prefix: env-set rows show " env", otherwise four spaces (default and function-set alike).
    if env_prefix {
        out.extend_from_slice(b" env");
    } else {
        out.extend_from_slice(b"    ");
    }
    // ": %-*s : " -- name left-justified to hdlen.
    out.extend_from_slice(b": ");
    out.extend_from_slice(env_name.as_bytes());
    for _ in env_name.len()..hdlen {
        out.push(b' ');
    }
    out.extend_from_slice(b" : ");
    // value + plen2 padding (min_conf - vl, or 1 when equal, else 0).
    out.extend_from_slice(value.as_bytes());
    let vl = value.len();
    let plen2 = if vl < min_conf {
        min_conf - vl
    } else if vl == min_conf {
        1
    } else {
        0
    };
    for _ in 0..plen2 {
        out.push(b' ');
    }
    // "(set by X)" -- emitted whenever an origin is known (alias env var or getlogin()).
    if let Some(by) = set_by {
        out.push(b' ');
        out.extend_from_slice(format!("(set by {by})").as_bytes());
    }
    // Default block -- only for rows not set by env/config/function; "(default)" unless "not set".
    if run_default {
        out.push(b' ');
        if value != NOT_SET {
            out.extend_from_slice(b"(default)");
        }
    }
    out.push(b'\n');
}

/// Render the full `cobcrun --runtime-config` report, byte-for-byte, into a buffer.
pub fn print_runtime_conf(sys: &SystemConf) -> Vec<u8> {
    let mut out = Vec::new();

    // Header: "GnuCOBOL <ver> runtime configuration\n via  <config>\n\n" (common.c:9779-9810).
    let ver = format!(
        "{}.{}.{}",
        crate::common::LIBCOB_VERSION,
        crate::common::LIBCOB_VERSION_MINOR,
        crate::common::LIBCOB_VERSION_PATCHLEVEL
    );
    out.extend_from_slice(format!("GnuCOBOL {ver} runtime configuration\n").as_bytes());
    out.extend_from_slice(b" via  ");
    out.extend_from_slice(sys.config_file.as_bytes());
    out.push(b'\n');
    out.push(b'\n');

    // hdlen = max(15, longest env_name/conf_name) (common.c:9811). min_conf_length = strlen("not set")
    // + 1, clamped to [6,15] (common.c:8149) -> 8.
    let mut hdlen = 15usize;
    for r in GC_CONF_DISPLAY {
        hdlen = hdlen.max(r.env.len()).max(r.conf.len());
    }
    let min_conf = {
        let m = NOT_SET.len() + 1;
        m.clamp(6, 15)
    };

    // Grouped settings, groups 1..MAX, rows in table order within each group (common.c:9824).
    for j in 1..grp::MAX {
        let mut dohdg = true;
        for row in GC_CONF_DISPLAY {
            if row.group != j {
                continue;
            }
            if dohdg {
                dohdg = false;
                if j > 1 {
                    out.push(b'\n');
                }
                out.extend_from_slice(format!(" {}\n", SETTING_GROUP[j as usize]).as_bytes());
            }
            // Resolve the displayed value + origin.
            match row.env {
                "USERNAME" => match &sys.username {
                    // Env-set (USERNAME/LOGNAME): " env" prefix; LOGNAME is an alias -> "(set by LOGNAME)".
                    // getlogin() is function-set: default "    " prefix, "(set by getlogin())".
                    Some((name, origin)) => {
                        let (env_prefix, set_by) = match origin {
                            UserOrigin::Username => (true, None),
                            UserOrigin::Logname => (true, Some("LOGNAME")),
                            UserOrigin::Getlogin => (false, Some("getlogin()")),
                        };
                        push_line(&mut out, hdlen, min_conf, row.env, &format!("'{name}'"), env_prefix, set_by, false);
                    }
                    None => push_line(&mut out, hdlen, min_conf, row.env, NOT_SET, false, None, true),
                },
                "LANG" | "OSTYPE" | "TERM" => {
                    let envval = match row.env {
                        "LANG" => &sys.lang,
                        "OSTYPE" => &sys.ostype,
                        _ => &sys.term,
                    };
                    match envval {
                        Some(v) => push_line(&mut out, hdlen, min_conf, row.env, &format!("'{v}'"), true, None, false),
                        None => push_line(&mut out, hdlen, min_conf, row.env, NOT_SET, false, None, true),
                    }
                }
                "COB_EXIT_MSG" => {
                    // Runtime default from cob_init_screenio(): a quoted string, shown "(default)".
                    push_line(&mut out, hdlen, min_conf, row.env, &format!("'{EXIT_MSG_DEFAULT}'"), false, None, true);
                }
                _ => {
                    let value = render_value(row);
                    push_line(&mut out, hdlen, min_conf, row.env, &value, false, None, true);
                }
            }
        }
    }

    // System block tail: LOCALEDIR + the LC_* locales (common.c:9938-9949). Plain "    : name : val".
    let tail = |out: &mut Vec<u8>, name: &str, val: &str| {
        out.extend_from_slice(b"    : ");
        out.extend_from_slice(name.as_bytes());
        for _ in name.len()..hdlen {
            out.push(b' ');
        }
        out.extend_from_slice(b" : ");
        out.extend_from_slice(val.as_bytes());
        out.push(b'\n');
    };
    tail(&mut out, "LOCALEDIR", &sys.localedir);
    tail(&mut out, "LC_CTYPE", &sys.lc_ctype);
    tail(&mut out, "LC_NUMERIC", &sys.lc_numeric);
    tail(&mut out, "LC_COLLATE", &sys.lc_collate);
    tail(&mut out, "LC_MESSAGES", &sys.lc_messages);
    tail(&mut out, "LC_MONETARY", &sys.lc_monetary);
    tail(&mut out, "LC_TIME", &sys.lc_time);

    out
}

/// The overlay for a RESOLVED `--runtime-conf` report (the cobcrun `-c <cfg>` / env surface):
/// config-file values by `conf_name`, env overrides by `env_name` (environment priority), and the
/// config file the report's `via` line should name (the base report keeps the compiled default).
pub struct ConfOverlay<'a> {
    pub config_file: Option<&'a str>,
    /// `conf_name` -> raw config-file value (already applied in file order; last wins).
    pub applied: &'a BTreeMap<String, Vec<u8>>,
    /// `env_name` -> value (process env or `setenv`-directive overrides; wins over config).
    pub env: &'a BTreeMap<String, String>,
}

/// Render one applied/env value in libcob's `get_config_val` shape (bool yes/no, enum back-
/// translation, size with K/M/G, quoted STR, bare FILE/PATH, `${...}` expansion for strings).
fn render_resolved_value(row: &ConfRow, raw: &str) -> String {
    let pos = cb_lookup_config(row.conf, GC_CONF);
    if pos < GC_CONF.len() {
        let e = &GC_CONF[pos];
        let stored = match row.kind {
            Bool => {
                let lv = raw.to_ascii_lowercase();
                let n = if matches!(lv.as_str(), "true" | "yes" | "on" | "1") { 1 } else { 0 };
                StoredVal::Int(n)
            }
            Uint | Size => {
                let n = parse_size_i64(raw).unwrap_or(0);
                StoredVal::Int(n)
            }
            Char => StoredVal::Char(raw.bytes().next().unwrap_or(0)),
            Str | Path => StoredVal::Str(Some(raw)),
        };
        let (mut value, _org) = get_config_val(stored, pos, GC_CONF);
        if row.kind == Str || row.kind == Path {
            // libcob expands `${...}` in string values when rendering the report
            let expanded = cob_expand_env_string(
                value.as_bytes(),
                &|name: &str| std::env::var(name).ok(),
                std::process::id() as i32,
                "",
                "",
            );
            value = String::from_utf8_lossy(&expanded).into_owned();
            if row.kind == Str {
                value = format!("'{value}'");
            }
        }
        let _ = e;
        value
    } else {
        // row not in the (representative) config table: a faithful fallback for the report
        match row.kind {
            Bool => {
                let lv = raw.to_ascii_lowercase();
                if matches!(lv.as_str(), "true" | "yes" | "on" | "1") {
                    "yes".to_string()
                } else {
                    "no".to_string()
                }
            }
            Char => raw.to_string(),
            Str => format!("'{}'", expand_env_plain(raw)),
            Path => expand_env_plain(raw),
            _ => raw.to_string(),
        }
    }
}

fn expand_env_plain(raw: &str) -> String {
    let expanded = cob_expand_env_string(
        raw.as_bytes(),
        &|name: &str| std::env::var(name).ok(),
        std::process::id() as i32,
        "",
        "",
    );
    String::from_utf8_lossy(&expanded).into_owned()
}

fn parse_size_i64(v: &str) -> Option<i64> {
    let v = v.trim();
    let (num, mult) = if let Some(rest) = v.strip_suffix(['K', 'k']) {
        (rest, 1024i64)
    } else if let Some(rest) = v.strip_suffix(['M', 'm']) {
        (rest, 1024i64 * 1024)
    } else if let Some(rest) = v.strip_suffix(['G', 'g']) {
        (rest, 1024i64 * 1024 * 1024)
    } else {
        (v, 1i64)
    };
    num.trim().parse::<i64>().ok().map(|n| n * mult)
}

/// The RESOLVED `cobcrun --runtime-conf` report: like [`print_runtime_conf`], but the `via` line and
/// the per-row values reflect the loaded config file (`-c <cfg>` / `COB_RUNTIME_CONFIG`), applied
/// config values, env overrides and `${...}` expansion. With an empty overlay this is byte-identical
/// to [`print_runtime_conf`].
pub fn print_runtime_conf_resolved(sys: &SystemConf, overlay: &ConfOverlay) -> Vec<u8> {
    let mut out = Vec::new();
    let ver = format!(
        "{}.{}.{}",
        crate::common::LIBCOB_VERSION,
        crate::common::LIBCOB_VERSION_MINOR,
        crate::common::LIBCOB_VERSION_PATCHLEVEL
    );
    out.extend_from_slice(format!("GnuCOBOL {ver} runtime configuration\n").as_bytes());
    out.extend_from_slice(b" via  ");
    match overlay.config_file {
        Some(f) => out.extend_from_slice(f.as_bytes()),
        None => out.extend_from_slice(sys.config_file.as_bytes()),
    }
    out.push(b'\n');
    out.push(b'\n');

    let mut hdlen = 15usize;
    for r in GC_CONF_DISPLAY {
        hdlen = hdlen.max(r.env.len()).max(r.conf.len());
    }
    let min_conf = {
        let m = NOT_SET.len() + 1;
        m.clamp(6, 15)
    };

    for j in 1..grp::MAX {
        let mut dohdg = true;
        for row in GC_CONF_DISPLAY {
            if row.group != j {
                continue;
            }
            if dohdg {
                dohdg = false;
                if j > 1 {
                    out.push(b'\n');
                }
                out.extend_from_slice(format!(" {}\n", SETTING_GROUP[j as usize]).as_bytes());
            }
            // env override wins (environment priority), then the config-file value, then the default.
            let env_val = overlay.env.get(row.env).cloned();
            let app_val = overlay.applied.get(row.conf).cloned();
            match row.env {
                "USERNAME" | "LANG" | "OSTYPE" | "TERM" if env_val.is_some() => {
                    let v = env_val.as_deref().unwrap_or("");
                    push_line(&mut out, hdlen, min_conf, row.env, &format!("'{v}'"), true, None, false);
                }
                _ if env_val.is_some() => {
                    let v = env_val.as_deref().unwrap_or("");
                    let value = render_resolved_value(row, v);
                    push_line(&mut out, hdlen, min_conf, row.env, &value, true, None, false);
                }
                _ if app_val.is_some() => {
                    let v = String::from_utf8_lossy(app_val.as_deref().unwrap_or(&[])).into_owned();
                    let value = render_resolved_value(row, &v);
                    push_line(&mut out, hdlen, min_conf, row.env, &value, false, None, false);
                }
                "USERNAME" => match &sys.username {
                    Some((name, origin)) => {
                        let (env_prefix, set_by) = match origin {
                            UserOrigin::Username => (true, None),
                            UserOrigin::Logname => (true, Some("LOGNAME")),
                            UserOrigin::Getlogin => (false, Some("getlogin()")),
                        };
                        push_line(&mut out, hdlen, min_conf, row.env, &format!("'{name}'"), env_prefix, set_by, false);
                    }
                    None => push_line(&mut out, hdlen, min_conf, row.env, NOT_SET, false, None, true),
                },
                "LANG" | "OSTYPE" | "TERM" => {
                    let envval = match row.env {
                        "LANG" => &sys.lang,
                        "OSTYPE" => &sys.ostype,
                        _ => &sys.term,
                    };
                    match envval {
                        Some(v) => push_line(&mut out, hdlen, min_conf, row.env, &format!("'{v}'"), true, None, false),
                        None => push_line(&mut out, hdlen, min_conf, row.env, NOT_SET, false, None, true),
                    }
                }
                "COB_EXIT_MSG" => {
                    push_line(&mut out, hdlen, min_conf, row.env, &format!("'{EXIT_MSG_DEFAULT}'"), false, None, true);
                }
                _ => {
                    let value = render_value(row);
                    push_line(&mut out, hdlen, min_conf, row.env, &value, false, None, true);
                }
            }
        }
    }

    let tail = |out: &mut Vec<u8>, name: &str, val: &str| {
        out.extend_from_slice(b"    : ");
        out.extend_from_slice(name.as_bytes());
        for _ in name.len()..hdlen {
            out.push(b' ');
        }
        out.extend_from_slice(b" : ");
        out.extend_from_slice(val.as_bytes());
        out.push(b'\n');
    };
    tail(&mut out, "LOCALEDIR", &sys.localedir);
    tail(&mut out, "LC_CTYPE", &sys.lc_ctype);
    tail(&mut out, "LC_NUMERIC", &sys.lc_numeric);
    tail(&mut out, "LC_COLLATE", &sys.lc_collate);
    tail(&mut out, "LC_MESSAGES", &sys.lc_messages);
    tail(&mut out, "LC_MONETARY", &sys.lc_monetary);
    tail(&mut out, "LC_TIME", &sys.lc_time);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A clean-environment `SystemConf` matching the pinned oracle sweep env.
    fn clean_sys() -> SystemConf {
        SystemConf {
            config_file: "/p/share/gnucobol/config/runtime.cfg".to_string(),
            username: Some(("one".to_string(), UserOrigin::Logname)),
            lang: Some("en_GB.UTF-8".to_string()),
            ostype: None,
            term: None,
            localedir: "/p/share/locale".to_string(),
            lc_ctype: "C".to_string(),
            lc_numeric: "C".to_string(),
            lc_collate: "C.UTF-8".to_string(),
            lc_messages: "C.UTF-8".to_string(),
            lc_monetary: "C.UTF-8".to_string(),
            lc_time: "C.UTF-8".to_string(),
        }
    }

    #[test]
    fn header_and_via_line() {
        let s = String::from_utf8(print_runtime_conf(&clean_sys())).unwrap();
        assert!(s.starts_with("GnuCOBOL 3.2.0 runtime configuration\n via  /p/share/gnucobol/config/runtime.cfg\n\n"));
    }

    #[test]
    fn raw_default_echo_not_numeric_size() {
        // The trailing-block rule: a non-enum, non-bool field echoes its RAW default ("256K", not "256 KB").
        let s = String::from_utf8(print_runtime_conf(&clean_sys())).unwrap();
        assert!(s.contains(": COB_SORT_CHUNK         : 256K     (default)\n"), "{s}");
        assert!(s.contains(": COB_SORT_MEMORY        : 128M     (default)\n"));
        // Strings echo unquoted from the default; runtime-default COB_EXIT_MSG is quoted.
        assert!(s.contains(": COB_TRACE_FORMAT       : %P %S Line: %L (default)\n"));
        assert!(s.contains(": COB_EXIT_MSG           : 'end of program, please press a key to exit' (default)\n"));
    }

    #[test]
    fn enum_and_bool_rendering() {
        let s = String::from_utf8(print_runtime_conf(&clean_sys())).unwrap();
        assert!(s.contains(": COB_LOAD_CASE          : not set  \n")); // enum 0 -> "not set", no "(default)"
        assert!(s.contains(": COB_BELL               : BEEP     (default)\n")); // enum 0 -> BEEP
        assert!(s.contains(": COB_PHYSICAL_CANCEL    : no       (default)\n")); // bool 0 -> no
        assert!(s.contains(": COB_COL_JUST_LRC       : yes      (default)\n")); // bool "true" -> yes
    }

    #[test]
    fn username_origins() {
        // LOGNAME -> env prefix + "(set by LOGNAME)".
        let s = String::from_utf8(print_runtime_conf(&clean_sys())).unwrap();
        assert!(s.contains(" env: USERNAME               : 'one'    (set by LOGNAME)\n"));
        // getlogin() -> default "    " prefix + "(set by getlogin())".
        let mut sys = clean_sys();
        sys.username = Some(("one".to_string(), UserOrigin::Getlogin));
        let s = String::from_utf8(print_runtime_conf(&sys)).unwrap();
        assert!(s.contains("    : USERNAME               : 'one'    (set by getlogin())\n"));
    }

    #[test]
    fn system_tail_locales() {
        let s = String::from_utf8(print_runtime_conf(&clean_sys())).unwrap();
        assert!(s.contains("    : LOCALEDIR              : /p/share/locale\n"));
        assert!(s.contains("    : LC_CTYPE               : C\n"));
        assert!(s.ends_with("    : LC_TIME                : C.UTF-8\n"));
    }
}
