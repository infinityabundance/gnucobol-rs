//! Phase 2 — the GnuCOBOL native Autotest suite extractor.
//!
//! Extracts the suite's `AT_CHECK` steps into materialized, replayable program packages
//! (`package.rs`), classifies them from the upstream oracle contract (never from candidate
//! behaviour), replays the valid packages against the admitted host oracle (`oracle.rs`),
//! probes the candidate phase by phase (`candidate.rs`), compares the stable 3.2 and current
//! upstream lanes, and writes the Phase-2 reports under `reports/valid-corpus/gnucobol-testsuite/`.
//!
//! No regular-expression parse: `m4.rs` + `at.rs` are the syntax-aware front end and fail closed
//! on uncertain constructs.

pub mod at;
pub mod candidate;
pub mod ccvs85;
pub mod extras;
pub mod m4;
pub mod manual;
pub mod omp;
pub mod oracle;
pub mod package;
pub mod report;
pub mod xcobol;

use crate::extract::at::{parse_at, AtGroup};
use crate::extract::oracle::{compare_contract, condition_holds, run_step, OracleEnv};
use crate::extract::package::{build_step, Expected, StepClass, StepPackage};
use crate::store::sha256_hex;
use package::CommandShape::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// One lane of the suite: the admitted source tree + its label.
#[derive(Debug, Clone, Copy)]
pub struct SuiteLane {
    /// `stable-3.2` or `current`.
    pub label: &'static str,
    /// Tests directory relative to the workspace root.
    pub tests_dir: &'static str,
    /// The pinned revision (recorded in reports).
    pub revision: &'static str,
}

pub const STABLE_3_2: SuiteLane = SuiteLane {
    label: "stable-3.2",
    tests_dir: "lab/admit/gnucobol-3.2/tests",
    revision: "3.2.0 (admitted source tree)",
};

pub const CURRENT: SuiteLane = SuiteLane {
    label: "current",
    tests_dir: "lab/admit/gnucobol-upstream-current/tests",
    revision: "5568b8fc770ff310e5017300d561d8f3deec257c",
};

/// Resolve the workspace root (two levels above this crate: `crates/<crate-name>` -> repo root).
pub fn workspace_root() -> Result<PathBuf, String> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .ancestors()
        .nth(2)
        .map(|p| p.to_path_buf())
        .ok_or_else(|| "cannot resolve workspace root".to_string())
}

/// Recursively load a suite source file, resolving `m4_include`s depth-first (the same assembly
/// the suite does), and parse every group. Returns the groups in suite order plus any parse
/// errors.
pub fn load_suite_groups(lane: SuiteLane) -> Result<(Vec<AtGroup>, Vec<String>), String> {
    let root = workspace_root()?;
    let tests = root.join(lane.tests_dir);
    let mut errors = Vec::new();
    let mut groups = Vec::new();
    let mut visited = BTreeSet::new();
    load_file(
        &tests,
        &tests.join("testsuite.at"),
        &mut groups,
        &mut errors,
        &mut visited,
    )?;
    // testsuite_manual.at is a separate suite (AT_INIT) run with a different harness; parse it
    // too -- its groups are screen tests, compiled here and classified with their platform
    // boundary recorded.
    let manual = tests.join("testsuite_manual.at");
    if manual.exists() {
        load_file(&tests, &manual, &mut groups, &mut errors, &mut visited)?;
    }
    Ok((groups, errors))
}

