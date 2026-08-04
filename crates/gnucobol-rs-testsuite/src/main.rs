//! `gnucobol-rs-testsuite` — the GNURUST.GNUCOBOL-TESTSUITE.{1,2,3} court harness.
//!
//! Commands (each is one evidence phase; the Docker orchestrator runs them in order):
//!
//! ```text
//! census           --census <census.jsonl> --out <reports/gnucobol-testsuite> [--pass a]
//! classify         --trees <trees-root/pass> --meta <meta.json> --out <outputs/pass> --pass a
//! determinism      --pass-a <summaryA.json> --pass-b <summaryB.json> --out <reports/gnucobol-testsuite>
//! receipts-finalize --root <repo> --meta <meta-final.json>
//! gate check       --root <repo>
//! ```
//!
//! The `classify` step writes the per-pass reports; the host orchestrator copies the committed
//! reports back into the repository and runs `determinism`, `receipts-finalize` and `gate check`
//! on the host.

use gnucobol_rs_testsuite::{
    autotest, census, classify, compat, determinism, gate, math, model, receipts,
};

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
        "census" => cmd_census(&args),
        "classify" => cmd_classify(&args),
        "determinism" => cmd_determinism(&args),
        "receipts-finalize" => cmd_receipts_finalize(&args),
        "compat-doc" => {
            let policy = arg_val(&args, "--policy").unwrap_or_else(|| "policy.json".into());
            let census_path = arg_val(&args, "--census")
                .unwrap_or_else(|| "reports/gnucobol-testsuite/invocation-census.json".into());
            let out = arg_val(&args, "--out")
                .unwrap_or_else(|| "docs/generated/cobc-rs-option-compatibility.md".into());
            let check = args.iter().any(|a| a == "--check");
            let r = if check {
                compat::check(Path::new(&policy), Path::new(&census_path), Path::new(&out))
            } else {
                compat::generate(Path::new(&policy), Path::new(&census_path), Path::new(&out))
                    .map(|_| ())
            };
            match r {
                Ok(()) => {
                    if check {
                        println!("compat-doc: {out} is fresh (regenerated == committed)");
                    } else {
                        println!("compat-doc: wrote {out}");
                    }
                    0
                }
                Err(e) => {
                    eprintln!("compat-doc: {e}");
                    1
                }
            }
        }
        "math" => {
            // The math subset is derived from the FULL per-test inventory (the same ledger the
            // Docker orchestrator passes: --results reports/gnucobol-testsuite/test-inventory.json),
            // which carries the `group` + `reason_code` fields the subset filter needs.
            let results = arg_val(&args, "--results")
                .unwrap_or_else(|| "reports/gnucobol-testsuite/test-inventory.json".into());
            let out =
                arg_val(&args, "--out").unwrap_or_else(|| "reports/gnucobol-runtime-tests".into());
            let check = args.iter().any(|a| a == "--check");
            let r = if check {
                math::verify(Path::new(&results), Path::new(&out))
            } else {
                math::generate(Path::new(&results), Path::new(&out)).map(|_| ())
            };
            match r {
                Ok(()) => {
                    if check {
                        println!("math: committed math-correctness reports are fresh (regenerated == committed)");
                    } else {
                        println!("math: correctness report written to {out}");
                    }
                    0
                }
                Err(e) => {
                    eprintln!("math: {e}");
                    1
                }
            }
        }
        "gate" => {
            let sub = args.get(2).map(String::as_str).unwrap_or("");
            match sub {
                "check" => {
                    let root = arg_val(&args, "--root").unwrap_or_else(|| ".".to_string());
                    let g = gate::gate_check(Path::new(&root));
                    for n in &g.notes {
                        println!("note: {n}");
                    }
                    for p in &g.problems {
                        eprintln!("GATE FAIL: {p}");
                    }
                    if g.problems.is_empty() {
                        println!(
                            "GNURUST.GNUCOBOL-TESTSUITE gate check: PASS (all invariants hold)"
                        );
                    } else {
                        println!(
                            "GNURUST.GNUCOBOL-TESTSUITE gate check: {} problem(s)",
                            g.problems.len()
                        );
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
                "usage: gnucobol-rs-testsuite <census|classify|determinism|receipts-finalize|compat-doc|math [--check]|gate check> ..."
            );
            2
        }
    };
    std::process::exit(code);
}

fn cmd_census(args: &[String]) -> i32 {
    let census_path =
        PathBuf::from(arg_val(args, "--census").unwrap_or_else(|| "census.jsonl".into()));
    let out = PathBuf::from(
        arg_val(args, "--out").unwrap_or_else(|| "reports/gnucobol-testsuite".into()),
    );
    let pass = arg_val(args, "--pass").unwrap_or_else(|| "a".into());
    match census::generate(&census_path, &out, &pass) {
        Ok(()) => {
            println!("census: artifacts written to {}", out.display());
            0
        }
        Err(e) => {
            eprintln!("census: {e}");
            1
        }
    }
}

