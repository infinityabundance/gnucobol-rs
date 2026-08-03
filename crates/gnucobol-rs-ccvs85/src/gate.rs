//! `gate check` — the host-side anti-freshness / invariant gate for `GNURUST.CCVS85.2/.3/.4`.
//!
//! Runs WITHOUT Docker against the committed evidence. It fails only on real problems:
//! corpus identity mismatch, unaccounted-for units, malformed result schema, missing required
//! evidence, receipt freshness failure, result totals that do not reconcile, candidate delegation
//! to `cobc`, missing raw evidence, or comparison-logic failure. It must NOT fail because the
//! candidate rejects units or outputs differ — those are benchmark findings, not harness failures.

use crate::corpus::sha256_hex;
use crate::model::FinalClassification;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;

pub const EXPECTED_UNITS: usize = 512;
pub const CLASSIFICATIONS: [&str; 23] = [
    "NON_EXECUTABLE_LIBRARY",
    "NON_EXECUTABLE_DATA",
    "ORACLE_COMPILE_PASS",
    "ORACLE_COMPILE_REJECT",
    "ORACLE_COMPILE_ERROR",
    "ORACLE_RUN_PASS",
    "ORACLE_RUN_FAIL",
    "ORACLE_TIMEOUT",
    "RUST_ACCEPT_AND_RUN",
    "RUST_ACCEPT_BUT_RUNTIME_FAIL",
    "RUST_REJECT_UNSUPPORTED",
    "RUST_REJECT_PARSE",
    "RUST_REJECT_LAYOUT",
    "RUST_REJECT_RUNTIME_BOUNDARY",
    "RUST_TIMEOUT",
    "RAW_OUTPUT_MATCH",
    "CANONICAL_OUTPUT_MATCH",
    "OUTPUT_MISMATCH",
    "EXIT_STATUS_MISMATCH",
    "GENERATED_FILE_MISMATCH",
    "HARNESS_BLOCKED",
    "DEPENDENCY_BLOCKED",
    "INFRASTRUCTURE_ERROR",
];

