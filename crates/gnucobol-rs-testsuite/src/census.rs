//! Invocation census: parses the baseline recorder's JSONL (one line per `cobc`/`cobcrun`
//! invocation, argv boundaries preserved) and produces:
//!   invocation-census.json   — every invocation (cwd aliased, env subset)
//!   options-frequency.csv    — per-option-token frequency
//!   option-coverage.md       — per-option policy classification (semantic categories)
//!
//! Option categories follow the prompt (§0.3): semantic source option, dialect, source format,
//! include/copybook, preprocessor definition, output selection, compile/link mode, runtime/module,
//! diagnostic-only, optimization/debugging, test-harness-only, unknown.

use crate::model::{Invocation, OptionCategory};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

pub fn parse_census(path: &Path) -> Result<Vec<Invocation>, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read census {}: {e}", path.display()))?;
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let v: Value = serde_json::from_str(line)
            .map_err(|e| format!("census line {} malformed: {e}", i + 1))?;
        let inv: Invocation =
            serde_json::from_value(v).map_err(|e| format!("census line {} shape: {e}", i + 1))?;
        out.push(inv);
    }
    Ok(out)
}

/// The tool name from argv[0]'s basename (`cobc`, `cobcrun`, …).
pub fn tool_name(inv: &Invocation) -> String {
    if !inv.tool.is_empty() {
        return inv.tool.clone();
    }
    inv.argv
        .first()
        .map(|a| {
            Path::new(a)
                .file_name()
                .map(|f| f.to_string_lossy().into_owned())
                .unwrap_or_else(|| a.clone())
        })
        .unwrap_or_else(|| "?".to_string())
}

/// Split an argv (without argv[0]) into option tokens (each leading-dash arg; attached values like
/// `-o` followed by a non-dash arg stay separate — the census keeps the RAW argv, so options with
/// separate values are counted by the option token alone; `-std=...` counts under `-std=`).
pub fn option_tokens(argv: &[String]) -> Vec<String> {
    argv.iter()
        .skip(1) // argv[0] is the tool name
        .filter(|a| a.starts_with('-') && a.len() > 1)
        .map(|a| {
            if let Some(eq) = a.find('=') {
                a[..eq + 1].to_string()
            } else {
                a.clone()
            }
        })
        .collect()
}

/// Classify one option token into a semantic category (prompt §0.3). This is the *census-side*
/// classification; the cobc-rs adapter's own policy registry is the authoritative per-option policy
/// (translated / accepted-noop / rejected).
pub fn classify_option(opt: &str) -> OptionCategory {
    let o = opt.trim_end_matches('=');
    match o {
        // compile/link mode
        "-x" | "-m" | "-c" | "-S" | "-C" | "-E" | "-M" | "-MT" | "-MF" | "-fsyntax-only" => {
            OptionCategory::CompileLinkMode
        }
        // dialect
        s if s.starts_with("-std=") || s.starts_with("-std") => OptionCategory::Dialect,
        // source format
        "-free" | "-fixed" | "-fformat" | "-ffixed" | "-ffixed-line-length" | "-ftab-width" => {
            OptionCategory::SourceFormat
        }
        // include / copybook
        "-I" | "-i" | "-ext" | "-fresolve-locate" => OptionCategory::IncludeCopybook,
        // preprocessor definitions
        "-D" | "-U" | "-d" | "-conf" => OptionCategory::Preprocessor,
        // output selection
        "-o" | "-t" | "-T" | "-save-temps" | "-foutput" => OptionCategory::OutputSelection,
        // runtime / module
        "-j" | "-e" | "-W" | "-fmodule" | "-fnot-reserved" | "-fstack" => {
            OptionCategory::RuntimeModule
        }
        // diagnostic-only
        s if s.starts_with("-Wall")
            || s.starts_with("-Wno")
            || s.starts_with("-W")
            || s.starts_with("-fdiagnostics")
            || s.starts_with("-fno-diagnostics") =>
        {
            OptionCategory::Diagnostic
        }
        // optimization / debugging
        "-O" | "-O0" | "-O1" | "-O2" | "-O3" | "-g" | "-debug" | "-fdebug" | "-ftrace" | "-fec"
        | "-fec-" => OptionCategory::OptimizationDebug,
        // test-harness-only
        "-fgen-c-line-directives"
        | "-fgen-c-labels"
        | "-fno-ttimestamp"
        | "-fttitle"
        | "-fno-diagnostics-show-option"
        | "-fmsgfmt"
        | "-funchecked"
        | "-fstatic-call" => OptionCategory::TestHarness,
        _ => OptionCategory::Unknown,
    }
}

