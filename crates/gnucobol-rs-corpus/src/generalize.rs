//! Phase 10.1/10.2/10.3 combined — the generalization report.
//!
//! Writes `reports/valid-corpus/generalization.json` with three sections: development-set
//! results, validation-set results (both computed from the RECORDED admission measurements in
//! `programs.json` — no re-probing), held-out results (reusing the `held-out` command's pure
//! measurement), and the overfitting summary (reusing the `overfit` command's checks). The
//! summary.md Phase-10 pointer is appended by the CLI layer; this module never rewrites the
//! existing summary content.

use crate::heldout::{evaluate_held_out, HeldOutReport, XcobolRow};
use crate::overfit::{run_checks, OverfitReport};
use crate::store::CorpusStore;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;

/// Development/validation set results computed from the recorded admission measurements.
#[derive(Debug, Clone, Serialize)]
pub struct SetResults {
    pub set: String,
    pub files: usize,
    pub candidate_accepted: usize,
    pub accept_rate: f64,
    pub first_failure_by_phase: BTreeMap<String, usize>,
    pub structural_distribution: BTreeMap<String, usize>,
}

/// The `generalization.json` report shape.
#[derive(Debug, Clone, Serialize)]
pub struct GeneralizationReport {
    pub development: SetResults,
    pub validation: SetResults,
    pub held_out: HeldOutReport,
    pub overfitting: OverfitReport,
    pub note: String,
}

/// Compute set results from the RECORDED candidate outcomes (no probing here — this is a
/// roll-up of the admission-time measurements, distinct from the held-out re-measurement).
pub fn compute_set_results(rows: &[XcobolRow], partition: &str) -> SetResults {
    let files: Vec<&XcobolRow> = rows.iter().filter(|r| r.partition == partition).collect();
    let mut first_failure_by_phase: BTreeMap<String, usize> = BTreeMap::new();
    let mut structural_distribution: BTreeMap<String, usize> = BTreeMap::new();
    let mut accepted = 0usize;
    for r in &files {
        *structural_distribution
            .entry(r.structural_class.clone())
            .or_default() += 1;
        match &r.candidate_first_failure {
            Some((phase, _)) => {
                *first_failure_by_phase.entry(phase.clone()).or_default() += 1;
            }
            None => {
                *first_failure_by_phase
                    .entry("none".to_string())
                    .or_default() += 1;
            }
        }
        if r.candidate_phases_ok {
            accepted += 1;
        }
    }
    SetResults {
        set: partition.to_string(),
        files: files.len(),
        candidate_accepted: accepted,
        accept_rate: if files.is_empty() {
            0.0
        } else {
            accepted as f64 / files.len() as f64
        },
        first_failure_by_phase,
        structural_distribution,
    }
}

/// Assemble the generalization report (held-out evaluation is a live bounded re-measurement and
/// requires the admitted X-COBOL dataset; the recorded-set sections and overfitting checks do
/// not).
pub fn run_generalize(root: &Path, store: &CorpusStore) -> Result<GeneralizationReport, String> {
    let rows = crate::heldout::load_xcobol_programs(root)?;
    let development = compute_set_results(&rows, "DEVELOPMENT");
    let validation = compute_set_results(&rows, "VALIDATION");
    let held_out = evaluate_held_out(root, store, &rows, true)?;
    let overfitting = run_checks(root)?;
    Ok(GeneralizationReport {
        development,
        validation,
        held_out,
        overfitting,
        note: "development and validation results are roll-ups of the recorded admission-time \
               measurements (programs.json); the held-out section is a fresh bounded re-\
               measurement that was never used for implementation tuning."
            .to_string(),
    })
}

/// The static Phase-10 pointer section appended to `reports/valid-corpus/summary.md` (idempotent;
/// never rewrites the existing summary content).
pub const SUMMARY_SECTION: &str = "\n## Phase 10 — generalization & overfitting\n\
See `held-out-results.json` (pure held-out measurement), `mutation-results.json` (metamorphic\n\
variant equivalence), `overfitting.json` (automated overfitting-indicator checks) and\n\
`generalization.json` (development/validation/held-out + overfitting summary). Run the\n\
`held-out`, `mutation`, `overfit` and `generalize` commands to (re)generate them.\n";

#[cfg(test)]
mod tests {
    use super::*;

    fn row(part: &str, class: &str, ok: bool, ff: Option<&str>) -> XcobolRow {
        XcobolRow {
            file_id: String::new(),
            repo: String::new(),
            path: String::new(),
            bytes: 0,
            extension: String::new(),
            structural_class: class.to_string(),
            encoding: String::new(),
            dialect_accepted: None,
            candidate_first_failure: ff.map(|p| (p.to_string(), "diag".to_string())),
            candidate_phases_ok: ok,
            partition: part.to_string(),
            exact_sha256: String::new(),
        }
    }

    #[test]
    fn compute_set_results_rolls_up_recorded_outcomes() {
        let rows = vec![
            row("DEVELOPMENT", "COMPLETE_PROGRAM", true, None),
            row("DEVELOPMENT", "COMPLETE_PROGRAM", false, Some("parse")),
            row("DEVELOPMENT", "COPYBOOK_OR_DATA", false, Some("preprocess")),
            row("VALIDATION", "COMPLETE_PROGRAM", true, None),
        ];
        let dev = compute_set_results(&rows, "DEVELOPMENT");
        assert_eq!(dev.files, 3);
        assert_eq!(dev.candidate_accepted, 1);
        assert_eq!(dev.first_failure_by_phase["parse"], 1);
        assert_eq!(dev.first_failure_by_phase["none"], 1);
        assert_eq!(dev.structural_distribution["COMPLETE_PROGRAM"], 2);
        let val = compute_set_results(&rows, "VALIDATION");
        assert_eq!(val.files, 1);
        assert_eq!(val.accept_rate, 1.0);
    }
}