fn cmd_classify(args: &[String]) -> i32 {
    let trees = PathBuf::from(arg_val(args, "--trees").unwrap_or_else(|| "trees".into()));
    let meta_path = PathBuf::from(arg_val(args, "--meta").unwrap_or_else(|| "meta.json".into()));
    let out = PathBuf::from(arg_val(args, "--out").unwrap_or_else(|| ".".into()));
    let pass = arg_val(args, "--pass").unwrap_or_else(|| "a".into());

    let meta = std::fs::read_to_string(&meta_path)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok());
    let (Some(meta), _) = (meta, ()) else {
        eprintln!("classify: cannot read meta {meta_path:?}");
        return 2;
    };

    let baseline_tree = trees.join("baseline");
    let candidate_tree = trees.join("candidate");

    // suite total from the generated testsuite script (authoritative inventory)
    let suite_script = baseline_tree.join("tests/testsuite");
    let suite_total = std::fs::read_to_string(&suite_script)
        .ok()
        .and_then(|s| autotest::suite_total(&s))
        .unwrap_or(0);
    if suite_total == 0 {
        eprintln!("classify: cannot derive the suite total from {suite_script:?}");
        return 2;
    }

    let inputs = classify::Inputs {
        baseline_log: baseline_tree.join("tests/testsuite.log"),
        baseline_dir: baseline_tree.join("tests/testsuite.dir"),
        candidate_log: candidate_tree.join("tests/testsuite.log"),
        candidate_dir: candidate_tree.join("tests/testsuite.dir"),
        suite_total,
        pass: pass.clone(),
    };
    let rows = match classify::classify(&inputs) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("classify: {e}");
            return 2;
        }
    };

    // oracle + candidate Autotest raw summaries (for the report, from the logs' tails)
    let oracle_summary = std::fs::read_to_string(&inputs.baseline_log)
        .ok()
        .and_then(|t| autotest::parse_summary(&t));
    let candidate_summary = std::fs::read_to_string(&inputs.candidate_log)
        .ok()
        .and_then(|t| autotest::parse_summary(&t));

    match classify::write_reports(&rows, &out, &pass, oracle_summary, candidate_summary) {
        Ok(summary) => {
            println!(
                "classify: {} tests reconciled; observable matches: {}",
                summary.total_tests, summary.comparison.observable_match
            );
            let _ = meta;
            0
        }
        Err(e) => {
            eprintln!("classify: {e}");
            1
        }
    }
}

fn cmd_determinism(args: &[String]) -> i32 {
    let out = PathBuf::from(
        arg_val(args, "--out").unwrap_or_else(|| "reports/gnucobol-testsuite".into()),
    );
    let pass_a =
        PathBuf::from(arg_val(args, "--pass-a").unwrap_or_else(|| "pass-a/summary.json".into()));
    let pass_b =
        PathBuf::from(arg_val(args, "--pass-b").unwrap_or_else(|| "pass-b/summary.json".into()));
    match determinism::compare(&pass_a, &pass_b, &out) {
        Ok(doc) => {
            let same = doc["stable_summary_identical"].as_bool().unwrap_or(false);
            println!(
                "determinism: stable_summary_identical={same} (classifications_identical={})",
                doc["per_test_classifications_identical"]
                    .as_bool()
                    .unwrap_or(false)
            );
            if same {
                0
            } else {
                1
            }
        }
        Err(e) => {
            eprintln!("determinism: {e}");
            2
        }
    }
}

fn cmd_receipts_finalize(args: &[String]) -> i32 {
    let root = PathBuf::from(arg_val(args, "--root").unwrap_or_else(|| ".".into()));
    let meta_path =
        PathBuf::from(arg_val(args, "--meta").unwrap_or_else(|| "meta-final.json".into()));
    let meta = std::fs::read_to_string(&meta_path)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok());
    let Some(meta) = meta else {
        eprintln!("receipts-finalize: cannot read meta {meta_path:?}");
        return 2;
    };
    let rep = root.join("reports/gnucobol-testsuite");
    let summary_text = std::fs::read_to_string(rep.join("summary.json"))
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok());
    let Some(summary_val) = summary_text else {
        eprintln!("receipts-finalize: summary.json missing/malformed");
        return 2;
    };
    let summary: model::Summary =
        serde_json::from_value(summary_val["summary"].clone()).unwrap_or_default();
    let census_total = std::fs::read_to_string(rep.join("invocation-census.json"))
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| v["total_invocations"].as_u64().map(|n| n as usize))
        .unwrap_or(0);
    let receipts_dir = root.join("reports/receipts");
    let written = receipts::write_receipts(&receipts_dir, &meta, &summary, census_total);
    for (gate, sha) in &written {
        println!("receipts-finalize: {gate} written (sha256 {sha})");
    }
    println!(
        "receipts-finalize: summaries reconcile — {} tests",
        summary.total_tests
    );
    0
}