fn load_file(
    tests: &Path,
    path: &Path,
    groups: &mut Vec<AtGroup>,
    errors: &mut Vec<String>,
    visited: &mut BTreeSet<String>,
) -> Result<(), String> {
    let key = path
        .strip_prefix(tests)
        .unwrap_or(path)
        .display()
        .to_string();
    if !visited.insert(key.clone()) {
        return Ok(()); // include cycles: the suite itself would fail; we fail closed on repeats
    }
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let parsed = parse_at(path, &text);
    errors.extend(parsed.parse_errors.iter().cloned());
    // Resolve includes in order; a group belongs to the file that declares it. The suite keeps
    // its per-banner sources under `testsuite.src/` and includes them by bare name, so an include
    // resolves against the including file's directory first, then `testsuite.src/`.
    for inc in &parsed.includes {
        let direct = tests.join(inc);
        let in_src = tests.join("testsuite.src").join(inc);
        let inc_path = if direct.exists() {
            direct
        } else if in_src.exists() {
            in_src
        } else {
            errors.push(format!(
                "{}: m4_include target missing: {inc}",
                path.display()
            ));
            continue;
        };
        load_file(tests, &inc_path, groups, errors, visited)?;
    }
    for mut g in parsed.groups {
        // family-relative source identity (lane + path)
        let rel = g.source_file.replace(&format!("{}/", tests.display()), "");
        g.source_file = rel.clone();
        groups.push(g);
    }
    Ok(())
}

/// One extracted step with its measured outcomes.
#[derive(Debug, Clone, Serialize)]
pub struct StepResult {
    pub identity: String,
    pub lane: String,
    pub source_file: String,
    pub group_title: String,
    pub group_line: usize,
    pub step_index: usize,
    pub check_line: usize,
    pub command: String,
    pub expanded_command: String,
    pub status_expected: Option<i32>,
    pub stdout_expected_bytes: Option<usize>,
    pub stderr_expected_bytes: Option<usize>,
    pub contract_class: String,
    pub command_shape: String,
    pub oracle_label: String,
    pub skip_reason: String,
    /// Oracle replay verdict: empty = replayed exactly as the contract declares.
    pub replay_mismatches: Vec<String>,
    pub replay_exit: Option<i32>,
    /// Candidate phase probes (empty when the step is not a valid-program candidate).
    pub candidate_phases: Vec<candidate::PhaseOutcome>,
    pub first_failure: Option<(String, String)>,
    pub classification: String,
    pub source_format: String,
    pub dialect: String,
    /// Generated-file expectations (`AT_CAPTURE_FILE`): (name, exists after replay).
    pub captured_files: Vec<(String, bool)>,
}

