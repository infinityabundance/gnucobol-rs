//! GNURUST.CCVS85.4 — differential comparison and classification.
//!
//! Compares the real GnuCOBOL outcome with the gnucobol-rs outcome for every indexed unit,
//! classifies each unit into exactly one primary [`FinalClassification`], and produces the
//! machine-readable + human-readable summaries, preserving raw evidence and separating:
//! oracle rejection / candidate rejection / candidate defect / intentionally unsupported /
//! harness limitation / missing library / missing data / timeout / nondeterminism / output
//! mismatch / exit-status mismatch / infrastructure failure.
//!
//! Comparison layers (each applied identically to both sides, documented + versioned):
//!   1. raw byte comparison of the primary output (oracle report file vs candidate stdout);
//!   2. canonical comparison (documented normalization — see [`canonicalize`]);
//!   3. CCVS85 verdict-count comparison (parsed PASS/FAIL/DELETED/INSPECT/INFO counts).

use crate::model::{
    section_of, CandidateSide, ComparisonSide, FinalClassification, Invocation, MaterializedUnit,
    OracleSide, Summary, UnitResult, VerdictCounts,
};
use crate::runner::read_bytes;
use std::collections::BTreeMap;
use std::path::Path;

/// Canonicalization schema version — bump ONLY when the normalization rules change (and re-run the
/// whole court; the version is recorded in every comparison result).
pub const CANONICAL_SCHEMA: &str = "gnurust-ccvs85-canonical-v1";

/// Documented symmetric canonicalization. Rules:
///   1. split on '\n';
///   2. strip trailing whitespace (spaces/tabs/CR) from each line;
///   3. drop trailing blank lines;
///   4. normalize any run of blank lines to a single blank line (paragraph collapses);
///   5. keep everything else byte-exact (no case folding, no digit normalization, no sort).
///
/// Applied identically to oracle and candidate outputs. Versioned by [`CANONICAL_SCHEMA`].
pub fn canonicalize(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let text = String::from_utf8_lossy(bytes);
    let mut blank = false;
    for line in text.split('\n') {
        let trimmed = line.trim_end_matches([' ', '\t', '\r']);
        if trimmed.is_empty() {
            if !blank {
                out.push(b'\n');
                blank = true;
            }
        } else {
            out.extend_from_slice(trimmed.as_bytes());
            out.push(b'\n');
            blank = false;
        }
    }
    // drop the trailing blank line(s)
    while out.last() == Some(&b'\n') {
        out.pop();
    }
    if !out.is_empty() {
        out.push(b'\n');
    }
    out
}

/// The oracle's primary output for comparison: the CCVS85 report file (PRINT-FILE) — the suite's
/// intended result channel. When no report was produced, the run stdout is used.
pub fn oracle_primary_output(side: &OracleSide, work_root: &Path) -> Vec<u8> {
    if let Some(run) = &side.run_invocation {
        if let Some(ev) = &run.stdout_path {
            let ev_dir = Path::new(ev).parent().unwrap_or(work_root);
            let report = ev_dir.join("REPORT");
            if report.exists() {
                let b = read_bytes(&report);
                if !b.is_empty() {
                    return b;
                }
            }
            return read_bytes(Path::new(ev));
        }
    }
    Vec::new()
}

/// The candidate's primary output: the materialized file store's report (PRINT-FILE) when the unit
/// wrote one -- mirrored from the oracle side (`--dump-files` puts the files in the run dir beside the
/// evidence dir) -- else cobrun stdout.
pub fn candidate_primary_output(side: &CandidateSide) -> Vec<u8> {
    if let Some(inv) = &side.run_invocation {
        if let Some(p) = &inv.stdout_path {
            let run_dir = Path::new(p)
                .parent()
                .and_then(|e| e.parent())
                .unwrap_or(Path::new("."));
            for name in ["REPORT", "XXXXX055"] {
                let report = run_dir.join(name);
                if report.exists() {
                    let b = read_bytes(&report);
                    if !b.is_empty() {
                        return b;
                    }
                }
            }
            return read_bytes(Path::new(p));
        }
    }
    Vec::new()
}

/// Determine the classification reason code from the candidate's first cobrun message.
pub fn candidate_reason_code(side: &CandidateSide) -> String {
    if let Some(inv) = &side.prepare_invocation {
        if let Some(p) = &inv.stderr_path {
            let f = crate::runner::first_line(Path::new(p));
            if !f.is_empty() {
                return summarize_reason(&f);
            }
        }
    }
    "CANDIDATE_REJECTED".to_string()
}

