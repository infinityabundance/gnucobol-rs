//! GNURUST.CCVS85.3 — the gnucobol-rs candidate execution baseline.
//!
//! Runs the SAME materialized units through the current Rust front-end (`cobrun`, the canonical
//! parse+execute path of `gnucobol-rs`). The candidate phase NEVER invokes `cobc` and never links
//! or loads `libcob`; the caller (the Docker harness) enforces this mechanically by renaming the
//! oracle prefix away during the candidate phase, scrubbing `cobc` from PATH, and scanning
//! `cobrun`'s dynamic dependencies. This module records parse/prepare/run/timeout/failure data and
//! preserves raw stdout/stderr. It makes no suite-pass claim.

use crate::model::{CandidateSide, MaterializedUnit};
use crate::runner::{first_line, read_bytes, run_invocation};
use std::collections::BTreeMap;
use std::path::Path;

pub const CANDIDATE_RUN_TIMEOUT_SECS: u64 = 30;

/// Run one unit through `cobrun -fixed <unit.cob>`. The subprogram resolution (SUBRTN units are
/// separate `<sub>.cob` files beside the main) is handled by cobrun itself (`resolve_separate_calls`).
/// The data file (if any) is piped on stdin, mirroring the oracle side.
pub fn candidate_unit(
    unit: &MaterializedUnit,
    materialized_root: &Path,
    work_root: &Path,
    cobrun: &Path,
    env: &[(String, String)],
) -> CandidateSide {
    let mut side = CandidateSide::default();

    // Subprogram-only units are exercised through their main (the main's cobrun run resolves and
    // appends them); a SUBRTN unit is not an executable itself.
    if unit.subprogram.is_some() {
        side.prepare = "bound-to-main".to_string();
        side.run = "not-applicable".to_string();
        return side;
    }

    if !unit.missing_copybooks.is_empty() {
        side.prepare = "dependency-blocked".to_string();
        side.run = "not-applicable".to_string();
        return side;
    }

    let unit_dir = work_root.join(format!("u{}", unit.unit_index));
    let run_dir = unit_dir.join("run");
    std::fs::create_dir_all(&run_dir).ok();

    let src = materialized_root.join(&unit.adapted_path);
    let mut argv = vec![cobrun.to_string_lossy().into_owned()];
    argv.push("-fixed".to_string());
    argv.push(src.to_string_lossy().into_owned());

    let stdin = if unit.data_dependencies.is_empty() {
        None
    } else {
        let dat = materialized_root
            .join("data")
            .join(format!("{}.dat", unit.data_dependencies[0]));
        Some(read_bytes(&dat))
    };

    let inv = run_invocation(
        &argv,
        &run_dir,
        env,
        CANDIDATE_RUN_TIMEOUT_SECS,
        &run_dir.join("evidence"),
        stdin.as_deref(),
    );
    side.prepare_invocation = Some(inv.clone());
    side.prepare_invocation_rc = inv.exit_code;
    side.stdout_sha256 = inv.stdout_sha256.clone();
    side.run_invocation = Some(inv.clone());

    // cobrun's exit contract: 0 = ran to STOP RUN (RETURN-CODE 0); n = program RETURN-CODE; 2 with
    // an `unsupported:`/`undefined data name:`/`runtime error:` message = fail-closed rejection.
    let rc = inv.exit_code.unwrap_or(-1);
    let err = inv
        .stderr_path
        .as_deref()
        .map(|s| first_line(Path::new(s)))
        .unwrap_or_default();
    if inv.timed_out {
        side.prepare = "accepted".to_string();
        side.run = "timeout".to_string();
        return side;
    }
    if rc == 2 {
        let low = err.to_ascii_lowercase();
        if low.contains("unsupported") {
            side.prepare = "reject-unsupported".to_string();
        } else if low.contains("undefined data name") {
            side.prepare = "reject-parse".to_string();
        } else if low.contains("runtime error") {
            side.prepare = "reject-runtime-boundary".to_string();
        } else if low.contains("layout") {
            side.prepare = "reject-layout".to_string();
        } else {
            side.prepare = "reject-parse".to_string();
        }
        side.run = "not-run".to_string();
        return side;
    }
    // Any other non-zero exit: cobrun ran the program and the program's RETURN-CODE was non-zero
    // (STOP RUN n / MOVE n TO RETURN-CODE) — that is a run outcome, not a parse failure.
    side.prepare = "accepted".to_string();
    if rc == 0 {
        side.run = "pass".to_string();
    } else {
        side.run = "fail".to_string();
    }
    side
}

/// The full candidate phase over every COBOL unit (parallel workers).
pub fn run_candidate_phase(
    units: &[MaterializedUnit],
    materialized_root: &Path,
    work_root: &Path,
    cobrun: &Path,
    env: &[(String, String)],
    jobs: usize,
) -> BTreeMap<usize, CandidateSide> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    // COBOL unit positions in the units array (CLBRY/DATA* units are not exercised).
    let cobol_pos: Vec<usize> = units
        .iter()
        .enumerate()
        .filter(|(_, u)| u.kind == "COBOL")
        .map(|(i, _)| i)
        .collect();
    let total = cobol_pos.len();
    let counter = Arc::new(AtomicUsize::new(0));
    let results: Arc<Mutex<BTreeMap<usize, CandidateSide>>> = Arc::new(Mutex::new(BTreeMap::new()));
    let mut handles = Vec::new();
    for _ in 0..jobs.max(1) {
        let units = units.to_vec();
        let cobol_pos = cobol_pos.clone();
        let materialized_root = materialized_root.to_path_buf();
        let work_root = work_root.to_path_buf();
        let cobrun = cobrun.to_path_buf();
        let env = env.to_vec();
        let results = Arc::clone(&results);
        let counter = Arc::clone(&counter);
        handles.push(std::thread::spawn(move || loop {
            let i = counter.fetch_add(1, Ordering::SeqCst);
            if i >= total {
                break;
            }
            let unit = &units[cobol_pos[i]];
            let side = candidate_unit(unit, &materialized_root, &work_root, &cobrun, &env);
            results.lock().unwrap().insert(unit.unit_index, side);
        }));
    }
    for h in handles {
        let _ = h.join();
    }
    {
        let g = results.lock().unwrap();
        g.clone()
    }
}

/// Serialize the candidate phase results.
pub fn write_candidate_results(
    path: &Path,
    units: &[MaterializedUnit],
    results: &BTreeMap<usize, CandidateSide>,
) {
    let v: Vec<serde_json::Value> = units
        .iter()
        .map(|u| {
            serde_json::json!({
                "unit_index": u.unit_index,
                "kind": u.kind,
                "name": u.name,
                "source_path": u.source_path,
                "candidate": results.get(&u.unit_index),
            })
        })
        .collect();
    let _ = std::fs::write(path, serde_json::to_string_pretty(&v).unwrap() + "\n");
}