/// Build all step results for one lane: materialize the group packages, replay against the
/// oracle, probe the candidate for contract-valid steps.
pub fn extract_lane(
    lane: SuiteLane,
    oracle: &OracleEnv,
    packages_root: &Path,
    with_replay: bool,
    with_candidate: bool,
) -> Result<(Vec<StepResult>, BTreeMap<String, usize>), String> {
    let (groups, errors) = load_suite_groups(lane)?;
    if !errors.is_empty() {
        return Err(format!(
            "{} suite parse errors (fail closed): {}",
            lane.label,
            errors.join("; ")
        ));
    }
    let mut results = Vec::new();
    let mut stats: BTreeMap<String, usize> = BTreeMap::new();
    for (gi, group) in groups.iter().enumerate() {
        if group.checks.is_empty() {
            continue;
        }
        let group_no = gi + 1;
        // Materialize the group package (all AT_DATA files + the capture list).
        let group_dir = packages_root
            .join(lane.label)
            .join(group.source_file.replace('/', "-"))
            .join(format!("group-{group_no:04}"));
        std::fs::create_dir_all(&group_dir).map_err(|e| e.to_string())?;
        let mut file_hashes: BTreeMap<String, String> = BTreeMap::new();
        for data in &group.data_files {
            let name = &data.filename;
            let content = &data.content;
            let path = group_dir.join(name);
            std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
            std::fs::write(&path, content).map_err(|e| e.to_string())?;
            file_hashes.insert(name.clone(), sha256_hex(content.as_bytes()));
        }
        // skip evaluation (oracle env), group-level
        let mut skip_reason = String::new();
        for cond in &group.skip {
            if condition_holds(oracle, cond).unwrap_or(false) {
                skip_reason = format!("AT_SKIP_IF holds: {cond}");
                break;
            }
        }
        let xfail = group
            .xfail
            .iter()
            .find(|c| condition_holds(oracle, c).unwrap_or(false))
            .cloned();

        // Replay the group's checks in order (stateful, like the harness) when asked.
        let mut replay_outcomes: Vec<oracle::StepOutcome> = Vec::new();
        if with_replay && skip_reason.is_empty() {
            for _check in &group.checks {
                let pkg = build_step(
                    lane.label,
                    &oracle.label,
                    &group.source_file,
                    group,
                    replay_outcomes.len(),
                    group_no,
                );
                if check_is_replayable(&pkg) {
                    let outcome = run_step(oracle, &group_dir, &pkg.expanded_command, &[]);
                    replay_outcomes.push(outcome);
                } else {
                    replay_outcomes.push(oracle::StepOutcome {
                        exit: None,
                        stdout: Vec::new(),
                        stderr: Vec::new(),
                        exec_error: Some("command shape not replayable".to_string()),
                        skipped: false,
                        skip_reason: String::new(),
                    });
                }
            }
        }
        // Generated-file expectations (`AT_CAPTURE_FILE`): exists after the group replay.
        let mut captured: Vec<(String, bool)> = Vec::new();
        for name in &group.capture_files {
            captured.push((name.clone(), group_dir.join(name).exists()));
        }

        for (si, _check) in group.checks.iter().enumerate() {
            let pkg = build_step(
                lane.label,
                &oracle.label,
                &group.source_file,
                group,
                si,
                group_no,
            );
            let outcome = replay_outcomes
                .get(si)
                .cloned()
                .unwrap_or(oracle::StepOutcome {
                    exit: None,
                    stdout: Vec::new(),
                    stderr: Vec::new(),
                    exec_error: if with_replay {
                        None
                    } else {
                        Some("replay disabled".to_string())
                    },
                    skipped: !skip_reason.is_empty(),
                    skip_reason: skip_reason.clone(),
                });
            let mismatches = if outcome.skipped || !skip_reason.is_empty() {
                vec![]
            } else if let Some(xf) = &xfail {
                vec![format!("xfail condition holds: {xf}")]
            } else {
                compare_contract(&pkg, &outcome)
            };
            let class = classify_step(&pkg, &outcome, &mismatches, skip_reason.as_str());
            *stats.entry(class.clone()).or_default() += 1;
            // store the per-step package manifest (files + hashes) for custody BEFORE the probe:
            // the probe subprocess reads it.
            write_step_manifest(&pkg, &group_dir, &file_hashes)?;

            // candidate probe for contract-valid steps: bounded subprocess (hard `timeout`), so
            // no suite program can hang the corpus run (spec: no input hangs the candidate).
            let candidate_phases = if with_candidate
                && matches!(
                    class.as_str(),
                    "VALID_EXECUTABLE_PROGRAM"
                        | "VALID_COMPILE_ONLY_PROGRAM"
                        | "VALID_MODULE_PROGRAM"
                )
                && has_cobol_main(&pkg)
            {
                spawn_probe(&pkg, &group_dir)
            } else {
                Vec::new()
            };
            let first_failure = candidate_phases
                .iter()
                .find(|p| !p.ok)
                .map(|p| (p.phase.clone(), p.diagnostic.clone()));
            let candidate_class = match &first_failure {
                Some((phase, _)) => format!("CANDIDATE_{}_REJECT", phase.to_uppercase()),
                None if !candidate_phases.is_empty() => "CANDIDATE_ALL_PHASES_OK".to_string(),
                None => String::new(),
            };

            let mut rec = StepResult {
                identity: pkg.identity.clone(),
                lane: pkg.lane.clone(),
                source_file: pkg.source_file.clone(),
                group_title: pkg.group_title.clone(),
                group_line: pkg.group_line,
                step_index: pkg.step_index,
                check_line: pkg.check_line,
                command: pkg.command.clone(),
                expanded_command: pkg.expanded_command.clone(),
                status_expected: pkg.status_expected,
                stdout_expected_bytes: match &pkg.stdout_expected {
                    Expected::Ignore => None,
                    Expected::Text(t) => Some(t.len()),
                },
                stderr_expected_bytes: match &pkg.stderr_expected {
                    Expected::Ignore => None,
                    Expected::Text(t) => Some(t.len()),
                },
                contract_class: format!("{:?}", pkg.contract_class()),
                command_shape: format!("{:?}", pkg.command_shape()),
                oracle_label: oracle.label.clone(),
                skip_reason: skip_reason.clone(),
                replay_mismatches: mismatches,
                replay_exit: outcome.exit,
                candidate_phases,
                first_failure,
                classification: class.clone(),
                source_format: pkg.source_format.clone(),
                dialect: pkg.dialect.clone(),
                captured_files: captured.clone(),
            };
            rec.classification =
                if !candidate_class.is_empty() && rec.classification.starts_with("VALID_") {
                    format!("{}|{}", rec.classification, candidate_class)
                } else {
                    rec.classification
                };
            results.push(rec);
        }
    }
    Ok((results, stats))
}

