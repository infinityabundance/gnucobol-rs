//! The interpreter execution boundary for the `cobc-rs` adapter: source -> process exit status,
//! mirroring the `cobrun` host boundary (locale resolution, `COB_DISPLAY_PRINT_FILE` redirect,
//! RETURN-CODE propagation, per-run file-store dump). The interpreter itself is the
//! `#![forbid(unsafe_code)]` gnucobol-rs front end — no `cobc`, no `libcob`, ever.

use gnucobol_rs::dialect::Dialect;
use std::io::Write;
use std::path::Path;

/// The GnuCOBOL version this front end targets (the ported `libcob` version constants).
pub fn target_version() -> String {
    use gnucobol_rs::common::{LIBCOB_VERSION, LIBCOB_VERSION_MINOR, LIBCOB_VERSION_PATCHLEVEL};
    format!("{LIBCOB_VERSION}.{LIBCOB_VERSION_MINOR}.{LIBCOB_VERSION_PATCHLEVEL}")
}

/// Resolve one `LC_*` category as libcob's `cob_init` does (glibc precedence).
fn resolve_locale(category: &str) -> String {
    if let Ok(v) = std::env::var("LC_ALL") {
        if !v.is_empty() {
            return v;
        }
    }
    if let Ok(v) = std::env::var(category) {
        if !v.is_empty() {
            return v;
        }
    }
    if let Ok(v) = std::env::var("LANG") {
        if !v.is_empty() {
            return v;
        }
    }
    "C".to_string()
}

/// Build the host-supplied [`SystemConf`] for `--runtime-conf` from the process environment
/// (reproducing libcob's `cob_init` locale setup and username resolution).
pub fn build_system_conf() -> gnucobol_rs::common_runtimeconf::SystemConf {
    use gnucobol_rs::common_runtimeconf::{SystemConf, UserOrigin};
    let nonempty = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
    let config_dir = std::env::var("COB_CONFIG_DIR")
        .unwrap_or_else(|_| "/usr/local/share/gnucobol/config".to_string());
    let config_file = format!("{config_dir}/runtime.cfg");
    let username = nonempty("LOGNAME")
        .map(|v| (v, UserOrigin::Logname))
        .or_else(|| nonempty("USERNAME").map(|v| (v, UserOrigin::Username)))
        .or_else(|| nonempty("USER").map(|v| (v, UserOrigin::Getlogin)));
    let localedir = nonempty("LOCALEDIR").unwrap_or_else(|| {
        std::path::Path::new(&config_dir)
            .parent()
            .and_then(|p| p.parent())
            .map(|share| share.join("locale").to_string_lossy().into_owned())
            .unwrap_or_else(|| format!("{config_dir}/locale"))
    });
    SystemConf {
        config_file,
        username,
        lang: nonempty("LANG"),
        ostype: nonempty("OSTYPE"),
        term: nonempty("TERM"),
        localedir,
        lc_ctype: "C".to_string(),
        lc_numeric: "C".to_string(),
        lc_collate: resolve_locale("LC_COLLATE"),
        lc_messages: resolve_locale("LC_MESSAGES"),
        lc_monetary: resolve_locale("LC_MONETARY"),
        lc_time: resolve_locale("LC_TIME"),
    }
}

/// Options for one interpreted run.
pub struct RunOpts {
    /// The FULL source text (already define/copy-expanded by the adapter).
    pub source: String,
    pub dialect: Dialect,
    pub fixed: bool,
    /// The original source file path (for FUNCTION MODULE-SOURCE + sibling CALL resolution).
    pub source_file: String,
    /// Directory for the per-run in-memory file-store dump (None = no dump).
    pub dump_dir: Option<String>,
}

