//! cobcrun runtime-configuration surface (the GnuCOBOL suite's `configuration.at` tests):
//!
//! * `-c <cfg>` / `--config=<cfg>` / `COB_RUNTIME_CONFIG` select the runtime config file;
//! * the file is loaded with libcob-shaped error reporting: the `configuration error:` banner,
//!   `file:line` location prefixes, unknown-tag / invalid-value (enum, bool, size-bounds, no-`:`
//!   for file/path) diagnostics, `WARNING - '...' without a value - ignored!`, and recursive-include
//!   detection with the `... included here` chain;
//! * `--runtime-conf` reflects the loaded file (the `via` line), the applied values, env overrides
//!   (environment priority) and `${...}` env-string expansion.
//!
//! The parsing/validation primitives are the faithful ports in `gnucobol-rs::common_configload`
//! (`cb_config_entry`, `cb_lookup_config`, `cob_expand_env_string`, `GC_CONF`) and the error
//! composition in `gnucobol-rs::common_runerr` (`conf_runtime_error`, `conf_runtime_error_value`).
//! The file read / include recursion / cycle detection / value store are this binary's boundary.

use gnucobol_rs::common_configload::{self, ConfigDirective, ConfigEntryError, GC_CONF};
use gnucobol_rs::common_runerr::{
    conf_runtime_error, conf_runtime_error_value, ConfErrorState, SourceLocation,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

/// Applied config values by `conf_name` (last assignment wins, like the C's in-order application).
fn store() -> &'static Mutex<BTreeMap<String, Vec<u8>>> {
    static STORE: OnceLock<Mutex<BTreeMap<String, Vec<u8>>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

/// The loaded config file (the `via` line of `--runtime-conf`).
fn config_file() -> &'static Mutex<Option<String>> {
    static CFG: OnceLock<Mutex<Option<String>>> = OnceLock::new();
    CFG.get_or_init(|| Mutex::new(None))
}

/// Per-run env overrides (`setenv`/`unsetenv` directives), consulted by env-string expansion.
fn env_overrides() -> &'static Mutex<BTreeMap<String, String>> {
    static ENV: OnceLock<Mutex<BTreeMap<String, String>>> = OnceLock::new();
    ENV.get_or_init(|| Mutex::new(BTreeMap::new()))
}

/// Normalize a `-c`/`--config=` value to a path (empty values are ignored).
pub fn config_path_from_value(value: &str) -> Option<String> {
    let v = value.trim();
    if v.is_empty() {
        None
    } else {
        Some(v.to_string())
    }
}

/// The config file `--runtime-conf` / a cobcrun run should load: an explicit `-c` wins, then
/// `COB_RUNTIME_CONFIG`, then `COB_CONFIG_DIR/runtime.cfg` when present, else the compiled default.
pub fn resolve_config_file(explicit: Option<&str>) -> Option<String> {
    if let Some(c) = explicit.and_then(config_path_from_value) {
        return Some(c);
    }
    if let Ok(v) = std::env::var("COB_RUNTIME_CONFIG") {
        if !v.is_empty() {
            return Some(v);
        }
    }
    if let Ok(dir) = std::env::var("COB_CONFIG_DIR") {
        let p = PathBuf::from(dir).join("runtime.cfg");
        if p.is_file() {
            return Some(p.to_string_lossy().into_owned());
        }
    }
    None
}

/// Load a runtime config file for the current process; on any configuration error prints the
/// libcob-shaped diagnostics and returns `Err(())` (the caller exits 1, matching cobcrun).
pub fn load_for_run(path: &str) -> Result<(), ()> {
    store().lock().unwrap().clear();
    env_overrides().lock().unwrap().clear();
    let mut conf_err = ConfErrorState::default();
    let p = PathBuf::from(path);
    let mut chain: Vec<(PathBuf, u32)> = Vec::new(); // (file, include-directive line)
    let ok = load_file(&p, &mut chain, &mut conf_err);
    if !ok {
        return Err(());
    }
    *config_file().lock().unwrap() = Some(path.to_string());
    Ok(())
}

/// The env lookup for expansion: process env, with `setenv`-directive overrides first.
fn getenv(name: &str) -> Option<String> {
    env_overrides()
        .lock()
        .unwrap()
        .get(name)
        .cloned()
        .or_else(|| std::env::var(name).ok())
}