/// Whether the package contains a COBOL main source (steps that compile C-only sources have
/// nothing to probe).
fn has_cobol_main(pkg: &StepPackage) -> bool {
    pkg.files
        .iter()
        .any(|(n, _)| n.ends_with(".cob") && !n.contains("expout") && !n.contains("experr"))
}

/// Run the candidate phase probe in a bounded subprocess (hard 90s `timeout`). The probe reads
/// the step manifest + package files itself; a timeout produces a typed `run`/`check` outcome,
/// never a hang.
fn spawn_probe(pkg: &StepPackage, group_dir: &Path) -> Vec<candidate::PhaseOutcome> {
    use std::process::Command;
    let exe = match std::env::current_exe() {
        Ok(e) => e,
        Err(_) => {
            return vec![candidate::PhaseOutcome {
                phase: "check".to_string(),
                ok: false,
                diagnostic: "cannot resolve the corpus binary for the probe subprocess".to_string(),
            }]
        }
    };
    let manifest = group_dir.join(format!("step-{:03}.json", pkg.step_index));
    let out = group_dir.join(format!("step-{:03}.candidate.json", pkg.step_index));
    let run = candidate::run_shape(&pkg.expanded_command);
    let status = Command::new("timeout")
        .arg("90")
        .arg(&exe)
        .arg("probe-step")
        .arg(&manifest)
        .arg("--out")
        .arg(&out)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    match status {
        Ok(s) if s.success() => match std::fs::read_to_string(&out) {
            Ok(json) => serde_json::from_str(&json).unwrap_or_else(|e| {
                vec![candidate::PhaseOutcome {
                    phase: "check".to_string(),
                    ok: false,
                    diagnostic: format!("probe output unreadable: {e}"),
                }]
            }),
            Err(e) => vec![candidate::PhaseOutcome {
                phase: "check".to_string(),
                ok: false,
                diagnostic: format!("probe output missing: {e}"),
            }],
        },
        _ => vec![candidate::PhaseOutcome {
            phase: if run {
                "run".to_string()
            } else {
                "check".to_string()
            },
            ok: false,
            diagnostic: "candidate probe timed out after 90s or failed to start (no hang "
                .to_string()
                + "may remain unrecorded)",
        }],
    }
}

fn check_is_replayable(pkg: &StepPackage) -> bool {
    !pkg.expanded_command.contains('$') || {
        // a leftover $ inside single quotes is shell-literal and safe
        let mut in_single = false;
        pkg.expanded_command.chars().all(|c| {
            match c {
                '\'' => in_single = !in_single,
                '$' if !in_single => return false,
                _ => {}
            }
            true
        })
    }
}

