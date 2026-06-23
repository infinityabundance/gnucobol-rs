//! `cobrun [-std=NAME] <file.cob>` -- the CLEAN-ROOM COBOL front-end: parse + EXECUTE a COBOL program on
//! the ported libcob runtime (no `cobc`, no `libcob` linked) and write its stdout. Exits 2 with the
//! reason on anything outside the sealed subset (`gnucobol_rs::frontend`). This is the turn-key form
//! of the end-to-end execution proof: feed it source, it runs it. `-std=` selects a dialect (e.g. the
//! `defaultbyte` fill of uninitialized storage), mirroring `cobc -std=`.

use gnucobol_rs::dialect::Dialect;
use std::io::Write;

/// The GnuCOBOL version this front-end targets (the ported `libcob` version constants), e.g. `3.2.0`.
fn target_version() -> String {
    use gnucobol_rs::common::{LIBCOB_VERSION, LIBCOB_VERSION_MINOR, LIBCOB_VERSION_PATCHLEVEL};
    format!("{LIBCOB_VERSION}.{LIBCOB_VERSION_MINOR}.{LIBCOB_VERSION_PATCHLEVEL}")
}

/// Resolve one `LC_*` category as libcob's `cob_init` does -- `setlocale(LC_ALL,"")` follows glibc
/// precedence (`LC_ALL` > `LC_<category>` > `LANG` > `"C"`), returning the env string verbatim.
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

/// Build the host-supplied [`SystemConf`] for `--runtime-config` from the process environment,
/// reproducing libcob's `cob_init` locale setup and username resolution. This is the boundary the
/// `#![forbid(unsafe_code)]` core cannot cross itself (it needs `getenv`/`setlocale`/`getlogin`).
fn build_system_conf() -> gnucobol_rs::common_runtimeconf::SystemConf {
    use gnucobol_rs::common_runtimeconf::{SystemConf, UserOrigin};
    let nonempty = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());

    // Main config file: $COB_CONFIG_DIR/runtime.cfg (else the compiled default dir).
    let config_dir = std::env::var("COB_CONFIG_DIR")
        .unwrap_or_else(|_| "/usr/local/share/gnucobol/config".to_string());
    let config_file = format!("{config_dir}/runtime.cfg");

    // Username resolution, in libcob's effective precedence: the config loader reads USERNAME first
    // then LOGNAME (a hidden alias at the same slot), so LOGNAME *overwrites* USERNAME and wins when
    // both are set -- shown "(set by LOGNAME)". USERNAME-only is plain env. With neither set, libcob
    // falls to getlogin() (a libc leaf the safe core can't call); $USER is the portable stand-in,
    // reported under the same "(set by getlogin())" origin (default "    " prefix, not env).
    let username = nonempty("LOGNAME")
        .map(|v| (v, UserOrigin::Logname))
        .or_else(|| nonempty("USERNAME").map(|v| (v, UserOrigin::Username)))
        .or_else(|| nonempty("USER").map(|v| (v, UserOrigin::Getlogin)));

    // LOCALEDIR: $LOCALEDIR, else derive the compiled <prefix>/share/locale from COB_CONFIG_DIR
    // (which is <prefix>/share/gnucobol/config) so it matches the admitted oracle's compiled path.
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
        // cob_init forces LC_NUMERIC and LC_CTYPE to "C" after setlocale(LC_ALL,"").
        lc_ctype: "C".to_string(),
        lc_numeric: "C".to_string(),
        lc_collate: resolve_locale("LC_COLLATE"),
        lc_messages: resolve_locale("LC_MESSAGES"),
        lc_monetary: resolve_locale("LC_MONETARY"),
        lc_time: resolve_locale("LC_TIME"),
    }
}

fn main() {
    // cobrun [-std=NAME] [-fixed|-free] <file.cob> -- dialect selector + source format (default free).
    let mut dialect = Dialect::DEFAULT;
    let mut fixed = false;
    let mut path: Option<String> = None;
    for arg in std::env::args().skip(1) {
        if let Some(name) = arg.strip_prefix("-std=").or_else(|| arg.strip_prefix("--std=")) {
            dialect = Dialect::from_std(name);
        } else if arg == "-fixed" || arg == "--fixed" {
            fixed = true;
        } else if arg == "-free" || arg == "--free" {
            fixed = false;
        } else if arg == "--runtime-config" || arg == "-runtime-config" {
            // cobcrun --runtime-config: print the resolved runtime configuration, byte-identical to
            // the oracle (native-Rust port of libcob print_runtime_conf).
            let sys = build_system_conf();
            std::io::stdout()
                .write_all(&gnucobol_rs::common_runtimeconf::print_runtime_conf(&sys))
                .unwrap();
            return;
        } else if arg == "-x" || arg == "-j" || arg == "--job" {
            // cobc-compatibility: `cobc -x`/`-j` build-and-run an executable; for the interpreter, running
            // the source IS that step, so these are accepted as no-ops (cobrun <file> == cobc -x <file>).
        } else if arg == "-dumpversion" || arg == "--dumpversion" {
            // the targeted GnuCOBOL version, byte-identical to `cobc -dumpversion` / `cobcrun -dumpversion`.
            println!("{}", target_version());
            return;
        } else if arg == "-V" || arg == "--version" {
            // Identify honestly as the gnucobol-rs port (NOT a masquerade of GnuCOBOL), reporting the
            // GnuCOBOL version it reproduces, in the GnuCOBOL --version block format.
            print!(
                "cobrun (gnucobol-rs, reproducing GnuCOBOL) {ver}\n\
                 A clean-room native-Rust reimplementation of the GnuCOBOL {ver} runtime + a COBOL\n\
                 interpreter front-end, proven byte-identical to the admitted cobc oracle.\n\
                 License LGPL-3.0-or-later. Not GnuCOBOL; not affiliated with the GNU project.\n",
                ver = target_version(),
            );
            return;
        } else if arg == "-h" || arg == "--help" {
            print!(
                "cobrun -- run a COBOL program on the native-Rust GnuCOBOL runtime (no cobc/libcob linked)\n\n\
                 Usage: cobrun [options] <file.cob>\n\n\
                 Options:\n  \
                 -free | -fixed              source format (default: free)\n  \
                 -std=<name>                 dialect (default | ibm | mvs | mf | cobol2014 | ...)\n  \
                 -V, --version               version information and exit\n  \
                 -dumpversion                the targeted GnuCOBOL version (e.g. {}) and exit\n  \
                 -h, --help                  this help and exit\n\n\
                 The program's RETURN-CODE becomes the process exit status.\n",
                target_version(),
            );
            return;
        } else if arg.starts_with('-') && arg != "-" {
            // an unknown leading-dash token is a flag, not the source file (avoid mis-reading it as
            // <file>). Checked AFTER the known flags so they are not shadowed by this catch-all.
            eprintln!("cobrun: unknown option `{arg}` (try --help)");
            std::process::exit(2);
        } else {
            path = Some(arg);
        }
    }
    let path = match path {
        Some(p) => p,
        None => {
            eprintln!("usage: cobrun [-std=NAME] [-fixed|-free] <file.cob>");
            std::process::exit(2);
        }
    };
    let raw = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("cobrun: cannot read {path}: {e}");
            std::process::exit(2);
        }
    };
    return run_after_read(raw, path, fixed, dialect);
}