/// Load one config file (recursively following `include`/`includeif`), detecting include cycles.
fn load_file(path: &Path, chain: &mut Vec<(PathBuf, u32)>, conf_err: &mut ConfErrorState) -> bool {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => {
            let out = conf_runtime_error(
                conf_err,
                true,
                &SourceLocation {
                    file: Some(path.to_string_lossy().into_owned().into_bytes()),
                    line: 0,
                },
                b"No such file or directory",
            );
            eprint!("{}", String::from_utf8_lossy(&out));
            return false;
        }
    };
    // recursive-include detection: the file is already being loaded
    if chain.iter().any(|(f, _)| f == path) {
        let out = conf_runtime_error(
            conf_err,
            true,
            &SourceLocation {
                file: Some(path.to_string_lossy().into_owned().into_bytes()),
                line: 0,
            },
            b"recursive inclusion",
        );
        eprint!("{}", String::from_utf8_lossy(&out));
        for (f, l) in chain.iter().rev() {
            let out = conf_runtime_error(
                conf_err,
                true,
                &SourceLocation {
                    file: Some(f.to_string_lossy().into_owned().into_bytes()),
                    line: *l,
                },
                b"configuration file was included here",
            );
            eprint!("{}", String::from_utf8_lossy(&out));
        }
        return false;
    }

    let lines: Vec<&[u8]> = bytes.split(|&b| b == b'\n').collect();
    let mut ok = true;
    for (idx, line) in lines.iter().enumerate() {
        if common_configload::classify_config_line(line) == common_configload::ConfigLineKind::Skip
        {
            continue;
        }
        let lineno = (idx + 1) as u32;
        let loc = || SourceLocation {
            file: Some(path.to_string_lossy().into_owned().into_bytes()),
            line: lineno,
        };
        let directive = common_configload::cb_config_entry(
            line,
            GC_CONF,
            &getenv,
            std::process::id() as i32,
            &std::env::var("COB_CONFIG_DIR").unwrap_or_default(),
            &std::env::var("COB_COPY_DIR").unwrap_or_default(),
        );
        match directive {
            ConfigDirective::Set { pos, value } => {
                if !apply_set(path, lineno, line, pos, &value, conf_err) {
                    ok = false;
                }
            }
            ConfigDirective::Reset { pos } => {
                if pos < GC_CONF.len() {
                    store().lock().unwrap().remove(GC_CONF[pos].conf_name);
                }
            }
            ConfigDirective::SetEnv { name, value } => {
                env_overrides().lock().unwrap().insert(
                    String::from_utf8_lossy(&name).into_owned(),
                    String::from_utf8_lossy(&value).into_owned(),
                );
            }
            ConfigDirective::UnsetEnv { name } => {
                env_overrides()
                    .lock()
                    .unwrap()
                    .remove(&String::from_utf8_lossy(&name).into_owned());
            }
            ConfigDirective::Include { file } => {
                chain.push((path.to_path_buf(), lineno));
                if let Some(inc) = resolve_include(&file) {
                    if !load_file(&inc, chain, conf_err) {
                        ok = false;
                    }
                } else {
                    // a missing include file is a configuration error (the C's fopen failure)
                    let out = conf_runtime_error(
                        conf_err,
                        true,
                        &SourceLocation {
                            file: Some(String::from_utf8_lossy(&file).into_owned().into_bytes()),
                            line: 0,
                        },
                        b"No such file or directory",
                    );
                    eprint!("{}", String::from_utf8_lossy(&out));
                    ok = false;
                }
                chain.pop();
            }
            ConfigDirective::IncludeIf { file } => {
                if let Some(inc) = resolve_include(&file) {
                    chain.push((path.to_path_buf(), lineno));
                    if !load_file(&inc, chain, conf_err) {
                        ok = false;
                    }
                    chain.pop();
                }
            }
            ConfigDirective::WarningIgnored { .. } => {
                // The C prints the WHOLE (trimmed) line: `WARNING - '<line>' without a value - ignored!`
                let raw = String::from_utf8_lossy(line).trim().to_string();
                let out = conf_runtime_error(
                    conf_err,
                    true,
                    &loc(),
                    format!("WARNING - '{raw}' without a value - ignored!").as_bytes(),
                );
                eprint!("{}", String::from_utf8_lossy(&out));
            }
            ConfigDirective::Error(ConfigEntryError::UnknownTag(kw)) => {
                let out = conf_runtime_error(
                    conf_err,
                    true,
                    &loc(),
                    format!(
                        "unknown configuration tag '{}'",
                        String::from_utf8_lossy(&kw)
                    )
                    .as_bytes(),
                );
                eprint!("{}", String::from_utf8_lossy(&out));
                ok = false;
            }
            ConfigDirective::Error(ConfigEntryError::IncludeWithoutValue(_)) => {
                let raw = String::from_utf8_lossy(line).trim().to_string();
                let out = conf_runtime_error(
                    conf_err,
                    true,
                    &loc(),
                    format!("{raw}: configuration directive without a value").as_bytes(),
                );
                eprint!("{}", String::from_utf8_lossy(&out));
                ok = false;
            }
        }
    }
    ok
}

