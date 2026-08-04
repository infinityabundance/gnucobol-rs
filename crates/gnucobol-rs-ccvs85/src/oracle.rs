//! GNURUST.CCVS85.2 — the real-GnuCOBOL oracle baseline.
//!
//! For every applicable unit: materialize the complete source + copybook environment, compile with
//! the pinned GnuCOBOL 3.2 `cobc`, record the exact compile outcome, and (when the compile passes
//! and the unit is an executable candidate) execute under a timeout and record the run outcome,
//! the produced report file, and the parsed CCVS85 verdict counts.
//!
//! This gate makes NO claim about gnucobol-rs; it is the baseline the later gates compare against.

use crate::corpus::sha256_hex;
use crate::model::{Invocation, MaterializedUnit, OracleSide, VerdictCounts};
use crate::runner::{read_bytes, run_invocation};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub const ORACLE_COMPILE_TIMEOUT_SECS: u64 = 180;
pub const ORACLE_RUN_TIMEOUT_SECS: u64 = 30;

/// The dialect/flags for the corpus: CCVS85 is fixed-format COBOL-85; GnuCOBOL's default dialect
/// (with `-fixed`) is the corpus's natural home. Copybooks are resolved from the materialized,
/// site-adapted `copybooks-adapted/` dir via `-I` (the raw copybooks may carry column-7
/// site-adaptation markers that cobc would reject as invalid indicators).
pub fn cobc_compile_args(unit: &MaterializedUnit, source: &Path) -> Vec<String> {
    // `source` is the materialized root; the adapted copybooks/ dir lives directly under it.
    // Fall back to the raw copybooks for backward-compatible host-side runs.
    let adapted = source.join("copybooks-adapted");
    let copy_dir = if adapted.exists() {
        adapted
    } else {
        source.join("copybooks")
    };
    let mut args = vec![
        "-x".to_string(),
        "-fixed".to_string(),
        "-I".to_string(),
        copy_dir.to_string_lossy().into_owned(),
        "-o".to_string(),
        "program".to_string(),
    ];
    args.push(unit.adapted_path.clone());
    args
}

/// Whether the unit's site-adapted source references the RAW-DATA file card (XXXXX062).
pub fn adapted_references_raw_data(unit: &MaterializedUnit, materialized_root: &Path) -> bool {
    let src = materialized_root.join(&unit.adapted_path);
    match std::fs::read(&src) {
        Ok(b) => {
            let up = String::from_utf8_lossy(&b).to_ascii_uppercase();
            up.contains("XXXXX062") || up.contains("RAW-DATA")
        }
        Err(_) => false,
    }
}

/// Execute the compiled program in a scratch dir, with the data file (if any) piped on stdin and
/// the PRINT-FILE (XXXXX055) directed to a canonical report path via the GnuCOBOL env-var file map.
pub fn oracle_run(
    unit: &MaterializedUnit,
    materialized_root: &Path,
    work_root: &Path,
    prefix: &Path,
    env: &[(String, String)],
    binary: &Path,
) -> Invocation {
    let run_dir = work_root.join(format!("u{}", unit.unit_index)).join("run");
    std::fs::create_dir_all(&run_dir).ok();

    // The CCVS85 RAW-DATA harness file (ASSIGN name XXXXX062) is an indexed control file the site
    // provides. This harness seeds it with an EMPTY starter (created by the oracle at build time,
    // mirrored under `data/XXXXX062`) so `OPEN I-O RAW-DATA` succeeds and the module runs its own
    // literal-expectation tests standalone; a module without the starter would abort on OPEN
    // (status 35) for a reason unrelated to the source. Units that do not reference XXXXX062 are
    // unaffected.
    if adapted_references_raw_data(unit, materialized_root) {
        let starter = materialized_root.join("data/XXXXX062");
        if starter.exists() {
            let _ = std::fs::copy(&starter, run_dir.join("XXXXX062"));
        }
    }

    // GnuCOBOL resolves the external file name of `ASSIGN TO XXXXX055` (the CCVS85 PRINT-FILE)
    // through the environment variable of the same name, so we direct the report to a stable path.
    let report_path = run_dir.join("REPORT");
    let mut full_env = env.to_vec();
    full_env.push((
        "LD_LIBRARY_PATH".into(),
        prefix.join("lib").to_string_lossy().into_owned(),
    ));
    full_env.push((
        "COB_CONFIG_DIR".into(),
        prefix
            .join("share/gnucobol/config")
            .to_string_lossy()
            .into_owned(),
    ));
    full_env.push((
        "XXXXX055".into(),
        report_path.to_string_lossy().into_owned(),
    ));
    // Also pin the CCVS85 file-map variables for the data file (XXXXX001 etc. are not used by the
    // corpus's file-less modules; the ACCEPT data arrives on stdin).
    full_env.push(("COB_CURRENT_DATE".into(), env_current_date(env)));

    // stdin: the DATA* unit of the same name, if any.
    let stdin = if unit.data_dependencies.is_empty() {
        None
    } else {
        let dat = materialized_root
            .join("data")
            .join(format!("{}.dat", unit.data_dependencies[0]));
        Some(read_bytes(&dat))
    };

    let argv = vec![binary.to_string_lossy().into_owned()];
    run_invocation(
        &argv,
        &run_dir,
        &full_env,
        ORACLE_RUN_TIMEOUT_SECS,
        &run_dir.join("evidence"),
        stdin.as_deref(),
    )
}