/// Run the source and return the process exit code (RETURN-CODE / STOP RUN n). Never returns a
/// `cobc`/`libcob` delegation — the front end is the in-process Rust interpreter.
pub fn run_interpreted(opts: &RunOpts) -> i32 {
    let mut src = if opts.fixed {
        gnucobol_rs::frontend::fixed_to_free(&opts.source)
    } else {
        opts.source.clone()
    };
    // Separate-file CALL: a `CALL "NAME"` to a program that is not a unit in THIS source is
    // resolved to a sibling NAME.<ext> file (mirroring cobc compiling the callee as a module and
    // linking it at the CALL). The compile pipeline already appended the callees into the expanded
    // source, so this is a no-op here except when the manifest records an unexpanded source.
    if let Some(dir) = std::path::Path::new(&opts.source_file).parent() {
        src = resolve_separate_calls(src, dir, opts.fixed);
    }
    gnucobol_rs::frontend::set_source_file(&opts.source_file);
    if let Some(dir) = &opts.dump_dir {
        gnucobol_rs::frontend::set_file_dump_dir(std::path::PathBuf::from(dir));
    }
    let print_file = std::env::var_os("COB_DISPLAY_PRINT_FILE");
    let redirect = print_file.is_some();
    match gnucobol_rs::frontend::run_program_redirected(&src, opts.dialect, redirect) {
        Ok((out, printer, rc)) => {
            let stdout = std::io::stdout();
            let mut lock = stdout.lock();
            let _ = lock.write_all(&out);
            let _ = lock.flush();
            if let Some(path) = print_file {
                if !printer.is_empty() {
                    if let Ok(mut f) = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&path)
                    {
                        let _ = f.write_all(&printer);
                    }
                }
            }
            rc
        }
        Err(e) => {
            eprintln!("cobrun: {e}");
            2
        }
    }
}

/// The `PROGRAM-ID name` units already present in `src` (uppercased), so a CALL to one of them is
/// NOT resolved to a file (it is in-source / contained).
pub fn program_units(src: &str) -> std::collections::HashSet<String> {
    let up = src.to_ascii_uppercase();
    let mut set = std::collections::HashSet::new();
    let mut rest = up.as_str();
    while let Some(p) = rest.find("PROGRAM-ID") {
        rest = &rest[p + "PROGRAM-ID".len()..];
        let bytes = rest.as_bytes();
        let mut i = 0;
        while i < bytes.len() && (bytes[i].is_ascii_whitespace() || bytes[i] == b'.') {
            i += 1;
        }
        let start = i;
        while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-') {
            i += 1;
        }
        if i > start {
            set.insert(rest[start..i].to_string());
        }
        rest = &rest[i..];
    }
    set
}

/// Every `CALL "literal"` / `CALL 'literal'` program name in `src` (uppercased).
pub fn call_literals(src: &str) -> Vec<String> {
    let up = src.to_ascii_uppercase();
    let mut out = Vec::new();
    let rest = up.as_str();
    let mut off = 0usize;
    while let Some(p) = rest[off..].find("CALL") {
        let at = off + p;
        let prev_ok = at == 0
            || !(up.as_bytes()[at - 1].is_ascii_alphanumeric() || up.as_bytes()[at - 1] == b'-');
        let mut j = at + 4;
        let b = up.as_bytes();
        while j < b.len() && b[j].is_ascii_whitespace() {
            j += 1;
        }
        if prev_ok && j < b.len() && (b[j] == b'"' || b[j] == b'\'') {
            let q = b[j];
            let s = j + 1;
            let mut e = s;
            while e < b.len() && b[e] != q {
                e += 1;
            }
            if e <= b.len() {
                let name = up[s..e].trim().to_string();
                if !name.is_empty() {
                    out.push(name);
                }
            }
        }
        off = at + 4;
    }
    out
}

/// Resolve separate-file CALLs by appending each called sibling source as an extra program unit.
pub fn resolve_separate_calls(mut src: String, dir: &Path, fixed: bool) -> String {
    let exts = ["CBL", "cbl", "COB", "cob", "Cob", "Cbl"];
    let mut present: std::collections::HashSet<String> = program_units(&src);
    loop {
        let mut added = false;
        for name in call_literals(&src) {
            if present.contains(&name) {
                continue;
            }
            for ext in &exts {
                let cand = dir.join(format!("{name}.{ext}"));
                if cand.is_file() {
                    if let Ok(raw) = std::fs::read_to_string(&cand) {
                        let unit = if fixed {
                            gnucobol_rs::frontend::fixed_to_free(&raw)
                        } else {
                            raw
                        };
                        src.push('\n');
                        src.push_str(&unit);
                        present.insert(name.clone());
                        added = true;
                    }
                    break;
                }
            }
            present.insert(name);
        }
        if !added {
            break;
        }
    }
    src
}
