//! Phase 10.3 — held-out evaluation (pure measurement; never a tuning input).
//!
//! Selects every X-COBOL file whose recorded partition is `HELD_OUT_EVALUATION`, probes the
//! candidate over each file's source bytes under a hard wall-clock bound (2 s per probe, via a
//! spawned thread + channel timeout), and produces the `held-out-results.json` report. The
//! command NEVER feeds the results back into the candidate: this is a pure measurement of
//! generalization, and the report states that the held-out set was not used for implementation
//! tuning.
//!
//! Source bytes are resolved in order: the content-addressed store (by the recorded
//! `exact_sha256`), the admitted X-COBOL extraction tree
//! (`lab/corpus/x-cobol/extracted/X-COBOL/COBOL_Files`), or the per-repo package work directory.
//! When none is present the command fails with a clear message (the X-COBOL dataset must be
//! admitted first); it never fabricates results.
//!
//! Execution is only attempted for `COMPLETE_PROGRAM` files (the structural class that can be
//! run), and even then the run is bounded by the same wall timeout, because some files contain
//! PERFORM loops.

use crate::extract::candidate::PhaseOutcome;
use crate::store::CorpusStore;
use gnucobol_rs::copybook::{self, CopyResolver};
use gnucobol_rs::dialect::Dialect;
use gnucobol_rs::frontend::probe_phases;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The X-COBOL Zenodo DOI (recorded in the custody report).
pub const XCOBOL_DOI: &str = "10.5281/zenodo.7968845";

/// Hard wall-clock bound for every candidate probe and run. A PERFORM loop must never hang a
/// report; exceeding the bound is recorded as `timed_out`, never as a crash or a pass.
pub const TIMEOUT: Duration = Duration::from_secs(2);
pub const TIMEOUT_SECS: u64 = 2;

/// Truncate a diagnostic for report rows (~200 chars).
pub fn truncate(s: &str, n: usize) -> String {
    let mut out: String = s.chars().take(n).collect();
    if s.chars().count() > n {
        out.push('…');
    }
    out
}

/// One row of `reports/valid-corpus/xcobol/programs.json` (the fields Phase 10 consumes; unknown
/// fields are ignored, missing fields default so older reports still load).
#[derive(Debug, Clone, Deserialize)]
pub struct XcobolRow {
    pub file_id: String,
    #[serde(default)]
    pub repo: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub bytes: usize,
    #[serde(default)]
    pub extension: String,
    #[serde(default)]
    pub structural_class: String,
    #[serde(default)]
    pub encoding: String,
    #[serde(default)]
    pub dialect_accepted: Option<String>,
    #[serde(default)]
    pub candidate_first_failure: Option<(String, String)>,
    #[serde(default)]
    pub candidate_phases_ok: bool,
    #[serde(default)]
    pub partition: String,
    #[serde(default)]
    pub exact_sha256: String,
}

impl XcobolRow {
    /// The file name (last path component), for the package-work-dir lookup.
    pub fn file_name(&self) -> &str {
        if self.path.is_empty() {
            self.file_id.rsplit('/').next().unwrap_or(&self.file_id)
        } else {
            self.path.rsplit('/').next().unwrap_or(&self.path)
        }
    }
}

/// `reports/valid-corpus/xcobol/programs.json`, the single recorded admission view.
pub fn xcobol_programs_path(root: &Path) -> PathBuf {
    root.join("reports")
        .join("valid-corpus")
        .join("xcobol")
        .join("programs.json")
}

/// Load the recorded X-COBOL file records (fails with a clear message when the dataset has not
/// been admitted — never fabricates).
pub fn load_xcobol_programs(root: &Path) -> Result<Vec<XcobolRow>, String> {
    let p = xcobol_programs_path(root);
    let bytes = std::fs::read(&p).map_err(|e| {
        format!(
            "cannot read {}: {e} — the X-COBOL dataset must be admitted first (run \
             `extract-xcobol`; DOI {XCOBOL_DOI})",
            p.display()
        )
    })?;
    serde_json::from_slice(&bytes).map_err(|e| format!("cannot parse {}: {e}", p.display()))
}

