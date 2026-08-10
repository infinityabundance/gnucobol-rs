//! Phase-2 report writers: the required files under
//! `reports/valid-corpus/gnucobol-testsuite/`.

use crate::extract::at::AtGroup;
use crate::extract::StepResult;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Serialize)]
struct DiscoveredStepView {
    identity: String,
    lane: String,
    source_file: String,
    group_title: String,
    group_line: usize,
    step_index: usize,
    check_line: usize,
    command: String,
    status_expected: Option<i32>,
    stdout_expected_bytes: Option<usize>,
    stderr_expected_bytes: Option<usize>,
    contract_class: String,
    command_shape: String,
    replay_exit: Option<i32>,
    replay_mismatches: Vec<String>,
    skip_reason: String,
}

#[derive(Serialize)]
struct ValidProgramView {
    identity: String,
    lane: String,
    source_file: String,
    group_title: String,
    step_index: usize,
    command: String,
    classification: String,
    replay_exit: Option<i32>,
    replay_mismatches: Vec<String>,
    first_failure: Option<(String, String)>,
    source_format: String,
    dialect: String,
    captured_files: Vec<(String, bool)>,
}

#[derive(Serialize)]
struct InvalidProgramView {
    identity: String,
    lane: String,
    source_file: String,
    group_title: String,
    step_index: usize,
    command: String,
    status_expected: Option<i32>,
    replay_exit: Option<i32>,
    classification: String,
}

#[derive(Serialize)]
struct MixedGroupView {
    group: String,
    steps: usize,
    valid_steps: usize,
    invalid_steps: usize,
}

#[derive(Serialize)]
struct DependencyView {
    source_file: String,
    group_title: String,
    files: Vec<String>,
    capture_files: Vec<String>,
    skip_conditions: Vec<String>,
}

#[derive(Serialize)]
struct DriftView {
    group_title: String,
    stable_contract: Option<String>,
    current_contract: Option<String>,
    drift: String,
}

/// Write every Phase-2 report. Returns the summary counts.
pub fn write_reports(
    out_dir: &Path,
    stable: &[StepResult],
    current: &[StepResult],
    groups: &BTreeMap<String, Vec<AtGroup>>,
) -> Result<BTreeMap<String, usize>, String> {
    std::fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();

    // discovered-steps.json: every AT_CHECK step in both lanes
    let mut discovered: Vec<DiscoveredStepView> = Vec::new();
    for r in stable.iter().chain(current.iter()) {
        discovered.push(DiscoveredStepView {
            identity: r.identity.clone(),
            lane: r.lane.clone(),
            source_file: r.source_file.clone(),
            group_title: r.group_title.clone(),
            group_line: r.group_line,
            step_index: r.step_index,
            check_line: r.check_line,
            command: r.command.clone(),
            status_expected: r.status_expected,
            stdout_expected_bytes: r.stdout_expected_bytes,
            stderr_expected_bytes: r.stderr_expected_bytes,
            contract_class: r.contract_class.clone(),
            command_shape: r.command_shape.clone(),
            replay_exit: r.replay_exit,
            replay_mismatches: r.replay_mismatches.clone(),
            skip_reason: r.skip_reason.clone(),
        });
    }
    write_json(out_dir, "discovered-steps.json", &discovered)?;
    counts.insert("discovered_steps".into(), discovered.len());

    // valid-programs.json + invalid-programs.json
    let mut valid: Vec<ValidProgramView> = Vec::new();
    let mut invalid: Vec<InvalidProgramView> = Vec::new();
    for r in stable.iter().chain(current.iter()) {
        let is_valid = r.classification.starts_with("VALID_")
            && !r.classification.starts_with("VALID_COPYBOOK");
        if is_valid {
            valid.push(ValidProgramView {
                identity: r.identity.clone(),
                lane: r.lane.clone(),
                source_file: r.source_file.clone(),
                group_title: r.group_title.clone(),
                step_index: r.step_index,
                command: r.command.clone(),
                classification: r.classification.clone(),
                replay_exit: r.replay_exit,
                replay_mismatches: r.replay_mismatches.clone(),
                first_failure: r.first_failure.clone(),
                source_format: r.source_format.clone(),
                dialect: r.dialect.clone(),
                captured_files: r.captured_files.clone(),
            });
        } else if r.classification.contains("INVALID_EXPECTED_REJECT") {
            invalid.push(InvalidProgramView {
                identity: r.identity.clone(),
                lane: r.lane.clone(),
                source_file: r.source_file.clone(),
                group_title: r.group_title.clone(),
                step_index: r.step_index,
                command: r.command.clone(),
                status_expected: r.status_expected,
                replay_exit: r.replay_exit,
                classification: r.classification.clone(),
            });
        }
    }
    write_json(out_dir, "valid-programs.json", &valid)?;
    write_json(out_dir, "invalid-programs.json", &invalid)?;
    counts.insert("valid_programs".into(), valid.len());
    counts.insert("invalid_programs".into(), invalid.len());

    // mixed-groups.json
    let mixed = crate::extract::mixed_groups(
        &stable
            .iter()
            .chain(current.iter())
            .cloned()
            .collect::<Vec<_>>(),
    );
    let mixed_view: Vec<MixedGroupView> = mixed
        .iter()
        .map(|(g, steps, v, i)| MixedGroupView {
            group: g.clone(),
            steps: *steps,
            valid_steps: *v,
            invalid_steps: *i,
        })
        .collect();
    write_json(out_dir, "mixed-groups.json", &mixed_view)?;
    counts.insert("mixed_groups".into(), mixed_view.len());

    // dependency-graph.json
    let mut deps: Vec<DependencyView> = Vec::new();
    for (file, gs) in groups {
        for g in gs {
            deps.push(DependencyView {
                source_file: file.clone(),
                group_title: g.title.clone(),
                files: g.data_files.iter().map(|d| d.filename.clone()).collect(),
                capture_files: g.capture_files.clone(),
                skip_conditions: g.skip.clone(),
            });
        }
    }
    write_json(out_dir, "dependency-graph.json", &deps)?;

    // stable-current-drift.json
    let drift = drift_report(stable, current);
    write_json(out_dir, "stable-current-drift.json", &drift)?;
    counts.insert(
        "drift_changed".into(),
        drift.iter().filter(|d| d.drift != "unchanged").count(),
    );

    // summary.md
    let md = summary_md(stable, current, &counts);
    std::fs::write(out_dir.join("summary.md"), md).map_err(|e| e.to_string())?;
    Ok(counts)
}

