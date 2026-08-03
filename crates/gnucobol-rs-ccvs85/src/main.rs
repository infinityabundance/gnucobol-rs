//! `gnucobol-rs-ccvs85` — the GNURUST.CCVS85.2/.3/.4 court harness.
//!
//! Commands (each is one evidence phase; the Docker orchestrator runs them in order):
//!
//! ```text
//! materialize  --input <newcob.val.Z> --work <dir> [--root <repo>]
//! oracle-run   --work <dir> [--prefix <oracle-prefix>] [--jobs N]
//! candidate-run --work <dir> --cobrun <path> [--jobs N]
//! classify     --work <dir> [--meta <meta.json>] [--out reports/ccvs85]
//! gate check   --root <repo>
//! determinism  --work <dir> --pass-a <summaryA.json> --pass-b <summaryB.json> [--out reports/ccvs85]
//! ```
//!
//! The `classify` step also writes the three receipts and the casefile inputs; the Docker host
//! orchestrator copies the committed reports back into the repository.

use gnucobol_rs_ccvs85::{candidate, compare, corpus, gate, model, oracle, receipts};

use std::path::{Path, PathBuf};

fn arg_val(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("");
    let code = match cmd {
        "materialize" => cmd_materialize(&args),
        "oracle-run" => cmd_oracle_run(&args),
        "candidate-run" => cmd_candidate_run(&args),
        "classify" => cmd_classify(&args),
        "receipts-finalize" => cmd_receipts_finalize(&args),
        "determinism" => cmd_determinism(&args),
        "gate" => {
            let sub = args.get(2).map(String::as_str).unwrap_or("");
            match sub {
                "check" => {
                    let root = arg_val(&args, "--root").unwrap_or_else(|| ".".to_string());
                    let g = gate::gate_check(Path::new(&root), None);
                    for n in &g.notes {
                        println!("note: {n}");
                    }
                    for p in &g.problems {
                        eprintln!("GATE FAIL: {p}");
                    }
                    if g.problems.is_empty() {
                        println!("GNURUST.CCVS85 gate check: PASS (all invariants hold)");
                    } else {
                        println!("GNURUST.CCVS85 gate check: {} problem(s)", g.problems.len());
                    }
                    gate::exit_code(&g)
                }
                _ => {
                    eprintln!("gate: use `gate check --root <repo>`");
                    2
                }
            }
        }
        _ => {
            eprintln!(
                "usage: gnucobol-rs-ccvs85 <materialize|oracle-run|candidate-run|classify|determinism|gate check> ..."
            );
            2
        }
    };
    std::process::exit(code);
}

fn cmd_materialize(args: &[String]) -> i32 {
    let input =
        arg_val(args, "--input").unwrap_or_else(|| "lab/corpus/ccvs85/newcob.val.Z".to_string());
    let work = arg_val(args, "--work").unwrap_or_else(|| "work".to_string());
    let root = arg_val(args, "--root").unwrap_or_else(|| ".".to_string());
    let work = PathBuf::from(&work);
    let input = PathBuf::from(&input);
    eprintln!("materialize: start (input={input:?})");

    let (custody, units) = match corpus::derive_custody(&input) {
        Some(v) => v,
        None => {
            eprintln!("materialize: cannot decompress/read corpus spine {input:?}");
            return 2;
        }
    };
    eprintln!(
        "materialize: custody derived ({} units)",
        custody.unit_count
    );
    // verify against the committed GNURUST.CCVS85.1 receipt
    let committed = std::fs::read_to_string(
        Path::new(&root).join("reports/provenance/ccvs85-corpus-ingest-receipt.json"),
    )
    .ok()
    .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok());
    if let Some(rec) = &committed {
        let mut mismatches = Vec::new();
        if rec["compressed_sha256"].as_str() != Some(&custody.compressed_sha256) {
            mismatches.push("compressed_sha256");
        }
        if rec["decompressed_sha256"].as_str() != Some(&custody.decompressed_sha256) {
            mismatches.push("decompressed_sha256");
        }
        if rec["unit_count"].as_u64() != Some(custody.unit_count as u64) {
            mismatches.push("unit_count");
        }
        if !mismatches.is_empty() {
            eprintln!(
                "materialize: corpus identity MISMATCH vs committed receipt: {}",
                mismatches.join(", ")
            );
            eprintln!("  (GNURUST.CCVS85.1 must stay green; the corpus spine is the admitted one)");
            return 1;
        }
        println!("materialize: corpus identity verified against committed CCVS85.1 receipt");
    } else {
        eprintln!("materialize: warning — committed CCVS85.1 receipt not found; corpus identity unverified");
    }

    let decompressed = match corpus::decompress(&input) {
        Some(d) => d,
        None => {
            eprintln!("materialize: cannot decompress corpus spine {input:?}");
            return 2;
        }
    };
    let text = String::from_utf8_lossy(&decompressed);
    let lines: Vec<&str> = text.lines().collect();
    eprintln!("materialize: decompressed {} lines", lines.len());
    let materialized = corpus::materialize(&lines, &units, &work);
    eprintln!("materialize: split done");
    corpus::write_index_json(&work.join("materialized-units.json"), &materialized);
    println!(
        "materialize: {} units materialized under {work:?} ({} COBOL, {} CLBRY, {} DATA*)",
        materialized.len(),
        materialized.iter().filter(|u| u.kind == "COBOL").count(),
        materialized.iter().filter(|u| u.kind == "CLBRY").count(),
        materialized.iter().filter(|u| u.kind == "DATA*").count(),
    );
    let missing: Vec<&str> = materialized
        .iter()
        .flat_map(|u| u.missing_copybooks.iter().map(|s| s.as_str()))
        .collect();
    println!(
        "materialize: copybook refs with no CLBRY unit: {}",
        missing.len()
    );
    0
}

