//! `cobc-rs` — the cobc-shaped compatibility driver for the gnucobol-rs interpreter.
//!
//! Roles (argv[0]-based dispatch, busybox-style):
//!   cobc / cobc-rs        — driver (compile modes, info modes)
//!   cobcrun / cobcrun-rs  — module runner + cobcrun info modes
//!   <anything else>       — launcher mode: `<base>` reads `<base>.cobr.json` (the manifest written
//!                           by `cobc-rs -x` / `-m`) and interprets the recorded source.
//!
//! The launcher + manifest mechanism is the TRUTHFUL artifact model: the test suite expects an
//! executable it can run later, so `cobc-rs -x -o prog` writes `prog` (a symlink to this binary),
//! `prog.cobr.json` (manifest) and `prog.cobr-src` (the define/copy-expanded source). `./prog`
//! dispatches here by argv[0] and interprets. This is NOT a native COBOL executable and is never
//! presented as one.

mod args;
mod capabilities;
mod compile;
mod copy;
mod info;
mod policy;
mod run;
mod runtime_config;

use std::path::PathBuf;

const EXIT_OK: i32 = 0;
const EXIT_ERROR: i32 = 1; // compile/check/rejection error (matches cobc's error exit)
const EXIT_INFRA: i32 = 2; // launcher/manifest/run infrastructure error (cobrun convention)

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    let argv0 = argv
        .first()
        .map(|a| {
            std::path::Path::new(a)
                .file_name()
                .map(|f| f.to_string_lossy().into_owned())
                .unwrap_or_else(|| a.clone())
        })
        .unwrap_or_default();
    let rest: Vec<String> = argv.iter().skip(1).cloned().collect();

    // Record every candidate invocation to $GNURUST_COBCRS_RECORD (JSONL) when set — the
    // candidate-phase invocation ledger feeding the evidence.
    if let Ok(rec) = std::env::var("GNURUST_COBCRS_RECORD") {
        if !rec.is_empty() {
            let _ = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&rec)
                .and_then(|mut f| {
                    use std::io::Write;
                    let entry = serde_json::json!({
                        "t": chrono_like_now(),
                        "cwd": std::env::current_dir().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default(),
                        "tool": argv0,
                        "argv": rest,
                    });
                    writeln!(f, "{entry}")
                });
        }
    }

    let code = dispatch(&argv0, &rest);
    std::process::exit(code);
}

fn chrono_like_now() -> String {
    // deterministic UTC timestamp (no chrono dependency)
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86400;
    let (y, m, d) = civil_from_days(days as i64);
    let tod = secs % 86400;
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        tod / 3600,
        (tod % 3600) / 60,
        tod % 60
    )
}

/// Convert days since 1970-01-01 to (year, month, day) — Howard Hinnant's civil algorithm.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

fn dispatch(argv0: &str, rest: &[String]) -> i32 {
    let base = argv0.to_ascii_lowercase();
    let is_cobc = base == "cobc" || base == "cobc-rs" || base == "cobc-rs.exe";
    let is_cobcrun = base == "cobcrun" || base == "cobcrun-rs" || base == "cobcrun-rs.exe";

    if is_cobc {
        return driver(rest);
    }
    if is_cobcrun {
        return runner(rest);
    }
    // launcher mode: <base> with a <base>.cobr.json manifest next to argv[0] or in cwd
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_default();
    let mut manifests = vec![
        exe_dir.join(format!("{argv0}.cobr.json")),
        PathBuf::from(format!("{argv0}.cobr.json")),
    ];
    if argv0.contains('/') {
        if let Some(dir) = std::path::Path::new(argv0).parent() {
            let b = std::path::Path::new(argv0)
                .file_name()
                .map(|f| f.to_string_lossy().into_owned())
                .unwrap_or_default();
            manifests.insert(0, dir.join(format!("{b}.cobr.json")));
        }
    }
    for m in manifests {
        if m.is_file() {
            return compile::run_launcher(&m, argv0);
        }
    }
    eprintln!(
        "cobc-rs: {argv0}: no launch manifest ({argv0}.cobr.json) found next to the executable or in the current directory"
    );
    EXIT_INFRA
}

