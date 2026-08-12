//! kani+fuzz coverage receipts (GNURUST.VERIFY.KANI-FUZZ.1). Faithful Rust port of lab/kani-fuzz/run.py.
//! Scans crate src for `// KANIFOR: <id>` and fuzz targets for `//! FUZZFOR: <id>`, maps every court to its
//! {kani, fuzz} evidence, and (check) FAILS if any impl court lacks either.
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;

fn na_reason(cid: &str) -> Option<&'static str> {
    Some(match cid {
        "GNURUST.COVERAGE.1" => "meta: the coverage map itself (a generated receipt, not a byte kernel)",
        "GNURUST.INTRINSIC.ATLAS.1" => "observed atlas: records oracle behavior, no Rust implementation to prove/fuzz",
        "GNURUST.PROCEDURE.FLOW.ATLAS.1" => "observed atlas: control-flow observation, no execution kernel",
        "SIZE.ERROR.ATLAS.1" => "observed atlas: size-error observation, no Rust implementation",
        "GNURUST.FILE.STATUS.1" => "observed atlas: file-status observation, no Rust implementation",
        "GNURUST.LINEAGE.CORPUS.20M.0" => "meta: differential corpus engine (drives the real cobc oracle); no single Rust byte kernel -- its own seal-grade gate is replay+Merkle+isolation",
        "GNURUST.LINEAGE.CORPUS.20M.SMOKE" => "meta: 200K real-cobc witness burn classified by the engine; no single Rust byte kernel",
        "GNURUST.LINEAGE.CORPUS.20M.1" => "meta: completed 4M real-cobc witness run; no single Rust byte kernel",
        "GNURUST.VALUE.NEGZERO.EDGE.1" => "characterization/regression-lock of the VALUE -0 oracle rule; exercises value_image which is kani+fuzz-covered under GNURUST.8",
        "GNURUST.CCVS85.2" => "meta: NIST CCVS85 corpus materialization + real-GnuCOBOL oracle baseline; no single Rust byte kernel -- its own seal-grade gate is replay+all-512-accounted+raw-evidence",
        "GNURUST.CCVS85.3" => "meta: NIST CCVS85 corpus cobrun baseline (isolated, no-delegation); no single Rust byte kernel -- its own seal-grade gate is replay+no-delegation+raw-evidence",
        "GNURUST.CCVS85.4" => "meta: NIST CCVS85 differential comparison + per-unit classification; no single Rust byte kernel -- its own seal-grade gate is replay+classification-reconciliation",
        "GNURUST.GNUCOBOL-TESTSUITE.1" => "meta: admitted GnuCOBOL 3.2 native Autotest suite custody + real-compiler baseline + invocation census; no Rust byte kernel -- its own seal-grade gate is replay+raw-evidence+census",
        "GNURUST.GNUCOBOL-TESTSUITE.2" => "meta: candidate execution through the native harness (COBC=cobc-rs) with no-delegation proof; no single byte kernel -- its own seal-grade gate is replay+no-delegation+all-accounted",
        "GNURUST.GNUCOBOL-TESTSUITE.3" => "meta: differential classification of the suite; no single byte kernel -- its own seal-grade gate is replay+classification-reconciliation",
        "GNURUST.GNUCOBOL-TESTSUITE.4" => "meta: full suite re-measured after the boundary reductions; no single byte kernel -- replay+reconciliation+determinism is its gate",
        "GNURUST.GNUCOBOL-TESTSUITE.BOUNDARY-REDUCTION.1" => "meta: per-test before/after transition ledger over the rerun; no byte kernel -- ledger+raw-evidence is its gate",
        "GNURUST.GNUCOBOL-TESTSUITE.DIAGNOSTIC-UNBLOCKED.1" => "meta: mechanically restricted derivative lane (only proven compiler-diagnostic streams become ignore) measuring semantic reachability; no byte kernel -- its seal-grade gates are the independent patch-policy gate + two-pass determinism + reconciliation",
        "GNURUST.MODULE.REGISTRY.1" => "meta: interpreted module lifecycle (cobcrun-rs runner + module search + -m artifacts); end-to-end integration court, no single byte kernel -- crates/cobc-rs tests are its gate",
        "GNURUST.MODULE.CALL.1" => "meta: CALL across separately compiled modules; integration court -- cobc-rs module courts are its gate",
        "GNURUST.MODULE.CANCEL.1" => "meta: CANCEL semantics (state reset + active-program fatal); integration court -- cobc-rs module courts are its gate",
        "GNURUST.MODULE.SEARCH.1" => "meta: cobcrun module search paths + error messages; integration court -- cobc-rs module courts are its gate",
        "GNURUST.MODULE.PARALLEL.1" => "meta: parallel module isolation; integration/stress court -- the 100-way cobc-rs test is its gate",
        "GNURUST.COBC-RS.NATIVE-MODE-BOUNDARY.1" => "meta: native-code-mode typed boundary (option policy); no byte kernel -- policy registry + census are its gate",
        "GNURUST.COBC-RS.POLICY-COMPLETE.1" => "meta: option-policy registry completeness vs the invocation census; no byte kernel -- census reconciliation is its gate",
        "GNURUST.GNUCOBOL-RUNTIME-MATH.2" => "meta: math subset re-measured from the full ledger; no byte kernel -- the 323 reconciliation invariant is its gate",
        "GNURUST.GNUCOBOL-RUNTIME-MATH.1" => "meta: math-subset classification derived from TESTSUITE.3 (no separate byte kernel); performance is separately labeled",
        "GNURUST.METHODOLOGY.LIBCOB.1" => "meta: runtime port methodology/provenance documentation + machine records; no byte kernel",
        "GNURUST.METHODOLOGY.PARSER.1" => "meta: parser provenance audit documentation + machine records; no byte kernel",
        "GNURUST.COBC-RS.ARGS.1" => "covered by the cobc-rs argument-policy integration tests (crates/cobc-rs/tests/cli.rs) + the generated option-compatibility gate; kani/fuzz markers live in the gnucobol-rs byte kernel, not the driver",
        "GNURUST.COBC-RS.LAUNCHER.1" => "covered by the cobc-rs launcher/manifest integration tests (tamper guard, self-hash, exit status); no kani/fuzz marker in the driver crate",
        "GNURUST.COBC-RS.PARALLEL.1" => "covered by the cobc-rs 100-way parallel integration test; no kani/fuzz marker in the driver crate",
        "GNURUST.CORPUS.CUSTODY.1" => "meta: corpus custody gate over the committed evidence tree (presence + freeze); no single Rust byte kernel -- its own seal-grade gate is the corpus_court_sweep replay",
        "GNURUST.CORPUS.LICENCE.1" => "meta: corpus licence-decisions gate over the committed licences.json + quarantine report; no byte kernel -- the sweep is its gate",
        "GNURUST.CORPUS.DEDUP.1" => "meta: corpus deduplication gate over the committed dedup evidence; no single byte kernel -- the sweep + dedup reports are its gate",
        "GNURUST.VALID-PROGRAMS.GNUCOBOL-TESTSUITE.1" => "meta: valid-program corpus classification of the Autotest suite; no byte kernel -- the committed step-level reports + sweep are its gate",
        "GNURUST.VALID-PROGRAMS.CCVS85.1" => "meta: valid-program corpus CCVS85 classification + packages; no byte kernel -- the 512-unit reconciliation + sweep are its gate",
        "GNURUST.VALID-PROGRAMS.MANUAL.1" => "meta: manual-examples classification court; no byte kernel -- the committed lane reports + sweep are its gate",
        "GNURUST.VALID-PROGRAMS.EXTRAS.1" => "meta: shipped-programs + contributions inventory court; no byte kernel -- the committed custody/classification reports + sweep are its gate",
        "GNURUST.VALID-PROGRAMS.OMP.1" => "meta: Open Mainframe course inventory + platform-typing court; no byte kernel -- the committed inventory/programs reports + sweep are its gate",
        "GNURUST.VALID-PROGRAMS.XCOBOL.1" => "meta: X-COBOL immutable custody + classification + partition court; no byte kernel -- the committed custody/robustness/partitions reports + sweep are its gate",
        "GNURUST.VALID-PROGRAMS.HELD-OUT.1" => "meta: held-out evaluation measurement court; no byte kernel -- the bounded candidate probe (spec 10.3) + committed report + sweep are its gate",
        "GNURUST.VALID-PROGRAMS.ACCURACY.1" => "meta: raw-byte accuracy dimensions aggregation; no byte kernel -- the per-family accuracy evidence + sweep are its gate",
        "GNURUST.PERFORMANCE.FRONTEND.1" => "meta: front-end-only performance views (View B); no byte kernel -- the correctness-gated phase-metrics + raw samples + sweep are its gate",
        "GNURUST.PERFORMANCE.PREPARED.1" => "meta: prepared-program execution views (View C); no byte kernel -- the no-reparse prepared lane + raw samples + sweep are its gate",
        "GNURUST.PERFORMANCE.BUSINESS.1" => "meta: purpose-built business workload correctness gates; no byte kernel -- the byte-exact-before-timing benchmarks.json + sweep are its gate",
        "GNURUST.PERFORMANCE.CORPUS.1" => "meta: corpus throughput views (View E); no byte kernel -- the raw-sample-preserving throughput report + sweep are its gate",
        _ => return None,
    })
}

