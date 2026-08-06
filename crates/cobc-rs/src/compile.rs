//! The compile pipeline: read + expand the source (defines, COPY, sibling CALLs), resolve the
//! dialect, and produce the requested artifacts — launch manifest + launcher (NEVER a native COBOL
//! executable), expanded source, dependency file, or the syntax-only check.

use crate::args::ParsedInvocation;
use crate::copy::{self, FsCopyResolver};
use crate::run;
use gnucobol_rs::copybook::CopyError;
use gnucobol_rs::dialect::Dialect;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// Read the main source (a path, or stdin for `-`).
///
/// Since upstream `470f7db12` (pplex `ppopen_get_file`), cobc detects a **binary** source file and
/// errors out directly; the candidate mirrors that: a NUL byte in the source is rejected as a
/// binary file before any further processing.
pub fn read_source(path: &str) -> Result<String, String> {
    let bytes = if path == "-" {
        use std::io::Read;
        let mut buf = Vec::new();
        std::io::stdin()
            .read_to_end(&mut buf)
            .map_err(|e| format!("stdin: {e}"))?;
        buf
    } else {
        std::fs::read(path).map_err(|e| format!("cannot read {path}: {e}"))?
    };
    if bytes.contains(&0) {
        return Err(format!("cobc: {path}: binary file"));
    }
    String::from_utf8(bytes).map_err(|e| format!("cobc: {path}: not valid UTF-8 source: {e}"))
}

/// The fully expanded program text (defines prepended, sibling CALLs appended, COPY expanded).
pub struct ExpandedSource {
    pub text: String,
    /// Copybooks consumed (absolute-ish paths as resolved), for -MF/-MT.
    pub deps: Vec<PathBuf>,
    /// The original source file path (for the manifest + MODULE-SOURCE).
    pub source_file: String,
    /// Fixed-format hint.
    pub fixed: bool,
}

/// Build the expanded source for the first source of `inv`.
pub fn expand_source(inv: &ParsedInvocation, source_path: &str) -> Result<ExpandedSource, String> {
    let raw = read_source(source_path)?;
    // Upstream e36a124b2: `--copy <file>` pre-copies copybook text at the beginning of the source
    // (before parsing), so it can carry REPLACEments or COBOL prototypes. The candidate prepends
    // each file's text before preprocessing, exactly like the upstream "COPY file at beginning".
    let mut raw = raw;
    for cf in &inv.copy_files {
        let text = std::fs::read_to_string(cf)
            .map_err(|e| format!("cobc-rs: cannot read --copy file {cf}: {e}"))?;
        raw = format!("{text}\n{raw}");
    }
    let _ = raw;
    let fixed = match inv.format.as_deref() {
        Some("fixed") => true,
        Some("free") => false,
        Some("auto") | None => detect_fixed(&raw),
        Some(other) => {
            return Err(format!(
                "cobc-rs: -fformat={other}: only fixed/free/auto are supported by the candidate (fail closed)"
            ))
        }
    };
    // 1) append sibling CALL targets (fixed sources converted the same way cobrun does)
    let mut text = if let Some(dir) = Path::new(source_path).parent() {
        let with_siblings = run::resolve_separate_calls(raw.clone(), dir, fixed);
        // re-convert: resolve_separate_calls converted fixed callees already; the main body stays raw
        with_siblings
    } else {
        raw.clone()
    };
    // 2) prepend >>DEFINE lines for -D (the front-end preprocessor consumes them)
    let mut define_lines = String::new();
    for (name, value) in &inv.defines {
        define_lines.push_str(&format!("        >>DEFINE {} AS {}\n", name, value));
    }
    if !define_lines.is_empty() {
        text = format!("{define_lines}{text}");
    }
    // 3) COPY expansion
    let resolver = FsCopyResolver::new(
        inv.includes.iter().map(PathBuf::from).collect(),
        inv.extensions.clone(),
    );
    let deps = copy::collect_deps(&text, &resolver);
    let expanded = gnucobol_rs::copybook::expand(&text, &resolver)
        .map_err(|e: CopyError| format!("cobc-rs: COPY expansion failed (fail closed): {e}"))?;
    Ok(ExpandedSource {
        text: expanded.text(),
        deps,
        source_file: source_path.to_string(),
        fixed,
    })
}