fn cmd_oracle_run(args: &[String]) -> i32 {
    let work = PathBuf::from(arg_val(args, "--work").unwrap_or_else(|| "work".to_string()));
    let prefix =
        PathBuf::from(arg_val(args, "--prefix").unwrap_or_else(|| "lab/oracle/prefix".to_string()));
    let jobs: usize = arg_val(args, "--jobs")
        .and_then(|j| j.parse().ok())
        .unwrap_or(4);

    if !prefix.join("bin/cobc").exists() {
        eprintln!("oracle-run: cobc not found under {prefix:?} — the oracle must be built first");
        return 2;
    }
    let units = load_materialized(&work);
    let env = oracle::deterministic_env();
    let (results, warnings) =
        oracle::run_oracle_phase(&units, &work, &work.join("oracle"), &prefix, &env, jobs);
    oracle::write_oracle_results(&work.join("oracle-results.json"), &units, &results);
    for w in &warnings {
        println!("oracle-run: {w}");
    }
    let ok = results.values().filter(|s| s.compile == "pass").count();
    let reject = results
        .values()
        .filter(|s| s.compile == "reject" || s.compile == "error")
        .count();
    println!(
        "oracle-run: {} units compiled (pass {ok}, reject/error {reject})",
        results.len()
    );
    0
}

fn cmd_candidate_run(args: &[String]) -> i32 {
    let work = PathBuf::from(arg_val(args, "--work").unwrap_or_else(|| "work".to_string()));
    let cobrun = PathBuf::from(
        arg_val(args, "--cobrun").unwrap_or_else(|| "target/release/examples/cobrun".to_string()),
    );
    let jobs: usize = arg_val(args, "--jobs")
        .and_then(|j| j.parse().ok())
        .unwrap_or(4);

    if !cobrun.exists() {
        eprintln!("candidate-run: cobrun not found at {cobrun:?} — build `cargo build --release -p gnucobol-rs --example cobrun` first");
        return 2;
    }
    let units = load_materialized(&work);
    let env = oracle::deterministic_env();
    let results =
        candidate::run_candidate_phase(&units, &work, &work.join("candidate"), &cobrun, &env, jobs);
    candidate::write_candidate_results(&work.join("candidate-results.json"), &units, &results);
    let accepted = results.values().filter(|s| s.prepare == "accepted").count();
    let rejected = results
        .values()
        .filter(|s| s.prepare.starts_with("reject"))
        .count();
    println!(
        "candidate-run: {} units (accepted {accepted}, rejected {rejected})",
        results.len()
    );
    0
}