/// The fixed `COB_CURRENT_DATE` value (YYYYMMDDHHMMSS[+/-HHMM]) used to pin date/time ACCEPTs.
/// Honors an externally supplied value (the harness's `SOURCE_DATE_EPOCH`-style pin) so the oracle
/// baseline is deterministic across runs.
fn env_current_date(env: &[(String, String)]) -> String {
    for (k, v) in env {
        if k == "COB_CURRENT_DATE" {
            return v.clone();
        }
    }
    // A fixed historical date (UTC) — deterministic by construction.
    "19920101000000+0000".to_string()
}

/// The full oracle phase: compile every COBOL unit; group SUBRTN subprograms with their mains;
/// run compiled executable candidates. Returns (unit_index -> OracleSide).
pub fn run_oracle_phase(
    units: &[MaterializedUnit],
    materialized_root: &Path,
    work_root: &Path,
    prefix: &Path,
    env: &[(String, String)],
    jobs: usize,
) -> (BTreeMap<usize, OracleSide>, Vec<String>) {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    // COBOL unit positions in the units array (CLBRY/DATA* units are not compiled).
    let cobol_pos: Vec<usize> = units
        .iter()
        .enumerate()
        .filter(|(_, u)| u.kind == "COBOL")
        .map(|(i, _)| i)
        .collect();
    let total = cobol_pos.len();
    let counter = Arc::new(AtomicUsize::new(0));
    let results: Arc<Mutex<BTreeMap<usize, OracleSide>>> = Arc::new(Mutex::new(BTreeMap::new()));
    let warnings: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    // Group SUBRTN units under their main: main units get their subprogram sources appended to the
    // cobc command line (compiled + linked together, mirroring `cobc -x main.cob sub.cob`).
    let sub_by_main: BTreeMap<String, Vec<&MaterializedUnit>> = {
        let mut m: BTreeMap<String, Vec<&MaterializedUnit>> = BTreeMap::new();
        for u in units {
            if u.kind == "COBOL" && u.subprogram.is_some() {
                if let Some(main) = &u.main_program {
                    m.entry(main.clone()).or_default().push(u);
                }
            }
        }
        m
    };

    // A tiny work-stealing executor: each worker pulls COBOL unit positions off an atomic counter.
    let mut handles = Vec::new();
    for _ in 0..jobs.max(1) {
        let units = units.to_vec();
        let cobol_pos = cobol_pos.clone();
        let materialized_root = materialized_root.to_path_buf();
        let work_root = work_root.to_path_buf();
        let prefix = prefix.to_path_buf();
        let env = env.to_vec();
        let sub_by_main: BTreeMap<String, Vec<String>> = sub_by_main
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    v.iter().map(|u| u.adapted_path.clone()).collect(),
                )
            })
            .collect();
        let results = Arc::clone(&results);
        let warnings = Arc::clone(&warnings);
        let counter = Arc::clone(&counter);
        handles.push(std::thread::spawn(move || loop {
            let i = counter.fetch_add(1, Ordering::SeqCst);
            if i >= total {
                break;
            }
            let unit = &units[cobol_pos[i]];
            let side = oracle_unit(
                unit,
                &materialized_root,
                &work_root,
                &prefix,
                &env,
                &sub_by_main,
                &warnings,
            );
            results.lock().unwrap().insert(unit.unit_index, side);
        }));
    }
    for h in handles {
        let _ = h.join();
    }
    let out = {
        let g = results.lock().unwrap();
        g.clone()
    };
    let warns = {
        let g = warnings.lock().unwrap();
        g.clone()
    };
    (out, warns)
}