/// The admitted X-COBOL extraction tree's `COBOL_Files` directory.
pub fn xcobol_files_dir(root: &Path) -> PathBuf {
    root.join("lab")
        .join("corpus")
        .join("x-cobol")
        .join("extracted")
        .join("X-COBOL")
        .join("COBOL_Files")
}

/// Fetch the original source bytes of one X-COBOL file. Resolution order: store blob (by the
/// recorded `exact_sha256`), the admitted extraction tree, the per-repo package work directory.
pub fn resolve_source(
    store: &CorpusStore,
    root: &Path,
    row: &XcobolRow,
) -> Result<Vec<u8>, String> {
    if !row.exact_sha256.is_empty() {
        if let Some(b) = store.get_bytes(&row.exact_sha256) {
            return Ok(b);
        }
    }
    let rel = if row.path.is_empty() {
        format!("{}/{}", row.repo, row.file_name())
    } else {
        row.path.clone()
    };
    let in_tree = xcobol_files_dir(root).join(&rel);
    if let Ok(b) = std::fs::read(&in_tree) {
        return Ok(b);
    }
    let in_packages = store
        .root()
        .join("packages")
        .join("xcobol")
        .join(&row.repo)
        .join(row.file_name());
    if let Ok(b) = std::fs::read(&in_packages) {
        return Ok(b);
    }
    Err(format!(
        "source for {} is unavailable: no store blob {}, no file {}, no file {} — the X-COBOL \
         dataset must be admitted first (run `extract-xcobol`; DOI {XCOBOL_DOI})",
        row.file_id,
        row.exact_sha256,
        in_tree.display(),
        in_packages.display()
    ))
}

/// Filesystem copybook resolver rooted at a repository directory, with the candidate's system
/// copybooks as the last resort — the same order the suite probes use.
struct RepoCopyResolver {
    repo_dir: PathBuf,
    system: PathBuf,
}

impl CopyResolver for RepoCopyResolver {
    fn resolve(&self, name: &str) -> Option<String> {
        for base in [&self.repo_dir, &self.system] {
            for cand in [base.join(name), base.join(format!("{name}.cpy"))] {
                if let Ok(s) = std::fs::read_to_string(&cand) {
                    return Some(s);
                }
            }
        }
        None
    }
    fn resolve_in(&self, name: &str, dir: &str) -> Option<String> {
        for cand in [
            self.repo_dir.join(dir).join(name),
            self.repo_dir.join(dir).join(format!("{name}.cpy")),
        ] {
            if let Ok(s) = std::fs::read_to_string(&cand) {
                return Some(s);
            }
        }
        self.resolve(name)
    }
}

/// The outcome of one bounded candidate probe.
#[derive(Debug, Clone, Serialize)]
pub struct BoundedProbe {
    /// The phase probes in order. When the probe was killed by the wall bound or the candidate
    /// panicked inside the thread, a synthetic failing probe is produced so first-failure
    /// bucketing stays truthful.
    pub probes: Vec<PhaseOutcome>,
    pub timed_out: bool,
    pub crashed: bool,
}

/// Run `probe_phases` under a hard wall-clock bound. A panic inside the candidate is contained by
/// the spawned thread (all candidate state is thread-local) and reported as `crashed`; a timeout
/// is reported as `timed_out`.
pub fn probe_bounded(source: &str, dialect: Dialect, run: bool) -> BoundedProbe {
    let src = source.to_string();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(probe_phases(&src, dialect, run));
    });
    let synthetic = || PhaseOutcome {
        phase: if run { "run" } else { "check" }.to_string(),
        ok: false,
        diagnostic: format!(
            "candidate probe exceeded the {TIMEOUT_SECS}s wall bound or crashed (possible \
             PERFORM loop)"
        ),
    };
    match rx.recv_timeout(TIMEOUT) {
        Ok(probes) => BoundedProbe {
            probes: probes
                .into_iter()
                .map(|p| PhaseOutcome {
                    phase: match p.phase.as_str() {
                        "execute" => "run".to_string(),
                        other => other.to_string(),
                    },
                    ok: p.ok,
                    diagnostic: p.diagnostic,
                })
                .collect(),
            timed_out: false,
            crashed: false,
        },
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => BoundedProbe {
            probes: vec![synthetic()],
            timed_out: true,
            crashed: false,
        },
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => BoundedProbe {
            probes: vec![synthetic()],
            timed_out: false,
            crashed: true,
        },
    }
}

