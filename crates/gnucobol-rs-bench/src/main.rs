//! `gnucobol-rs-bench` — performance-corpus validation + measurement CLI.
//!
//! `validate [workload] [scale]` — correctness gate: generate, compile with the host oracle,
//! run, and require a byte-exact match against the independent expected output BEFORE any
//! timing is reported.
//!
//! `validate-all` — the full correctness gate across all workloads and scales.
//!
//! `measure [view] [workload] [scale] [--iters N]` — Phase 9 performance views (spec 9.5):
//! `view-a` end-to-end one-shot, `view-b` front-end only, `view-c` repeated execution,
//! `view-d` runtime-operation microbenchmarks, `view-e` corpus throughput; `all` runs every
//! view. Workload/scale defaults: all workloads, `small` for views A/B/C (View E always runs
//! all scales, View D runs the micro set). Every view correctness-gates before timing.
//!
//! `report` — regenerate the Phase-8 report.
//!
//! `list` — list the corpus workloads.

use gnucobol_rs_bench::views;
use gnucobol_rs_bench::{validate, validate_all, workload, WORKLOADS};
use std::path::PathBuf;
use std::process::ExitCode;

fn write_report(
    all: &std::collections::BTreeMap<String, Vec<gnucobol_rs_bench::BenchResult>>,
) -> Result<(), String> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest
        .ancestors()
        .nth(2)
        .ok_or_else(|| "workspace root".to_string())?;
    let out = root.join("reports/valid-corpus/performance");
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(all).map_err(|e| e.to_string())?;
    std::fs::write(out.join("benchmarks.json"), json).map_err(|e| e.to_string())?;
    let mut md = String::new();
    md.push_str("# Performance corpus (Phase 8)\n\n");
    md.push_str(
        "Correctness-gated: every workload at every scale is byte-exact against the host\n",
    );
    md.push_str("GnuCOBOL 3.2.0 oracle BEFORE any timing is reported (spec 8.3). Inputs are\n");
    md.push_str(
        "deterministic (seeded generators, integer-exact); expected outputs are computed\n",
    );
    md.push_str("independently in Rust -- never by the candidate.\n\n");
    md.push_str("| workload | scale | records | compile ms | run ms | byte-exact |\n|---|---|---|---|---|---|\n");
    for (w, results) in all {
        for r in results {
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                r.workload, r.scale, r.records, r.oracle_compile_ms, r.oracle_run_ms, r.byte_exact
            ));
        }
        let _ = w;
    }
    std::fs::write(out.join("summary.md"), md).map_err(|e| e.to_string())?;
    Ok(())
}

fn run_validate(name: &str, scale: &str, work_root: &std::path::Path) -> ExitCode {
    let Some(w) = workload(name) else {
        eprintln!("unknown workload {name:?}");
        return ExitCode::FAILURE;
    };
    match validate(w, scale, work_root) {
        Ok(r) => {
            println!(
                "{} @ {}: {} records, compile {}ms run {}ms, {}",
                r.workload, r.scale, r.records, r.oracle_compile_ms, r.oracle_run_ms, r.note
            );
            if r.byte_exact {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(e) => {
            eprintln!("validate FAILED: {e}");
            ExitCode::FAILURE
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let work_root = std::env::var_os("GNURUST_COBOL_BENCH_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp/gnucobol-rs-bench"));
    std::fs::create_dir_all(&work_root).expect("bench root");
    match args.first().map(String::as_str) {
        Some("validate") => {
            let name = args.get(1).map(String::as_str).unwrap_or("all");
            let scale = args.get(2).map(String::as_str).unwrap_or("small");
            if name == "all" {
                match validate_all(&work_root) {
                    Ok(all) => {
                        for (w, results) in &all {
                            for r in results {
                                println!(
                                    "{} @ {}: {} records, compile {}ms run {}ms, {}",
                                    r.workload,
                                    r.scale,
                                    r.records,
                                    r.oracle_compile_ms,
                                    r.oracle_run_ms,
                                    r.note
                                );
                            }
                            let _ = w;
                        }
                        println!("validate-all: ALL CORRECTNESS GATES PASSED");
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("validate-all FAILED: {e}");
                        ExitCode::FAILURE
                    }
                }
            } else {
                run_validate(name, scale, &work_root)
            }
        }
        Some("measure") => {
            let rest: Vec<String> = args.iter().skip(1).cloned().collect();
            let parsed = match views::parse_measure_args(&rest) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("measure: {e}");
                    return ExitCode::FAILURE;
                }
            };
            match views::measure(&parsed, &work_root) {
                Ok(out) => {
                    views::print_console(&out);
                    match views::write_reports(&out) {
                        Ok(()) => {
                            println!(
                                "measure: reports written under reports/valid-corpus/performance/"
                            );
                            ExitCode::SUCCESS
                        }
                        Err(e) => {
                            eprintln!("measure: report write failed: {e}");
                            ExitCode::FAILURE
                        }
                    }
                }
                Err(e) => {
                    eprintln!("measure FAILED: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        Some("report") => match validate_all(&work_root) {
            Ok(all) => match write_report(&all) {
                Ok(()) => {
                    println!("reports/valid-corpus/performance/ written (all gates passed)");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("report write failed: {e}");
                    ExitCode::FAILURE
                }
            },
            Err(e) => {
                eprintln!("report FAILED: {e}");
                ExitCode::FAILURE
            }
        },
        Some("list") => {
            for w in WORKLOADS {
                println!("{} — {}", w.name, w.description);
            }
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!(
                "gnucobol-rs-bench validate <workload|all> [scale] | measure [view-a|view-b|view-c|view-d|view-e|all] [workload] [scale] [--iters N] | report | list"
            );
            ExitCode::FAILURE
        }
    }
}