fn oracle_unit(
    unit: &MaterializedUnit,
    materialized_root: &Path,
    work_root: &Path,
    prefix: &Path,
    env: &[(String, String)],
    sub_by_main: &BTreeMap<String, Vec<String>>,
    warnings: &Mutex<Vec<String>>,
) -> OracleSide {
    let mut side = OracleSide::default();

    // Subprogram-only units are compiled and linked with their main (the main's record carries the
    // combined compile); a SUBRTN unit is not an executable itself.
    if unit.subprogram.is_some() {
        side.compile = "bound-to-main".to_string();
        side.run = "not-applicable".to_string();
        return side;
    }

    // Missing copybooks -> the compile would fail for a reason unrelated to the source; record
    // DEPENDENCY_BLOCKED signal via the compile record (classifier promotes it).
    if !unit.missing_copybooks.is_empty() {
        side.compile = "dependency-blocked".to_string();
        side.run = "not-applicable".to_string();
        return side;
    }

    // EXEC85 is the master driver: it CALLs the whole module library. Running it standalone is not
    // the corpus's intended execution mode; the compile is still a real oracle result.
    let is_driver = unit.name == "EXEC85";

    // Build the compile command: the unit plus any SUBRTN subprograms bound to it (compiled and
    // linked together, mirroring `cobc -x main.cob sub.cob`).
    let subs: Vec<String> = sub_by_main.get(&unit.name).cloned().unwrap_or_default();
    let mut argv = vec![prefix.join("bin/cobc").to_string_lossy().into_owned()];
    let mut args = cobc_compile_args(unit, materialized_root);
    let n = args.len();
    args[n - 1] = materialized_root
        .join(&unit.adapted_path)
        .to_string_lossy()
        .into_owned();
    for s in &subs {
        args.push(materialized_root.join(s).to_string_lossy().into_owned());
    }
    argv.extend(args);

    let mut full_env = env.to_vec();
    full_env.push((
        "LD_LIBRARY_PATH".into(),
        prefix.join("lib").to_string_lossy().into_owned(),
    ));
    full_env.push((
        "COB_CONFIG_DIR".into(),
        prefix
            .join("share/gnucobol/config")
            .to_string_lossy()
            .into_owned(),
    ));

    let unit_dir = work_root.join(format!("u{}", unit.unit_index));
    let comp = run_invocation(
        &argv,
        &unit_dir,
        &full_env,
        ORACLE_COMPILE_TIMEOUT_SECS,
        &unit_dir.join("compile"),
        None,
    );
    let comp_ok = comp.exit_code == Some(0);
    side.compile_invocation = Some(comp.clone());
    side.compile = if comp_ok {
        "pass".to_string()
    } else if comp.timed_out {
        "timeout".to_string()
    } else if comp.exit_code.map(|c| c >= 128).unwrap_or(false) {
        "error".to_string()
    } else {
        "reject".to_string()
    };

    if !comp_ok || !unit.is_executable_candidate {
        side.run = "not-applicable".to_string();
        return side;
    }

    if is_driver {
        // EXEC85 needs the 459-module callable library (a different execution mode). Compile is
        // recorded; the run is marked harness-blocked so it is never counted as a pass/fail.
        side.run = "harness-blocked".to_string();
        warnings.lock().unwrap().push(
            "EXEC85: compile passed; run deferred (driver requires module library)".to_string(),
        );
        return side;
    }

    let binary = unit_dir.join("program");
    let run = oracle_run(unit, materialized_root, work_root, prefix, env, &binary);
    let run_ok = run.exit_code == Some(0);
    side.run_invocation = Some(run.clone());
    side.run = if run.timed_out {
        "timeout".to_string()
    } else if run_ok {
        "pass".to_string()
    } else {
        "fail".to_string()
    };

    // Preserve produced artifacts (report file, generated files) + hashes.
    let run_dir = unit_dir.join("run");
    let report = find_report_file(&run_dir);
    if report.exists() {
        let bytes = read_bytes(&report);
        side.report_sha256 = sha256_hex(&bytes);
        // mirror the report into the evidence dir for raw preservation
        if let Some(ev) = &run.stdout_path {
            let ev_dir = Path::new(ev).parent().unwrap_or(&run_dir);
            let _ = std::fs::create_dir_all(ev_dir);
            let _ = std::fs::copy(&report, ev_dir.join("REPORT"));
        }
    }
    // Parse the CCVS85 verdict counts from the report.
    side.verdict_counts = parse_verdict_counts(&read_bytes(&report));

    side
}