/// The outcome of one bounded prepare+run (used by the mutation harness).
#[derive(Debug, Clone, Serialize)]
pub struct BoundedRun {
    /// Exit code of the prepared program; `None` when preparation failed or the run was killed.
    pub exit: Option<i32>,
    /// Byte-identical stdout captured from the run.
    pub stdout: Vec<u8>,
    /// The failure diagnostic, when the program did not run to completion.
    pub error: Option<String>,
    pub timed_out: bool,
    pub crashed: bool,
}

/// Prepare + run under a hard wall-clock bound. Preparation failure, timeout and panic are all
/// contained by the spawned thread and reported distinctly.
pub fn run_bounded(source: &str) -> BoundedRun {
    let src = source.to_string();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let out = (|| -> Result<(Vec<u8>, i32), String> {
            let pp = gnucobol_rs::frontend::prepare_program(&src, Dialect::DEFAULT)
                .map_err(|e| e.to_string())?;
            let (stdout, _printer, rc) = pp.run(false).map_err(|e| e.to_string())?;
            Ok((stdout, rc))
        })();
        let _ = tx.send(out);
    });
    match rx.recv_timeout(TIMEOUT) {
        Ok(Ok((stdout, exit))) => BoundedRun {
            exit: Some(exit),
            stdout,
            error: None,
            timed_out: false,
            crashed: false,
        },
        Ok(Err(e)) => BoundedRun {
            exit: None,
            stdout: Vec::new(),
            error: Some(e),
            timed_out: false,
            crashed: false,
        },
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => BoundedRun {
            exit: None,
            stdout: Vec::new(),
            error: Some(format!(
                "run exceeded the {TIMEOUT_SECS}s wall bound (possible PERFORM loop)"
            )),
            timed_out: true,
            crashed: false,
        },
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => BoundedRun {
            exit: None,
            stdout: Vec::new(),
            error: Some("candidate run panicked inside the worker thread".to_string()),
            timed_out: false,
            crashed: true,
        },
    }
}

/// Per-file held-out row.
#[derive(Debug, Clone, Serialize)]
pub struct HeldOutFileResult {
    pub file_id: String,
    pub repo: String,
    pub partition: String,
    pub bytes: usize,
    pub structural_class: String,
    pub parse_ok: bool,
    pub check_ok: bool,
    pub run_attempted: bool,
    pub run_ok: bool,
    pub exit_code: Option<i32>,
    pub first_failure: Option<String>,
    pub crashed: bool,
    pub timed_out: bool,
    pub unsupported_diagnostics: Vec<String>,
    pub diagnostics: Vec<String>,
}

/// Aggregated held-out totals.
#[derive(Debug, Clone, Serialize, Default)]
pub struct HeldOutTotals {
    pub files: usize,
    pub parse_ok: usize,
    pub check_ok: usize,
    pub run_attempted: usize,
    pub run_ok: usize,
    pub crashed: usize,
    pub timed_out: usize,
    /// Files the candidate already accepted at admission time (recorded, not re-derived).
    pub candidate_ok: usize,
}

/// The `held-out-results.json` report shape.
#[derive(Debug, Clone, Serialize)]
pub struct HeldOutReport {
    pub timeout_seconds: u64,
    pub disclaimer: String,
    pub totals: HeldOutTotals,
    pub first_failure_by_phase: BTreeMap<String, usize>,
    pub dialect_distribution: BTreeMap<String, usize>,
    pub structural_distribution: BTreeMap<String, usize>,
    pub size_distribution: BTreeMap<String, usize>,
    pub files: Vec<HeldOutFileResult>,
}

fn size_bucket(bytes: usize) -> &'static str {
    if bytes < 1024 {
        "<1KB"
    } else if bytes < 10 * 1024 {
        "1-10KB"
    } else if bytes < 100 * 1024 {
        "10-100KB"
    } else {
        ">100KB"
    }
}

