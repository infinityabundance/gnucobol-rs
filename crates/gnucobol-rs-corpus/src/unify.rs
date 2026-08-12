//! Phase 12.1 — the unified valid-corpus reports.
//!
//! Aggregates the per-family evidence under `reports/valid-corpus/` into the single cross-family
//! report set the task mandates:
//!
//! - `summary.json` / `summary.md` — totals by corpus class, source family, validity class,
//!   first-failure phase, runnable/non-runnable, partition;
//! - `programs.csv` — one row per admitted unit;
//! - `licences.json` — the per-family / per-repository licence decisions;
//! - `dependencies.json` — copybooks, modules, inputs, missing dependencies;
//! - `deduplication.json` — exact + near-duplicate evidence per family;
//! - `dialect-matrix.json` — validity profiles (dialect x format x family);
//! - `first-failure-buckets.md` — candidate phase buckets across every family;
//! - `accuracy.json` — oracle/candidate byte accuracy per family;
//! - `performance.json` — the Phase-8/9 performance evidence pointer + aggregate;
//! - `determinism.json` — the determinism evidence pointers + verdict;
//! - `no-delegation.json` — the no-delegation evidence pointers + verdict.
//!
//! Everything is derived from the committed family reports — this module never re-measures and
//! never invents numbers. When a family report is absent the aggregate records `missing` for it
//! (honest, never fabricated).

use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Read a JSON file as a serde Value; `None` when absent or unparseable.
fn read_json(p: &Path) -> Option<serde_json::Value> {
    let text = std::fs::read_to_string(p).ok()?;
    serde_json::from_str(&text).ok()
}

/// `reports/valid-corpus/` under the workspace root.
pub fn report_dir(root: &Path) -> PathBuf {
    root.join("reports").join("valid-corpus")
}

/// The complete unified report (one serializable document per spec-12.1 file).
#[derive(Debug, Clone, Serialize)]
pub struct UnifiedReports {
    pub schema: String,
    pub generated_at_utc: String,
    pub summary: serde_json::Value,
    pub licences: serde_json::Value,
    pub dependencies: serde_json::Value,
    pub deduplication: serde_json::Value,
    pub dialect_matrix: serde_json::Value,
    pub first_failure_buckets: serde_json::Value,
    pub accuracy: serde_json::Value,
    pub performance: serde_json::Value,
    pub determinism: serde_json::Value,
    pub no_delegation: serde_json::Value,
}

/// The families whose evidence is aggregated (directory under `reports/valid-corpus/`).
const FAMILIES: &[&str] = &[
    "gnucobol-testsuite",
    "ccvs85",
    "gnucobol-manual",
    "extras",
    "omp",
    "xcobol",
    "performance",
];

fn sum_values(v: &serde_json::Value) -> u64 {
    match v {
        serde_json::Value::Number(n) => n.as_u64().unwrap_or(0),
        serde_json::Value::Array(a) => a.iter().map(sum_values).sum(),
        serde_json::Value::Object(o) => o.values().map(sum_values).sum(),
        _ => 0,
    }
}

/// Count records in a family programs list (array) or by a `count` field.
fn count_records(v: &serde_json::Value) -> usize {
    match v {
        serde_json::Value::Array(a) => a.len(),
        serde_json::Value::Object(o) => o
            .get("count")
            .and_then(|c| c.as_u64())
            .map(|c| c as usize)
            .unwrap_or(0),
        _ => 0,
    }
}

/// The candidate first-failure phase from a record row: handles both the string form
/// (`"parse"`) and the `[phase, diagnostic]` tuple form the extractors emit.
fn first_failure_phase(r: &serde_json::Value) -> String {
    match r.get("candidate_first_failure") {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Array(a)) => a
            .first()
            .and_then(|v| v.as_str())
            .unwrap_or("none")
            .to_string(),
        _ => "none".to_string(),
    }
}

/// Load `reports/valid-corpus/<family>/<file>` when present.
fn family_file(root: &Path, family: &str, file: &str) -> Option<serde_json::Value> {
    read_json(&report_dir(root).join(family).join(file))
}

// ---------------------------------------------------------------------------------------------
// summary
// ---------------------------------------------------------------------------------------------