/// Heuristic for the default source format when neither -free nor -fixed is given (cobc defaults
/// to fixed-format). Free format wins only when the source is unambiguously free (a column-7 area
/// that is never non-blank in the first 72 columns, i.e. no indicator column usage).
fn detect_fixed(src: &str) -> bool {
    for line in src.lines() {
        let chars: Vec<char> = line.chars().collect();
        // any non-blank char at index 6 (column 7) -> fixed-format indicator area in use
        if chars.get(6).is_some_and(|c| !c.is_whitespace()) {
            return true;
        }
        // code in the sequence area (columns 1-6) -> fixed
        let seq: String = chars.iter().take(6).collect();
        if seq.chars().any(|c| !c.is_whitespace()) {
            return true;
        }
    }
    false
}

/// Resolve the runtime [`Dialect`] from -std / -conf / -fdefaultbyte overlays.
pub fn resolve_dialect(inv: &ParsedInvocation) -> Dialect {
    let mut d = match &inv.conf {
        Some(conf) => Dialect::from_conf(conf, &conf_reader).unwrap_or_else(|| {
            eprintln!("cobc-rs: warning: cannot read dialect config {conf:?}; using -std/default");
            Dialect::from_std(inv.dialect.as_deref().unwrap_or(""))
        }),
        None => Dialect::from_std(inv.dialect.as_deref().unwrap_or("")),
    };
    for (key, val) in &inv.conf_overrides {
        if key == "defaultbyte" {
            d.defaultbyte = parse_defaultbyte(val);
        }
    }
    d
}

/// `Dialect::from_conf`'s `read` callback: resolve the config file against cwd and COB_CONFIG_DIR.
fn conf_reader(name: &str) -> Option<Vec<u8>> {
    let mut candidates = vec![PathBuf::from(name)];
    if let Ok(dir) = std::env::var("COB_CONFIG_DIR") {
        candidates.push(PathBuf::from(dir).join(name));
    }
    for c in candidates {
        if let Ok(bytes) = std::fs::read(&c) {
            return Some(bytes);
        }
    }
    None
}

fn parse_defaultbyte(v: &str) -> gnucobol_rs::dialect::DefaultByte {
    use gnucobol_rs::dialect::DefaultByte;
    match v {
        "init" => DefaultByte::Init,
        "0" | "none" => DefaultByte::Fill(0),
        " " | "32" | "space" => DefaultByte::Fill(b' '),
        other => {
            if let Ok(n) = other.parse::<u8>() {
                DefaultByte::Fill(n)
            } else if let Some(c) = other.bytes().next() {
                DefaultByte::Fill(c)
            } else {
                DefaultByte::Init
            }
        }
    }
}

/// The default artifact name for an invocation (cobc derives it from the first source basename).
pub fn default_output(inv: &ParsedInvocation) -> String {
    if let Some(o) = &inv.output {
        return o.clone();
    }
    match inv.sources.first() {
        Some(src) if src != "-" => {
            let p = Path::new(src);
            let base = p
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "a".to_string());
            match inv.mode {
                crate::policy::Mode::Module => format!("{base}.so"),
                _ => base,
            }
        }
        _ => "a.out".to_string(),
    }
}

