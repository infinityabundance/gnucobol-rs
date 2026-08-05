//! Argument parsing: `cobc`-shaped argv -> a `ParsedInvocation`, driven by the policy registry.
//! Every option token is matched to an explicit policy; unsupported/unknown options fail closed.

use crate::policy::{self, CompatMode, Mode};
use std::collections::BTreeMap;

/// The parsed, normalized invocation.
#[derive(Debug, Clone, Default)]
pub struct ParsedInvocation {
    /// Sources (non-option arguments; `-` = stdin).
    pub sources: Vec<String>,
    /// Output path from `-o` (default: derived from the first source basename).
    pub output: Option<String>,
    /// `-std=` dialect name.
    pub dialect: Option<String>,
    /// `-conf=` config file name.
    pub conf: Option<String>,
    /// `-free` / `-fixed` / `-fformat=` resolution ("fixed" | "free" | other).
    pub format: Option<String>,
    /// `-I` include paths (repeatable).
    pub includes: Vec<String>,
    /// `-ext=` copybook extension(s).
    pub extensions: Vec<String>,
    /// `--copy <file>` copybooks prepended to each source before preprocessing (upstream e36a124b2).
    pub copy_files: Vec<String>,
    /// `-fdiagnostics-absolute-path`: render the source-file prefix of diagnostics as absolute.
    pub diag_absolute_path: bool,
    /// `-D` defines (name -> value).
    pub defines: BTreeMap<String, String>,
    /// `-fdefaultbyte=` overlays applied on top of the dialect (config-line form).
    pub conf_overrides: Vec<(String, String)>,
    /// The compile mode requested.
    pub mode: Mode,
    /// `-MF` dependency file.
    pub depfile: Option<String>,
    /// `-MT` dependency targets.
    pub deptargets: Vec<String>,
    /// Proven no-op options accepted (invocation ledger).
    pub noops: Vec<String>,
    /// Rejected options (kept for the ledger even when parsing fails).
    pub rejected: Vec<String>,
    /// Driver-level introspection requests.
    pub explain: bool,
    pub capabilities: bool,
    pub dump_invocation_json: Option<String>,
    /// `--info` / `--version` / `--runtime-conf` / `--help` request, when in Info mode.
    pub info_request: Option<String>,
    pub compat: CompatMode,
}

/// A parse error: the offending token + a stable reason.
#[derive(Debug, Clone)]
pub struct ArgError {
    pub option: String,
    pub message: String,
}

impl core::fmt::Display for ArgError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "cobc-rs: {}: {}", self.option, self.message)
    }
}

/// GCC-style short-option attached value: `-DNAME` -> `("-D", "NAME")`, `-Ipath` -> `("-I", "path")`.
/// Only fires when the 2-char key is a registry entry that consumes a value; other tokens return
/// `(raw, None)` unchanged (e.g. `-Wno-unsupported` stays whole for the `-W` family fallback).
fn split_short_attached(raw: &str) -> (String, Option<String>) {
    let b = raw.as_bytes();
    if raw.len() > 2
        && raw.starts_with('-')
        && !raw.starts_with("--")
        && b[1].is_ascii_alphabetic()
        && !raw.contains('=')
    {
        let key = raw[..2].to_string();
        if policy::lookup(&key)
            .map(|e| e.consumes_value)
            .unwrap_or(false)
        {
            return (key, Some(raw[2..].to_string()));
        }
    }
    (raw.to_string(), None)
}