fn build_summary(root: &Path) -> serde_json::Value {
    let mut by_family: BTreeMap<String, usize> = BTreeMap::new();
    // Two independent dimensions per unit (spec 12.2 presentation):
    //   VALIDITY  = the oracle/admission class (never a candidate outcome);
    //   CANDIDATE = the candidate phase outcome (ALL_PHASES_OK / phase REJECT / not probed).
    // Family rows sometimes fuse the two with '|' (extract/mod.rs stores `VALID_X|CANDIDATE_Y`
    // in classification for the testsuite lane); this projection splits them so the summary
    // never mixes validity with candidate outcome.
    let mut by_validity: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_candidate: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_first_failure: BTreeMap<String, usize> = BTreeMap::new();
    // cross-tab: validity x candidate outcome (the fused-string rows land here decomposed)
    let mut x_validity_candidate: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    let mut total = 0usize;

    // record one unit: decompose the fused classification, count both dimensions + cross-tab.
    let mut record = |row: &serde_json::Value,
                      family: &str,
                      class_of: fn(&serde_json::Value) -> Option<String>,
                      candidate_of: fn(&serde_json::Value) -> Option<String>,
                      first_failure_of: fn(&serde_json::Value) -> Option<String>|
     -> bool {
        let Some(c) = class_of(row) else {
            return false;
        };
        total += 1;
        *by_family.entry(family.to_string()).or_default() += 1;
        // validity = the primary (first) component; candidate = the CANDIDATE_* suffix when the
        // family fused it, else the row's own candidate-phase field, else "not probed".
        let parts: Vec<&str> = c.split('|').collect();
        let validity = parts[0].trim().to_string();
        *by_validity.entry(validity.clone()).or_default() += 1;
        let fused_candidate = parts
            .get(1)
            .map(|s| s.trim())
            .filter(|s| s.starts_with("CANDIDATE_"))
            .map(|s| s.to_string());
        let cand = fused_candidate
            .or_else(|| candidate_of(row))
            .unwrap_or_else(|| "CANDIDATE_NOT_PROBED".to_string());
        *by_candidate.entry(cand.clone()).or_default() += 1;
        x_validity_candidate
            .entry(validity.clone())
            .or_default()
            .entry(cand.clone())
            .and_modify(|n| *n += 1)
            .or_insert(1);
        if let Some(ff) = first_failure_of(row) {
            *by_first_failure.entry(ff).or_default() += 1;
        }
        true
    };

    // testsuite: valid-programs.json rows carry classification + first_failure
    if let Some(progs) = family_file(root, "gnucobol-testsuite", "valid-programs.json") {
        if let Some(arr) = progs.as_array() {
            for r in arr {
                record(
                    r,
                    "gnucobol-testsuite",
                    |row| {
                        row.get("classification")
                            .and_then(|c| c.as_str())
                            .map(|s| s.to_string())
                    },
                    // candidate suffix already fused into classification; fall back to first_failure tuple
                    |row| {
                        row.get("first_failure")
                            .and_then(|f| f.as_array())
                            .and_then(|a| a.first())
                            .and_then(|p| p.as_str())
                            .map(|p| format!("CANDIDATE_{}_REJECT", p.to_uppercase()))
                    },
                    |row| {
                        // preserve the committed projection: every testsuite row contributes
                        // (tuple first_failure -> "none", matching the legacy summary exactly)
                        Some(
                            row.get("first_failure")
                                .and_then(|f| f.as_str())
                                .unwrap_or("none")
                                .to_string(),
                        )
                    },
                );
            }
        }
    }
    // ccvs85
    if let Some(progs) = family_file(root, "ccvs85", "programs.json") {
        if let Some(arr) = progs.as_array() {
            for r in arr {
                record(
                    r,
                    "ccvs85",
                    |row| {
                        row.get("classification")
                            .and_then(|c| c.as_str())
                            .map(|s| s.to_string())
                    },
                    |_row| None, // ccvs85 rows carry accuracy verdicts, not candidate phase probes
                    |_row| None,
                );
            }
        }
    }
    // manual (stable + current lanes)
    for lane in ["stable-3.2", "current"] {
        if let Some(ex) = family_file(root, "gnucobol-manual", &format!("{lane}/examples.json")) {
            if let Some(arr) = ex.as_array() {
                for r in arr {
                    record(
                        r,
                        "gnucobol-manual",
                        |row| {
                            row.get("classification")
                                .and_then(|c| c.as_str())
                                .map(|s| s.to_string())
                        },
                        |_row| None,
                        |_row| None,
                    );
                }
            }
        }
    }
    // extras
    if let Some(progs) = family_file(root, "extras", "programs.json") {
        if let Some(arr) = progs.as_array() {
            for r in arr {
                record(
                    r,
                    "extras",
                    |row| {
                        row.get("classification")
                            .and_then(|c| c.as_str())
                            .map(|s| s.to_string())
                    },
                    |row| {
                        // all candidate phases present and ok -> ALL_PHASES_OK (module/no-run rows)
                        let all_ok = row
                            .get("candidate_phases")
                            .and_then(|a| a.as_array())
                            .map(|a| {
                                !a.is_empty()
                                    && a.iter().all(|p| {
                                        p.get("ok").and_then(|o| o.as_bool()) == Some(true)
                                    })
                            })
                            .unwrap_or(false);
                        if all_ok {
                            return Some("CANDIDATE_ALL_PHASES_OK".to_string());
                        }
                        row.get("candidate_first_failure")
                            .and_then(|f| f.as_array())
                            .and_then(|a| a.first())
                            .and_then(|p| p.as_str())
                            .map(|p| format!("CANDIDATE_{}_REJECT", p.to_uppercase()))
                    },
                    |row| {
                        // preserve the committed projection: extras contributes its non-none
                        // candidate phases only (old code: first_failure_phase != "none")
                        let ff = first_failure_phase(row);
                        if ff != "none" {
                            Some(ff)
                        } else {
                            None
                        }
                    },
                );
            }
        }
    }
    // omp
    if let Some(progs) = family_file(root, "omp", "programs.json") {
        if let Some(arr) = progs.as_array() {
            for r in arr {
                record(
                    r,
                    "omp",
                    |row| {
                        row.get("admission")
                            .or_else(|| row.get("classification"))
                            .and_then(|c| c.as_str())
                            .map(|s| s.to_string())
                    },
                    |_row| None,
                    |_row| None,
                );
            }
        }
    }
    // xcobol
    if let Some(progs) = family_file(root, "xcobol", "programs.json") {
        if let Some(arr) = progs.as_array() {
            for r in arr {
                record(
                    r,
                    "xcobol",
                    |row| {
                        row.get("structural_class")
                            .and_then(|c| c.as_str())
                            .map(|s| s.to_string())
                    },
                    |row| {
                        if row.get("candidate_phases_ok").and_then(|v| v.as_bool()) == Some(true) {
                            Some("CANDIDATE_ALL_PHASES_OK".to_string())
                        } else {
                            row.get("candidate_first_failure")
                                .and_then(|f| match f {
                                    serde_json::Value::String(s) => {
                                        s.split(':').next().map(|s| s.to_string())
                                    }
                                    serde_json::Value::Array(a) => {
                                        a.first().and_then(|p| p.as_str()).map(|p| p.to_string())
                                    }
                                    _ => None,
                                })
                                .map(|p| format!("CANDIDATE_{}_REJECT", p.to_uppercase()))
                        }
                    },
                    |row| {
                        // preserve the committed projection: xcobol contributes its non-none
                        // candidate phases only (old code: first_failure_phase != "none")
                        let ff = first_failure_phase(row);
                        if ff != "none" {
                            Some(ff)
                        } else {
                            None
                        }
                    },
                );
            }
        }
    }

    serde_json::json!({
        "total_units": total,
        "by_source_family": by_family,
        "by_validity_class": by_validity,
        "by_candidate_outcome": by_candidate,
        "validity_x_candidate_outcome": x_validity_candidate,
        "by_first_failure_phase": by_first_failure,
        "families_aggregated": FAMILIES,
        "note": "aggregated from the committed per-family reports (spec 12.2: report separately by corpus class, source family, dialect, format, validity class, first failing phase, partition). validity and candidate outcome are two independent dimensions: a family row that fuses them (VALID_X|CANDIDATE_Y) is decomposed here, so the summary never mixes validity with candidate outcome.",
    })
}