/// Compress a diagnostics line into a stable, bucketing reason code (e.g. the first construct the
/// front-end refused). Keeps the raw line in `first_failure_line` for audit.
pub fn summarize_reason(line: &str) -> String {
    let low = line.to_ascii_uppercase();
    let words: Vec<String> = low
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| w.len() >= 3)
        .map(|w| w.to_string())
        .collect();
    if words.is_empty() {
        return "UNCLASSIFIED".to_string();
    }
    // join the first 8 significant words to form a stable key
    let key = words.iter().take(8).cloned().collect::<Vec<_>>().join("_");
    key
}

/// The oracle compile-reject reason (first cobc diagnostics line, summarized). The stderr can
/// open with a cc1/-fpermissive warning before the real cobc error, so the first line containing
/// `error` is used when present.
pub fn oracle_reject_reason(inv: &Invocation) -> String {
    if let Some(p) = &inv.stderr_path {
        if let Ok(text) = std::fs::read_to_string(p) {
            for line in text.lines() {
                if line.to_ascii_lowercase().contains("error") {
                    let f = line.trim();
                    if !f.is_empty() {
                        return summarize_reason(f);
                    }
                }
            }
        }
        let f = crate::runner::first_line(Path::new(p));
        if !f.is_empty() {
            return summarize_reason(&f);
        }
    }
    "COBC_REJECTED".to_string()
}