fn cmd_classify(args: &[String]) -> i32 {
    let work = PathBuf::from(arg_val(args, "--work").unwrap_or_else(|| "work".to_string()));
    let out = PathBuf::from(arg_val(args, "--out").unwrap_or_else(|| "reports/ccvs85".to_string()));
    let meta_path = arg_val(args, "--meta").map(PathBuf::from);

    let units = load_materialized(&work);
    let oracle_results = load_oracle(&work);
    let candidate_results = load_candidate(&work);
    let work_root = work.join("oracle");
    let (results, summary) =
        compare::classify_all(&units, &oracle_results, &candidate_results, &work_root);

    let meta = meta_path
        .as_deref()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .unwrap_or_else(|| serde_json::json!({"generated_at": "unstamped", "git_commit": "unstamped", "crate_version": "0.1.0"}));

    std::fs::create_dir_all(&out).ok();
    compare::write_comparison_results(&out.join("comparison-results.json"), &results);
    compare::write_summary_json(&out.join("summary.json"), &summary, &meta);
    std::fs::write(
        out.join("summary.md"),
        compare::render_summary_md(&summary, &meta),
    )
    .ok();
    compare::write_csv(&out.join("results.csv"), &results);
    std::fs::write(
        out.join("failure-buckets.md"),
        compare::render_failure_buckets(&results, &summary),
    )
    .ok();

    // also copy the manifests/ledgers to the repo out dir
    for f in [
        "materialized-units.json",
        "oracle-results.json",
        "candidate-results.json",
    ] {
        let src = work.join(f);
        if src.exists() {
            let _ = std::fs::copy(&src, out.join(f));
        }
    }

    // receipts
    let receipts_dir = Path::new(&out).parent().unwrap().join("receipts");
    let written = receipts::write_receipts(&receipts_dir, &meta, &summary);
    for (gate, sha) in &written {
        println!("classify: receipt {gate} written (sha256 {sha})");
    }

    println!(
        "classify: {} units classified — oracle compile pass {}, reject {}, run pass {}, candidate accepted {}, unsupported {}, timeouts {}",
        summary.units_total,
        summary.oracle_compile_pass,
        summary.oracle_compile_reject,
        summary.oracle_run_pass,
        summary.candidate_accepted,
        summary.candidate_unsupported,
        summary.oracle_timeout + summary.candidate_timeout,
    );
    0
}