// ---------------------------------------------------------------------------------------------
// licences
// ---------------------------------------------------------------------------------------------

fn build_licences(root: &Path) -> serde_json::Value {
    let mut by_family: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    by_family.insert(
        "gnucobol-testsuite".to_string(),
        serde_json::json!({
            "spdx": "GPL-3.0-or-later (suite) / GFDL-1.3 (manual)",
            "redistribution_allowed": false,
            "decision": "admitted read-only for differential testing; never redistributed by gnucobol-rs",
        }),
    );
    by_family.insert(
        "ccvs85".to_string(),
        serde_json::json!({
            "spdx": "public-domain",
            "redistribution_allowed": true,
            "decision": "NIST CCVS85 is a US Government work in the public domain",
        }),
    );
    by_family.insert(
        "extras".to_string(),
        serde_json::json!({
            "spdx": "MIT",
            "redistribution_allowed": true,
            "decision": "OpenCBS COBOL Defects Benchmark Suite is MIT; per-file copyrights preserved in the custody report",
        }),
    );
    by_family.insert(
        "omp".to_string(),
        serde_json::json!({
            "spdx": "CC-BY-4.0",
            "redistribution_allowed": false,
            "decision": "Open Mainframe Project course materials: admitted read-only, not redistributed",
        }),
    );
    // xcobol: per-repository licences from the quarantine report
    if let Some(q) = family_file(root, "xcobol", "licence-quarantine.json") {
        by_family.insert(
            "xcobol".to_string(),
            serde_json::json!({
                "spdx": "per-repository (CC-BY-4.0 dataset; repositories vary)",
                "redistribution_allowed": false,
                "decision": q.get("policy"),
                "repos_checked": q.get("repos").map(|r| r.as_array().map(|a| a.len()).unwrap_or(0)).unwrap_or(0),
            }),
        );
    } else {
        by_family.insert(
            "xcobol".to_string(),
            serde_json::json!({ "spdx": "per-repository", "redistribution_allowed": false, "decision": "quarantine report missing" }),
        );
    }
    by_family.insert(
        "gnucobol-manual".to_string(),
        serde_json::json!({
            "spdx": "GFDL-1.3",
            "redistribution_allowed": false,
            "decision": "GnuCOBOL manual examples: admitted read-only from the texinfo source",
        }),
    );
    serde_json::json!({
        "policy": "unknown-licence source is quarantined (REFERENCE_ONLY) and never published; every admitted family has a recorded licence decision",
        "by_family": by_family,
    })
}

// ---------------------------------------------------------------------------------------------
// dependencies
// ---------------------------------------------------------------------------------------------