/// Build the full per-unit comparison + classification.
pub fn classify_unit(
    unit: &MaterializedUnit,
    oracle: &OracleSide,
    candidate: &CandidateSide,
    work_root: &Path,
) -> UnitResult {
    let mut r = UnitResult {
        unit_index: unit.unit_index,
        kind: unit.kind.clone(),
        name: unit.name.clone(),
        source_path: unit.source_path.clone(),
        source_sha256: unit.source_sha256.clone(),
        oracle: oracle.clone(),
        candidate: candidate.clone(),
        comparison: ComparisonSide::default(),
        final_classification: FinalClassification::InfrastructureError,
        reason_code: String::new(),
        nondeterministic: false,
        determinism: None,
        first_failure_line: String::new(),
    };

    // ---- non-executable kinds -------------------------------------------------------------
    if unit.kind == "CLBRY" {
        r.final_classification = FinalClassification::NonExecutableLibrary;
        r.reason_code = "LIBRARY_TEXT_UNIT".to_string();
        r.comparison.raw_stdout = "not_comparable".into();
        r.comparison.canonical_stdout = "not_comparable".into();
        r.comparison.generated_files = "not_comparable".into();
        r.comparison.exit_status = "not_comparable".into();
        r.comparison.verdict_counts = "not_comparable".into();
        return r;
    }
    if unit.kind == "DATA*" {
        r.final_classification = FinalClassification::NonExecutableData;
        r.reason_code = "DATA_UNIT".to_string();
        r.comparison.raw_stdout = "not_comparable".into();
        r.comparison.canonical_stdout = "not_comparable".into();
        r.comparison.generated_files = "not_comparable".into();
        r.comparison.exit_status = "not_comparable".into();
        r.comparison.verdict_counts = "not_comparable".into();
        return r;
    }
    // SUBRTN-only units: exercised through their main; not independently executable.
    if unit.subprogram.is_some() {
        r.final_classification = FinalClassification::NonExecutableLibrary;
        r.reason_code = "SUBPROGRAM_BOUND_TO_MAIN".to_string();
        r.comparison.raw_stdout = "not_comparable".into();
        r.comparison.canonical_stdout = "not_comparable".into();
        r.comparison.generated_files = "not_comparable".into();
        r.comparison.exit_status = "not_comparable".into();
        r.comparison.verdict_counts = "not_comparable".into();
        return r;
    }

    // ---- dependency-blocked ---------------------------------------------------------------
    if oracle.compile == "dependency-blocked" || candidate.prepare == "dependency-blocked" {
        r.final_classification = FinalClassification::DependencyBlocked;
        r.reason_code = format!("MISSING_COPYBOOK_{}", unit.missing_copybooks.join("_"));
        r.comparison.raw_stdout = "not_comparable".into();
        r.comparison.canonical_stdout = "not_comparable".into();
        r.comparison.generated_files = "not_comparable".into();
        r.comparison.exit_status = "not_comparable".into();
        r.comparison.verdict_counts = "not_comparable".into();
        return r;
    }

    // ---- oracle compile outcome ------------------------------------------------------------
    if oracle.compile == "timeout"
        || oracle
            .compile_invocation
            .as_ref()
            .map(|i| i.timed_out)
            .unwrap_or(false)
    {
        r.final_classification = FinalClassification::InfrastructureError;
        r.reason_code = "ORACLE_COMPILE_TIMEOUT".to_string();
        return r;
    }
    if oracle.compile == "reject" {
        let inv = oracle
            .compile_invocation
            .as_ref()
            .cloned()
            .unwrap_or_default();
        r.final_classification = FinalClassification::OracleCompileReject;
        r.reason_code = oracle_reject_reason(&inv);
        r.first_failure_line = oracle_reject_reason(&inv);
        r.comparison.raw_stdout = "not_comparable".into();
        r.comparison.canonical_stdout = "not_comparable".into();
        r.comparison.generated_files = "not_comparable".into();
        r.comparison.exit_status = "not_comparable".into();
        r.comparison.verdict_counts = "not_comparable".into();
        return r;
    }
    if oracle.compile == "error" {
        r.final_classification = FinalClassification::OracleCompileError;
        r.reason_code = "COBC_CRASHED".to_string();
        return r;
    }
    if oracle.compile == "harness-blocked" || oracle.run == "harness-blocked" {
        r.final_classification = FinalClassification::HarnessBlocked;
        r.reason_code = "EXEC85_DRIVER_REQUIRES_MODULE_LIBRARY".to_string();
        return r;
    }
    if oracle.compile != "pass" {
        r.final_classification = FinalClassification::InfrastructureError;
        r.reason_code = format!(
            "UNEXPECTED_ORACLE_COMPILE_STATE_{}",
            oracle.compile.to_ascii_uppercase()
        );
        return r;
    }

    // ---- oracle run outcome (compile passed, executable candidate) -------------------------
    let oracle_run = oracle.run_invocation.as_ref().cloned().unwrap_or_default();
    let oracle_rc = oracle_run.exit_code;
    if oracle.run == "timeout" || oracle_run.timed_out {
        r.final_classification = FinalClassification::OracleTimeout;
        r.reason_code = "ORACLE_RUN_TIMEOUT".to_string();
        return r;
    }
    if oracle.run != "pass" {
        // oracle ran but failed (nonzero exit / runtime error)
        r.final_classification = FinalClassification::OracleRunFail;
        r.reason_code = if oracle_rc.map(|c| c >= 128).unwrap_or(false) {
            "ORACLE_RUN_SIGNAL".to_string()
        } else {
            "ORACLE_RUN_NONZERO_EXIT".to_string()
        };
        r.first_failure_line = crate::runner::first_line(
            oracle_run
                .stderr_path
                .as_deref()
                .map(Path::new)
                .unwrap_or_else(|| Path::new("/dev/null")),
        );
        if r.first_failure_line.is_empty() {
            r.first_failure_line = format!("exit {}", oracle_rc.unwrap_or(-1));
        }
        return r;
    }

    // ---- candidate outcome ----------------------------------------------------------------
    let candidate_rc = candidate.prepare_invocation_rc;
    match candidate.prepare.as_str() {
        "reject-unsupported" => {
            r.final_classification = FinalClassification::RustRejectUnsupported;
            r.reason_code = candidate_reason_code(candidate);
            r.first_failure_line = candidate_first_line(candidate);
            set_not_comparable(&mut r);
            return r;
        }
        "reject-parse" => {
            r.final_classification = FinalClassification::RustRejectParse;
            r.reason_code = candidate_reason_code(candidate);
            r.first_failure_line = candidate_first_line(candidate);
            set_not_comparable(&mut r);
            return r;
        }
        "reject-layout" => {
            r.final_classification = FinalClassification::RustRejectLayout;
            r.reason_code = candidate_reason_code(candidate);
            r.first_failure_line = candidate_first_line(candidate);
            set_not_comparable(&mut r);
            return r;
        }
        "reject-runtime-boundary" => {
            r.final_classification = FinalClassification::RustRejectRuntimeBoundary;
            r.reason_code = candidate_reason_code(candidate);
            r.first_failure_line = candidate_first_line(candidate);
            set_not_comparable(&mut r);
            return r;
        }
        "bound-to-main" | "dependency-blocked" => {
            // handled above for dependency-blocked; bound-to-main is unreachable for executables
            r.final_classification = FinalClassification::InfrastructureError;
            r.reason_code = "UNREACHABLE".to_string();
            return r;
        }
        _ => {}
    }

    // candidate accepted: compare observables
    if candidate.run == "timeout" || candidate_rc == Some(124) {
        r.final_classification = FinalClassification::RustTimeout;
        r.reason_code = "CANDIDATE_RUN_TIMEOUT".to_string();
        return r;
    }

    let oracle_out = oracle_primary_output(oracle, work_root);
    let cand_out = candidate_primary_output(candidate);
    let o_canon = canonicalize(&oracle_out);
    let c_canon = canonicalize(&cand_out);

    let raw_match = oracle_out == cand_out;
    let canon_match = o_canon == c_canon;
    let exit_match = oracle_rc == candidate_rc;
    // generated files: oracle's run dir files (report excluded — it IS the primary output)
    let o_files = crate::oracle::generated_files(
        &work_root.join(format!("u{}", unit.unit_index)).join("run"),
    );
    let c_files = candidate_generated_files(candidate, work_root, unit.unit_index);
    let gen_match = o_files == c_files;

    r.comparison.raw_stdout = if raw_match { "match" } else { "mismatch" }.into();
    r.comparison.canonical_stdout = if canon_match { "match" } else { "mismatch" }.into();
    r.comparison.generated_files = if gen_match { "match" } else { "mismatch" }.into();
    r.comparison.exit_status = if exit_match { "match" } else { "mismatch" }.into();

    // verdict counts comparison
    let o_counts = oracle.verdict_counts.clone();
    let c_counts = candidate_verdict_counts(candidate, work_root, unit.unit_index);
    r.comparison.oracle_counts = o_counts.clone();
    r.comparison.candidate_counts = c_counts.clone();
    r.comparison.verdict_counts = match (&o_counts, &c_counts) {
        (Some(a), Some(b)) if a == b => "match".to_string(),
        (Some(_), Some(_)) => "mismatch".to_string(),
        _ => "not_comparable".to_string(),
    };

    // primary classification priority (deepest observable first)
    if candidate.run == "fail" {
        r.final_classification = FinalClassification::RustAcceptButRuntimeFail;
        r.reason_code = "CANDIDATE_RUNTIME_FAIL".to_string();
        return r;
    }
    if !gen_match {
        r.final_classification = FinalClassification::GeneratedFileMismatch;
        r.reason_code = "GENERATED_FILES_DIFFER".to_string();
        return r;
    }
    if raw_match {
        r.final_classification = FinalClassification::RawOutputMatch;
        r.reason_code = "RAW_OUTPUT_IDENTICAL".to_string();
        return r;
    }
    if canon_match {
        r.final_classification = FinalClassification::CanonicalOutputMatch;
        r.reason_code = "CANONICAL_OUTPUT_IDENTICAL".to_string();
        return r;
    }
    if !exit_match {
        r.final_classification = FinalClassification::ExitStatusMismatch;
        r.reason_code = format!(
            "EXIT_ORACLE_{}_CANDIDATE_{}",
            oracle_rc.unwrap_or(-1),
            candidate_rc.unwrap_or(-1)
        );
        return r;
    }
    r.final_classification = FinalClassification::OutputMismatch;
    r.reason_code = "OUTPUT_BYTES_DIFFER".to_string();
    r
}

