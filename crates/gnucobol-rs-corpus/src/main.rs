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
use std::process::ExitCode;

fn main() -> ExitCode {
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
            let specs_dir = cwd.join("corpus").join("specs");
            let reports = cli::cmd_check_updates(&specs_dir)?;
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
    }
}