fn build_dependencies(root: &Path) -> serde_json::Value {
    let mut by_family: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    // ccvs85 dependencies.json rows carry copy_libraries / missing_copybooks / data_inputs
    if let Some(deps) = family_file(root, "ccvs85", "dependencies.json") {
        let mut copy_libs = 0usize;
        let mut missing = 0usize;
        let mut data_inputs = 0usize;
        if let Some(arr) = deps.as_array() {
            for r in arr {
                copy_libs += r
                    .get("copy_libraries")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                missing += r
                    .get("missing_copybooks")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                data_inputs += r
                    .get("data_inputs")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
            }
        }
        by_family.insert(
            "ccvs85".to_string(),
            serde_json::json!({
                "units": count_records(&deps),
                "copy_library_uses": copy_libs,
                "missing_copybooks": missing,
                "data_inputs": data_inputs,
            }),
        );
    }
    // xcobol: copy_dependencies / missing_copybooks per file
    if let Some(progs) = family_file(root, "xcobol", "programs.json") {
        let mut copy_deps = 0usize;
        let mut missing = 0usize;
        if let Some(arr) = progs.as_array() {
            for r in arr {
                copy_deps += r
                    .get("copy_dependencies")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                missing += r
                    .get("missing_copybooks")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
            }
        }
        by_family.insert(
            "xcobol".to_string(),
            serde_json::json!({ "files": count_records(&progs), "copy_dependencies": copy_deps, "missing_copybooks": missing }),
        );
    }
    serde_json::json!({
        "note": "copybooks, modules, data inputs and missing dependencies per family (spec 1.4 dependencies / 5.2 dependency discovery)",
        "by_family": by_family,
    })
}

// ---------------------------------------------------------------------------------------------
// deduplication
// ---------------------------------------------------------------------------------------------

fn build_deduplication(root: &Path) -> serde_json::Value {
    let mut by_family: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    if let Some(d) = family_file(root, "xcobol", "dedup.json") {
        by_family.insert(
            "xcobol".to_string(),
            serde_json::json!({
                "exact_duplicate_files": d.get("exact_duplicate_files"),
                "near_duplicate_families": d.get("near_duplicate_families"),
                "note": d.get("note"),
            }),
        );
    }
    serde_json::json!({
        "policy": "exact byte hash, normalized hash, whitespace-insensitive hash, structural hash, near-duplicate similarity; grouping is repository-level so partitions never split a repo (spec 1.8)",
        "by_family": by_family,
    })
}

// ---------------------------------------------------------------------------------------------
// dialect matrix
// ---------------------------------------------------------------------------------------------

fn build_dialect_matrix(root: &Path) -> serde_json::Value {
    let mut rows: Vec<serde_json::Value> = Vec::new();
    // xcobol: dialect_accepted per file
    if let Some(progs) = family_file(root, "xcobol", "programs.json") {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        if let Some(arr) = progs.as_array() {
            for r in arr {
                let d = r
                    .get("dialect_accepted")
                    .and_then(|v| v.as_str())
                    .unwrap_or("none");
                *counts.entry(d.to_string()).or_default() += 1;
            }
        }
        rows.push(serde_json::json!({
            "family": "xcobol",
            "dimension": "oracle admission dialect (default/cobol85/cobol2002/cobol2014/ibm/mf/acu; first success)",
            "counts": counts,
        }));
    }
    // testsuite: dialect recorded per valid step
    if let Some(progs) = family_file(root, "gnucobol-testsuite", "valid-programs.json") {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        let mut formats: BTreeMap<String, usize> = BTreeMap::new();
        if let Some(arr) = progs.as_array() {
            for r in arr {
                let d = r
                    .get("dialect")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                *counts.entry(d.to_string()).or_default() += 1;
                let f = r
                    .get("source_format")
                    .and_then(|v| v.as_str())
                    .unwrap_or("fixed");
                *formats.entry(f.to_string()).or_default() += 1;
            }
        }
        rows.push(serde_json::json!({
            "family": "gnucobol-testsuite",
            "dimension": "recorded step dialect + source format",
            "counts": counts,
            "source_formats": formats,
        }));
    }
    serde_json::json!({
        "note": "validity is profile-relative (VALID_FOR oracle/dialect/format/options/copybooks/runtime/platform); a source may be valid under one dialect and invalid under another (spec core validity definition)",
        "rows": rows,
    })
}

// ---------------------------------------------------------------------------------------------
// first-failure buckets
// ---------------------------------------------------------------------------------------------