pub fn category_str(c: OptionCategory) -> &'static str {
    match c {
        OptionCategory::Semantic => "semantic",
        OptionCategory::Dialect => "dialect",
        OptionCategory::SourceFormat => "source-format",
        OptionCategory::IncludeCopybook => "include-copybook",
        OptionCategory::Preprocessor => "preprocessor",
        OptionCategory::OutputSelection => "output-selection",
        OptionCategory::CompileLinkMode => "compile-link-mode",
        OptionCategory::RuntimeModule => "runtime-module",
        OptionCategory::Diagnostic => "diagnostic",
        OptionCategory::OptimizationDebug => "optimization-debug",
        OptionCategory::TestHarness => "test-harness",
        OptionCategory::Unknown => "unknown",
    }
}

/// Build the three census artifacts. `out` receives `invocation-census.json`,
/// `options-frequency.csv` and `option-coverage.md`. `cwd_root` is the container build-tree root
/// used to alias per-invocation cwds (`<tree>/tests/testsuite.dir/<NN>` -> `<tree>/...` stays, the
/// full container path is preserved under the raw key).
pub fn generate(census_path: &Path, out: &Path, pass: &str) -> Result<(), String> {
    let invs = parse_census(census_path)?;
    let mut freq: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_tool: BTreeMap<String, usize> = BTreeMap::new();
    let mut category_freq: BTreeMap<String, usize> = BTreeMap::new();
    let mut uniq_opts: BTreeMap<String, OptionCategory> = BTreeMap::new();

    for inv in &invs {
        *by_tool.entry(tool_name(inv)).or_insert(0) += 1;
        for tok in option_tokens(&inv.argv[1..]) {
            *freq.entry(tok.clone()).or_insert(0) += 1;
            let cat = classify_option(&tok);
            uniq_opts.entry(tok.clone()).or_insert(cat);
            *category_freq
                .entry(category_str(cat).to_string())
                .or_insert(0) += 1;
        }
    }

    let census_json = json!({
        "schema": "gnurust-gnucobol-testsuite-invocation-census-v1",
        "pass": pass,
        "total_invocations": invs.len(),
        "by_tool": by_tool,
        "invocations": invs.iter().map(|inv| json!({
            "t": inv.t,
            "cwd": inv.cwd,
            "tool": tool_name(inv),
            "argv": inv.argv,
            "env": inv.env,
        })).collect::<Vec<_>>(),
    });
    std::fs::write(
        out.join("invocation-census.json"),
        serde_json::to_string_pretty(&census_json).unwrap(),
    )
    .map_err(|e| format!("write invocation-census.json: {e}"))?;

    // options-frequency.csv: option,count,category
    let mut csv = String::from("option,count,category\n");
    for (opt, count) in &freq {
        csv.push_str(&format!("{opt},{count},{}\n", category_str(uniq_opts[opt])));
    }
    std::fs::write(out.join("options-frequency.csv"), csv)
        .map_err(|e| format!("write options-frequency.csv: {e}"))?;

    // option-coverage.md: per unique option: category + policy note (policy itself is authoritative
    // in the cobc-rs registry; this table is the observed census view).
    let mut md = String::from("# Observed GnuCOBOL-testsuite option census\n\n");
    md.push_str(&format!(
        "{} recorded invocations (tools: {}). Every unique option token classified by semantic category; \
         the authoritative per-option policy (translated / proven-no-op / rejected) lives in the `cobc-rs` option registry.\n\n",
        invs.len(),
        by_tool.iter().map(|(t, n)| format!("{t}×{n}")).collect::<Vec<_>>().join(", ")
    ));
    md.push_str("| option | count | category |\n|---|---|---|\n");
    for (opt, count) in &freq {
        md.push_str(&format!(
            "| `{opt}` | {count} | {} |\n",
            category_str(uniq_opts[opt])
        ));
    }
    md.push_str("\n## Category totals\n\n");
    for (cat, n) in &category_freq {
        md.push_str(&format!("- {cat}: {n}\n"));
    }
    std::fs::write(out.join("option-coverage.md"), md)
        .map_err(|e| format!("write option-coverage.md: {e}"))?;

    Ok(())
}
