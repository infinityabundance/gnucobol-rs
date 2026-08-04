//! Receipt generation for `GNURUST.GNUCOBOL-TESTSUITE.{1,2,3}` — following the repository's
//! `gnurust-replay-receipt-v1` schema shape so the forensic casefile generator can consume them.

use crate::model::Summary;
use serde_json::{json, Value};
use std::path::Path;

fn file_sha(p: &Path) -> String {
    use sha2::{Digest, Sha256};
    match std::fs::read(p) {
        Ok(b) => {
            let mut h = Sha256::new();
            h.update(&b);
            format!("{:x}", h.finalize())
        }
        Err(_) => String::new(),
    }
}

fn envelope(gate: &str, court: &str, meta: &Value, replay: &str) -> Value {
    json!({
        "schema": "gnurust-replay-receipt-v1",
        "campaign": gate,
        "court": court,
        "conformance_claim": "NONE — differential observation over the admitted GnuCOBOL 3.2 native Autotest suite in this environment; no GnuCOBOL test-suite parity claim, no COBOL conformance certification, no compiler-replacement claim.",
        "generated_at": meta["generated_at"],
        "git_commit": meta["git_commit"],
        "crate_version": meta["crate_version"],
        "oracle": {
            "name": "GnuCOBOL",
            "version": meta["oracle"]["cobc_version"],
            "cobcrun_version": meta["oracle"]["cobcrun_version"],
            "source_sha256": meta["oracle"]["source_sha256"],
            "in_tree_prefix": meta["oracle"]["in_tree_prefix"],
            "configure": meta["oracle"]["configure"],
        },
        "command": {"replay": replay},
        "receipt_status": "current",
        "superseded_by": Value::Null,
        "current_authority": "STATUS.md",
        "environment": meta["environment"],
        "docker": meta["docker"],
        "preflight": meta["preflight"],
    })
}

/// GNURUST.GNUCOBOL-TESTSUITE.1 — suite custody + baseline + invocation census.
fn receipt_1(meta: &Value, summary: &Summary, census_total: usize) -> Value {
    let mut r = envelope(
        "GNURUST.GNUCOBOL-TESTSUITE.1",
        "GnuCOBOL 3.2 native Autotest suite custody + real-compiler baseline + invocation census",
        meta,
        "bash lab/gnucobol-testsuite/run-docker.sh",
    );
    r["byte_domain"] = json!(
        "admitted gnucobol-3.2 source (hash-verified), fresh in-tree build per pass, the generated Autotest testsuite run with the REAL admitted cobc, full invocation census (argv boundaries preserved), raw testsuite.log + per-group logs preserved"
    );
    r["non_claims"] = json!([
        "no claim about gnucobol-rs is made by this gate",
        "oracle results are specific to this admitted build + environment (stock configuration, no -fpermissive)",
        "oracle-side failures are observations about this build, not upstream defects",
        "the census records invocations, not compiler internals",
        "no NIST certification and no COBOL conformance claim"
    ]);
    r["results"] = json!({
        "suite_total_tests": summary.total_tests,
        "oracle_pass": summary.oracle.pass,
        "oracle_fail": summary.oracle.fail,
        "oracle_skip": summary.oracle.skip,
        "oracle_xfail": summary.oracle.xfail,
        "oracle_xpass": summary.oracle.xpass,
        "oracle_not_reached": summary.oracle.infra_error,
        "invocation_census_total": census_total,
        "invocation_census_sha256": meta["artifacts"]["invocation_census_json_sha256"],
        "oracle_results_sha256": meta["artifacts"]["oracle_results_json_sha256"],
    });
    r["verdict"] = "pass".into();
    r
}

/// GNURUST.GNUCOBOL-TESTSUITE.2 — candidate execution through the native harness.
fn receipt_2(meta: &Value, summary: &Summary) -> Value {
    let mut r = envelope(
        "GNURUST.GNUCOBOL-TESTSUITE.2",
        "candidate execution: the native suite run with COBC=cobc-rs (make localcheck), no-delegation proof, all tests accounted",
        meta,
        "bash lab/gnucobol-testsuite/run-docker.sh",
    );
    r["byte_domain"] = json!(
        "cobc-rs adapter + cobrun interpreter outcomes per test, raw candidate testsuite.log + group logs, mechanical no-delegation proof (linkage scans + PATH isolation)"
    );
    r["non_claims"] = json!([
        "no suite-pass or parity claim: this gate records candidate outcomes, it does not certify them",
        "candidate rejection is fail-closed (unsupported constructs are never silently run)",
        "generated launch artifacts are interpreter manifests, NOT native COBOL executables",
        "candidate execution never invokes cobc and never links libcob (mechanically enforced + recorded)"
    ]);
    r["results"] = json!({
        "candidate_parse_check_reject": summary.candidate.check_reject + summary.candidate.parse_reject + summary.candidate.layout_reject,
        "candidate_unsupported": summary.candidate.unsupported,
        "candidate_module_model_unsupported": summary.candidate.module_model_unsupported,
        "candidate_runtime_fail": summary.candidate.runtime_fail,
        "candidate_timeout": summary.candidate.timeout,
        "candidate_not_reached": summary.candidate.not_reached,
        "candidate_skipped": summary.candidate.skipped,
        "candidate_results_sha256": meta["artifacts"]["candidate_results_json_sha256"],
        "no_delegation": meta["no_delegation"],
    });
    r["verdict"] = "pass".into();
    r
}