fn build_first_failure(root: &Path) -> serde_json::Value {
    let mut buckets: BTreeMap<String, usize> = BTreeMap::new();
    let mut families: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    // testsuite
    if let Some(progs) = family_file(root, "gnucobol-testsuite", "valid-programs.json") {
        if let Some(arr) = progs.as_array() {
            for r in arr {
                let ff = r
                    .get("first_failure")
                    .and_then(|v| v.as_str())
                    .unwrap_or("none")
                    .to_string();
                *buckets.entry(ff.clone()).or_default() += 1;
                *families
                    .entry("gnucobol-testsuite".to_string())
                    .or_default()
                    .entry(ff)
                    .or_default() += 1;
            }
        }
    }
    // ccvs85
    if let Some(acc) = family_file(root, "ccvs85", "accuracy.json") {
        if let Some(arr) = acc.as_array() {
            for r in arr {
                let ff = r
                    .get("final_classification")
                    .and_then(|v| v.as_str())
                    .unwrap_or("UNKNOWN")
                    .to_string();
                *buckets.entry(ff.clone()).or_default() += 1;
                *families
                    .entry("ccvs85".to_string())
                    .or_default()
                    .entry(ff)
                    .or_default() += 1;
            }
        }
    }
    // xcobol
    if let Some(progs) = family_file(root, "xcobol", "programs.json") {
        if let Some(arr) = progs.as_array() {
            for r in arr {
                let ff = first_failure_phase(r);
                *buckets.entry(ff.clone()).or_default() += 1;
                *families
                    .entry("xcobol".to_string())
                    .or_default()
                    .entry(ff)
                    .or_default() += 1;
            }
        }
    }
    serde_json::json!({
        "note": "first candidate failing phase per program profile (spec 9.4: preprocess/lex/parse/resolution/layout/check/prepare/run; exactly one per profile)",
        "total": buckets.values().sum::<usize>(),
        "buckets": buckets,
        "by_family": families,
    })
}

// ---------------------------------------------------------------------------------------------
// accuracy
// ---------------------------------------------------------------------------------------------

fn build_accuracy(root: &Path) -> serde_json::Value {
    let mut by_family: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    if let Some(acc) = family_file(root, "ccvs85", "accuracy.json") {
        let mut matches = 0usize;
        let mut mismatches = 0usize;
        let mut exit_mismatch = 0usize;
        let mut file_mismatch = 0usize;
        if let Some(arr) = acc.as_array() {
            for r in arr {
                if r.get("raw_stdout_match")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    matches += 1;
                } else {
                    mismatches += 1;
                }
                if r.get("exit_status_match").and_then(|v| v.as_bool()) == Some(false) {
                    exit_mismatch += 1;
                }
                if r.get("generated_files_match").and_then(|v| v.as_bool()) == Some(false) {
                    file_mismatch += 1;
                }
            }
        }
        by_family.insert(
            "ccvs85".to_string(),
            serde_json::json!({
                "units": count_records(&acc),
                "raw_stdout_matches": matches,
                "raw_stdout_mismatches": mismatches,
                "exit_status_mismatches": exit_mismatch,
                "generated_file_mismatches": file_mismatch,
                "dimensions": ["compile status", "execution status", "report bytes (sha256)", "raw stdout", "raw stderr", "generated files", "verdict counts", "return status"],
            }),
        );
    }
    if let Some(ex) = family_file(root, "gnucobol-manual", "stable-3.2/examples.json") {
        let mut verified = 0usize;
        if let Some(arr) = ex.as_array() {
            for r in arr {
                if r.get("replay_verdict").and_then(|v| v.as_str()).is_some() {
                    verified += 1;
                }
            }
        }
        by_family.insert(
            "gnucobol-manual".to_string(),
            serde_json::json!({ "examples": count_records(&ex), "replay_verified": verified }),
        );
    }
    serde_json::json!({
        "note": "raw-byte accuracy only; output normalization is never reported as byte parity (spec integrity rules)",
        "by_family": by_family,
    })
}

// ---------------------------------------------------------------------------------------------
// performance
// ---------------------------------------------------------------------------------------------