fn candidate_first_line(candidate: &CandidateSide) -> String {
    if let Some(inv) = &candidate.prepare_invocation {
        if let Some(p) = &inv.stderr_path {
            let f = crate::runner::first_line(Path::new(p));
            if !f.is_empty() {
                return f;
            }
        }
        if let Some(p) = &inv.stdout_path {
            let f = crate::runner::first_line(Path::new(p));
            if !f.is_empty() {
                return f;
            }
        }
    }
    "candidate-rejected".to_string()
}

fn candidate_generated_files(
    candidate: &CandidateSide,
    work_root: &Path,
    unit_index: usize,
) -> Vec<String> {
    if let Some(inv) = &candidate.run_invocation {
        if let Some(ev) = &inv.stdout_path {
            // The candidate's generated files are the materialized file store: `--dump-files` writes
            // them into the RUN dir (the evidence dir's parent), mirroring the oracle's disk files.
            let ev_dir = Path::new(ev).parent().unwrap_or(work_root);
            let run_dir = ev_dir.parent().unwrap_or(work_root);
            if let Ok(rd) = std::fs::read_dir(run_dir) {
                let mut out = Vec::new();
                for e in rd.flatten() {
                    let p = e.path();
                    if p.is_file() {
                        if let Some(n) = p.file_name().map(|n| n.to_string_lossy().into_owned()) {
                            // the report / harness files are the PRIMARY output, never "generated"
                            if n != "stdout" && n != "stderr" && n != "REPORT" && n != "XXXXX055" {
                                out.push(n);
                            }
                        }
                    }
                }
                out.sort();
                return out;
            }
        }
    }
    let _ = unit_index;
    Vec::new()
}