/// Write the manifest + launcher for an executable/module artifact.
pub fn write_artifacts(
    inv: &ParsedInvocation,
    expanded: &ExpandedSource,
    output: &str,
) -> Result<(), String> {
    let out_path = PathBuf::from(output);
    let out_dir = out_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_default();
    let base = out_path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| output.to_string());

    // expanded source next to the artifact (atomic write)
    let src_name = format!("{base}.cobr-src");
    let src_path = out_dir.join(&src_name);
    let tmp = out_dir.join(format!("{base}.cobr-src.tmp"));
    std::fs::write(&tmp, &expanded.text).map_err(|e| format!("write expanded source: {e}"))?;
    std::fs::rename(&tmp, &src_path).map_err(|e| format!("rename expanded source: {e}"))?;

    // manifest
    let manifest = json!({
        "schema": "gnucobol-rs-launch-manifest-v1",
        "source": expanded.source_file,
        "expanded_source": src_name,
        "dialect": inv.dialect.clone().unwrap_or_default(),
        "conf": inv.conf.clone().unwrap_or_default(),
        "conf_overrides": inv.conf_overrides,
        "source_format": if expanded.fixed { "fixed" } else { "free" },
        "diag_absolute_path": inv.diag_absolute_path,
        "main_program": null,
        "module_paths": [],
        "runtime_environment": {},
        "mode": inv.mode.as_str(),
        "manifest_sha256": "",
    });
    let manifest_path = out_dir.join(format!("{base}.cobr.json"));
    let manifest_bytes = serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?;
    // manifest_sha256 is the hash of the manifest body WITHOUT the sha field itself (the field is
    // inserted after hashing; the launcher removes it and re-hashes to verify integrity).
    let mut body_val = manifest.clone();
    if let Some(m) = body_val.as_object_mut() {
        m.remove("manifest_sha256");
    }
    let body = serde_json::to_vec(&body_val).unwrap();
    let hash = sha256_hex(&body);
    let mut final_manifest = manifest.clone();
    if let Some(m) = final_manifest.as_object_mut() {
        m.insert("manifest_sha256".to_string(), Value::String(hash));
    }
    let tmp = out_dir.join(format!("{base}.cobr.json.tmp"));
    std::fs::write(&tmp, serde_json::to_vec_pretty(&final_manifest).unwrap())
        .map_err(|e| format!("write manifest: {e}"))?;
    std::fs::rename(&tmp, &manifest_path).map_err(|e| format!("rename manifest: {e}"))?;
    let _ = manifest_bytes;

    // launcher: a symlink to the cobc-rs binary (argv[0]-based dispatch -> launcher mode)
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let launcher_tmp = out_dir.join(format!(".{base}.cobr-launch.tmp"));
    let _ = std::fs::remove_file(&launcher_tmp);
    std::os::unix::fs::symlink(&exe, &launcher_tmp)
        .map_err(|e| format!("symlink launcher: {e}"))?;
    std::fs::rename(&launcher_tmp, &out_path).map_err(|e| format!("rename launcher: {e}"))?;

    // dependency file
    if let Some(df) = &inv.depfile {
        let targets: Vec<String> = if inv.deptargets.is_empty() {
            vec![output.to_string()]
        } else {
            inv.deptargets.clone()
        };
        copy::write_depfile(
            Path::new(df),
            &targets,
            &expanded.deps,
            Path::new(&expanded.source_file),
            copy::DepfileOpts {
                phony: inv.dep_phony,
                quote_targets: inv.dep_makefile_quote,
                copybook_only: inv.copybook_deps,
            },
        )?;
    }
    Ok(())
}

/// `-E` preprocess-only output (GnuCOBOL's `#line`-prefixed shape).
pub fn preprocess_output(expanded: &ExpandedSource) -> String {
    let mut out = String::new();
    out.push_str(&format!("#line 1 \"{}\"\n", expanded.source_file));
    out.push('\n');
    out.push_str(&expanded.text);
    if !expanded.text.ends_with('\n') {
        out.push('\n');
    }
    out
}

pub fn sha256_hex(b: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(b);
    format!("{:x}", h.finalize())
}

/// Syntax-only: run the real candidate check pipeline (no execution, no artifacts).
pub fn syntax_only(inv: &ParsedInvocation) -> Result<(), String> {
    let source_path = inv
        .sources
        .first()
        .ok_or_else(|| "cobc-rs: -fsyntax-only needs a source file".to_string())?;
    let expanded = expand_source(inv, source_path)?;
    let dialect = resolve_dialect(inv);
    let src = if expanded.fixed {
        gnucobol_rs::frontend::fixed_to_free(&expanded.text)
    } else {
        expanded.text.clone()
    };
    gnucobol_rs::frontend::check_program(&src, dialect).map_err(|e| format!("cobc-rs: {e}"))
}