fn cmd_receipts_finalize(args: &[String]) -> i32 {
    let root = PathBuf::from(arg_val(args, "--root").unwrap_or_else(|| ".".to_string()));
    let meta_path = arg_val(args, "--meta").map(PathBuf::from);
    let meta = meta_path
        .as_deref()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok());
    let Some(meta) = meta else {
        eprintln!("receipts-finalize: cannot read meta JSON");
        return 2;
    };

    // recompute the summary from the committed comparison-results.json (the binding ledger)
    let ccvs85 = root.join("reports/ccvs85");
    let comp: serde_json::Value = std::fs::read_to_string(ccvs85.join("comparison-results.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(serde_json::Value::Null);
    let Some(units) = comp["units"].as_array() else {
        eprintln!("receipts-finalize: comparison-results.json missing/malformed");
        return 2;
    };
    let unit_results: Vec<model::UnitResult> = units
        .iter()
        .filter_map(|u| serde_json::from_value(u.clone()).ok())
        .collect();
    let unit_meta: Vec<model::MaterializedUnit> =
        std::fs::read_to_string(ccvs85.join("materialized-units.json"))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
    let summary = compare::summarize(&unit_results, &unit_meta);
    let receipts_dir = root.join("reports/receipts");
    let written = receipts::write_receipts(&receipts_dir, &meta, &summary);
    for (gate, sha) in &written {
        println!("receipts-finalize: {gate} written (sha256 {sha})");
    }
    println!(
        "receipts-finalize: summaries reconcile — {} units",
        summary.units_total
    );
    0
}

fn cmd_determinism(args: &[String]) -> i32 {
    let out = PathBuf::from(arg_val(args, "--out").unwrap_or_else(|| "reports/ccvs85".to_string()));
    let pass_a = PathBuf::from(
        arg_val(args, "--pass-a").unwrap_or_else(|| "pass-a/summary.json".to_string()),
    );
    let pass_b = PathBuf::from(
        arg_val(args, "--pass-b").unwrap_or_else(|| "pass-b/summary.json".to_string()),
    );
    let _work = PathBuf::from(arg_val(args, "--work").unwrap_or_else(|| "work".to_string()));

    let a = std::fs::read_to_string(&pass_a)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok());
    let b = std::fs::read_to_string(&pass_b)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok());
    let (Some(a), Some(b)) = (a, b) else {
        eprintln!("determinism: cannot read both pass summaries");
        return 2;
    };
    let sa = &a["summary"];
    let sb = &b["summary"];

    // compare the stable summary surface (counts + classifications; timestamps excluded)
    let stable_a = serde_json::json!({
        "units_total": sa["units_total"], "by_final_classification": sa["by_final_classification"],
        "by_reason_code": sa["by_reason_code"], "by_section": sa["by_section"],
        "oracle_candidate_pair": sa["oracle_candidate_pair"],
    });
    let stable_b = serde_json::json!({
        "units_total": sb["units_total"], "by_final_classification": sb["by_final_classification"],
        "by_reason_code": sb["by_reason_code"], "by_section": sb["by_section"],
        "oracle_candidate_pair": sb["oracle_candidate_pair"],
    });
    let identical = stable_a == stable_b;

    // ALSO compare the per-unit oracle REPORT bytes between the two fresh runs: the stable-summary
    // gate tolerates timestamps, but a unit whose REPORT bytes differ run-to-run is a genuine
    // nondeterminism that must be recorded and explicitly classified (never concealed by retries).
    // GnuCOBOL's `COB_CURRENT_DATE` pins the date+time but the fractional-second field of ACCEPT
    // FROM TIME still comes from the real clock; a CCVS85 TIME test prints it into its report.
    let mut report_drift: Vec<serde_json::Value> = Vec::new();
    if let (Some(oa), Some(ob)) = (
        read_json_sibling(&pass_a, "oracle-results.json"),
        read_json_sibling(&pass_b, "oracle-results.json"),
    ) {
        let oa_units = oa.as_array().cloned().unwrap_or_default();
        let ob_units = ob.as_array().cloned().unwrap_or_default();
        for ua in &oa_units {
            let idx = ua["unit_index"].as_u64().unwrap_or(u64::MAX);
            let name = ua["name"].as_str().unwrap_or("?");
            let ha = ua["oracle"]["report_sha256"].as_str().unwrap_or("");
            let Some(ub) = ob_units
                .iter()
                .find(|x| x["unit_index"].as_u64() == Some(idx))
            else {
                continue;
            };
            let hb = ub["oracle"]["report_sha256"].as_str().unwrap_or("");
            if !ha.is_empty() && ha != hb {
                report_drift.push(serde_json::json!({
                    "unit_index": idx,
                    "name": name,
                    "pass_a_report_sha256": ha,
                    "pass_b_report_sha256": hb,
                    "note": "oracle REPORT bytes differ between two fresh runs (e.g. a TIME test printing real fractional seconds); the unit is explicitly classified nondeterministic"
                }));
            }
        }
    }
    let doc = serde_json::json!({
        "schema": "gnurust-ccvs85-determinism-v1",
        "pass_a": {
            "summary_sha256": corpus::sha256_hex(&std::fs::read(&pass_a).unwrap_or_default()),
            "path": pass_a,
        },
        "pass_b": {
            "summary_sha256": corpus::sha256_hex(&std::fs::read(&pass_b).unwrap_or_default()),
            "path": pass_b,
        },
        "stable_summary_identical": identical,
        "report_byte_nondeterminism": report_drift,
        "note": "summary counts + classifications + reason buckets must be identical across two fresh full runs (timestamps deliberately excluded); per-unit oracle REPORT hashes are compared separately and any drift is recorded + explicitly classified",
    });
    std::fs::write(
        out.join("determinism.json"),
        serde_json::to_string_pretty(&doc).unwrap() + "\n",
    )
    .ok();

    // Mark every drifted unit nondeterministic in the committed comparison ledger and recompute the
    // summary so `nondeterministic` reconciles and the unit is visible in every report.
    if !report_drift.is_empty() {
        let ccvs85 = out.clone();
        let comp_path = ccvs85.join("comparison-results.json");
        if let Ok(text) = std::fs::read_to_string(&comp_path) {
            if let Ok(mut comp) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(units) = comp["units"].as_array_mut() {
                    for d in &report_drift {
                        let idx = d["unit_index"].as_u64().unwrap_or(u64::MAX);
                        if let Some(u) = units
                            .iter_mut()
                            .find(|x| x["unit_index"].as_u64() == Some(idx))
                        {
                            u["nondeterministic"] = serde_json::json!(true);
                            u["determinism"] = serde_json::json!({
                                "pass_a": d["pass_a_report_sha256"],
                                "pass_b": d["pass_b_report_sha256"],
                            });
                        }
                    }
                    let _ = std::fs::write(
                        &comp_path,
                        serde_json::to_string_pretty(&comp).unwrap() + "\n",
                    );
                    // recompute the summary from the (now annotated) ledger
                    let munits: Vec<model::MaterializedUnit> =
                        std::fs::read_to_string(ccvs85.join("materialized-units.json"))
                            .ok()
                            .and_then(|s| serde_json::from_str(&s).ok())
                            .unwrap_or_default();
                    if !munits.is_empty() {
                        let unit_results: Vec<model::UnitResult> = comp["units"]
                            .as_array()
                            .cloned()
                            .unwrap_or_default()
                            .iter()
                            .filter_map(|u| serde_json::from_value(u.clone()).ok())
                            .collect();
                        let summary = compare::summarize(&unit_results, &munits);
                        let meta = std::fs::read_to_string(ccvs85.join("summary.json"))
                            .ok()
                            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                            .unwrap_or_else(|| serde_json::json!({"generated_at": "unstamped"}));
                        compare::write_summary_json(&ccvs85.join("summary.json"), &summary, &meta);
                        let _ = std::fs::write(
                            ccvs85.join("summary.md"),
                            compare::render_summary_md(&summary, &meta),
                        );
                        let _ = std::fs::write(
                            ccvs85.join("failure-buckets.md"),
                            compare::render_failure_buckets(&unit_results, &summary),
                        );
                    }
                }
            }
        }
    }

    if identical {
        if report_drift.is_empty() {
            println!("determinism: two fresh full runs produce identical stable summaries — PASS");
        } else {
            println!(
                "determinism: stable summaries identical — PASS, with {} explicitly-classified report-byte nondeterminism(s):",
                report_drift.len()
            );
            for d in &report_drift {
                println!(
                    "  u{} {}: report sha pass_a {} vs pass_b {}",
                    d["unit_index"],
                    d["name"],
                    d["pass_a_report_sha256"],
                    d["pass_b_report_sha256"]
                );
            }
        }
        0
    } else {
        // Find which unit classifications differ between the two passes.
        let units_a = a["units"].as_array();
        let units_b = b["units"].as_array();
        let mut diffs = Vec::new();
        if let (Some(ua), Some(ub)) = (units_a, units_b) {
            for u in ua {
                let idx = u["unit_index"].as_u64().unwrap_or(0);
                if let Some(ub_entry) = ub.iter().find(|x| x["unit_index"].as_u64() == Some(idx)) {
                    let fa = u["final_classification"].as_str().unwrap_or("");
                    let fb = ub_entry["final_classification"].as_str().unwrap_or("");
                    if fa != fb {
                        diffs.push(format!("u{idx}: {fa} vs {fb}"));
                    }
                }
            }
        }
        eprintln!("determinism: FAIL — stable summaries differ between two fresh runs");
        for d in diffs.iter().take(20) {
            eprintln!("  diff: {d}");
        }
        eprintln!(
            "  ({} divergent unit(s); the affected units must be classified as nondeterministic)",
            diffs.len()
        );
        1
    }
}