fn candidate_verdict_counts(
    candidate: &CandidateSide,
    _work_root: &Path,
    _unit_index: usize,
) -> Option<VerdictCounts> {
    // The candidate (cobrun) writes DISPLAY output to stdout; its report-equivalent is the stdout
    // bytes, so verdict counts are extracted from the same canonical format when present.
    let out = candidate_primary_output(candidate);
    if out.is_empty() {
        return None;
    }
    crate::oracle::parse_verdict_counts(&out)
}

fn set_not_comparable(r: &mut UnitResult) {
    r.comparison.raw_stdout = "not_comparable".into();
    r.comparison.canonical_stdout = "not_comparable".into();
    r.comparison.generated_files = "not_comparable".into();
    r.comparison.exit_status = "not_comparable".into();
    r.comparison.verdict_counts = "not_comparable".into();
}

/// Classify every unit and compute the summary. Returns (results, summary).
pub fn classify_all(
    units: &[MaterializedUnit],
    oracle: &BTreeMap<usize, OracleSide>,
    candidate: &BTreeMap<usize, CandidateSide>,
    work_root: &Path,
) -> (Vec<UnitResult>, Summary) {
    let mut results = Vec::new();
    for u in units {
        let o = oracle.get(&u.unit_index).cloned().unwrap_or_default();
        let c = candidate.get(&u.unit_index).cloned().unwrap_or_default();
        results.push(classify_unit(u, &o, &c, work_root));
    }
    let summary = summarize(&results, units);
    (results, summary)
}

/// Compute the summary counts + groupings. All 512 units must be accounted for exactly once.
pub fn summarize(results: &[UnitResult], units: &[MaterializedUnit]) -> Summary {
    let mut s = Summary {
        units_total: units.len(),
        ..Default::default()
    };
    for u in units {
        *s.units_by_kind.entry(u.kind.clone()).or_insert(0) += 1;
        if u.is_executable_candidate {
            s.executable_candidates += 1;
        }
    }
    for r in results {
        let c = r.final_classification.as_str();
        *s.by_final_classification.entry(c.to_string()).or_insert(0) += 1;
        if !r.reason_code.is_empty() {
            *s.by_reason_code.entry(r.reason_code.clone()).or_insert(0) += 1;
        }
        *s.by_section.entry(section_of(&r.name)).or_insert(0) += 1;
        let pair = format!(
            "oracle:{} / candidate:{}",
            r.oracle.run_or_else_compile(),
            r.candidate.prepare_or_run(),
        );
        *s.oracle_candidate_pair.entry(pair).or_insert(0) += 1;
        if r.nondeterministic {
            s.nondeterministic += 1;
        }
        // Field-based oracle/candidate counters (orthogonal to the primary classification): a unit
        // the oracle runs and the candidate rejects counts once as oracle_run_pass AND once as
        // candidate_unsupported — distinct failure classes are never conflated.
        match r.oracle.compile.as_str() {
            "pass" => s.oracle_compile_pass += 1,
            "reject" => s.oracle_compile_reject += 1,
            "error" => s.oracle_compile_error += 1,
            _ => {}
        }
        match r.oracle.run.as_str() {
            "pass" => s.oracle_run_pass += 1,
            "fail" => s.oracle_run_fail += 1,
            "timeout" => s.oracle_timeout += 1,
            _ => {}
        }
        match r.candidate.prepare.as_str() {
            "accepted" => s.candidate_accepted += 1,
            "reject-unsupported" => s.candidate_unsupported += 1,
            "reject-parse" | "reject-layout" | "reject-runtime-boundary" => {
                s.candidate_parse_fail += 1
            }
            _ => {}
        }
        match r.candidate.run.as_str() {
            "fail" => s.candidate_runtime_fail += 1,
            "timeout" => s.candidate_timeout += 1,
            _ => {}
        }
        match c {
            "NON_EXECUTABLE_LIBRARY" => s.non_executable_library += 1,
            "NON_EXECUTABLE_DATA" => s.non_executable_data += 1,
            "RAW_OUTPUT_MATCH" => s.raw_output_match += 1,
            "CANONICAL_OUTPUT_MATCH" => s.canonical_output_match += 1,
            "RUST_ACCEPT_BUT_RUNTIME_FAIL" => {}
            "RUST_REJECT_UNSUPPORTED" => {}
            "RUST_REJECT_PARSE" | "RUST_REJECT_LAYOUT" | "RUST_REJECT_RUNTIME_BOUNDARY" => {}
            "RUST_TIMEOUT" => {}
            "OUTPUT_MISMATCH" => s.output_mismatch += 1,
            "EXIT_STATUS_MISMATCH" => s.exit_status_mismatch += 1,
            "GENERATED_FILE_MISMATCH" => s.generated_file_mismatch += 1,
            "HARNESS_BLOCKED" => s.harness_blocked += 1,
            "DEPENDENCY_BLOCKED" => s.dependency_blocked += 1,
            "INFRASTRUCTURE_ERROR" => s.infrastructure_error += 1,
            _ => {}
        }
    }
    s
}