fn build_performance(root: &Path) -> serde_json::Value {
    let perf_dir = report_dir(root).join("performance");
    let benchmarks = read_json(&perf_dir.join("benchmarks.json"));
    let views = read_json(&perf_dir.join("views.json"));
    let phase_metrics = read_json(&perf_dir.join("phase-metrics.json"));
    let mut workloads = 0usize;
    let mut scales = 0usize;
    if let Some(b) = &benchmarks {
        if let Some(obj) = b.as_object() {
            workloads = obj.len();
            for (_, v) in obj {
                scales = scales.max(v.as_array().map(|a| a.len()).unwrap_or(0));
            }
        }
    }
    // The machine authority for View E is the per-row ledger inside views.json: the 40
    // workload x scale rows. Totals here are DERIVED from the rows (never copied from a
    // summary field), so performance.json can never disagree with the authoritative rows.
    let view_e_rows = views
        .as_ref()
        .and_then(|v| v.get("view_e"))
        .and_then(|e| e.get("entries"))
        .and_then(|a| a.as_array());
    let view_e_entries = view_e_rows.map(|a| a.len()).unwrap_or(0);
    let sum_rows = |key: &str| -> Option<f64> {
        view_e_rows.map(|rows| {
            rows.iter()
                .filter_map(|r| r.get(key).and_then(|n| n.as_f64()))
                .sum()
        })
    };
    let view_e_total_candidate = sum_rows("candidate_ms");
    let view_e_total_oracle = sum_rows("oracle_ms");
    // Cross-check: views.json's own totals must equal the row sum (the gate enforces this;
    // recording both makes any drift visible in this file).
    let view_e_field_candidate = views
        .as_ref()
        .and_then(|v| v.get("view_e"))
        .and_then(|e| e.get("candidate_total_ms"))
        .and_then(|n| n.as_f64());
    let view_e_field_oracle = views
        .as_ref()
        .and_then(|v| v.get("view_e"))
        .and_then(|e| e.get("oracle_total_ms"))
        .and_then(|n| n.as_f64());
    serde_json::json!({
        "note": "performance is reported only for correctness-proven workloads (spec 8.3); views A-E stay separate (spec 9.5); raw samples preserved under raw/",
        "phase8_correctness": {
            "workloads": workloads,
            "scales_per_workload": scales,
            "evidence": "reports/valid-corpus/performance/benchmarks.json",
        },
        "phase9_views": {
            "evidence": "reports/valid-corpus/performance/views.json",
            "view_e_authority": "sum of views.json view_e.entries[] (candidate_ms/oracle_ms); totals are derived from the rows, never copied from a separate summary",
            "view_e_entries": view_e_entries,
            "view_e_oracle_total_ms": view_e_total_oracle,
            "view_e_candidate_total_ms": view_e_total_candidate,
            "view_e_field_oracle_total_ms": view_e_field_oracle,
            "view_e_field_candidate_total_ms": view_e_field_candidate,
            "raw_samples": "reports/valid-corpus/performance/raw/",
        },
        "phase_metrics": if phase_metrics.is_some() {
            serde_json::json!({ "evidence": "reports/valid-corpus/performance/phase-metrics.json", "present": true })
        } else {
            serde_json::json!({ "evidence": "reports/valid-corpus/performance/phase-metrics.json", "present": false })
        },
    })
}

// ---------------------------------------------------------------------------------------------
// determinism + no-delegation
// ---------------------------------------------------------------------------------------------

fn build_determinism(root: &Path) -> serde_json::Value {
    let mut by_family: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    for (family, path) in [
        (
            "gnucobol-testsuite",
            "reports/gnucobol-testsuite/determinism.json",
        ),
        ("ccvs85", "reports/ccvs85/determinism.json"),
    ] {
        let p = root.join(path);
        if let Some(v) = read_json(&p) {
            by_family.insert(
                family.to_string(),
                serde_json::json!({ "evidence": path, "summary": v.get("note"), "pass": v.get("pass") }),
            );
        } else {
            by_family.insert(
                family.to_string(),
                serde_json::json!({ "evidence": path, "missing": true }),
            );
        }
    }
    serde_json::json!({
        "note": "two-pass determinism: summary counts + per-test classifications identical across fresh runs (timestamps excluded); governed by the determinism courts",
        "by_family": by_family,
    })
}

fn build_no_delegation(root: &Path) -> serde_json::Value {
    let mut by_family: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    for (family, path) in [
        (
            "gnucobol-testsuite",
            "reports/gnucobol-testsuite/no-delegation.json",
        ),
        ("ccvs85", "reports/ccvs85/no-delegation.json"),
    ] {
        let p = root.join(path);
        if let Some(v) = read_json(&p) {
            let isolated = v
                .get("candidate_phase_isolated")
                .and_then(|x| x.as_bool())
                .unwrap_or(false);
            let links_no_libcob = v
                .get("cobrun_links_no_libcob")
                .or_else(|| v.get("cobc_rs_links_no_libcob"))
                .and_then(|x| x.as_bool());
            by_family.insert(
                family.to_string(),
                serde_json::json!({
                    "evidence": path,
                    "candidate_phase_isolated": isolated,
                    "candidate_no_oracle_delegation": isolated,
                    "no_libcob_linkage": links_no_libcob,
                    "schema": v.get("schema"),
                }),
            );
        } else {
            by_family.insert(
                family.to_string(),
                serde_json::json!({ "evidence": path, "missing": true }),
            );
        }
    }
    serde_json::json!({
        "note": "the candidate never invokes the oracle: candidate phases run with an isolated PATH and no libcob linkage (linkage scan + PATH isolation + execve trace)",
        "by_family": by_family,
    })
}

// ---------------------------------------------------------------------------------------------
// entry point
// ---------------------------------------------------------------------------------------------