const IS_ATLAS_EXTRA: [&str; 47] = [
    "GNURUST.COVERAGE.1",
    "GNURUST.FILE.STATUS.1",
    "GNURUST.PUBLIC.CORPUS.1",
    "GNURUST.BUILD.PROFILE.1",
    "GNURUST.PUBLIC.GAP.1",
    "GNURUST.LINEAGE.CORPUS.20M.0",
    "GNURUST.LINEAGE.CORPUS.20M.SMOKE",
    "GNURUST.LINEAGE.CORPUS.20M.1",
    "GNURUST.VALUE.NEGZERO.EDGE.1",
    "GNURUST.CCVS85.2",
    "GNURUST.CCVS85.3",
    "GNURUST.CCVS85.4",
    "GNURUST.GNUCOBOL-TESTSUITE.1",
    "GNURUST.GNUCOBOL-TESTSUITE.2",
    "GNURUST.GNUCOBOL-TESTSUITE.3",
    "GNURUST.GNUCOBOL-TESTSUITE.4",
    "GNURUST.GNUCOBOL-TESTSUITE.BOUNDARY-REDUCTION.1",
    "GNURUST.GNUCOBOL-TESTSUITE.DIAGNOSTIC-UNBLOCKED.1",
    "GNURUST.MODULE.REGISTRY.1",
    "GNURUST.MODULE.CALL.1",
    "GNURUST.MODULE.CANCEL.1",
    "GNURUST.MODULE.SEARCH.1",
    "GNURUST.MODULE.PARALLEL.1",
    "GNURUST.COBC-RS.NATIVE-MODE-BOUNDARY.1",
    "GNURUST.COBC-RS.POLICY-COMPLETE.1",
    "GNURUST.GNUCOBOL-RUNTIME-MATH.2",
    "GNURUST.GNUCOBOL-RUNTIME-MATH.1",
    "GNURUST.METHODOLOGY.LIBCOB.1",
    "GNURUST.METHODOLOGY.PARSER.1",
    "GNURUST.COBC-RS.ARGS.1",
    "GNURUST.COBC-RS.LAUNCHER.1",
    "GNURUST.COBC-RS.PARALLEL.1",
    "GNURUST.CORPUS.CUSTODY.1",
    "GNURUST.CORPUS.LICENCE.1",
    "GNURUST.CORPUS.DEDUP.1",
    "GNURUST.VALID-PROGRAMS.GNUCOBOL-TESTSUITE.1",
    "GNURUST.VALID-PROGRAMS.CCVS85.1",
    "GNURUST.VALID-PROGRAMS.MANUAL.1",
    "GNURUST.VALID-PROGRAMS.EXTRAS.1",
    "GNURUST.VALID-PROGRAMS.OMP.1",
    "GNURUST.VALID-PROGRAMS.XCOBOL.1",
    "GNURUST.VALID-PROGRAMS.HELD-OUT.1",
    "GNURUST.VALID-PROGRAMS.ACCURACY.1",
    "GNURUST.PERFORMANCE.FRONTEND.1",
    "GNURUST.PERFORMANCE.PREPARED.1",
    "GNURUST.PERFORMANCE.BUSINESS.1",
    "GNURUST.PERFORMANCE.CORPUS.1",
];

