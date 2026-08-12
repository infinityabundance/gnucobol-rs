//! `gnucobol-rs-corpus` — the valid-COBOL corpus CLI.
//!
//! Admission is state-driven and every command supports structured JSON output (`--json`).
//! No source can be admitted without oracle validation: the `admit` engine walks the strictly
//! ordered chain DISCOVERED -> CUSTODY_VERIFIED -> LICENCE_VERIFIED -> DEPENDENCIES_RESOLVED ->
//! ORACLE_COMPILE_VERIFIED -> ORACLE_RUN_VERIFIED -> DETERMINISM_VERIFIED -> ADMITTED and
//! `--finalize` refuses to mark a record ADMITTED unless the chain was walked and the oracle
//! contract (plus reviewed licence) is present.

use gnucobol_rs_corpus::cli::{self, Command};
use gnucobol_rs_corpus::dedup::DedupIndex;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    // Deterministic probe environment: the candidate's WHEN-COMPILED / MODULE-DATE / MODULE-TIME
    // support requires a pinned SOURCE_DATE_EPOCH (the reproducible-builds standard) and must
    // never depend on the ambient shell (the lane containers pin it; a bare host shell does
    // not). Pin the canonical corpus epoch when the caller did not choose one -- the probe
    // subprocesses inherit it, so probe results are environment-independent.
    if std::env::var_os("SOURCE_DATE_EPOCH").is_none() {
        std::env::set_var("SOURCE_DATE_EPOCH", "725846400");
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = match cli::parse(&args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    match run(cmd) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Emit a value as JSON when `json` is set (all commands support structured output).
fn emit<T: serde::Serialize>(v: &T, json: bool) -> Result<(), String> {
    if json {
        let s = serde_json::to_string_pretty(v).map_err(|e| e.to_string())?;
        println!("{s}");
    }
    Ok(())
}

/// Build the dedup index from every record's stored source bytes (repository-level grouping).
fn dedup_index(store: &gnucobol_rs_corpus::CorpusStore, ms: &cli::ManifestStore) -> DedupIndex {
    let mut idx = DedupIndex::new();
    if let Ok(recs) = ms.list() {
        for r in &recs {
            if !r.source.content_sha256.is_empty() {
                if let Some(bytes) = store.get_bytes(&r.source.content_sha256) {
                    idx.register(&r.program_id, &bytes);
                    idx.note_normalized(
                        &gnucobol_rs_corpus::dedup::normalized_hash(&bytes),
                        &r.program_id,
                    );
                }
            }
        }
    }
    idx
}

fn run(cmd: Command) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    match cmd {
        Command::Discover { dir, json } => {
            let (store, ms) = cli::stores()?;
            let n = cli::cmd_discover(&store, &ms, &dir)?;
            if !json {
                println!("discovered {n} candidate unit(s) under {}", dir.display());
            }
            emit(&serde_json::json!({ "discovered": n }), json)
        }
        Command::Fetch { spec, json } => {
            let (store, _ms) = cli::stores()?;
            let r = cli::cmd_fetch(&store, &spec)?;
            if !json {
                println!(
                    "fetched {} @ {} (sha {}): {}",
                    r.family, r.revision, r.archive_sha256, r.source
                );
            }
            emit(&r, json)
        }
        Command::Admit(args) => {
            let (store, ms) = cli::stores()?;
            let rec = cli::cmd_admit(&store, &ms, &args)?;
            if !args.json {
                println!(
                    "{} -> {} ({})",
                    rec.program_id,
                    rec.admission_state,
                    rec.classification.as_str()
                );
            }
            emit(&rec, args.json)
        }
        Command::Verify { id, json } => {
            let (store, ms) = cli::stores()?;
            let issues = cli::cmd_verify(&store, &ms, &id)?;
            if !json {
                if issues.is_empty() {
                    println!("{id}: custody OK");
                } else {
                    for i in &issues {
                        println!("{id}: {i}");
                    }
                }
            }
            emit(
                &serde_json::json!({ "program_id": id, "issues": issues }),
                json,
            )
        }
        Command::List { json } => {
            let (_store, ms) = cli::stores()?;
            let recs = ms.list()?;
            let view: Vec<serde_json::Value> = recs
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "program_id": r.program_id,
                        "corpus_class": format!("{:?}", r.corpus_class),
                        "classification": r.classification.as_str(),
                        "admission_state": r.admission_state,
                        "first_failure": r.candidate.first_failure,
                    })
                })
                .collect();
            if json {
                emit(&view, true)
            } else {
                for r in &recs {
                    println!(
                        "{}\t{}\t{}\t{}",
                        r.program_id,
                        r.classification.as_str(),
                        r.admission_state,
                        r.candidate
                            .first_failure
                            .as_ref()
                            .map(|(p, _)| p.as_str())
                            .unwrap_or("")
                    );
                }
                Ok(())
            }
        }
        Command::Classify { id, class, json } => {
            let (_store, ms) = cli::stores()?;
            let rec = cli::cmd_classify(&ms, &id, class)?;
            if !json {
                println!("{} classified as {}", rec.program_id, class.as_str());
            }
            emit(&rec, json)
        }
        Command::RunOracle(args) => {
            // run-oracle records the oracle contract; the actual replay happens in the family
            // extractors (testsuite/ccvs85/manual), which feed these steps. This entry point
            // records the contract through the same state machine as `admit`.
            let (store, ms) = cli::stores()?;
            let rec = cli::cmd_admit(&store, &ms, &args)?;
            if !args.json {
                println!(
                    "{} oracle contract -> {}",
                    rec.program_id, rec.admission_state
                );
            }
            emit(&rec, args.json)
        }
        Command::RunCandidate(args) => {
            let (store, ms) = cli::stores()?;
            let rec = cli::cmd_admit(&store, &ms, &args)?;
            if !args.json {
                println!(
                    "{} candidate outcomes -> {} (first failure: {:?})",
                    rec.program_id, rec.admission_state, rec.candidate.first_failure
                );
            }
            emit(&rec, args.json)
        }
        Command::Compare { id, json } => {
            let (_store, ms) = cli::stores()?;
            let v = cli::cmd_compare(&ms, &id)?;
            if !json {
                println!("{}\t{}", v.program_id, v.verdict);
            }
            emit(&v, json)
        }
        Command::Report { json } => {
            let (store, ms) = cli::stores()?;
            let dedup = dedup_index(&store, &ms);
            let rep = cli::cmd_report(&ms, &cwd, &dedup)?;
            if !json {
                println!(
                    "reports/valid-corpus: {} units, {} admitted, {} unknown classifications",
                    rep.total, rep.admitted, rep.unknown_classifications
                );
            }
            emit(&rep, json)
        }
        Command::Gate { json } => {
            let (_store, ms) = cli::stores()?;
            let fails = cli::cmd_gate(&ms)?;
            if !json {
                if fails.is_empty() {
                    println!("gate: GREEN");
                } else {
                    for f in &fails {
                        println!("gate FAIL: {f}");
                    }
                }
            }
            emit(&serde_json::json!({ "failures": fails }), json)?;
            if fails.is_empty() {
                Ok(())
            } else {
                Err(format!("{} gate failure(s)", fails.len()))
            }
        }
        Command::DiagUnblocked { sub, args, json } => {
            match sub.as_str() {
                "transform" => {
                    // diag-unblocked transform <suite-src-root> <report-root> [--revision REV]
                    let src = PathBuf::from(args.first().ok_or_else(|| {
                        "diag-unblocked transform: missing suite src root".to_string()
                    })?);
                    let out = PathBuf::from(args.get(1).ok_or_else(|| {
                        "diag-unblocked transform: missing report root".to_string()
                    })?);
                    let revision = args
                        .iter()
                        .find_map(|a| a.strip_prefix("--revision=").map(|v| v.to_string()))
                        .unwrap_or_else(|| "unknown".to_string());
                    let rep =
                        gnucobol_rs_corpus::diag_unblocked::cmd_transform(&src, &out, &revision)?;
                    if !json {
                        println!(
                            "diag-unblocked transform: {} files, {} transformations, {} ignored, {} errors",
                            rep.files_scanned,
                            rep.transformations.len(),
                            rep.transformations
                                .iter()
                                .filter(|t| t.disposition.ignores_stdout() || t.disposition.ignores_stderr())
                                .count(),
                            rep.errors.len()
                        );
                    }
                    emit(&rep, json)
                }
                "gate" => {
                    // diag-unblocked gate <patch> <pristine-root> <transformed-root> <transformations.json>
                    let patch =
                        PathBuf::from(args.first().ok_or_else(|| {
                            "diag-unblocked gate: missing patch path".to_string()
                        })?);
                    let pristine =
                        PathBuf::from(args.get(1).ok_or_else(|| {
                            "diag-unblocked gate: missing pristine root".to_string()
                        })?);
                    let transformed = PathBuf::from(args.get(2).ok_or_else(|| {
                        "diag-unblocked gate: missing transformed root".to_string()
                    })?);
                    let manifest = PathBuf::from(args.get(3).ok_or_else(|| {
                        "diag-unblocked gate: missing transformations.json".to_string()
                    })?);
                    let verdict = gnucobol_rs_corpus::diag_unblocked::cmd_gate(
                        &patch,
                        &pristine,
                        &transformed,
                        &manifest,
                    )?;
                    if !json {
                        if verdict.failures.is_empty() {
                            println!("diag-unblocked gate: GREEN");
                        } else {
                            for f in &verdict.failures {
                                println!("diag-unblocked gate FAIL: {f}");
                            }
                        }
                    }
                    emit(&verdict, json)?;
                    if verdict.failures.is_empty() {
                        Ok(())
                    } else {
                        Err(format!(
                            "{} diag-unblocked gate failure(s)",
                            verdict.failures.len()
                        ))
                    }
                }
                "reachability" => {
                    // diag-unblocked reachability [--pristine-log P] [--pristine-dir P]
                    //   [--unblocked-log P] [--unblocked-dir P] [--pristine-oracle-log P]
                    //   [--unblocked-oracle-log P] [--transformations P] [--out P]
                    let flag = |name: &str, dflt: &str| {
                        args.iter()
                            .find_map(|a| a.strip_prefix(name).map(|v| v.trim_start_matches('=')))
                            .map(PathBuf::from)
                            .unwrap_or_else(|| PathBuf::from(dflt))
                    };
                    let report = gnucobol_rs_corpus::diag_reach::cmd_reachability(
                        &flag("--pristine-log", "reports/gnucobol-testsuite/raw/candidate/testsuite.log"),
                        &flag("--pristine-dir", "reports/gnucobol-testsuite/raw/candidate/testsuite.dir"),
                        &flag("--unblocked-log", "reports/gnucobol-testsuite/diagnostic-unblocked/raw/candidate/testsuite.log"),
                        &flag("--unblocked-dir", "reports/gnucobol-testsuite/diagnostic-unblocked/raw/candidate/testsuite.dir"),
                        &flag("--pristine-oracle-log", "reports/gnucobol-testsuite/raw/baseline/testsuite.log"),
                        &flag("--unblocked-oracle-log", "reports/gnucobol-testsuite/diagnostic-unblocked/raw/oracle/testsuite.log"),
                        &flag("--transformations", "reports/gnucobol-testsuite/diagnostic-unblocked/transformations.json"),
                        &flag("--out", "reports/gnucobol-testsuite/diagnostic-unblocked"),
                    )?;
                    if !json {
                        let t = &report.totals;
                        println!(
                            "diag-unblocked reachability: {} ignored, {} groups affected, {} progressed, {} newly-reached checks",
                            t.get("diagnostic_expectations_ignored").and_then(|v| v.as_u64()).unwrap_or(0),
                            t.get("groups_affected").and_then(|v| v.as_u64()).unwrap_or(0),
                            t.get("groups_progressed_further").and_then(|v| v.as_u64()).unwrap_or(0),
                            t.get("newly_reached_checks").and_then(|v| v.as_u64()).unwrap_or(0),
                        );
                    }
                    emit(&report, json)
                }
                "reconcile" => {
                    // diag-unblocked reconcile [--pristine-src DIR] [--transformations P]
                    //   [--patch P] [--pristine-log P] [--unblocked-log P] [--pristine-dir P]
                    //   [--unblocked-dir P] [--pristine-oracle-log P] [--unblocked-oracle-log P]
                    //   [--out P]
                    let flag = |name: &str, dflt: &str| {
                        args.iter()
                            .find_map(|a| a.strip_prefix(name).map(|v| v.trim_start_matches('=')))
                            .map(PathBuf::from)
                            .unwrap_or_else(|| PathBuf::from(dflt))
                    };
                    let report = gnucobol_rs_corpus::diag_reach::cmd_reconcile(
                        &flag("--pristine-src", "lab/admit/gnucobol-3.2/tests/testsuite.src"),
                        &flag("--transformations", "reports/gnucobol-testsuite/diagnostic-unblocked/transformations.json"),
                        &flag("--patch", "reports/gnucobol-testsuite/diagnostic-unblocked/diagnostic-ignore.patch"),
                        &flag("--pristine-log", "reports/gnucobol-testsuite/raw/candidate/testsuite.log"),
                        &flag("--unblocked-log", "reports/gnucobol-testsuite/diagnostic-unblocked/raw/candidate/testsuite.log"),
                        &flag("--pristine-dir", "reports/gnucobol-testsuite/raw/candidate/testsuite.dir"),
                        &flag("--unblocked-dir", "reports/gnucobol-testsuite/diagnostic-unblocked/raw/candidate/testsuite.dir"),
                        &flag("--pristine-oracle-log", "reports/gnucobol-testsuite/raw/baseline/testsuite.log"),
                        &flag("--unblocked-oracle-log", "reports/gnucobol-testsuite/diagnostic-unblocked/raw/oracle/testsuite.log"),
                        &flag("--out", "reports/gnucobol-testsuite/diagnostic-unblocked"),
                    )?;
                    if !json {
                        println!(
                            "diag-unblocked reconcile: AT_SETUP {}=={}, AT_CHECK {}=={}, patch reproducible: {}",
                            report.at_setup_pristine,
                            report.at_setup_transformed,
                            report.at_check_pristine,
                            report.at_check_transformed,
                            report.patch_reproducible
                        );
                    }
                    emit(&report, json)
                }
                "cross-check" => {
                    // diag-unblocked cross-check [--unblocked-log P] [--unblocked-dir P]
                    //   [--transformations P] [--valid-programs P] [--out P]
                    let flag = |name: &str, dflt: &str| {
                        args.iter()
                            .find_map(|a| a.strip_prefix(name).map(|v| v.trim_start_matches('=')))
                            .map(PathBuf::from)
                            .unwrap_or_else(|| PathBuf::from(dflt))
                    };
                    let report = gnucobol_rs_corpus::diag_reach::cmd_cross_check(
                        &flag("--unblocked-log", "reports/gnucobol-testsuite/diagnostic-unblocked/raw/candidate/testsuite.log"),
                        &flag("--unblocked-dir", "reports/gnucobol-testsuite/diagnostic-unblocked/raw/candidate/testsuite.dir"),
                        &flag("--transformations", "reports/gnucobol-testsuite/diagnostic-unblocked/transformations.json"),
                        &flag("--valid-programs", "reports/valid-corpus/gnucobol-testsuite/valid-programs.json"),
                        &flag("--out", "reports/gnucobol-testsuite/diagnostic-unblocked"),
                    )?;
                    if !json {
                        println!(
                            "diag-unblocked cross-check: {} steps in logs, {} matched in corpus, {} agreed, {} disagreements",
                            report.totals["steps_in_unblocked_logs"].as_u64().unwrap_or(0),
                            report.totals["matched_in_corpus"].as_u64().unwrap_or(0),
                            report.totals["agreed"].as_u64().unwrap_or(0),
                            report.totals["disagreed"].as_u64().unwrap_or(0),
                        );
                    }
                    emit(&report, json)
                }
                other => Err(format!(
                    "diag-unblocked: unknown subcommand {other:?} (transform|gate|reachability|reconcile|cross-check)"
                )),
            }
        }
        Command::ExtractXcobol {
            candidate,
            oracle,
            json,
        } => {
            let counts = cli::cmd_extract_xcobol(candidate, oracle)?;
            if !json {
                println!(
                    "extract-xcobol: {} repos, {} files",
                    counts.get("total_repos").copied().unwrap_or(0),
                    counts.get("total_files").copied().unwrap_or(0)
                );
                for (k, v) in &counts {
                    if k != "total_repos" && k != "total_files" {
                        println!("  {k}: {v}");
                    }
                }
            }
            emit(&counts, json)
        }
        Command::ExtractOmp { candidate, json } => {
            let counts = cli::cmd_extract_omp(candidate)?;
            if !json {
                println!(
                    "extract-omp: {} programs",
                    counts.get("total_programs").copied().unwrap_or(0)
                );
                for (k, v) in &counts {
                    if k != "total_programs" && !k.starts_with("inventory:") {
                        println!("  {k}: {v}");
                    }
                }
            }
            emit(&counts, json)
        }
        Command::ExtractExtras { candidate, json } => {
            let counts = cli::cmd_extract_extras(candidate)?;
            if !json {
                println!(
                    "extract-extras: {} programs classified",
                    counts.get("total").copied().unwrap_or(0)
                );
                for (k, v) in &counts {
                    if k != "total" {
                        println!("  {k}: {v}");
                    }
                }
            }
            emit(&counts, json)
        }
        Command::ExtractManual {
            lane,
            candidate,
            json,
        } => {
            let counts = cli::cmd_extract_manual(&lane, candidate)?;
            if !json {
                let total: usize = counts.values().sum();
                println!("extract-manual: {total} classified units (lanes {lane})");
                for (k, v) in &counts {
                    println!("  {k}: {v}");
                }
            }
            emit(&counts, json)
        }
        Command::ExtractCcvs85 { json } => {
            let counts = cli::cmd_extract_ccvs85()?;
            if !json {
                println!(
                    "extract-ccvs85: {} units classified",
                    counts.get("total").copied().unwrap_or(0)
                );
                for (k, v) in &counts {
                    if k != "total" {
                        println!("  {k}: {v}");
                    }
                }
            }
            emit(&counts, json)
        }
        Command::ProbeFile {
            dir,
            file,
            run,
            out,
            json,
        } => {
            let probes = cli::cmd_probe_file(&dir, &file, run, &out)?;
            if !json {
                let first = probes.iter().find(|p| !p.ok);
                match first {
                    Some(p) => println!(
                        "probe-file {}: first failure at {} ({})",
                        file, p.phase, p.diagnostic
                    ),
                    None => println!("probe-file {}: all phases ok", file),
                }
            }
            emit(&probes, json)
        }
        Command::ProbeStep {
            manifest,
            out,
            json,
        } => {
            let probes = cli::cmd_probe_step(&manifest, &out)?;
            if !json {
                let first = probes.iter().find(|p| !p.ok);
                match first {
                    Some(p) => println!(
                        "probe-step {}: first failure at {} ({})",
                        manifest.display(),
                        p.phase,
                        p.diagnostic
                    ),
                    None => println!("probe-step {}: all phases ok", manifest.display()),
                }
            }
            emit(&probes, json)
        }
        Command::CheckUpdates { json } => {
            // fetch specs ship with the crate (crates/gnucobol-rs-corpus/specs); fall back to
            // a repo-root `corpus/specs` when the crate dir is unavailable (embedded installs).
            let crate_specs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("specs");
            let specs_dir = if crate_specs.is_dir() {
                crate_specs
            } else {
                cwd.join("corpus").join("specs")
            };
            let reports = cli::cmd_check_updates(&specs_dir)?;
            // persist the upstream-freshness + corpus-drift report (spec 11.3) alongside the
            // other valid-corpus reports; never mutates the admitted corpus.
            cli::write_upstream_drift(&reports)?;
            if !json {
                if reports.is_empty() {
                    println!("no fetch specs under {}", specs_dir.display());
                }
                for r in &reports {
                    println!(
                        "{}: pinned {} ({})",
                        r.family,
                        r.pinned_revision,
                        if r.has_newer {
                            "NEWER AVAILABLE"
                        } else {
                            "current"
                        }
                    );
                }
            }
            emit(&reports, json)
        }
        Command::ExtractTestsuite {
            lane,
            replay,
            candidate,
            json,
        } => {
            let sum = cli::cmd_extract_testsuite(&lane, replay, candidate)?;
            if !json {
                println!(
                    "extract-testsuite: {} steps, {} valid, {} invalid, {} drift, {} skipped (lanes {})",
                    sum.discovered_steps,
                    sum.valid_programs,
                    sum.invalid_programs,
                    sum.oracle_contract_drift,
                    sum.skipped_under_profile,
                    sum.lanes.join(",")
                );
                for r in &sum.reports {
                    println!("wrote {r}");
                }
            }
            emit(&sum, json)
        }
        Command::HeldOut { json } => {
            let rep = cli::cmd_held_out()?;
            if !json {
                println!(
                    "held-out: {} files, parse ok {}, check ok {}, run ok {} (first-failure by \
                     phase: {:?}; all probes bounded at {}s)",
                    rep.totals.files,
                    rep.totals.parse_ok,
                    rep.totals.check_ok,
                    rep.totals.run_ok,
                    rep.first_failure_by_phase,
                    rep.timeout_seconds
                );
                println!("wrote reports/valid-corpus/held-out-results.json");
            }
            emit(&rep, json)
        }
        Command::Mutation { json } => {
            let rep = cli::cmd_mutation()?;
            if !json {
                println!(
                    "mutation: {} bases, {} variants ({} equivalent, {} divergent, {} skipped; \
                     {}s bound per run)",
                    rep.summary.total_bases,
                    rep.summary.total_variants,
                    rep.summary.equivalent,
                    rep.summary.divergent,
                    rep.summary.skipped,
                    rep.timeout_seconds
                );
                println!("wrote reports/valid-corpus/mutation-results.json");
            }
            emit(&rep, json)
        }
        Command::Overfit { json } => {
            let rep = cli::cmd_overfit()?;
            if !json {
                for c in &rep.checks {
                    println!("overfit {:>28}: {}", c.name, c.result);
                }
                println!("overfit gate: {}", if rep.gate { "PASS" } else { "FAIL" });
                println!("wrote reports/valid-corpus/overfitting.json");
            }
            emit(&rep, json)
        }
        Command::Generalize { json } => {
            let rep = cli::cmd_generalize()?;
            if !json {
                println!(
                    "generalization: dev {}/{} accepted ({:.3}), val {}/{} accepted ({:.3}), \
                     held-out {} files, overfit gate {}",
                    rep.development.candidate_accepted,
                    rep.development.files,
                    rep.development.accept_rate,
                    rep.validation.candidate_accepted,
                    rep.validation.files,
                    rep.validation.accept_rate,
                    rep.held_out.totals.files,
                    if rep.overfitting.gate { "PASS" } else { "FAIL" }
                );
                println!("wrote reports/valid-corpus/generalization.json");
            }
            emit(&rep, json)
        }
        Command::Unify { json } => {
            let root = gnucobol_rs_corpus::extract::workspace_root()?;
            let rep = gnucobol_rs_corpus::unify::unify(&root)?;
            if !json {
                println!(
                    "unify: {} total units aggregated from {} families; wrote summary.json, \
                     programs.csv, licences.json, dependencies.json, deduplication.json, \
                     dialect-matrix.json, first-failure-buckets.json, accuracy.json, \
                     performance.json, determinism.json, no-delegation.json",
                    rep.summary
                        .get("total_units")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                    rep.summary
                        .get("families_aggregated")
                        .and_then(|v| v.as_array())
                        .map(|a| a.len())
                        .unwrap_or(0)
                );
            }
            emit(&rep, json)
        }
    }
}