/// Evaluate the held-out set: a pure measurement over every recorded `HELD_OUT_EVALUATION` file.
/// Fails (with a clear message) when any held-out source cannot be resolved — the dataset must be
/// admitted first; results are never fabricated.
pub fn evaluate_held_out(
    root: &Path,
    store: &CorpusStore,
    rows: &[XcobolRow],
    with_run: bool,
) -> Result<HeldOutReport, String> {
    let files_dir = xcobol_files_dir(root);
    let mut held: Vec<&XcobolRow> = rows
        .iter()
        .filter(|r| r.partition == "HELD_OUT_EVALUATION")
        .collect();
    held.sort_by(|a, b| a.file_id.cmp(&b.file_id));
    let mut rep = HeldOutReport {
        timeout_seconds: TIMEOUT_SECS,
        disclaimer: "PURE MEASUREMENT: the held-out set was never used for implementation \
                     tuning and this report feeds nothing back into the candidate."
            .to_string(),
        totals: HeldOutTotals {
            files: held.len(),
            ..HeldOutTotals::default()
        },
        first_failure_by_phase: BTreeMap::new(),
        dialect_distribution: BTreeMap::new(),
        structural_distribution: BTreeMap::new(),
        size_distribution: BTreeMap::new(),
        files: Vec::with_capacity(held.len()),
    };
    for row in &held {
        let src = resolve_source(store, root, row)?;
        let text = String::from_utf8_lossy(&src).into_owned();
        let run_this = with_run && row.structural_class == "COMPLETE_PROGRAM";
        let resolver = RepoCopyResolver {
            repo_dir: files_dir.join(&row.repo),
            system: copybook::system_copy_dir(),
        };
        let expanded = match copybook::expand(&text, &resolver) {
            Ok(e) => e.text(),
            Err(e) => {
                let diag = truncate(&format!("copybook expansion failed: {e}"), 200);
                let unsupported = diag.to_lowercase().contains("unsupported");
                *rep.first_failure_by_phase
                    .entry("preprocess".to_string())
                    .or_default() += 1;
                *rep.dialect_distribution
                    .entry(
                        row.dialect_accepted
                            .as_deref()
                            .map(|d| format!("oracle-accepted:{d}"))
                            .unwrap_or_else(|| "oracle-accepted:none".to_string()),
                    )
                    .or_default() += 1;
                *rep.structural_distribution
                    .entry(row.structural_class.clone())
                    .or_default() += 1;
                *rep.size_distribution
                    .entry(size_bucket(row.bytes).to_string())
                    .or_default() += 1;
                rep.files.push(HeldOutFileResult {
                    file_id: row.file_id.clone(),
                    repo: row.repo.clone(),
                    partition: row.partition.clone(),
                    bytes: row.bytes,
                    structural_class: row.structural_class.clone(),
                    parse_ok: false,
                    check_ok: false,
                    run_attempted: false,
                    run_ok: false,
                    exit_code: None,
                    first_failure: Some("preprocess".to_string()),
                    crashed: false,
                    timed_out: false,
                    unsupported_diagnostics: if unsupported {
                        vec![diag.clone()]
                    } else {
                        Vec::new()
                    },
                    diagnostics: vec![diag],
                });
                continue;
            }
        };
        let bp = probe_bounded(&expanded, Dialect::DEFAULT, run_this);
        let probes = &bp.probes;
        let first = probes.iter().find(|p| !p.ok);
        let parse_ok = probes
            .iter()
            .find(|p| p.phase == "parse")
            .map(|p| p.ok)
            .unwrap_or(false);
        let check_ok = probes
            .iter()
            .find(|p| p.phase == "check")
            .map(|p| p.ok)
            .unwrap_or(false);
        let run_probe = probes.iter().find(|p| p.phase == "run");
        let run_attempted = run_this && !bp.timed_out && !bp.crashed && run_probe.is_some();
        let run_ok = run_attempted && run_probe.map(|p| p.ok).unwrap_or(false);
        let exit_code = if run_ok {
            run_probe.and_then(|p| {
                p.diagnostic
                    .strip_prefix("exit ")
                    .and_then(|s| s.parse::<i32>().ok())
            })
        } else {
            None
        };
        let first_failure = first.map(|p| p.phase.clone());
        let unsupported_diagnostics: Vec<String> = probes
            .iter()
            .filter(|p| !p.ok && p.diagnostic.to_lowercase().contains("unsupported"))
            .map(|p| truncate(&p.diagnostic, 200))
            .collect();
        let diagnostics: Vec<String> = match first {
            Some(p) => vec![truncate(&p.diagnostic, 200)],
            None => Vec::new(),
        };
        if let Some(ff) = &first_failure {
            *rep.first_failure_by_phase.entry(ff.clone()).or_default() += 1;
        } else {
            *rep.first_failure_by_phase
                .entry("none".to_string())
                .or_default() += 1;
        }
        *rep.dialect_distribution
            .entry(
                row.dialect_accepted
                    .as_deref()
                    .map(|d| format!("oracle-accepted:{d}"))
                    .unwrap_or_else(|| "oracle-accepted:none".to_string()),
            )
            .or_default() += 1;
        *rep.structural_distribution
            .entry(row.structural_class.clone())
            .or_default() += 1;
        *rep.size_distribution
            .entry(size_bucket(row.bytes).to_string())
            .or_default() += 1;
        if parse_ok {
            rep.totals.parse_ok += 1;
        }
        if check_ok {
            rep.totals.check_ok += 1;
        }
        if run_attempted {
            rep.totals.run_attempted += 1;
        }
        if run_ok {
            rep.totals.run_ok += 1;
        }
        if bp.crashed {
            rep.totals.crashed += 1;
        }
        if bp.timed_out {
            rep.totals.timed_out += 1;
        }
        if row.candidate_phases_ok {
            rep.totals.candidate_ok += 1;
        }
        rep.files.push(HeldOutFileResult {
            file_id: row.file_id.clone(),
            repo: row.repo.clone(),
            partition: row.partition.clone(),
            bytes: row.bytes,
            structural_class: row.structural_class.clone(),
            parse_ok,
            check_ok,
            run_attempted,
            run_ok,
            exit_code,
            first_failure,
            crashed: bp.crashed,
            timed_out: bp.timed_out,
            unsupported_diagnostics,
            diagnostics,
        });
    }
    Ok(rep)
}