/// Run a launcher artifact: read the manifest next to `argv0`/cwd and interpret the source.
pub fn run_launcher(manifest_path: &Path, argv0_base: &str) -> i32 {
    let manifest: Value = match std::fs::read_to_string(manifest_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
    {
        Some(m) => m,
        None => {
            eprintln!("cobc-rs: {argv0_base}: cannot read launch manifest {manifest_path:?}");
            return 2;
        }
    };
    // Tamper guard: the manifest carries a self-hash (the body WITHOUT the manifest_sha256 field); a
    // mismatched hash means the manifest was hand-edited after `cobc-rs` wrote it — refuse to run
    // (fail closed) rather than interpret a manifest that no longer matches its recorded bytes.
    let mut body = manifest.clone();
    let expected = body
        .as_object_mut()
        .and_then(|m| m.remove("manifest_sha256"))
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default();
    if !expected.is_empty() {
        let actual = match serde_json::to_vec(&body) {
            Ok(b) => sha256_hex(&b),
            Err(_) => String::new(),
        };
        if actual != expected {
            eprintln!(
                "cobc-rs: {argv0_base}: launch manifest {manifest_path:?} failed its integrity check (tampered?)"
            );
            return 2;
        }
    }
    let mdir = manifest_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_default();
    let src_name = manifest["expanded_source"].as_str().unwrap_or("");
    let src_path = mdir.join(src_name);
    let text = match std::fs::read_to_string(&src_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("cobc-rs: {argv0_base}: cannot read expanded source {src_path:?}: {e}");
            return 2;
        }
    };
    let fixed = manifest["source_format"].as_str().unwrap_or("free") == "fixed";
    let dialect_name = manifest["dialect"].as_str().unwrap_or("");
    let conf = manifest["conf"].as_str().unwrap_or("");
    let mut dialect = if conf.is_empty() {
        Dialect::from_std(dialect_name)
    } else {
        Dialect::from_conf(conf, &conf_reader).unwrap_or_else(|| Dialect::from_std(dialect_name))
    };
    if let Some(overrides) = manifest["conf_overrides"].as_array() {
        for ov in overrides {
            if ov.is_array() && ov[0].as_str() == Some("defaultbyte") {
                if let Some(v) = ov[1].as_str() {
                    dialect.defaultbyte = parse_defaultbyte(v);
                }
            }
        }
    }
    let source_file = manifest["source"]
        .as_str()
        .unwrap_or("prog.cob")
        .to_string();
    let diag_absolute_path = manifest["diag_absolute_path"].as_bool().unwrap_or(false);
    let dump_dir = Some(".".to_string());
    let opts = run::RunOpts {
        source: text,
        dialect,
        fixed,
        source_file,
        diag_absolute_path,
        dump_dir,
    };
    run::run_interpreted(&opts)
}

/// cobcrun mode: `cobcrun [-M <dir>] [-c <cfg>] <program> [args...]` — resolve the build-local
/// module registry (the manifest written by `cobc-rs -m`) and interpret the module, passing the
/// remaining arguments to the program (COMMAND-LINE / ARGUMENT-VALUE). The `-M` value is a
/// DIRECTORY (cobcrun appends `/` when missing); the module file is `<dir><program>.so`. This is a
/// truthful interpreted-module model — never a native shared object.
pub struct CobcrunArgs {
    /// `-M <dir>` value (None when absent; Some even when empty — an empty value is the cobcrun
    /// `invalid module argument ''` error).
    pub module_dir: Option<String>,
    /// `-c <cfg>` / `--config=<cfg>` runtime config file (None when absent).
    pub config: Option<String>,
    /// `-q` quiet.
    pub quiet: bool,
    /// `-v` verbose.
    pub verbose: bool,
    /// `-r` direct-run flag (accepted; the interpreter always runs directly).
    pub run_flag: bool,
    /// The program name (first non-option argument).
    pub program: Option<String>,
    /// Program arguments (everything after the program name).
    pub args: Vec<String>,
}