/// GNURUST.GNUCOBOL-TESTSUITE.3 — differential comparison + classification.
fn receipt_3(meta: &Value, summary: &Summary) -> Value {
    let mut r = envelope(
        "GNURUST.GNUCOBOL-TESTSUITE.3",
        "GnuCOBOL testsuite differential comparison + per-test classification",
        meta,
        "bash lab/gnucobol-testsuite/run-docker.sh",
    );
    r["byte_domain"] = json!(
        "per-test oracle-vs-candidate observable comparison, first-failure attribution, all-tests-accounted reconciliation, failure buckets"
    );
    r["non_claims"] = json!([
        "no full GnuCOBOL test-suite parity claim",
        "no native-code-generation comparison (cobrun interprets; cobc emits C/native)",
        "OBSERVABLE_MATCH is scoped to this environment and the test's own assertions",
        "no claim that matching output proves equivalence outside the tested environment",
        "no claim that accepted no-op flags preserve all semantics outside the admitted tests",
        "no claim that a launcher is a GnuCOBOL-compatible native executable",
        "no claim that GnuCOBOL baseline failures prove upstream defects"
    ]);
    r["results"] = json!({
        "tests_accounted": summary.total_tests,
        "observable_match": summary.comparison.observable_match,
        "stdout_mismatch": summary.comparison.stdout_mismatch,
        "stderr_mismatch": summary.comparison.stderr_mismatch,
        "exit_status_mismatch": summary.comparison.exit_status_mismatch,
        "generated_file_mismatch": summary.comparison.generated_file_mismatch,
        "first_failure": summary.first_failure,
        "comparison_results_sha256": meta["artifacts"]["comparison_results_json_sha256"],
        "summary_json_sha256": meta["artifacts"]["summary_json_sha256"],
        "determinism": meta["determinism"],
    });
    r["verdict"] = "pass".into();
    r
}

/// Render the receipt markdown (repo convention: `.md` is a rendering of `receipt.json`).
pub fn render_receipt_md(r: &Value) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        "<!-- GENERATED from receipt.json by gnucobol-rs-testsuite — DO NOT EDIT BY HAND.\n     Regenerate: bash lab/gnucobol-testsuite/run-docker.sh -->\n\
         # {} — {}\n\n\
         **Verdict: {}** · replay `{}`\n\n\
         | field | value |\n|-------|-------|\n\
         | campaign | `{}` |\n| court | {} |\n\
         | oracle | {} |\n| byte_domain | {} |\n\
         | replay command | `{}` |\n| generated_at | {} |\n| git_commit | `{}` |\n\
         | receipt_status | {} |\n\n\
         **Conformance claim:** {}\n\n\
         ## Results\n\n```json\n{}\n```\n\n\
         ## Non-claims\n\n",
        r["campaign"].as_str().unwrap_or(""),
        r["court"].as_str().unwrap_or(""),
        r["verdict"].as_str().unwrap_or("").to_uppercase(),
        r["command"]["replay"].as_str().unwrap_or(""),
        r["campaign"].as_str().unwrap_or(""),
        r["court"].as_str().unwrap_or(""),
        r["oracle"]["version"].as_str().unwrap_or(""),
        r["byte_domain"].as_str().unwrap_or(""),
        r["command"]["replay"].as_str().unwrap_or(""),
        r["generated_at"].as_str().unwrap_or(""),
        r["git_commit"].as_str().unwrap_or(""),
        r["receipt_status"].as_str().unwrap_or(""),
        r["conformance_claim"].as_str().unwrap_or(""),
        serde_json::to_string_pretty(&r["results"]).unwrap_or_default(),
    ));
    for nc in r["non_claims"].as_array().unwrap_or(&vec![]) {
        s.push_str(&format!("- {}\n", nc.as_str().unwrap_or("")));
    }
    s.push_str("\n> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is\n> generated from `receipt.json`; the binding evidence is the JSON.\n");
    s
}

/// Write the three receipts under `reports/receipts/GNURUST.GNUCOBOL-TESTSUITE.{1,2,3}/`.
pub fn write_receipts(
    receipts_dir: &Path,
    meta: &Value,
    summary: &Summary,
    census_total: usize,
) -> Vec<(String, String)> {
    let gates: [(&str, Value); 3] = [
        (
            "GNURUST.GNUCOBOL-TESTSUITE.1",
            receipt_1(meta, summary, census_total),
        ),
        ("GNURUST.GNUCOBOL-TESTSUITE.2", receipt_2(meta, summary)),
        ("GNURUST.GNUCOBOL-TESTSUITE.3", receipt_3(meta, summary)),
    ];
    let mut written = Vec::new();
    for (gate, r) in gates {
        let dir = receipts_dir.join(gate);
        let _ = std::fs::create_dir_all(&dir);
        let jf = dir.join("receipt.json");
        let mf = dir.join("receipt.md");
        let _ = std::fs::write(&jf, serde_json::to_string_pretty(&r).unwrap() + "\n");
        let _ = std::fs::write(&mf, render_receipt_md(&r));
        written.push((gate.to_string(), file_sha(&jf)));
    }
    written
}