/// The `PROGRAM-ID name` units already present in `src` (uppercased), so a CALL to one of them is NOT
/// resolved to a file (it is in-source / contained).
fn program_units(src: &str) -> std::collections::HashSet<String> {
    let up = src.to_ascii_uppercase();
    let mut set = std::collections::HashSet::new();
    let mut rest = up.as_str();
    while let Some(p) = rest.find("PROGRAM-ID") {
        rest = &rest[p + "PROGRAM-ID".len()..];
        // skip the optional '.' and whitespace, then take the identifier (letters/digits/-).
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

/// Every `CALL "literal"` / `CALL 'literal'` program name in `src` (uppercased). `CALL identifier` (a
/// data-name target) is skipped -- only a literal names a resolvable separate file.
fn call_literals(src: &str) -> Vec<String> {
    let up = src.to_ascii_uppercase();
    let mut out = Vec::new();
    let mut rest = up.as_str();
    let mut off = 0usize;
    while let Some(p) = rest[off..].find("CALL") {
        let at = off + p;
        // require a word boundary before CALL (start or non-identifier char) so we don't match inside a word.
        let prev_ok = at == 0 || !(up.as_bytes()[at - 1].is_ascii_alphanumeric() || up.as_bytes()[at - 1] == b'-');
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
fn resolve_separate_calls(mut src: String, dir: &std::path::Path, fixed: bool) -> String {
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
                        let unit = if fixed { gnucobol_rs::frontend::fixed_to_free(&raw) } else { raw };
                        src.push('\n');
                        src.push_str(&unit);
                        present.insert(name.clone());
                        added = true;
                    }
                    break;
                }
            }
            // mark as seen even if no file found, so an external (.so-only) CALL is not retried each pass.
            present.insert(name);
        }
        if !added {
            break;
        }
    }
    src
}

fn run_after_read(raw: String, path: String, fixed: bool, dialect: Dialect) -> ! {
    let mut src = if fixed { gnucobol_rs::frontend::fixed_to_free(&raw) } else { raw };
    // Separate-file CALL: a `CALL "NAME"` to a program that is not a unit in THIS source is resolved to a
    // sibling NAME.<ext> file (mirroring cobc compiling the callee as a module and linking it at the CALL).
    // Append each resolved callee as another program unit so cobrun's CALL dispatch executes the real
    // subprogram (USING args bound by position). Transitive + cycle-safe.
    if let Some(dir) = std::path::Path::new(&path).parent() {
        src = resolve_separate_calls(src, dir, fixed);
    }
    // Record the source path for FUNCTION MODULE-SOURCE (cobc embeds the source name it was given).
    gnucobol_rs::frontend::set_source_file(&path);
    // DISPLAY ... UPON PRINTER redirect: when COB_DISPLAY_PRINT_FILE is set, libcob diverts UPON PRINTER
    // to that file (appending) instead of stdout. cobrun is the host that owns this env+file boundary; the
    // interpreter only separates the printer stream. (COB_DISPLAY_PRINT_PIPE -- a spawned pipe -- is a
    // boundary cobrun does not implement.)
    let print_file = std::env::var_os("COB_DISPLAY_PRINT_FILE");
    let redirect = print_file.is_some();
    match gnucobol_rs::frontend::run_program_redirected(&src, dialect, redirect) {
        Ok((out, printer, rc)) => {
            std::io::stdout().write_all(&out).unwrap();
            if let Some(path) = print_file {
                if !printer.is_empty() {
                    use std::io::Write as _;
                    if let Ok(mut f) =
                        std::fs::OpenOptions::new().create(true).append(true).open(&path)
                    {
                        let _ = f.write_all(&printer);
                    }
                }
            }
            // RETURN-CODE flows to the process exit status (MOVE n TO RETURN-CODE / STOP RUN n).
            std::process::exit(rc);
        }
        Err(e) => {
            eprintln!("cobrun: {e}");
            std::process::exit(2);
        }
    }
}