fn driver(rest: &[String]) -> i32 {
    // info-only requests (the harness probes these with no sources)
    if rest.iter().any(|a| a == "--version" || a == "-V") && rest.len() == 1 {
        print!("{}", info::version());
        return EXIT_OK;
    }
    if rest.iter().any(|a| a == "--dumpversion") && rest.len() == 1 {
        print!("{}", info::dumpversion());
        return EXIT_OK;
    }
    if rest.iter().any(|a| a == "--info") {
        print!("{}", info::info());
        return EXIT_OK;
    }
    if rest
        .iter()
        .any(|a| a == "--runtime-conf" || a == "--runtime-config")
    {
        print!("{}", info::runtime_conf());
        return EXIT_OK;
    }
    if rest.iter().any(|a| a == "--help" || a == "-h") {
        print!("{}", info::help());
        return EXIT_OK;
    }
    if rest.iter().any(|a| a == "--print-capabilities") {
        print!("{}", capabilities::print_capabilities());
        return EXIT_OK;
    }
    // machine-readable policy registry export (the freshness source for the generated
    // cobc-rs-option-compatibility table)
    if let Some(p) = rest.iter().find(|a| a.starts_with("--dump-policy-json=")) {
        let path = p.trim_start_matches("--dump-policy-json=").to_string();
        if std::fs::write(
            &path,
            serde_json::to_vec_pretty(&capabilities::capabilities_json()).unwrap(),
        )
        .is_err()
        {
            eprintln!("cobc-rs: cannot write policy JSON {path}");
            return EXIT_INFRA;
        }
        return EXIT_OK;
    }
    if let Some(i) = rest.iter().position(|a| a == "--explain-translation") {
        let explained: Vec<String> = rest.iter().skip(i + 1).cloned().collect();
        print!("{}", capabilities::explain(&explained));
        return EXIT_OK;
    }

    let parsed = args::parse(rest);
    // --dump-invocation-json writes the record regardless of parse success
    let dump_path = match &parsed {
        Ok(p) => p.dump_invocation_json.clone(),
        Err(_) => rest
            .iter()
            .position(|a| a.starts_with("--dump-invocation-json"))
            .and_then(|i| rest.get(i + 1).cloned()),
    };
    if let Some(p) = dump_path {
        let rec = capabilities::invocation_record(rest, &parsed);
        let _ = std::fs::write(&p, serde_json::to_string_pretty(&rec).unwrap() + "\n");
    }

    let inv = match parsed {
        Ok(inv) => inv,
        Err(e) => {
            eprintln!("{e}");
            return EXIT_ERROR;
        }
    };
    if inv.capabilities {
        print!("{}", capabilities::print_capabilities());
        return EXIT_OK;
    }
    if inv.explain {
        print!("{}", capabilities::explain(rest));
        return EXIT_OK;
    }

    match inv.mode {
        policy::Mode::Info => {
            match inv.info_request.as_deref() {
                Some("--version") => print!("{}", info::version()),
                Some("--dumpversion") => print!("{}", info::dumpversion()),
                Some("--info") => print!("{}", info::info()),
                Some("--runtime-conf") => print!("{}", info::runtime_conf()),
                _ => print!("{}", info::help()),
            }
            EXIT_OK
        }
        policy::Mode::SyntaxOnly => match compile::syntax_only(&inv) {
            Ok(()) => EXIT_OK,
            Err(e) => {
                eprintln!("{e}");
                EXIT_ERROR
            }
        },
        policy::Mode::Preprocess => {
            let src = match inv.sources.first() {
                Some(s) => s.clone(),
                None => {
                    eprintln!("cobc-rs: -E needs a source file");
                    return EXIT_ERROR;
                }
            };
            match compile::expand_source(&inv, &src) {
                Ok(expanded) => {
                    let out = compile::preprocess_output(&expanded);
                    match &inv.output {
                        Some(o) => {
                            let _ = std::fs::write(o, out);
                        }
                        None => print!("{out}"),
                    }
                    EXIT_OK
                }
                Err(e) => {
                    eprintln!("{e}");
                    EXIT_ERROR
                }
            }
        }
        policy::Mode::Dependency => {
            let src = match inv.sources.first() {
                Some(s) => s.clone(),
                None => {
                    eprintln!("cobc-rs: -M needs a source file");
                    return EXIT_ERROR;
                }
            };
            match compile::expand_source(&inv, &src) {
                Ok(expanded) => {
                    let deps = &expanded.deps;
                    let main = std::path::Path::new(&src);
                    let targets: Vec<String> = if inv.deptargets.is_empty() {
                        vec![compile::default_output(&inv)]
                    } else {
                        inv.deptargets.clone()
                    };
                    match &inv.depfile {
                        Some(df) => {
                            if let Err(e) =
                                copy::write_depfile(std::path::Path::new(df), &targets, deps, main)
                            {
                                eprintln!("cobc-rs: {e}");
                                return EXIT_ERROR;
                            }
                        }
                        None => {
                            let mut line = format!("{}:", targets.join(" "));
                            for d in deps {
                                line.push(' ');
                                line.push_str(&d.to_string_lossy());
                            }
                            line.push(' ');
                            line.push_str(main.to_string_lossy().as_ref());
                            println!("{line}");
                        }
                    }
                    EXIT_OK
                }
                Err(e) => {
                    eprintln!("{e}");
                    EXIT_ERROR
                }
            }
        }
        policy::Mode::Executable | policy::Mode::Module => {
            let src = match inv.sources.first() {
                Some(s) => s.clone(),
                None => {
                    eprintln!("cobc-rs: no source file");
                    return EXIT_ERROR;
                }
            };
            // -C / -c / -S etc. are rejected at parse time; here only -x/-m remain.
            match compile::expand_source(&inv, &src) {
                Ok(expanded) => {
                    let out = compile::default_output(&inv);
                    if let Err(e) = compile::write_artifacts(&inv, &expanded, &out) {
                        eprintln!("cobc-rs: {e}");
                        return EXIT_ERROR;
                    }
                    // NOTE: cobc -m / -x do NOT print the module name on success (a suite
                    // AT_CHECK expects empty stdout); the artifact is written silently.
                    EXIT_OK
                }
                Err(e) => {
                    eprintln!("{e}");
                    EXIT_ERROR
                }
            }
        }
    }
}