impl OracleSide {
    fn run_or_else_compile(&self) -> &str {
        if self.run == "pass" || self.run == "fail" || self.run == "timeout" {
            &self.run
        } else if self.compile == "pass" {
            "compile-pass"
        } else {
            &self.compile
        }
    }
}

impl CandidateSide {
    fn prepare_or_run(&self) -> &str {
        match self.prepare.as_str() {
            "accepted" => {
                if self.run == "pass" {
                    "run-pass"
                } else if self.run == "fail" {
                    "run-fail"
                } else if self.run == "timeout" {
                    "timeout"
                } else {
                    "run-?"
                }
            }
            other => other,
        }
    }
}

/// Write `comparison-results.json`.
pub fn write_comparison_results(path: &Path, results: &[UnitResult]) {
    let v: Vec<serde_json::Value> = results
        .iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect();
    let doc = serde_json::json!({
        "schema": "gnurust-ccvs85-comparison-v1",
        "canonical_schema": CANONICAL_SCHEMA,
        "units": v,
    });
    let _ = std::fs::write(path, serde_json::to_string_pretty(&doc).unwrap() + "\n");
}

/// Write `summary.json`.
pub fn write_summary_json(path: &Path, summary: &Summary, meta: &serde_json::Value) {
    let doc = serde_json::json!({
        "schema": "gnurust-ccvs85-summary-v1",
        "meta": meta,
        "summary": summary,
    });
    let _ = std::fs::write(path, serde_json::to_string_pretty(&doc).unwrap() + "\n");
}