impl HeldOutReport {
    /// A compact human-readable markdown summary of the held-out measurement.
    pub fn summary_md(&self) -> String {
        let mut md = String::new();
        md.push_str("# Held-out evaluation (Phase 10.3)\n\n");
        md.push_str(&format!("{}\n\n", self.disclaimer));
        md.push_str(&format!(
            "Every probe/run is bounded at {}s per file.\n\n",
            self.timeout_seconds
        ));
        md.push_str(&format!(
            "Every probe/run is bounded at {}s per file.\n\n",
            self.timeout_seconds
        ));
        md.push_str("| measure | count |\n|---|---|\n");
        md.push_str(&format!("| files | {} |\n", self.totals.files));
        md.push_str(&format!("| parse ok | {} |\n", self.totals.parse_ok));
        md.push_str(&format!("| check ok | {} |\n", self.totals.check_ok));
        md.push_str(&format!("| run ok | {} |\n", self.totals.run_ok));
        md.push_str(&format!("| crashed | {} |\n", self.totals.crashed));
        md.push_str(&format!("| timed out | {} |\n", self.totals.timed_out));
        md.push('\n');
        md.push_str("First-failure buckets:\n");
        for (k, v) in &self.first_failure_by_phase {
            md.push_str(&format!("- {k}: {v}\n"));
        }
        md.push('\n');
        md.push_str("See `held-out-results.json` for the per-file rows. This report is a pure\n");
        md.push_str("measurement; the held-out set is never used to modify the candidate.\n");
        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(partition: &str, class: &str, file_id: &str) -> XcobolRow {
        XcobolRow {
            file_id: file_id.to_string(),
            repo: "r".to_string(),
            path: format!("r/{file_id}"),
            bytes: 0,
            extension: "cob".to_string(),
            structural_class: class.to_string(),
            encoding: "UTF-8/ASCII".to_string(),
            dialect_accepted: Some("default".to_string()),
            candidate_first_failure: None,
            candidate_phases_ok: true,
            partition: partition.to_string(),
            exact_sha256: String::new(),
        }
    }

    fn hello() -> &'static str {
        "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       PROCEDURE DIVISION.\n           DISPLAY \"OK\".\n           STOP RUN.\n"
    }

