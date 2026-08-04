//! Capability introspection (`--print-capabilities`), translation explanation
//! (`--explain-translation`), and the structured invocation record (`--dump-invocation-json`) —
//! all generated from the option-policy registry (no hand-maintained duplication).

use crate::args::ParsedInvocation;
use crate::policy::{self, OptCategory, OptionPolicy};
use serde_json::{json, Value};

/// The machine-readable capability map (every registry entry with its policy).
#[allow(dead_code)]
pub fn capabilities_json() -> Value {
    let entries: Vec<Value> = policy::registry()
        .iter()
        .map(|e| {
            json!({
                "option": e.canonical,
                "aliases": e.aliases,
                "policy": e.policy.as_str(),
                "category": e.category.as_str(),
                "consumes_value": e.consumes_value,
                "justification": e.justification,
            })
        })
        .collect();
    json!({
        "schema": "gnucobol-rs-capabilities-v1",
        "candidate": {
            "name": "cobc-rs",
            "interpreter": "gnucobol-rs front end (no native code generation)",
            "links_libcob": false,
            "invokes_cobc": false,
        },
        "supported_modes": ["executable-launch-artifact", "module-launch-artifact", "syntax-only", "preprocess", "dependency", "info"],
        "supported_dialects": ["default", "ibm", "mf", "mvs", "cobol85", "cobol2002", "cobol2014", "conf-files"],
        "compat_modes": ["strict", "gnucobol-testsuite"],
        "options": entries,
    })
}

/// Human-readable `--print-capabilities` summary.
pub fn print_capabilities() -> String {
    let mut out = String::from(
        "cobc-rs capabilities (generated from the option-policy registry)\n\n\
         candidate: gnucobol-rs interpreter; NO native code generation; never links libcob; never invokes cobc\n\n",
    );
    for (policy, label) in [
        (OptionPolicy::Translated, "translated"),
        (OptionPolicy::AcceptedEquivalent, "accepted-equivalent"),
        (OptionPolicy::AcceptedProvenNoOp, "accepted-proven-no-op"),
        (OptionPolicy::RejectedUnsupported, "rejected-unsupported"),
        (OptionPolicy::RejectedAmbiguous, "rejected-ambiguous"),
    ] {
        let opts: Vec<&str> = policy::registry()
            .iter()
            .filter(|e| e.policy == policy)
            .map(|e| e.canonical)
            .collect();
        out.push_str(&format!(
            "\n== {label} ({}):\n  {}\n",
            opts.len(),
            opts.join(" ")
        ));
    }
    out
}

/// `--explain-translation`: the policy for each option in the given argv.
pub fn explain(argv: &[String]) -> String {
    let mut out = String::new();
    for raw in argv {
        let (key, attached) = policy::split_attached(raw);
        if !raw.starts_with('-') {
            out.push_str(&format!("{raw}: source file (not an option)\n"));
            continue;
        }
        match policy::lookup(&key) {
            Some(e) => {
                out.push_str(&format!(
                    "{}: {} [{}] {}{}\n",
                    raw,
                    e.policy.as_str(),
                    e.category.as_str(),
                    e.justification,
                    attached
                        .map(|v| format!(" (value {v:?})").to_string())
                        .unwrap_or_default()
                ));
            }
            None => out.push_str(&format!(
                "{}: REJECTED-AMBIGUOUS (no registry entry — fail closed)\n",
                raw
            )),
        }
    }
    out
}

/// The structured invocation record (`--dump-invocation-json`): raw argv, normalized, translated
/// options, no-ops, rejected, sources, output, mode — with container-internal cwd aliasing left to
/// the privacy sanitizer (raw record kept outside git by the orchestrator).
pub fn invocation_record(
    argv: &[String],
    inv: &Result<ParsedInvocation, crate::args::ArgError>,
) -> Value {
    let (ok, parsed) = match inv {
        Ok(p) => (true, Some(p)),
        Err(_) => (false, None),
    };
    let rejected: Vec<String> = parsed
        .map(|p| p.rejected.clone())
        .unwrap_or_else(|| argv.to_vec());
    let mut noops: Vec<String> = parsed.map(|p| p.noops.clone()).unwrap_or_default();
    noops.sort();
    noops.dedup();
    json!({
        "schema": "gnucobol-rs-invocation-record-v1",
        "raw_argv": argv,
        "parse_ok": ok,
        "mode": parsed.map(|p| p.mode.as_str().to_string()).unwrap_or_else(|| "rejected".into()),
        "compat": parsed.map(|p| p.compat.as_str().to_string()).unwrap_or_else(|| "strict".into()),
        "sources": parsed.map(|p| p.sources.clone()).unwrap_or_default(),
        "output": parsed.and_then(|p| p.output.clone()),
        "dialect": parsed.and_then(|p| p.dialect.clone()),
        "format": parsed.and_then(|p| p.format.clone()),
        "includes": parsed.map(|p| p.includes.clone()).unwrap_or_default(),
        "defines": parsed.map(|p| p.defines.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>()).unwrap_or_default(),
        "noop_options": noops,
        "rejected_options": rejected,
        "candidate_commit": env!("CARGO_PKG_VERSION"),
    })
}

impl OptCategory {
    #[allow(dead_code)]
    fn desc(&self) -> &'static str {
        self.as_str()
    }
}