/// Render `summary.md` (the human-readable differential report with the mandated wording).
pub fn render_summary_md(summary: &Summary, meta: &serde_json::Value) -> String {
    let mut s = String::new();
    s.push_str(
        "# GNURUST.CCVS85.4 — NIST CCVS85 differential execution report\n\n\
         **GENERATED** by `cargo run -p gnucobol-rs-ccvs85 -- classify` — do not edit by hand.\n\n\
         `GNURUST.CCVS85.4` is a differential execution report over the admitted NIST CCVS85 Version 4.0\n\
         corpus. It reports which indexed units the pinned GnuCOBOL 3.2 oracle compiles and runs, which\n\
         units the current `gnucobol-rs` front-end accepts and executes, and where their observable\n\
         results agree or differ. It is **not** a NIST certification, does **not** establish complete\n\
         COBOL-85 conformance, and does **not** turn unsupported or unexecuted units into passes.\n\n"
    );
    s.push_str("## Totals\n\n| measure | count |\n|---|---|\n");
    s.push_str(&format!(
        "| units indexed (must reconcile) | **{}** |\n",
        summary.units_total
    ));
    for (k, v) in &summary.units_by_kind {
        s.push_str(&format!("| units by kind `{k}` | {v} |\n"));
    }
    s.push_str(&format!(
        "| executable candidates | {} |\n",
        summary.executable_candidates
    ));
    s.push_str(&format!(
        "| oracle compile pass | {} |\n",
        summary.oracle_compile_pass
    ));
    s.push_str(&format!(
        "| oracle compile reject | {} |\n",
        summary.oracle_compile_reject
    ));
    s.push_str(&format!(
        "| oracle compile error | {} |\n",
        summary.oracle_compile_error
    ));
    s.push_str(&format!(
        "| oracle run pass | {} |\n",
        summary.oracle_run_pass
    ));
    s.push_str(&format!(
        "| oracle run fail | {} |\n",
        summary.oracle_run_fail
    ));
    s.push_str(&format!(
        "| oracle timeout | {} |\n",
        summary.oracle_timeout
    ));
    s.push_str(&format!(
        "| candidate accepted | {} |\n",
        summary.candidate_accepted
    ));
    s.push_str(&format!(
        "| candidate unsupported | {} |\n",
        summary.candidate_unsupported
    ));
    s.push_str(&format!(
        "| candidate parse/layout/boundary reject | {} |\n",
        summary.candidate_parse_fail
    ));
    s.push_str(&format!(
        "| candidate runtime fail | {} |\n",
        summary.candidate_runtime_fail
    ));
    s.push_str(&format!(
        "| candidate timeout | {} |\n",
        summary.candidate_timeout
    ));
    s.push_str(&format!(
        "| raw output match | {} |\n",
        summary.raw_output_match
    ));
    s.push_str(&format!(
        "| canonical output match | {} |\n",
        summary.canonical_output_match
    ));
    s.push_str(&format!(
        "| output mismatch | {} |\n",
        summary.output_mismatch
    ));
    s.push_str(&format!(
        "| exit-status mismatch | {} |\n",
        summary.exit_status_mismatch
    ));
    s.push_str(&format!(
        "| generated-file mismatch | {} |\n",
        summary.generated_file_mismatch
    ));
    s.push_str(&format!(
        "| harness-blocked | {} |\n",
        summary.harness_blocked
    ));
    s.push_str(&format!(
        "| dependency-blocked | {} |\n",
        summary.dependency_blocked
    ));
    s.push_str(&format!(
        "| infrastructure error | {} |\n",
        summary.infrastructure_error
    ));
    s.push_str(&format!(
        "| nondeterministic (explicitly classified) | {} |\n",
        summary.nondeterministic
    ));

    s.push_str("\n## By primary classification\n\n| classification | count |\n|---|---|\n");
    for (k, v) in &summary.by_final_classification {
        s.push_str(&format!("| `{k}` | {v} |\n"));
    }

    s.push_str("\n## By CCVS85 section (name prefix)\n\n| section | count |\n|---|---|\n");
    for (k, v) in &summary.by_section {
        s.push_str(&format!("| `{k}` | {v} |\n"));
    }

    s.push_str("\n## By reason code (top buckets)\n\n| reason | count |\n|---|---|\n");
    let mut reasons: Vec<(&String, &usize)> = summary.by_reason_code.iter().collect();
    reasons.sort_by(|a, b| b.1.cmp(a.1));
    for (k, v) in reasons.into_iter().take(40) {
        s.push_str(&format!("| `{k}` | {v} |\n"));
    }

    s.push_str("\n## Oracle × candidate outcome pairs\n\n| pair | count |\n|---|---|\n");
    for (k, v) in &summary.oracle_candidate_pair {
        s.push_str(&format!("| `{k}` | {v} |\n"));
    }

    s.push_str(&format!(
        "\n## Boundary\n\n\
         - **no NIST certification** — CCVS85 is a historical validation corpus; this report is not a\n\
           certification result and carries no NIST or GSA authority.\n\
         - **no full COBOL-85 conformance claim** — a unit the oracle compiles+runs and the candidate\n\
           matches does not imply full language conformance.\n\
         - **no full `cobc` replacement claim** — `cobrun` is a sealed-subset interpreter over the ported\n\
           runtime.\n\
         - **no native-code-generation comparison** — `cobc` emits C + native code; `cobrun` interprets;\n\
           observable stdout/report bytes are compared, not codegen.\n\
         - **no claim that an oracle rejection proves the source invalid** under every COBOL\n\
           implementation — rejection is specific to the pinned GnuCOBOL 3.2 oracle and its dialect.\n\
         - **no claim that matching output proves equivalence** outside the tested environment.\n\
         - **no claim that library/data units are executable tests** — CLBRY and DATA* units are\n\
           classified as non-executable support units.\n\
         - **no conversion of blocked units into passes** — HARNESS_BLOCKED / DEPENDENCY_BLOCKED /\n\
           INFRASTRUCTURE_ERROR units are never counted as passes.\n\n\
         ## Environment\n\n\
         ```json\n{}\n```\n",
        serde_json::to_string_pretty(meta).unwrap_or_default()
    ));
    s
}