/// Parse the CCVS85 summary counts from a report file. The NIST CCVS85 modules print a
/// "TEST RESULTS" block at the end of the report whose canonical form is (verified against the
/// oracle's actual reports):
///   `001 OF 001  TESTS WERE EXECUTED SUCCESSFULLY`
///   `NO  TEST(S) FAILED` / `003 TEST(S) FAILED`
///   `NO  TEST(S) DELETED` / `002 TEST(S) DELETED`
///   `NO  TEST(S) REQUIRE INSPECTION` / `057 TEST(S) REQUIRE INSPECTION`
/// Older corpora also print `TOTAL PASSED =    12  FAILED =     0  DELETED =     0  INSPECT =     0`;
/// both forms are recognized (the extractor is deliberately conservative: it only counts lines it
/// recognizes, and records the raw lines it used so the counts are auditable).
pub fn parse_verdict_counts(report: &[u8]) -> Option<VerdictCounts> {
    let text = String::from_utf8_lossy(report);
    let mut vc = VerdictCounts::default();
    let mut found = false;
    for line in text.lines() {
        let up = line.to_ascii_uppercase();
        // canonical NIST summary line (column-insensitive)
        if up.contains("TOTAL PASSED") && (up.contains("FAILED") || up.contains("FAIL ")) {
            for (label, field) in [
                ("PASSED", &mut vc.passed),
                ("FAILED", &mut vc.failed),
                ("DELETED", &mut vc.deleted),
                ("INSPECT", &mut vc.inspect),
            ] {
                if let Some(v) = extract_count(&up, label) {
                    *field = v;
                }
            }
            vc.source_lines.push(line.trim().to_string());
            found = true;
        }
        // CCVS85's actual per-module verdict block:
        //   `<n> OF <total>  TESTS WERE EXECUTED SUCCESSFULLY`   -> passed = <n>, total = <total>
        //   `NO  TEST(S) FAILED` | `<n> TEST(S) FAILED`         -> failed
        //   `NO  TEST(S) DELETED` | `<n> TEST(S) DELETED`       -> deleted
        //   `NO  TEST(S) REQUIRE INSPECTION` | `<n> ...`        -> inspect
        if up.contains("TESTS WERE EXECUTED SUCCESSFULLY") {
            // `<n> OF <total> TESTS WERE EXECUTED SUCCESSFULLY` -> passed = <n>
            let digits: String = up
                .trim_start()
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if let Ok(n) = digits.parse::<u64>() {
                vc.passed = n;
            }
            vc.source_lines.push(line.trim().to_string());
            found = true;
        }
        if up.contains("TEST(S) FAILED") || up.contains("TESTS FAILED") {
            vc.failed = count_or_no(&up);
            vc.source_lines.push(line.trim().to_string());
            found = true;
        }
        if up.contains("TEST(S) DELETED") || up.contains("TESTS DELETED") {
            vc.deleted = count_or_no(&up);
            vc.source_lines.push(line.trim().to_string());
            found = true;
        }
        if up.contains("REQUIRE INSPECTION") || up.contains("TESTS INSPECTED") {
            vc.inspect = count_or_no(&up);
            vc.source_lines.push(line.trim().to_string());
            found = true;
        }
        // informational marker: a line of the form `... INFO ... n` is not standard; the NIST
        // suite uses `INSPECT` for informational checks; count explicitly reported info lines.
        if up.contains("INFORMATIONAL") && up.contains('=') {
            if let Some(v) = extract_count(&up, "INFORMATIONAL") {
                vc.informational = v;
                vc.source_lines.push(line.trim().to_string());
                found = true;
            }
        }
    }
    if found {
        Some(vc)
    } else {
        None
    }
}