fn read_json(p: &Path) -> Option<Value> {
    std::fs::read_to_string(p)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

fn sha_of(p: &Path) -> String {
    match std::fs::read(p) {
        Ok(b) => sha256_hex(&b),
        Err(_) => String::new(),
    }
}

pub struct GateReport {
    pub problems: Vec<String>,
    pub notes: Vec<String>,
}

/// Check the committed CCVS85 evidence under `root` (the repository root). Returns the report.
pub fn gate_check(root: &Path, work_evidence: Option<&Path>) -> GateReport {
    let mut g = GateReport {
        problems: Vec::new(),
        notes: Vec::new(),
    };
    let ccvs85 = root.join("reports/ccvs85");

    // ---- 1. required result artifacts -------------------------------------------------------
    let required = [
        "materialized-units.json",
        "oracle-results.json",
        "candidate-results.json",
        "comparison-results.json",
        "summary.json",
        "summary.md",
        "results.csv",
        "failure-buckets.md",
        "no-delegation.json",
    ];
    for f in required {
        let p = ccvs85.join(f);
        if !p.exists() {
            g.problems.push(format!("missing required evidence {f}"));
        }
    }

    // ---- 2. materialized-units.json: 512 units, stable fields -------------------------------
    let materialized = read_json(&ccvs85.join("materialized-units.json"));
    match &materialized {
        Some(Value::Array(arr)) if arr.len() == EXPECTED_UNITS => {
            let mut seen = BTreeSet::new();
            for u in arr {
                let idx = u["unit_index"].as_u64();
                if let Some(i) = idx {
                    if !seen.insert(i) {
                        g.problems
                            .push(format!("materialized-units.json duplicate unit_index {i}"));
                    }
                }
                for field in [
                    "kind",
                    "name",
                    "source_path",
                    "source_sha256",
                    "start_line",
                    "end_line",
                ] {
                    if u[field].is_null() {
                        g.problems.push(format!(
                            "materialized unit {} missing field {field}",
                            u["unit_index"]
                        ));
                    }
                }
            }
            let kinds: BTreeSet<String> = arr
                .iter()
                .filter_map(|u| u["kind"].as_str().map(String::from))
                .collect();
            for k in &kinds {
                if k != "COBOL" && k != "CLBRY" && k != "DATA*" {
                    g.problems.push(format!("unexpected unit kind {k}"));
                }
            }
        }
        Some(Value::Array(arr)) => {
            g.problems.push(format!(
                "materialized-units.json has {} units, expected {EXPECTED_UNITS} (all-512-accounted invariant)",
                arr.len()
            ));
        }
        _ => {
            g.problems
                .push("materialized-units.json malformed or absent".to_string());
        }
    }

    // ---- 3. comparison-results.json: schema, 512 units, exactly one classification each ----
    let comparison = read_json(&ccvs85.join("comparison-results.json"));
    match &comparison {
        Some(Value::Object(o)) if o.get("units").is_some() => {
            let units = o["units"].as_array().cloned().unwrap_or_default();
            if units.len() != EXPECTED_UNITS {
                g.problems.push(format!(
                    "comparison-results.json has {} units, expected {EXPECTED_UNITS}",
                    units.len()
                ));
            }
            let valid: BTreeSet<&str> = CLASSIFICATIONS.iter().copied().collect();
            let mut seen = BTreeSet::new();
            for u in &units {
                let idx = u["unit_index"].as_u64();
                if let Some(i) = idx {
                    if !seen.insert(i) {
                        g.problems
                            .push(format!("comparison-results.json duplicate unit_index {i}"));
                    }
                }
                let fc = u["final_classification"].as_str().unwrap_or("");
                if fc.is_empty() {
                    g.problems
                        .push(format!("unit {idx:?} has no final_classification"));
                } else if !valid.contains(fc) {
                    g.problems.push(format!(
                        "unit {idx:?} has unknown final_classification {fc}"
                    ));
                }
            }
        }
        _ => {
            g.problems
                .push("comparison-results.json malformed (no units array)".to_string());
        }
    }

    // ---- 4. summary.json reconciles with comparison-results.json ----------------------------
    if let (Some(comp), Some(sum)) = (&comparison, read_json(&ccvs85.join("summary.json"))) {
        let units = comp["units"].as_array().cloned().unwrap_or_default();
        let mut by_class: BTreeSet<String> = BTreeSet::new();
        // Field-based counts (the summary's oracle/candidate counters are computed from the
        // orthogonal oracle/candidate fields, NOT from the primary classification, so e.g. a unit
        // the oracle runs and the candidate rejects counts once as oracle_run_pass AND once as
        // candidate_unsupported — never conflated).
        let mut n_non_exec = 0usize;
        let mut n_compile_pass = 0usize;
        let mut n_compile_reject = 0usize;
        let mut n_compile_error = 0usize;
        let mut n_run_pass = 0usize;
        let mut n_run_fail = 0usize;
        let mut n_timeout = 0usize;
        let mut n_cand_accepted = 0usize;
        let mut n_cand_unsupported = 0usize;
        let mut n_cand_parse = 0usize;
        let mut n_cand_runtime_fail = 0usize;
        let mut n_cand_timeout = 0usize;
        let mut n_raw = 0usize;
        let mut n_canon = 0usize;
        let mut n_mismatch = 0usize;
        let mut n_exit = 0usize;
        let mut n_gen = 0usize;
        let mut n_blocked = 0usize;
        let mut n_dep = 0usize;
        let mut n_infra = 0usize;
        let mut n_nondet = 0usize;
        for u in &units {
            let fc = FinalClassification::from(u["final_classification"].as_str().unwrap_or(""));
            by_class.insert(fc.as_str().to_string());
            match u["oracle"]["compile"].as_str().unwrap_or("") {
                "pass" => n_compile_pass += 1,
                "reject" => n_compile_reject += 1,
                "error" => n_compile_error += 1,
                _ => {}
            }
            match u["oracle"]["run"].as_str().unwrap_or("") {
                "pass" => n_run_pass += 1,
                "fail" => n_run_fail += 1,
                "timeout" => n_timeout += 1,
                _ => {}
            }
            match u["candidate"]["prepare"].as_str().unwrap_or("") {
                "accepted" => n_cand_accepted += 1,
                "reject-unsupported" => n_cand_unsupported += 1,
                "reject-parse" | "reject-layout" | "reject-runtime-boundary" => n_cand_parse += 1,
                _ => {}
            }
            match u["candidate"]["run"].as_str().unwrap_or("") {
                "fail" => n_cand_runtime_fail += 1,
                "timeout" => n_cand_timeout += 1,
                _ => {}
            }
            match fc {
                FinalClassification::NonExecutableLibrary
                | FinalClassification::NonExecutableData => n_non_exec += 1,
                FinalClassification::RawOutputMatch => n_raw += 1,
                FinalClassification::CanonicalOutputMatch => n_canon += 1,
                FinalClassification::OutputMismatch => n_mismatch += 1,
                FinalClassification::ExitStatusMismatch => n_exit += 1,
                FinalClassification::GeneratedFileMismatch => n_gen += 1,
                FinalClassification::HarnessBlocked => n_blocked += 1,
                FinalClassification::DependencyBlocked => n_dep += 1,
                FinalClassification::InfrastructureError => n_infra += 1,
                _ => {}
            }
            if u["nondeterministic"].as_bool().unwrap_or(false) {
                n_nondet += 1;
            }
        }
        let s = &sum["summary"];
        let check = |g: &mut GateReport, label: &str, got: usize, expected: usize| {
            if got != expected {
                g.problems.push(format!(
                    "summary reconcile failed: {label} computed {got}, summary says {expected}"
                ));
            }
        };
        check(
            &mut g,
            "units_total",
            units.len(),
            s["units_total"].as_u64().unwrap_or(0) as usize,
        );
        check(&mut g, "non_executable_library+data", n_non_exec, {
            s["non_executable_library"].as_u64().unwrap_or(0) as usize
                + s["non_executable_data"].as_u64().unwrap_or(0) as usize
        });
        check(
            &mut g,
            "oracle_compile_pass",
            n_compile_pass,
            s["oracle_compile_pass"].as_u64().unwrap_or(0) as usize,
        );
        check(
            &mut g,
            "oracle_compile_reject",
            n_compile_reject,
            s["oracle_compile_reject"].as_u64().unwrap_or(0) as usize,
        );
        check(
            &mut g,
            "oracle_compile_error",
            n_compile_error,
            s["oracle_compile_error"].as_u64().unwrap_or(0) as usize,
        );
        check(
            &mut g,
            "oracle_run_pass",
            n_run_pass,
            s["oracle_run_pass"].as_u64().unwrap_or(0) as usize,
        );
        check(
            &mut g,
            "oracle_run_fail",
            n_run_fail,
            s["oracle_run_fail"].as_u64().unwrap_or(0) as usize,
        );
        check(
            &mut g,
            "oracle_timeout",
            n_timeout,
            s["oracle_timeout"].as_u64().unwrap_or(0) as usize,
        );
        check(
            &mut g,
            "candidate_accepted",
            n_cand_accepted,
            s["candidate_accepted"].as_u64().unwrap_or(0) as usize,
        );
        check(
            &mut g,
            "candidate_unsupported",
            n_cand_unsupported,
            s["candidate_unsupported"].as_u64().unwrap_or(0) as usize,
        );
        check(
            &mut g,
            "candidate_parse_fail",
            n_cand_parse,
            s["candidate_parse_fail"].as_u64().unwrap_or(0) as usize,
        );
        check(
            &mut g,
            "candidate_runtime_fail",
            n_cand_runtime_fail,
            s["candidate_runtime_fail"].as_u64().unwrap_or(0) as usize,
        );
        check(
            &mut g,
            "candidate_timeout",
            n_cand_timeout,
            s["candidate_timeout"].as_u64().unwrap_or(0) as usize,
        );
        check(
            &mut g,
            "raw_output_match",
            n_raw,
            s["raw_output_match"].as_u64().unwrap_or(0) as usize,
        );
        check(
            &mut g,
            "canonical_output_match",
            n_canon,
            s["canonical_output_match"].as_u64().unwrap_or(0) as usize,
        );
        check(
            &mut g,
            "output_mismatch",
            n_mismatch,
            s["output_mismatch"].as_u64().unwrap_or(0) as usize,
        );
        check(
            &mut g,
            "exit_status_mismatch",
            n_exit,
            s["exit_status_mismatch"].as_u64().unwrap_or(0) as usize,
        );
        check(
            &mut g,
            "generated_file_mismatch",
            n_gen,
            s["generated_file_mismatch"].as_u64().unwrap_or(0) as usize,
        );
        check(
            &mut g,
            "harness_blocked",
            n_blocked,
            s["harness_blocked"].as_u64().unwrap_or(0) as usize,
        );
        check(
            &mut g,
            "dependency_blocked",
            n_dep,
            s["dependency_blocked"].as_u64().unwrap_or(0) as usize,
        );
        check(
            &mut g,
            "infrastructure_error",
            n_infra,
            s["infrastructure_error"].as_u64().unwrap_or(0) as usize,
        );
        check(
            &mut g,
            "nondeterministic",
            n_nondet,
            s["nondeterministic"].as_u64().unwrap_or(0) as usize,
        );
        // every classification used must be present in by_final_classification
        let sfc = s["by_final_classification"]
            .as_object()
            .cloned()
            .unwrap_or_default();
        for c in &by_class {
            if !sfc.contains_key(c) {
                g.problems.push(format!(
                    "summary missing by_final_classification bucket {c}"
                ));
            }
        }
        g.notes.push(format!(
            "summary reconciles: {} units, {} classifications",
            units.len(),
            by_class.len()
        ));
    }

    // ---- 5. no-delegation evidence ----------------------------------------------------------
    let nd = read_json(&ccvs85.join("no-delegation.json"));
    match &nd {
        Some(v) => {
            let ok = v["candidate_phase_isolated"].as_bool().unwrap_or(false)
                && v["cobrun_links_no_libcob"].as_bool().unwrap_or(false);
            if !ok {
                g.problems.push("no-delegation.json does not prove candidate isolation (candidate_phase_isolated / cobrun_links_no_libcob must be true)".to_string());
            } else {
                g.notes.push(
                    "candidate-no-oracle-delegation invariant holds (recorded proof)".to_string(),
                );
            }
        }
        None => {
            g.problems.push(
                "no-delegation.json missing — candidate delegation proof required".to_string(),
            );
        }
    }

    // ---- 6. raw evidence presence + hashes ---------------------------------------------------
    if let Some(units) = materialized.as_ref().and_then(|m| m.as_array()) {
        let raw_dir = ccvs85.join("raw");
        if !raw_dir.exists() {
            g.problems
                .push("reports/ccvs85/raw/ missing — raw per-unit evidence required".to_string());
        } else {
            let mut missing = 0usize;
            let mut checked = 0usize;
            for u in units.iter().take(EXPECTED_UNITS) {
                let sp = u["source_path"].as_str().unwrap_or("");
                let want = u["source_sha256"].as_str().unwrap_or("");
                // the raw evidence tree mirrors materialized files (either at raw/<sp> directly
                // or under raw/sources/<sp>)
                let mirrored = [raw_dir.join(sp), raw_dir.join("sources").join(sp)]
                    .into_iter()
                    .find(|p| p.exists());
                if let Some(m) = mirrored {
                    let got = sha_of(&m);
                    if !want.is_empty() && got != want {
                        g.problems.push(format!(
                            "raw evidence hash mismatch for {sp}: {got} != {want}"
                        ));
                    }
                    checked += 1;
                } else {
                    missing += 1;
                }
            }
            if missing > 0 {
                g.problems.push(format!(
                    "{missing} raw unit source files missing from reports/ccvs85/raw/"
                ));
            }
            g.notes
                .push(format!("raw evidence checked for {checked} unit sources"));
        }
    }

    // ---- 7. receipt freshness ----------------------------------------------------------------
    for gate in ["GNURUST.CCVS85.2", "GNURUST.CCVS85.3", "GNURUST.CCVS85.4"] {
        let jf = root
            .join("reports/receipts")
            .join(gate)
            .join("receipt.json");
        let mf = root.join("reports/receipts").join(gate).join("receipt.md");
        if !jf.exists() {
            g.problems.push(format!("{gate} receipt.json missing"));
            continue;
        }
        let r = read_json(&jf);
        match r {
            Some(v) => {
                // freshness: the receipt's embedded artifact hashes must match the current files
                let arts = &v["results"];
                for (field, file) in [
                    ("materialized_units_json_sha256", "materialized-units.json"),
                    ("oracle_results_sha256", "oracle-results.json"),
                    ("candidate_results_sha256", "candidate-results.json"),
                    ("comparison_results_sha256", "comparison-results.json"),
                ] {
                    if let Some(want) = arts[field].as_str() {
                        let got = sha_of(&ccvs85.join(file));
                        if !want.is_empty() && got != want {
                            g.problems.push(format!(
                                "{gate} receipt STALE: {file} hash {got} != receipt {want}"
                            ));
                        }
                    }
                }
                if !mf.exists() {
                    g.problems.push(format!("{gate} receipt.md missing"));
                }
            }
            None => {
                g.problems.push(format!("{gate} receipt.json malformed"));
            }
        }
    }

    // ---- 8. corpus identity vs committed GNURUST.CCVS85.1 receipt ----------------------------
    let committed = read_json(&root.join("reports/provenance/ccvs85-corpus-ingest-receipt.json"));
    let spine = root.join("lab/corpus/ccvs85/newcob.val.Z");
    if spine.exists() {
        if let Some(rec) = &committed {
            if let Ok(bytes) = std::fs::read(&spine) {
                let got = sha256_hex(&bytes);
                let want = rec["compressed_sha256"].as_str().unwrap_or("");
                if got != want {
                    g.problems.push(format!(
                        "corpus identity mismatch: spine sha256 {got} != committed {want}"
                    ));
                } else {
                    g.notes.push(
                        "corpus identity matches the committed GNURUST.CCVS85.1 receipt"
                            .to_string(),
                    );
                }
            }
        }
    }

    if let Some(w) = work_evidence {
        let _ = w;
        g.notes
            .push("work-dir evidence override accepted".to_string());
    }

    g
}

/// Exit-code decision: the gate fails ONLY on real problems (never on benchmark findings).
pub fn exit_code(g: &GateReport) -> i32 {
    if g.problems.is_empty() {
        0
    } else {
        1
    }
}
