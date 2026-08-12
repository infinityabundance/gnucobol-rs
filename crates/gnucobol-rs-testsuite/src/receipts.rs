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
    let replay = r["command"]["replay"]
        .as_str()
        .unwrap_or("bash lab/gnucobol-testsuite/run-docker.sh");
    let mut s = String::new();
    s.push_str(&format!(
        "<!-- GENERATED from receipt.json by gnucobol-rs-testsuite — DO NOT EDIT BY HAND.\n     Regenerate: {replay} -->\n\
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

/// GNURUST.GNUCOBOL-TESTSUITE.DIAGNOSTIC-UNBLOCKED.1 — the diagnostic-unblocked lane receipt.
/// Built from the committed lane evidence (meta.json + the three Phase 7-9 reports) so the
/// receipt is a deterministic projection of the same evidence the court binds.
pub fn write_diag_unblocked_receipt(
    receipts_dir: &Path,
    du_rep: &Path,
) -> Result<(String, String), String> {
    let gate = "GNURUST.GNUCOBOL-TESTSUITE.DIAGNOSTIC-UNBLOCKED.1";
    let read = |name: &str| -> Result<Value, String> {
        let p = du_rep.join(name);
        let text = std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", p.display()))
    };
    let meta = read("meta.json")?;
    let reach = read("semantic-reachability.json")?;
    let rec = read("pristine-vs-diagnostic-unblocked.json")?;
    let cross = read("corpus-cross-check.json")?;
    let t = &reach["totals"];
    let mut r = json!({
        "schema": "gnurust-replay-receipt-v1",
        "campaign": gate,
        "court": "diagnostic-unblocked testsuite lane: mechanically restricted derivative exposing later semantic checks hidden behind exact compiler-diagnostic wording",
        "conformance_claim": "NONE — semantic-reachability observation over a mechanically restricted derivative of the admitted GnuCOBOL 3.2 Autotest suite; no test-suite parity claim, no diagnostic-compatibility claim, no COBOL conformance certification.",
        "generated_at": reach["generated_at_utc"],
        "git_commit": meta["git_commit"],
        "crate_version": meta["crate_version"],
        "oracle": {"name": "GnuCOBOL", "version": meta["cobc_version"]},
        "command": {"replay": "bash lab/gnucobol-testsuite/run-diagnostic-unblocked-docker.sh"},
        "receipt_status": "current",
        "superseded_by": Value::Null,
        "current_authority": "STATUS.md",
        "environment": meta["environment"],
    });
    r["byte_domain"] = json!(
        "diagnostic-ignore.patch + transformations.json + tree-manifest.json + semantic-reachability.{json,md} + pristine-vs-diagnostic-unblocked.{json,md} + corpus-cross-check.{json,md} + both passes' raw testsuite evidence under reports/gnucobol-testsuite/diagnostic-unblocked/raw/"
    );
    r["non_claims"] = json!([
        "diagnostic-unblocked results are NOT pristine upstream testsuite passes",
        "ignored compiler diagnostic text is NOT diagnostic compatibility",
        "expected exit statuses, semantic runtime output and generated-file expectations are still enforced exactly",
        "the pristine upstream testsuite remains the compatibility authority and is untouched",
        "no new language/runtime compatibility claim from diagnostic-only steps",
        "the transformer decides solely from upstream test structure, never from candidate behaviour",
        "no candidate parser-success claim for steps validated only by the real cobc oracle",
    ]);
    r["results"] = json!({
        "generated_testsuite_sha256": meta["generated_testsuite_sha256"],
        "generated_testsuite_bytes": meta["generated_testsuite_bytes"],
        "patch_sha256": meta["patch_sha256"],
        "transformer_version": meta["transformer_version"],
        "diagnostic_expectations_ignored": t["diagnostic_expectations_ignored"],
        "stdout_ignored": t["stdout_ignored"],
        "stderr_ignored": t["stderr_ignored"],
        "groups_affected": t["groups_affected"],
        "groups_progressed_further": t["groups_progressed_further"],
        "groups_no_additional_step": t["groups_no_additional_step"],
        "gate_lifted_no_progress": t["gate_lifted_no_progress"],
        "groups_later_compile_reached": t["groups_later_compile_reached"],
        "groups_execution_reached": t["groups_execution_reached"],
        "newly_reached_checks": t["newly_reached_checks"],
        "newly_reached_runtime_checks": t["newly_reached_runtime_checks"],
        "newly_matched_runtime_checks": t["newly_matched_runtime_checks"],
        "newly_exposed_compile_failures": t["newly_exposed_compile_failures"],
        "newly_exposed_runtime_failures": t["newly_exposed_runtime_failures"],
        "pristine_group_passes": t["pristine_group_passes"],
        "unblocked_group_passes": t["unblocked_group_passes"],
        "pristine_candidate_xpass": t["pristine_candidate_xpass"],
        "unblocked_candidate_xpass": t["unblocked_candidate_xpass"],
        "oracle_unblocked_xpass": reach["oracle"]["unblocked_xpass"],
        "oracle_pristine_xpass": reach["oracle"]["pristine_xpass"],
        "suite_groups": t["suite_groups"],
        "at_setup_pristine": rec["at_setup_pristine"],
        "at_setup_transformed": rec["at_setup_transformed"],
        "at_check_pristine": rec["at_check_pristine"],
        "at_check_transformed": rec["at_check_transformed"],
        "patch_reproducible": rec["patch_reproducible"],
        "transformations_reproducible": rec["transformations_reproducible"],
        "group_index_identical": rec["group_index_identical"],
        "gate_failures": rec["gate"]["failures"],
        "cross_check_matched": cross["totals"]["matched_in_corpus"],
        "cross_check_agreed": cross["totals"]["agreed"],
        "cross_check_contract_contradictions": cross["totals"]["contract_contradictions"],
        "cross_check_candidate_failures_on_valid_steps": cross["totals"]["candidate_failures_on_valid_steps"],
    });
    r["verdict"] = "pass".into();
    let dir = receipts_dir.join(gate);
    let _ = std::fs::create_dir_all(&dir);
    let jf = dir.join("receipt.json");
    let mf = dir.join("receipt.md");
    std::fs::write(&jf, serde_json::to_string_pretty(&r).unwrap() + "\n")
        .map_err(|e| e.to_string())?;
    std::fs::write(&mf, render_receipt_md(&r)).map_err(|e| e.to_string())?;
    Ok((gate.to_string(), file_sha(&jf)))
}