fn drift_report(stable: &[StepResult], current: &[StepResult]) -> Vec<DriftView> {
    let contract = |r: &StepResult| -> String {
        format!(
            "status={:?} shape={} class={}",
            r.status_expected, r.command_shape, r.contract_class
        )
    };
    let mut stable_map: BTreeMap<String, String> = BTreeMap::new();
    for r in stable {
        stable_map
            .entry(r.group_title.clone())
            .or_insert_with(|| contract(r));
    }
    let mut current_map: BTreeMap<String, String> = BTreeMap::new();
    for r in current {
        current_map
            .entry(r.group_title.clone())
            .or_insert_with(|| contract(r));
    }
    let mut keys: BTreeSet<String> = BTreeSet::new();
    keys.extend(stable_map.keys().cloned());
    keys.extend(current_map.keys().cloned());
    keys.iter()
        .map(|k| {
            let s = stable_map.get(k).cloned();
            let c = current_map.get(k).cloned();
            let drift = match (&s, &c) {
                (Some(a), Some(b)) if a == b => "unchanged".to_string(),
                (Some(_), Some(_)) => "contract changed".to_string(),
                (Some(_), None) => "removed in current".to_string(),
                (None, Some(_)) => "added in current".to_string(),
                (None, None) => "unreachable".to_string(),
            };
            DriftView {
                group_title: k.clone(),
                stable_contract: s,
                current_contract: c,
                drift,
            }
        })
        .collect()
}

fn summary_md(
    stable: &[StepResult],
    current: &[StepResult],
    counts: &BTreeMap<String, usize>,
) -> String {
    let mut md = String::new();
    md.push_str("# GnuCOBOL Autotest suite — corpus extraction (Phase 2)\n\n");
    md.push_str("Classification happens at `AT_CHECK`-step level. Validity is profile-relative:\n");
    md.push_str(
        "every step carries its oracle identity, dialect, format and expected contract.\n\n",
    );
    md.push_str(&format!(
        "discovered steps (stable 3.2 + current): {}\n\n",
        counts.get("discovered_steps").copied().unwrap_or(0)
    ));
    for (lane, results) in [("stable-3.2", stable), ("current", current)] {
        let total = results.len();
        let valid = results
            .iter()
            .filter(|r| r.classification.starts_with("VALID_"))
            .count();
        let invalid = results
            .iter()
            .filter(|r| r.classification.contains("INVALID_EXPECTED_REJECT"))
            .count();
        let drift = results
            .iter()
            .filter(|r| r.classification.contains("ORACLE_CONTRACT_DRIFT"))
            .count();
        let skipped = results.iter().filter(|r| !r.skip_reason.is_empty()).count();
        md.push_str(&format!(
            "## {lane}\n- steps: {total}\n- contract-valid: {valid}\n- expected rejects: {invalid}\n- oracle-contract drift: {drift}\n- skipped under this oracle profile: {skipped}\n\n",
        ));
        // first-failure buckets (candidate)
        let mut buckets: BTreeMap<String, usize> = BTreeMap::new();
        for r in results {
            if let Some((phase, _)) = &r.first_failure {
                *buckets.entry(phase.clone()).or_default() += 1;
            }
        }
        if !buckets.is_empty() {
            md.push_str("### candidate first-failure buckets\n");
            for (k, v) in &buckets {
                md.push_str(&format!("- {k}: {v}\n"));
            }
            md.push('\n');
        }
    }
    md.push_str("## Notes\n\n");
    md.push_str(
        "- A step is valid only under the declared profile (oracle, dialect, format, options).\n",
    );
    md.push_str(
        "- `ORACLE_CONTRACT_DRIFT` = the suite declares the step valid but the admitted host\n",
    );
    md.push_str("  oracle disagreed on replay; kept as a first-class finding.\n");
    md.push_str(
        "- Screen tests (`$RUN_PROG_MANUAL`) and curses tests are skipped under this oracle\n",
    );
    md.push_str("  profile (no terminal); their sources are still extracted and compile-probed.\n");
    md.push_str("- Raw per-step evidence lives under `GNURUST_COBOL_CORPUS_ROOT/packages/`.\n");
    md
}

fn write_json<T: Serialize>(dir: &Path, name: &str, v: &T) -> Result<(), String> {
    let json = serde_json::to_string_pretty(v).map_err(|e| e.to_string())?;
    std::fs::write(dir.join(name), json).map_err(|e| e.to_string())
}

use std::collections::BTreeSet;