/// `NO TEST(S) FAILED`-style zero vs `<n> TEST(S) FAILED`-style count.
fn count_or_no(up: &str) -> u64 {
    // a leading `NO ` (or `NO\t`) means zero; otherwise the first number in the line is the count.
    let after = up.trim_start();
    if after.starts_with("NO ") || after.starts_with("NO\t") {
        return 0;
    }
    let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().unwrap_or(0)
}

fn extract_count(up: &str, label: &str) -> Option<u64> {
    // matches `LABEL = <digits>` (also `LABEL=<digits>`)
    let idx = up.find(label)?;
    let rest = &up[idx + label.len()..];
    let rest = rest.trim_start();
    let rest = rest.strip_prefix('=')?.trim_start();
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

/// Serialize the oracle phase results to `oracle-results.json` (work dir).
pub fn write_oracle_results(
    path: &Path,
    units: &[MaterializedUnit],
    results: &BTreeMap<usize, OracleSide>,
) {
    let v: Vec<serde_json::Value> = units
        .iter()
        .map(|u| {
            serde_json::json!({
                "unit_index": u.unit_index,
                "kind": u.kind,
                "name": u.name,
                "source_path": u.source_path,
                "source_sha256": u.source_sha256,
                "oracle": results.get(&u.unit_index),
            })
        })
        .collect();
    let _ = std::fs::write(path, serde_json::to_string_pretty(&v).unwrap() + "\n");
}

/// The default deterministic environment for oracle execution (LC_ALL/LANG/TZ/SOURCE_DATE_EPOCH).
/// PATH is included because the runner env_clear()s the process environment and cobc's C-compile
/// stage spawns gcc (which execs `cc1` via PATH).
pub fn deterministic_env() -> Vec<(String, String)> {
    vec![
        ("LC_ALL".into(), "C.UTF-8".into()),
        ("LANG".into(), "C.UTF-8".into()),
        ("TZ".into(), "UTC0".into()),
        ("SOURCE_DATE_EPOCH".into(), "725846400".into()), // 1993-01-01T00:00:00Z
        ("COB_CURRENT_DATE".into(), "19920101000000+0000".into()),
        ("COB_SORT_MEMORY".into(), "1M".into()),
        (
            "PATH".into(),
            "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".into(),
        ),
        ("HOME".into(), "/tmp".into()),
    ]
}

/// The CCVS85 report is written to the file named by ASSIGN TO XXXXX055. When GnuCOBOL's env-var
/// file map is not honoured (a config surprise), fall back to scanning the run dir for the report.
pub fn find_report_file(run_dir: &Path) -> PathBuf {
    let canonical = run_dir.join("REPORT");
    if canonical.exists() {
        return canonical;
    }
    if let Ok(rd) = std::fs::read_dir(run_dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_file() {
                let name = p
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                if name == "XXXXX055" {
                    return p;
                }
            }
        }
    }
    canonical
}

/// Collect generated-file differences: the set of files the oracle run produced in its run dir
/// (excluding the harness's own evidence dir).
pub fn generated_files(run_dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(run_dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_file() {
                if let Some(n) = p.file_name().map(|n| n.to_string_lossy().into_owned()) {
                    if n != "REPORT" && n != "evidence" {
                        out.push(n);
                    }
                }
            }
        }
    }
    out.sort();
    out
}