fn runner(rest: &[String]) -> i32 {
    // cobcrun info modes
    if rest
        .iter()
        .any(|a| a == "--runtime-conf" || a == "--runtime-config")
    {
        // `-c <cfg>` / `--config=<cfg>` (before `--runtime-conf`) selects the runtime config file;
        // otherwise COB_RUNTIME_CONFIG / COB_CONFIG_DIR/runtime.cfg (the report's `via` line must
        // reflect the file the oracle loads at cob_init). The report then reflects the loaded
        // file + applied values + env-string expansion.
        let explicit = rest
            .iter()
            .position(|a| a == "-c" || a == "--config")
            .and_then(|i| rest.get(i + 1))
            .cloned()
            .or_else(|| {
                rest.iter()
                    .find(|a| a.starts_with("--config="))
                    .map(|a| a.trim_start_matches("--config=").to_string())
            });
        if let Some(path) = crate::runtime_config::resolve_config_file(explicit.as_deref()) {
            if crate::runtime_config::load_for_run(&path).is_err() {
                return 1;
            }
        }
        print!("{}", info::runtime_conf());
        return EXIT_OK;
    }
    if rest.iter().any(|a| a == "--version" || a == "-V") {
        print!("{}", info::version());
        return EXIT_OK;
    }
    if rest.iter().any(|a| a == "--dumpversion") {
        print!("{}", info::dumpversion());
        return EXIT_OK;
    }
    if rest.iter().any(|a| a == "--info") {
        print!("{}", info::info());
        return EXIT_OK;
    }
    if rest.iter().any(|a| a == "--help" || a == "-h") {
        print!("{}", info::help());
        return EXIT_OK;
    }
    if rest.iter().any(|a| {
        a == "--list-reserved"
            || a == "--list-intrinsics"
            || a == "--list-mnemonics"
            || a == "--list-registers"
            || a == "--list-system"
            || a == "--list-exceptions"
    }) {
        eprintln!("cobc-rs (cobcrun): upstream-identical keyword lists are not provided by the candidate; failing closed");
        return EXIT_ERROR;
    }
    // cobcrun option parsing (module + args; `-M` dir; `-c` config).
    let args = match compile::CobcrunArgs::parse(rest) {
        Ok(a) => a,
        Err(msg) => {
            // `invalid module argument ''` is cobcrun's own fatal diagnostic (no prefix)
            if msg.starts_with("invalid module argument") {
                eprintln!("{msg}");
            } else {
                eprintln!("{msg}");
            }
            return 1;
        }
    };
    // `-c <cfg>` runtime config loading (errors are fatal, matching cobcrun).
    if let Some(cfg) = &args.config {
        if let Some(path) = crate::runtime_config::config_path_from_value(cfg) {
            if crate::runtime_config::load_for_run(&path).is_err() {
                return 1;
            }
        }
    }
    match &args.program {
        Some(_) => compile::cobcrun_run(&args, "cobcrun"),
        None => {
            // `cobcrun` with only options and no program name
            eprintln!("cobcrun: missing PROGRAM name\nTry 'cobcrun --help' for more information.");
            1
        }
    }
}