fn read_json(p: &Path) -> Value {
    std::fs::read_to_string(p)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(Value::Null)
}

fn scan_tags(dir: &Path, tag: &str) -> HashMap<String, Vec<String>> {
    let mut out: HashMap<String, Vec<String>> = HashMap::new();
    let mut files: Vec<std::path::PathBuf> = match std::fs::read_dir(dir) {
        Ok(rd) => rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().map(|x| x == "rs").unwrap_or(false))
            .collect(),
        Err(_) => return out,
    };
    files.sort();
    for f in files {
        let fname = f
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        for line in std::fs::read_to_string(&f).unwrap_or_default().lines() {
            if let Some(pos) = line.find(tag) {
                let rest = line[pos + tag.len()..]
                    .trim_start()
                    .trim_start_matches(':')
                    .trim_start();
                for cid in rest.split(|c: char| c == ',' || c.is_whitespace()) {
                    let cid = cid.trim().trim_end_matches('.');
                    if cid.starts_with("GNURUST.")
                        || cid.starts_with("KOBOLD.")
                        || cid.starts_with("SIZE.")
                    {
                        out.entry(cid.to_string()).or_default().push(fname.clone());
                    }
                }
            }
        }
    }
    out
}

fn build(root: &str) -> Value {
    let r = Path::new(root);
    let cl = read_json(&r.join("reports/claim-ladder.json"));
    let kani = scan_tags(&r.join("crates/gnucobol-rs/src"), "KANIFOR");
    let fz = scan_tags(&r.join("crates/gnucobol-rs/fuzz/fuzz_targets"), "FUZZFOR");
    let mut rows = Vec::new();
    if let Some(courts) = cl["courts"].as_array() {
        for c in courts {
            let cid = c["id"].as_str().unwrap_or("");
            let is_atlas = cid.contains("ATLAS") || IS_ATLAS_EXTRA.contains(&cid);
            if !cid.starts_with("GNURUST.") || is_atlas {
                let reason = na_reason(cid).unwrap_or(if is_atlas {
                    "observed atlas / meta: no Rust byte kernel"
                } else {
                    "KOBOLD composition / governance / view court: no gnucobol-rs byte kernel (composes sealed courts)"
                });
                rows.push(json!({"court": cid, "kani": "n/a", "fuzz": "n/a", "reason": reason}));
            } else {
                rows.push(json!({"court": cid, "kani": kani.get(cid).cloned().unwrap_or_default(), "fuzz": fz.get(cid).cloned().unwrap_or_default()}));
            }
        }
    }
    let impl_rows: Vec<&Value> = rows.iter().filter(|r| r["kani"] != "n/a").collect();
    let kani_cov = impl_rows
        .iter()
        .filter(|r| !r["kani"].as_array().map(|a| a.is_empty()).unwrap_or(true))
        .count();
    let fuzz_cov = impl_rows
        .iter()
        .filter(|r| !r["fuzz"].as_array().map(|a| a.is_empty()).unwrap_or(true))
        .count();
    json!({
        "schema": "gnurust-kani-fuzz-coverage-v1", "court": "GNURUST.VERIFY.KANI-FUZZ.1",
        "total_courts": rows.len(),
        "na_courts": rows.iter().filter(|r| r["kani"] == "n/a").count(),
        "impl_courts": impl_rows.len(),
        "kani_covered": kani_cov, "fuzz_covered": fuzz_cov,
        "rows": rows
    })
}