    #[test]
    fn selector_picks_only_held_out_partition() {
        let rows = vec![
            row("DEVELOPMENT", "COMPLETE_PROGRAM", "dev.cob"),
            row("VALIDATION", "COMPLETE_PROGRAM", "val.cob"),
            row("HELD_OUT_EVALUATION", "COMPLETE_PROGRAM", "held.cob"),
        ];
        let dir = tempfile::tempdir().unwrap();
        let store = CorpusStore::open_at(dir.path()).unwrap();
        // materialize the held-out file in the dataset tree
        let tree = xcobol_files_dir(dir.path()).join("r");
        std::fs::create_dir_all(&tree).unwrap();
        std::fs::write(tree.join("held.cob"), hello()).unwrap();
        let rep = evaluate_held_out(dir.path(), &store, &rows, true).unwrap();
        assert_eq!(rep.totals.files, 1);
        assert_eq!(rep.files.len(), 1);
        assert_eq!(rep.files[0].file_id, "held.cob");
        // a complete program that runs cleanly
        assert!(rep.files[0].parse_ok);
        assert!(rep.files[0].check_ok);
        assert!(rep.files[0].run_attempted);
        assert!(rep.files[0].run_ok);
        assert_eq!(rep.files[0].exit_code, Some(0));
        assert_eq!(rep.files[0].first_failure, None);
        assert_eq!(rep.totals.run_ok, 1);
    }

    #[test]
    fn missing_sources_fail_with_clear_message() {
        let rows = vec![row("HELD_OUT_EVALUATION", "COMPLETE_PROGRAM", "gone.cob")];
        let dir = tempfile::tempdir().unwrap();
        let store = CorpusStore::open_at(dir.path()).unwrap();
        let e = evaluate_held_out(dir.path(), &store, &rows, true).unwrap_err();
        assert!(e.contains("must be admitted first"), "{e}");
    }

    #[test]
    fn run_is_not_attempted_for_non_program_classes() {
        let rows = vec![row("HELD_OUT_EVALUATION", "COPYBOOK_OR_DATA", "book.cpy")];
        let dir = tempfile::tempdir().unwrap();
        let store = CorpusStore::open_at(dir.path()).unwrap();
        let tree = xcobol_files_dir(dir.path()).join("r");
        std::fs::create_dir_all(&tree).unwrap();
        std::fs::write(tree.join("book.cpy"), b"01 X PIC 9(4).").unwrap();
        let rep = evaluate_held_out(dir.path(), &store, &rows, true).unwrap();
        assert!(!rep.files[0].run_attempted);
        // a copybook is not a program: no run probe, and the probe stops at parse
        assert!(!rep.files[0].parse_ok);
        assert!(rep.files[0].first_failure.is_some());
    }

    #[test]
    fn run_bounded_captures_stdout_and_exit() {
        let r = run_bounded(hello());
        assert_eq!(r.exit, Some(0));
        assert_eq!(r.stdout, b"OK\n");
        assert!(r.error.is_none());
    }

    #[test]
    fn run_bounded_reports_prepare_failure() {
        let r = run_bounded("NOT COBOL AT ALL");
        assert_eq!(r.exit, None);
        assert!(r.error.is_some());
    }

    #[test]
    fn resolve_source_prefers_store_blob_then_tree() {
        let dir = tempfile::tempdir().unwrap();
        let store = CorpusStore::open_at(dir.path()).unwrap();
        let r = row("HELD_OUT_EVALUATION", "COMPLETE_PROGRAM", "f.cob");
        // store blob wins
        let sha = store.put_bytes(b"from-store").unwrap();
        let mut r2 = r.clone();
        r2.exact_sha256 = sha.clone();
        assert_eq!(
            resolve_source(&store, dir.path(), &r2).unwrap(),
            b"from-store"
        );
        // tree fallback
        let tree = xcobol_files_dir(dir.path()).join("r");
        std::fs::create_dir_all(&tree).unwrap();
        std::fs::write(tree.join("f.cob"), b"from-tree").unwrap();
        assert_eq!(
            resolve_source(&store, dir.path(), &r).unwrap(),
            b"from-tree"
        );
        // clear error when neither exists
        let e = resolve_source(
            &store,
            dir.path(),
            &row("HELD_OUT_EVALUATION", "COMPLETE_PROGRAM", "nope.cob"),
        )
        .unwrap_err();
        assert!(e.contains("must be admitted first"), "{e}");
    }
}