fn load_materialized(work: &Path) -> Vec<model::MaterializedUnit> {
    let p = work.join("materialized-units.json");
    let txt = std::fs::read_to_string(&p).unwrap_or_else(|_| {
        eprintln!("missing {p:?} — run `materialize` first");
        std::process::exit(2);
    });
    serde_json::from_str(&txt).unwrap_or_else(|e| {
        eprintln!("malformed {p:?}: {e}");
        std::process::exit(2);
    })
}

fn load_oracle(work: &Path) -> std::collections::BTreeMap<usize, model::OracleSide> {
    let p = work.join("oracle-results.json");
    let txt = std::fs::read_to_string(&p).unwrap_or_else(|_| {
        eprintln!("missing {p:?} — run `oracle-run` first");
        std::process::exit(2);
    });
    let v: Vec<serde_json::Value> = serde_json::from_str(&txt).unwrap_or_default();
    v.into_iter()
        .filter_map(|e| {
            let idx = e["unit_index"].as_u64()? as usize;
            let side = serde_json::from_value(e["oracle"].clone()).ok()?;
            Some((idx, side))
        })
        .collect()
}

fn load_candidate(work: &Path) -> std::collections::BTreeMap<usize, model::CandidateSide> {
    let p = work.join("candidate-results.json");
    let txt = std::fs::read_to_string(&p).unwrap_or_else(|_| {
        eprintln!("missing {p:?} — run `candidate-run` first");
        std::process::exit(2);
    });
    let v: Vec<serde_json::Value> = serde_json::from_str(&txt).unwrap_or_default();
    v.into_iter()
        .filter_map(|e| {
            let idx = e["unit_index"].as_u64()? as usize;
            let side = serde_json::from_value(e["candidate"].clone()).ok()?;
            Some((idx, side))
        })
        .collect()
}

/// Read a JSON file that sits next to `path` (e.g. `oracle-results.json` beside `summary.json`).
fn read_json_sibling(path: &Path, name: &str) -> Option<serde_json::Value> {
    let sibling = path.parent().unwrap_or(Path::new(".")).join(name);
    std::fs::read_to_string(&sibling)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}