pub fn run(cmd: &str, root: &str) -> i32 {
    let b = build(root);
    match cmd {
        "generate" => {
            let _ = std::fs::write(
                Path::new(root).join("reports/kani-fuzz-coverage.json"),
                serde_json::to_vec_pretty(&b).unwrap_or_default(),
            );
            println!(
                "kani-fuzz coverage: {}/{} kani, {}/{} fuzz ({} n/a)",
                b["kani_covered"],
                b["impl_courts"],
                b["fuzz_covered"],
                b["impl_courts"],
                b["na_courts"]
            );
            0
        }
        "check" => {
            let rows = b["rows"].as_array().unwrap();
            let missing_k: Vec<&str> = rows
                .iter()
                .filter(|r| {
                    r["kani"] != "n/a" && r["kani"].as_array().map(|a| a.is_empty()).unwrap_or(true)
                })
                .map(|r| r["court"].as_str().unwrap())
                .collect();
            let missing_f: Vec<&str> = rows
                .iter()
                .filter(|r| {
                    r["kani"] != "n/a" && r["fuzz"].as_array().map(|a| a.is_empty()).unwrap_or(true)
                })
                .map(|r| r["court"].as_str().unwrap())
                .collect();
            for c in &missing_k {
                println!("   kani-missing: {c}");
            }
            for c in &missing_f {
                println!("   fuzz-missing: {c}");
            }
            if !missing_k.is_empty() || !missing_f.is_empty() {
                println!(
                    "!! kani-fuzz coverage incomplete: {} kani + {} fuzz missing",
                    missing_k.len(),
                    missing_f.len()
                );
                return 1;
            }
            println!("kani-fuzz: all {} impl courts have BOTH a Kani proof and a fuzz target; {} n/a (declared)", b["impl_courts"], b["na_courts"]);
            0
        }
        _ => {
            eprintln!("usage: kani-fuzz generate|check");
            2
        }
    }
}