/// Parse `argv` (already without argv[0]).
pub fn parse(argv: &[String]) -> Result<ParsedInvocation, ArgError> {
    let mut p = ParsedInvocation {
        compat: CompatMode::Strict,
        ..Default::default()
    };
    // scan for the compatibility-mode switch first (any position)
    for a in argv {
        match a.as_str() {
            "--compat=gnucobol-testsuite" => p.compat = CompatMode::GnucobolTestsuite,
            "--compat=strict" | "--strict-args" => p.compat = CompatMode::Strict,
            _ => {}
        }
    }
    let mut i = 0usize;
    while i < argv.len() {
        let raw = argv[i].clone();
        if raw == "--" {
            for s in &argv[i + 1..] {
                p.sources.push(s.clone());
            }
            break;
        }
        if raw == "-" || !raw.starts_with('-') {
            p.sources.push(raw);
            i += 1;
            continue;
        }
        // driver-level introspection flags
        if raw == "--explain-translation" {
            p.explain = true;
            i += 1;
            continue;
        }
        if raw == "--print-capabilities" {
            p.capabilities = true;
            i += 1;
            continue;
        }
        if let Some(v) = raw.strip_prefix("--dump-invocation-json=") {
            p.dump_invocation_json = Some(v.to_string());
            i += 1;
            continue;
        }
        if raw == "--dump-invocation-json" {
            p.dump_invocation_json = Some(
                argv.get(i + 1)
                    .ok_or_else(|| ArgError {
                        option: raw.clone(),
                        message: "--dump-invocation-json needs a path".into(),
                    })?
                    .clone(),
            );
            i += 2;
            continue;
        }

        let (opt_key, attached) = policy::split_attached(&raw);
        // GCC-style short-option attached values: `-DNAME`, `-Ipath`, `-MFfile`, `-otarget` …
        // (no `=`). Only when the 2-char key is a registry entry that CONSUMES a value; every other
        // token keeps its exact spelling for the lookup.
        let (opt_key, attached) = if attached.is_none() {
            split_short_attached(&raw)
        } else {
            (opt_key, attached)
        };
        // the compatibility-mode switch was consumed by the pre-scan; skip it here (it is not an
        // option with a policy -- it selects the policy regime).
        if matches!(
            raw.as_str(),
            "--compat=gnucobol-testsuite" | "--compat=strict" | "--strict-args"
        ) {
            i += 1;
            continue;
        }
        let entry = policy::lookup(&opt_key).ok_or_else(|| ArgError {
            option: raw.clone(),
            message:
                "unknown option (no explicit policy; failing closed — see --print-capabilities)"
                    .into(),
        })?;
        // value for consumes_value options: attached form or the next token
        let mut value: Option<String> = attached.clone();
        let mut consumed_next = false;
        if value.is_none() && entry.consumes_value {
            if let Some(nxt) = argv.get(i + 1) {
                if !nxt.starts_with('-') || nxt == "-" {
                    value = Some(nxt.clone());
                    consumed_next = true;
                }
            }
        }

        match entry.policy {
            policy::OptionPolicy::Translated | policy::OptionPolicy::AcceptedEquivalent => {
                apply_translated(&mut p, entry.canonical, &raw, value.as_deref())?;
            }
            policy::OptionPolicy::AcceptedProvenNoOp => {
                p.noops.push(entry.canonical.to_string());
            }
            policy::OptionPolicy::RejectedUnsupported | policy::OptionPolicy::RejectedAmbiguous => {
                p.rejected.push(raw.clone());
                return Err(ArgError {
                    option: raw.clone(),
                    message: format!(
                        "unsupported option ({}; {}): {}",
                        entry.policy.as_str(),
                        entry.justification,
                        value.as_deref().unwrap_or("")
                    ),
                });
            }
        }
        i += if consumed_next { 2 } else { 1 };
    }
    Ok(p)
}

fn apply_translated(
    p: &mut ParsedInvocation,
    canonical: &str,
    raw: &str,
    value: Option<&str>,
) -> Result<(), ArgError> {
    match canonical {
        "-x" => p.mode = Mode::Executable,
        "-m" => p.mode = Mode::Module,
        "-E" => p.mode = Mode::Preprocess,
        "-M" => p.mode = Mode::Dependency,
        "-fsyntax-only" => p.mode = Mode::SyntaxOnly,
        "-o" => p.output = value.map(|v| v.to_string()),
        "-std" => p.dialect = value.map(|v| v.to_string()),
        "-conf" => p.conf = value.map(|v| v.to_string()),
        "-free" => p.format = Some("free".to_string()),
        "-fixed" => p.format = Some("fixed".to_string()),
        "-fformat" => {
            p.format = value.map(|v| v.to_string());
        }
        "-I" => {
            if let Some(v) = value {
                p.includes.push(v.to_string());
            }
        }
        "-ext" => {
            if let Some(v) = value {
                p.extensions.push(v.trim_start_matches('.').to_string());
            }
        }
        "--copy" | "-copy" => {
            // Upstream e36a124b2: --copy <file> pre-copies the copybook text before each source.
            if let Some(v) = value {
                p.copy_files.push(v.to_string());
            }
        }
        "-fdiagnostics-absolute-path" => p.diag_absolute_path = true,
        "-D" => {
            if let Some(v) = value {
                let (name, val) = match v.split_once('=') {
                    Some((n, vv)) => (n.to_string(), vv.to_string()),
                    None => (v.to_string(), String::new()),
                };
                p.defines.insert(name.to_ascii_uppercase(), val);
            }
        }
        "-MF" => p.depfile = value.map(|v| v.to_string()),
        "-MT" => {
            if let Some(v) = value {
                p.deptargets.push(v.to_string());
            }
        }
        "-fdefaultbyte" => {
            if let Some(v) = value {
                p.conf_overrides
                    .push(("defaultbyte".to_string(), v.to_string()));
            }
        }
        "-fprof" => {
            // upstream 7b6995042: -fprof gates the generated profiling calls; the candidate's
            // paragraph hooks are always present and COB_PROF_ENABLE activates them, so the flag
            // is recorded (translated) and needs no per-invocation state.
        }
        "--version" | "--info" | "--dumpversion" | "--runtime-conf" | "--help" => {
            p.mode = Mode::Info;
            p.info_request = Some(canonical.to_string());
        }
        // every other translated option is handled above; anything else reaching here means the
        // registry changed without the translator — fail loudly rather than silently.
        other => {
            return Err(ArgError {
                option: raw.to_string(),
                message: format!("internal: translated option {other} has no translator"),
            });
        }
    }
    Ok(())
}