/// Write `results.csv` (one row per unit).
pub fn write_csv(path: &Path, results: &[UnitResult]) {
    let mut out = String::new();
    out.push_str("unit_index,kind,name,source_path,source_sha256,oracle_compile,oracle_run,oracle_exit,candidate_prepare,candidate_run,candidate_exit,raw_stdout,canonical_stdout,generated_files,exit_status,verdict_counts,final_classification,reason_code,nondeterministic,first_failure_line\n");
    for r in results {
        let o_rc = r
            .oracle
            .run_invocation
            .as_ref()
            .and_then(|i| i.exit_code)
            .map(|c| c.to_string())
            .unwrap_or_default();
        let c_rc = r
            .candidate
            .prepare_invocation_rc
            .map(|c| c.to_string())
            .unwrap_or_default();
        let csv = |s: &str| format!("\"{}\"", s.replace('"', "\"\""));
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            r.unit_index,
            csv(&r.kind),
            csv(&r.name),
            csv(&r.source_path),
            r.source_sha256,
            csv(&r.oracle.compile),
            csv(&r.oracle.run),
            o_rc,
            csv(&r.candidate.prepare),
            csv(&r.candidate.run),
            c_rc,
            csv(&r.comparison.raw_stdout),
            csv(&r.comparison.canonical_stdout),
            csv(&r.comparison.generated_files),
            csv(&r.comparison.exit_status),
            csv(&r.comparison.verdict_counts),
            r.final_classification.as_str(),
            csv(&r.reason_code),
            if r.nondeterministic { "yes" } else { "no" },
            csv(&r.first_failure_line),
        ));
    }
    let _ = std::fs::write(path, out);
}

/// Render `failure-buckets.md` — the largest failure/unsupported-feature buckets.
pub fn render_failure_buckets(results: &[UnitResult], summary: &Summary) -> String {
    let mut s = String::new();
    s.push_str("# GNURUST.CCVS85.4 — failure & unsupported-feature buckets\n\n");
    s.push_str(
        "**GENERATED** by `cargo run -p gnucobol-rs-ccvs85 -- classify` — do not edit by hand.\n\n",
    );
    s.push_str("Buckets group units by their primary classification, reason code, and (for candidate\nrejections) the first refused construct, so no failure class is collapsed into a single\nnumber.\n\n");

    let mut reasons: Vec<(&String, &usize)> = summary.by_reason_code.iter().collect();
    reasons.sort_by(|a, b| b.1.cmp(a.1));
    s.push_str("## Top reason-code buckets\n\n| reason | count |\n|---|---|\n");
    for (k, v) in reasons.iter().take(50) {
        s.push_str(&format!("| `{k}` | {v} |\n"));
    }

    let interesting: Vec<&UnitResult> = results
        .iter()
        .filter(|r| {
            !matches!(
                r.final_classification,
                crate::model::FinalClassification::NonExecutableLibrary
                    | crate::model::FinalClassification::NonExecutableData
                    | crate::model::FinalClassification::RawOutputMatch
            )
        })
        .collect();
    s.push_str(&format!(
        "\n## Representative units per classification ({} non-trivial)\n\n",
        interesting.len()
    ));
    let mut by_class: BTreeMap<&str, Vec<&UnitResult>> = BTreeMap::new();
    for r in &interesting {
        by_class
            .entry(r.final_classification.as_str())
            .or_default()
            .push(r);
    }
    for (class, list) in by_class {
        s.push_str(&format!("\n### `{class}` ({} units)\n\n", list.len()));
        for r in list.iter().take(12) {
            s.push_str(&format!(
                "- `{}` (u{}) — `{}`{}",
                r.name,
                r.unit_index,
                r.reason_code,
                if r.first_failure_line.is_empty() {
                    String::new()
                } else {
                    format!(" — {}", r.first_failure_line)
                }
            ));
            s.push('\n');
        }
        if list.len() > 12 {
            s.push_str(&format!("- … and {} more\n", list.len() - 12));
        }
    }

    s.push_str("\n## Explicitly classified nondeterministic units\n\n");
    let nondet: Vec<&UnitResult> = results.iter().filter(|r| r.nondeterministic).collect();
    if nondet.is_empty() {
        s.push_str("(none)\n");
    } else {
        for r in nondet {
            let note = r
                .determinism
                .as_ref()
                .map(|d| {
                    format!(
                        "oracle REPORT bytes differ between the two fresh runs (pass A {} vs pass B {})",
                        &d.pass_a[..12],
                        &d.pass_b[..12]
                    )
                })
                .unwrap_or_else(|| "oracle REPORT bytes differ between the two fresh runs".to_string());
            s.push_str(&format!(
                "- `{}` — explicitly classified nondeterministic ({}) — {}\n",
                r.name,
                r.final_classification.as_str(),
                note
            ));
        }
    }
    s
}