impl CobcrunArgs {
    /// Parse cobcrun argv (already without argv[0]). Option parsing stops at the first non-option
    /// argument (the program name); everything after is passed to the program verbatim.
    pub fn parse(argv: &[String]) -> Result<CobcrunArgs, String> {
        let mut out = CobcrunArgs {
            module_dir: None,
            config: None,
            quiet: false,
            verbose: false,
            run_flag: false,
            program: None,
            args: Vec::new(),
        };
        let mut i = 0usize;
        let mut after_dd = false;
        while i < argv.len() {
            let a = &argv[i];
            if after_dd || a == "-" || !a.starts_with('-') {
                // first non-option = the program name; the rest are program args
                if out.program.is_none() {
                    out.program = Some(a.clone());
                } else {
                    out.args.push(a.clone());
                }
                i += 1;
                continue;
            }
            if a == "--" {
                after_dd = true;
                i += 1;
                continue;
            }
            let (opt, attached): (String, Option<String>) =
                if let Some(v) = a.strip_prefix("--config=") {
                    ("-c".to_string(), Some(v.to_string()))
                } else if a.starts_with("-M") && a.len() > 2 {
                    // `-M<value>` attached (GCC-style short option with an attached value); `-M` alone
                    // falls through to the separated-value path below.
                    ("-M".to_string(), Some(a[2..].to_string()))
                } else if a.starts_with("-c") && a.len() > 2 && !a.starts_with("--") {
                    // `-c<value>` attached; `-c` alone uses the next argument.
                    ("-c".to_string(), Some(a[2..].to_string()))
                } else {
                    (a.clone(), None)
                };
            match opt.as_str() {
                "-M" => {
                    let attached_is_some = attached.is_some();
                    let v = match attached.as_deref() {
                        Some(v) if !v.is_empty() => v.to_string(),
                        Some(_) => {
                            return Err("invalid module argument ''".into());
                        }
                        None => match argv.get(i + 1) {
                            Some(n) if !n.is_empty() => n.clone(),
                            Some(_) => {
                                return Err("invalid module argument ''".into());
                            }
                            None => {
                                return Err("cobcrun: option requires an argument -- 'M'".into());
                            }
                        },
                    };
                    out.module_dir = Some(v);
                    i += if attached_is_some { 1 } else { 2 };
                    continue;
                }
                "-c" => {
                    let attached_is_some = attached.is_some();
                    let v = match attached.as_deref() {
                        Some(v) => v.to_string(),
                        None => match argv.get(i + 1) {
                            Some(n) => n.clone(),
                            None => {
                                return Err("cobcrun: option requires an argument -- 'c'".into());
                            }
                        },
                    };
                    out.config = Some(v);
                    i += if attached_is_some { 1 } else { 2 };
                    continue;
                }
                "-q" => {
                    out.quiet = true;
                    i += 1;
                    continue;
                }
                "-v" => {
                    out.verbose = true;
                    i += 1;
                    continue;
                }
                "-r" => {
                    out.run_flag = true;
                    i += 1;
                    continue;
                }
                _ => {
                    // cobcrun's getopt-style diagnostic for an unknown short/long option
                    if let Some(c) = a.strip_prefix("--") {
                        return Err(format!("cobcrun: unrecognized option '--{c}'"));
                    }
                    let bytes = a.as_bytes();
                    let c = bytes
                        .get(1)
                        .copied()
                        .map(|b| (b as char).to_string())
                        .unwrap_or_else(|| "?".to_string());
                    return Err(format!("cobcrun: invalid option -- '{c}'"));
                }
            }
        }
        Ok(out)
    }
}

/// Resolve + run one cobcrun module invocation.
pub fn cobcrun_run(args: &CobcrunArgs, argv0_base: &str) -> i32 {
    let program = match &args.program {
        Some(p) => p.clone(),
        None => {
            eprintln!("cobcrun: missing PROGRAM name\nTry 'cobcrun --help' for more information.");
            return 1;
        }
    };
    // module file candidates: `<dir><prog>.so` / `<dir><prog>` with the launch manifest next to it.
    let dirs: Vec<String> = match &args.module_dir {
        Some(d) => {
            let d = if d.ends_with('/') {
                d.clone()
            } else {
                format!("{d}/")
            };
            vec![d]
        }
        None => {
            let mut dirs = vec![String::new()]; // cwd
            if let Ok(lib) = std::env::var("COB_LIBRARY_PATH") {
                for p in lib.split(':').filter(|p| !p.is_empty()) {
                    dirs.push(format!("{p}/"));
                }
            }
            dirs
        }
    };
    let base = {
        let p = std::path::Path::new(&program);
        p.file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_else(|| program.clone())
    };
    for d in &dirs {
        for cand in [
            format!("{d}{base}.so.cobr.json"),
            format!("{d}{base}.cobr.json"),
        ] {
            let cp = std::path::PathBuf::from(&cand);
            if cp.is_file() {
                // program args: set the interpreter command line, then run the module manifest.
                run::set_program_args(&args.args);
                return run_launcher(&cp, argv0_base);
            }
        }
    }
    eprintln!("cobcrun: cannot find module {program}");
    1
}