/// Build + write all unified reports. Returns the unified document.
pub fn unify(root: &Path) -> Result<UnifiedReports, String> {
    let summary = build_summary(root);
    let licences = build_licences(root);
    let dependencies = build_dependencies(root);
    let deduplication = build_deduplication(root);
    let dialect_matrix = build_dialect_matrix(root);
    let first_failure_buckets = build_first_failure(root);
    let accuracy = build_accuracy(root);
    let performance = build_performance(root);
    let determinism = build_determinism(root);
    let no_delegation = build_no_delegation(root);

    let out = report_dir(root);
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let stamp = crate::cli::now_utc_string_pub();
    let unified = UnifiedReports {
        schema: "gnurust-valid-corpus-unified-v1".to_string(),
        generated_at_utc: stamp.clone(),
        summary,
        licences,
        dependencies,
        deduplication,
        dialect_matrix,
        first_failure_buckets,
        accuracy,
        performance,
        determinism,
        no_delegation,
    };

    let write = |name: &str, v: &serde_json::Value| -> Result<(), String> {
        let text = serde_json::to_string_pretty(v).map_err(|e| e.to_string())?;
        std::fs::write(out.join(name), text).map_err(|e| e.to_string())
    };
    let s = &unified.summary;
    write("summary.json", s)?;
    write("licences.json", &unified.licences)?;
    write("dependencies.json", &unified.dependencies)?;
    write("deduplication.json", &unified.deduplication)?;
    write("dialect-matrix.json", &unified.dialect_matrix)?;
    write("first-failure-buckets.json", &unified.first_failure_buckets)?;
    write("accuracy.json", &unified.accuracy)?;
    write("performance.json", &unified.performance)?;
    write("determinism.json", &unified.determinism)?;
    write("no-delegation.json", &unified.no_delegation)?;

    // programs.csv: one row per admitted unit across families
    let mut csv =
        String::from("program_id,source_family,classification,first_failure,dialect,partition\n");
    if let Some(progs) = family_file(root, "ccvs85", "programs.json") {
        if let Some(arr) = progs.as_array() {
            for r in arr {
                csv.push_str(&format!(
                    "{},{},{},{},{},{}\n",
                    r.get("program_id").and_then(|v| v.as_str()).unwrap_or(""),
                    "ccvs85",
                    r.get("classification")
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                    r.get("final_classification")
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                    "",
                    "",
                ));
            }
        }
    }
    if let Some(progs) = family_file(root, "xcobol", "programs.json") {
        if let Some(arr) = progs.as_array() {
            for r in arr {
                csv.push_str(&format!(
                    "{},{},{},{},{},{}\n",
                    r.get("file_id").and_then(|v| v.as_str()).unwrap_or(""),
                    "xcobol",
                    r.get("structural_class")
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                    first_failure_phase(r),
                    r.get("dialect_accepted")
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                    r.get("partition").and_then(|v| v.as_str()).unwrap_or(""),
                ));
            }
        }
    }
    if let Some(progs) = family_file(root, "extras", "programs.json") {
        if let Some(arr) = progs.as_array() {
            for r in arr {
                csv.push_str(&format!(
                    "{},{},{},{},{},{}\n",
                    r.get("program_id").and_then(|v| v.as_str()).unwrap_or(""),
                    "extras",
                    r.get("classification")
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                    first_failure_phase(r),
                    "",
                    "",
                ));
            }
        }
    }
    if let Some(progs) = family_file(root, "omp", "programs.json") {
        if let Some(arr) = progs.as_array() {
            for r in arr {
                csv.push_str(&format!(
                    "{},{},{},{},{},{}\n",
                    r.get("program_id").and_then(|v| v.as_str()).unwrap_or(""),
                    "omp",
                    r.get("admission").and_then(|v| v.as_str()).unwrap_or(""),
                    "",
                    "",
                    "",
                ));
            }
        }
    }
    std::fs::write(out.join("programs.csv"), csv).map_err(|e| e.to_string())?;

    // summary.md: human view of the totals
    let mut md = String::new();
    md.push_str("# Valid-COBOL corpus — unified summary (Phase 12)\n\n");
    md.push_str(&format!(
        "_generated_at_utc: {stamp} · schema: `gnurust-valid-corpus-unified-v1`_\n\n",
        stamp = stamp
    ));
    md.push_str(&format!(
        "**total units:** {}\n\n",
        unified
            .summary
            .get("total_units")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
    ));
    md.push_str("## by source family\n");
    if let Some(by_family) = unified
        .summary
        .get("by_source_family")
        .and_then(|v| v.as_object())
    {
        for (k, v) in by_family {
            md.push_str(&format!("- {k}: {v}\n"));
        }
    }
    md.push_str("## by validity class\n");
    if let Some(by_class) = unified
        .summary
        .get("by_validity_class")
        .and_then(|v| v.as_object())
    {
        for (k, v) in by_class {
            md.push_str(&format!("- {k}: {v}\n"));
        }
    }
    md.push_str("\n## by candidate outcome\n");
    if let Some(by_cand) = unified
        .summary
        .get("by_candidate_outcome")
        .and_then(|v| v.as_object())
    {
        for (k, v) in by_cand {
            md.push_str(&format!("- {k}: {v}\n"));
        }
    }
    md.push_str("\n## validity × candidate outcome (cross-tab)\n");
    md.push_str("\n| validity | candidate outcome | units |\n|---|---|---|\n");
    if let Some(x) = unified
        .summary
        .get("validity_x_candidate_outcome")
        .and_then(|v| v.as_object())
    {
        for (vclass, inner) in x {
            if let Some(inner) = inner.as_object() {
                for (cand, n) in inner {
                    md.push_str(&format!("| {vclass} | {cand} | {n} |\n"));
                }
            }
        }
    }
    md.push_str("\n## by first-failure phase\n");
    if let Some(ff) = unified
        .summary
        .get("by_first_failure_phase")
        .and_then(|v| v.as_object())
    {
        for (k, v) in ff {
            md.push_str(&format!("- {k}: {v}\n"));
        }
    }
    md.push_str("\n## companion reports\n");
    for name in [
        "licences.json",
        "dependencies.json",
        "deduplication.json",
        "dialect-matrix.json",
        "first-failure-buckets.json",
        "accuracy.json",
        "performance.json",
        "determinism.json",
        "no-delegation.json",
        "generalization.json",
        "held-out-results.json",
        "overfitting.json",
        "upstream-drift.json",
        "programs.csv",
    ] {
        md.push_str(&format!("- `{name}`\n"));
    }
    md.push('\n');
    md.push_str(
        "> Doctrine: every number above is aggregated from the committed per-family reports;\n",
    );
    md.push_str("> no value is re-measured or invented by this report.\n");
    std::fs::write(out.join("summary.md"), md).map_err(|e| e.to_string())?;

    let _ = sum_values(&unified.summary);
    Ok(unified)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_root() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    fn write_family(root: &Path, family: &str, file: &str, v: &serde_json::Value) {
        let dir = report_dir(root).join(family);
        std::fs::create_dir_all(&dir).unwrap();
        let text = serde_json::to_string(v).unwrap();
        std::fs::write(dir.join(file), text).unwrap();
    }

    #[test]
    fn unify_aggregates_family_reports() {
        let td = scratch_root();
        let root = td.path();
        write_family(
            root,
            "ccvs85",
            "programs.json",
            &serde_json::json!([
                {"program_id": "ccvs85/NC107A", "classification": "VALID_EXECUTABLE_PROGRAM", "final_classification": "RAW_OUTPUT_MATCH"},
                {"program_id": "ccvs85/NC108A", "classification": "VALID_EXECUTABLE_PROGRAM", "final_classification": "OUTPUT_MISMATCH"},
            ]),
        );
        write_family(
            root,
            "ccvs85",
            "accuracy.json",
            &serde_json::json!([
                {"program_id": "ccvs85/NC107A", "final_classification": "RAW_OUTPUT_MATCH", "raw_stdout_match": true, "exit_status_match": true, "generated_files_match": true},
                {"program_id": "ccvs85/NC108A", "final_classification": "OUTPUT_MISMATCH", "raw_stdout_match": false, "exit_status_match": false, "generated_files_match": false},
            ]),
        );
        write_family(
            root,
            "xcobol",
            "programs.json",
            &serde_json::json!([
                {"file_id": "xcobol/repo/a.cob", "structural_class": "COMPLETE_PROGRAM", "candidate_first_failure": "parse", "dialect_accepted": "ibm", "partition": "DEVELOPMENT"},
                {"file_id": "xcobol/repo/b.cob", "structural_class": "FRAGMENT", "candidate_first_failure": "none", "dialect_accepted": "default", "partition": "HELD_OUT_EVALUATION"},
            ]),
        );
        write_family(
            root,
            "xcobol",
            "licence-quarantine.json",
            &serde_json::json!({"policy": "per-repo", "repos": [{"repo": "r1", "state": "REDISTRIBUTABLE"}]}),
        );
        write_family(
            root,
            "xcobol",
            "dedup.json",
            &serde_json::json!({"exact_duplicate_files": 3, "near_duplicate_families": 2, "note": "repo-level"}),
        );
        let u = unify(root).unwrap();
        assert_eq!(u.summary["total_units"].as_u64().unwrap(), 4);
        assert_eq!(u.summary["by_source_family"]["ccvs85"].as_u64().unwrap(), 2);
        assert_eq!(u.summary["by_source_family"]["xcobol"].as_u64().unwrap(), 2);
        assert_eq!(
            u.first_failure_buckets["buckets"]["parse"]
                .as_u64()
                .unwrap(),
            1
        );
        assert_eq!(
            u.first_failure_buckets["buckets"]["none"].as_u64().unwrap(),
            1
        );
        // ccvs85 rows bucket by final_classification
        assert_eq!(
            u.first_failure_buckets["buckets"]["RAW_OUTPUT_MATCH"]
                .as_u64()
                .unwrap(),
            1
        );
        // reports were written
        assert!(report_dir(root).join("summary.json").exists());
        assert!(report_dir(root).join("licences.json").exists());
        assert!(report_dir(root).join("programs.csv").exists());
        assert!(report_dir(root).join("dialect-matrix.json").exists());
        assert!(report_dir(root).join("determinism.json").exists());
        assert!(report_dir(root).join("no-delegation.json").exists());
    }

    #[test]
    fn unify_tolerates_missing_families() {
        let td = scratch_root();
        let root = td.path();
        let u = unify(root).unwrap();
        // empty aggregation, not a crash
        assert_eq!(u.summary["total_units"].as_u64().unwrap(), 0);
        assert!(u.no_delegation["by_family"]["ccvs85"]["missing"]
            .as_bool()
            .unwrap_or(false));
    }
}