/// Resolve an `include`/`includeif` file name like the C's `cob_load_config_file`: as given (cwd),
/// then `COB_CONFIG_DIR` (the suite's `configuration.at` tests include files that live in the
/// configured config directory, e.g. `runtime_empty.cfg`).
fn resolve_include(file: &[u8]) -> Option<PathBuf> {
    let name = String::from_utf8_lossy(file).into_owned();
    let p = PathBuf::from(&name);
    if p.is_file() {
        return Some(p);
    }
    if let Ok(dir) = std::env::var("COB_CONFIG_DIR") {
        let c = PathBuf::from(dir).join(&name);
        if c.is_file() {
            return Some(c);
        }
    }
    None
}

/// Validate + store one `Set` directive (libcob's `set_config_val` decision + errors).
fn apply_set(
    path: &Path,
    lineno: u32,
    line: &[u8],
    pos: usize,
    value: &[u8],
    conf_err: &mut ConfErrorState,
) -> bool {
    let loc = || SourceLocation {
        file: Some(path.to_string_lossy().into_owned().into_bytes()),
        line: lineno,
    };
    if pos >= GC_CONF.len() {
        return false;
    }
    let e = &GC_CONF[pos];
    let name = e.conf_name.as_bytes();
    let dt = e.data_type;
    let v = String::from_utf8_lossy(value).into_owned();
    let invalid = |conf_err: &mut ConfErrorState, body: &[u8]| {
        let first = conf_runtime_error_value(value, name);
        let out1 = conf_runtime_error(conf_err, false, &loc(), &first);
        eprint!("{}", String::from_utf8_lossy(&out1));
        let out2 = conf_runtime_error(conf_err, true, &loc(), body);
        eprint!("{}", String::from_utf8_lossy(&out2));
    };
    let _ = line;
    if (dt & common_configload::env::ENV_BOOL) != 0 {
        let lv = v.to_ascii_lowercase();
        let ok = matches!(
            lv.as_str(),
            "true" | "yes" | "on" | "1" | "false" | "no" | "off" | "0"
        );
        if !ok {
            invalid(
                conf_err,
                b"should be one of the following values: true, false",
            );
            return false;
        }
        store()
            .lock()
            .unwrap()
            .insert(e.conf_name.to_string(), value.to_vec());
        return true;
    }
    if (dt & (common_configload::env::ENV_UINT | common_configload::env::ENV_SINT)) != 0 {
        if (dt & common_configload::env::ENV_ENUMVAL) != 0 && !e.enums.is_empty() {
            let matched = e
                .enums
                .iter()
                .any(|en| v.eq_ignore_ascii_case(en.r#match) || v.eq_ignore_ascii_case(en.value));
            if !matched {
                let list = e
                    .enums
                    .iter()
                    .map(|en| {
                        if en.r#match == en.value {
                            en.value.to_string()
                        } else {
                            format!("{}({})", en.r#match, en.value)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                invalid(
                    conf_err,
                    format!("should be one of the following values: {list}").as_bytes(),
                );
                return false;
            }
        }
        store()
            .lock()
            .unwrap()
            .insert(e.conf_name.to_string(), value.to_vec());
        return true;
    }
    if (dt & common_configload::env::ENV_SIZE) != 0 {
        match parse_size(&v) {
            Some(n) => {
                if e.min_value != 0 && n < e.min_value {
                    invalid(
                        conf_err,
                        format!("minimum value: {}", e.min_value).as_bytes(),
                    );
                    return false;
                }
                if e.max_value != 0 && n > e.max_value {
                    invalid(
                        conf_err,
                        format!("maximum value: {}", e.max_value).as_bytes(),
                    );
                    return false;
                }
            }
            None => {
                invalid(conf_err, b"should be a size");
                return false;
            }
        }
        store()
            .lock()
            .unwrap()
            .insert(e.conf_name.to_string(), value.to_vec());
        return true;
    }
    if (dt & (common_configload::env::ENV_FILE | common_configload::env::ENV_PATH)) != 0 {
        if v.contains(':') {
            invalid(conf_err, b"should not contain ':'");
            return false;
        }
        store()
            .lock()
            .unwrap()
            .insert(e.conf_name.to_string(), value.to_vec());
        return true;
    }
    // STR / CHAR: accepted verbatim.
    store()
        .lock()
        .unwrap()
        .insert(e.conf_name.to_string(), value.to_vec());
    true
}

/// Parse `4K` / `128M` / `2G` / plain bytes (libcob's size parsing).
fn parse_size(v: &str) -> Option<i64> {
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

/// Read the resolved store for the `--runtime-conf` report.
pub fn snapshot() -> (
    Option<String>,
    BTreeMap<String, Vec<u8>>,
    BTreeMap<String, String>,
) {
    let cfg = config_file().lock().unwrap().clone();
    let applied = store().lock().unwrap().clone();
    let env = env_overrides().lock().unwrap().clone();
    (cfg, applied, env)
}