/// The corpus classification for a step, from the oracle contract + the replay evidence.
fn classify_step(
    pkg: &StepPackage,
    outcome: &oracle::StepOutcome,
    mismatches: &[String],
    skip_reason: &str,
) -> String {
    if !skip_reason.is_empty() {
        return "QUARANTINED".to_string(); // skipped under this oracle profile (e.g. curses)
    }
    match pkg.contract_class() {
        StepClass::Valid | StepClass::ValidWithExpectedWarning => {
            if mismatches.is_empty() {
                match pkg.command_shape() {
                    CompileOnly | Compile | Shell => "VALID_COMPILE_ONLY_PROGRAM".to_string(),
                    Run | ScreenRun => "VALID_EXECUTABLE_PROGRAM".to_string(),
                }
            } else {
                // the upstream contract declares valid but the admitted host oracle disagrees:
                // a first-class drift finding, never silently reclassified
                "ORACLE_CONTRACT_DRIFT".to_string()
            }
        }
        StepClass::InvalidExpectedReject => {
            if outcome.exit.is_none() {
                "INVALID_EXPECTED_REJECT".to_string()
            } else if outcome.exit == Some(0) {
                // the suite expects rejection but the oracle accepted the source: dialect drift
                "INVALID_EXPECTED_REJECT|ORACLE_ACCEPTS".to_string()
            } else {
                "INVALID_EXPECTED_REJECT".to_string()
            }
        }
        StepClass::ContractAnyStatus => {
            if mismatches.is_empty() {
                "DIAGNOSTIC_SHAPE_ONLY".to_string()
            } else {
                "INVALID_EXPECTED_REJECT".to_string()
            }
        }
    }
}

fn write_step_manifest(
    pkg: &StepPackage,
    group_dir: &Path,
    file_hashes: &BTreeMap<String, String>,
) -> Result<(), String> {
    #[derive(Serialize)]
    struct StepManifest<'a> {
        schema: &'static str,
        program_id: &'a str,
        corpus_class: &'static str,
        source_family: &'static str,
        lane: &'a str,
        group_title: &'a str,
        step_index: usize,
        command: &'a str,
        expanded_command: &'a str,
        status_expected: Option<i32>,
        files: &'a BTreeMap<String, String>,
        main_file: String,
        group_dir: String,
        capture_files: &'a [String],
        skip_conditions: &'a [String],
        xfail_conditions: &'a [String],
        source_format: &'a str,
        dialect: &'a str,
        oracle: &'a str,
    }
    let m = StepManifest {
        schema: "gnurust-gnucobol-testsuite-step-v1",
        program_id: &pkg.identity,
        corpus_class: "UPSTREAM_SEMANTIC",
        source_family: "GNUCOBOL_TESTSUITE",
        lane: &pkg.lane,
        group_title: &pkg.group_title,
        step_index: pkg.step_index,
        command: &pkg.command,
        expanded_command: &pkg.expanded_command,
        status_expected: pkg.status_expected,
        files: file_hashes,
        main_file: pkg
            .files
            .iter()
            .find(|(n, _)| n.ends_with(".cob") && !n.contains("expout") && !n.contains("experr"))
            .map(|(n, _)| n.clone())
            .unwrap_or_default(),
        group_dir: group_dir.display().to_string(),
        capture_files: &pkg.capture_files,
        skip_conditions: &pkg.skip_conditions,
        xfail_conditions: &pkg.xfail_conditions,
        source_format: &pkg.source_format,
        dialect: &pkg.dialect,
        oracle: &pkg.oracle,
    };
    let json = serde_json::to_string_pretty(&m).map_err(|e| e.to_string())?;
    let p = group_dir.join(format!("step-{:03}.json", pkg.step_index));
    std::fs::write(p, json).map_err(|e| e.to_string())
}

/// Mixed-group detection: groups containing both contract-valid and contract-invalid steps.
pub fn mixed_groups(results: &[StepResult]) -> Vec<(String, usize, usize, usize)> {
    let mut by_group: BTreeMap<String, Vec<&StepResult>> = BTreeMap::new();
    for r in results {
        by_group
            .entry(format!("{}/{}", r.source_file, r.group_title))
            .or_default()
            .push(r);
    }
    let mut out = Vec::new();
    for (k, steps) in by_group {
        let valid = steps
            .iter()
            .filter(|s| s.status_expected == Some(0))
            .count();
        let invalid = steps
            .iter()
            .filter(|s| s.status_expected.map(|v| v != 0).unwrap_or(false))
            .count();
        if valid > 0 && invalid > 0 {
            out.push((k, steps.len(), valid, invalid));
        }
    }
    out
}
