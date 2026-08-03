//! Receipt generation for `GNURUST.CCVS85.2/.3/.4` — the three new evidence gates.
//!
//! Receipts follow the repository's `gnurust-replay-receipt-v1` schema shape (so the forensic
//! casefile generator `kobold-courts` can consume them), extended with the CCVS85-specific facts.
//! They are generated from the live run's artifacts inside the container and copied back into the
//! repo; the `gate check` subcommand verifies their freshness (receipt hashes == current files).

use crate::corpus::sha256_hex;
use crate::model::Summary;
use serde_json::{json, Value};
use std::path::Path;

/// Read a file's sha256 ("" when absent).
fn file_sha(p: &Path) -> String {
    match std::fs::read(p) {
        Ok(b) => sha256_hex(&b),
        Err(_) => String::new(),
    }
}

/// Build the common receipt envelope.
fn envelope(gate: &str, court: &str, meta: &Value, replay: &str) -> Value {
    json!({
        "schema": "gnurust-replay-receipt-v1",
        "campaign": gate,
        "court": court,
        "conformance_claim": "NONE — differential observation over the admitted NIST CCVS85 corpus; no NIST certification, no full COBOL-85 conformance, no compiler-replacement claim.",
        "generated_at": meta["generated_at"],
        "git_commit": meta["git_commit"],
        "crate_version": meta["crate_version"],
        "oracle": {
            "name": "GnuCOBOL",
            "version": meta["oracle"]["cobc_version"],
            "source_sha256": meta["oracle"]["source_sha256"],
            "built_prefix": meta["oracle"]["built_prefix"],
        },
        "command": {"replay": replay},
        "receipt_status": "current",
        "superseded_by": Value::Null,
        "current_authority": "STATUS.md",
        "environment": meta["environment"],
        "docker": meta["docker"],
    })
}

/// The GNURUST.CCVS85.2 receipt (materialization + oracle baseline).
pub fn receipt_2(meta: &Value, summary: &Summary) -> Value {
    let mut r = envelope(
        "GNURUST.CCVS85.2",
        "CCVS85 materialization + real-GnuCOBOL oracle baseline",
        meta,
        "bash lab/ccvs85/run-docker.sh",
    );
    r["byte_domain"] = json!(
        "CCVS85 corpus units materialized to stable files (hashes recorded) + per-unit cobc compile/run outcomes (real GnuCOBOL 3.2, pinned source)"
    );
    r["non_claims"] = json!([
        "no claim about gnucobol-rs is made by this gate",
        "oracle acceptance/rejection is specific to the pinned GnuCOBOL 3.2 build and its dialect",
        "no NIST certification and no COBOL-85 conformance claim",
        "CLBRY/DATA* units are support units, not executable tests"
    ]);
    r["results"] = json!({
        "units_indexed": summary.units_total,
        "units_by_kind": summary.units_by_kind,
        "executable_candidates": summary.executable_candidates,
        "oracle_compile_pass": summary.oracle_compile_pass,
        "oracle_compile_reject": summary.oracle_compile_reject,
        "oracle_compile_error": summary.oracle_compile_error,
        "oracle_run_pass": summary.oracle_run_pass,
        "oracle_run_fail": summary.oracle_run_fail,
        "oracle_timeout": summary.oracle_timeout,
        "harness_blocked": summary.harness_blocked,
        "dependency_blocked": summary.dependency_blocked,
        "materialized_manifest_sha256": meta["artifacts"]["materialized_units_json_sha256"],
        "oracle_results_sha256": meta["artifacts"]["oracle_results_json_sha256"],
    });
    r["verdict"] = "pass".into();
    r
}

/// The GNURUST.CCVS85.3 receipt (gnucobol-rs execution baseline).
pub fn receipt_3(meta: &Value, summary: &Summary) -> Value {
    let mut r = envelope(
        "GNURUST.CCVS85.3",
        "gnucobol-rs execution baseline over the materialized CCVS85 units",
        meta,
        "bash lab/ccvs85/run-docker.sh",
    );
    r["byte_domain"] = json!(
        "per-unit cobrun (native-Rust front-end + ported runtime) prepare/run/timeout outcomes with raw stdout/stderr preserved"
    );
    r["non_claims"] = json!([
        "no suite-pass claim: this gate records candidate outcomes, it does not certify them",
        "candidate rejection is fail-closed (unsupported constructs are never silently run)",
        "no claim that candidate acceptance implies COBOL-85 conformance",
        "candidate execution never invokes cobc and never links libcob (mechanically enforced and recorded)"
    ]);
    r["results"] = json!({
        "candidate_accepted": summary.candidate_accepted,
        "candidate_unsupported": summary.candidate_unsupported,
        "candidate_parse_fail": summary.candidate_parse_fail,
        "candidate_runtime_fail": summary.candidate_runtime_fail,
        "candidate_timeout": summary.candidate_timeout,
        "candidate_results_sha256": meta["artifacts"]["candidate_results_json_sha256"],
        "no_delegation": meta["no_delegation"],
    });
    r["verdict"] = "pass".into();
    r
}

/// The GNURUST.CCVS85.4 receipt (differential comparison + classification).
pub fn receipt_4(meta: &Value, summary: &Summary) -> Value {
    let mut r = envelope(
        "GNURUST.CCVS85.4",
        "CCVS85 differential comparison + per-unit classification",
        meta,
        "bash lab/ccvs85/run-docker.sh",
    );
    r["byte_domain"] = json!(
        "per-unit oracle-vs-candidate observable comparison: raw output, canonical output, generated files, exit status, CCVS85 verdict counts"
    );
    r["non_claims"] = json!([
        "no NIST certification",
        "no full COBOL-85 conformance claim",
        "no full cobc replacement claim",
        "no native-code-generation comparison (cobrun interprets; cobc emits C/native)",
        "no claim that an oracle rejection proves the source invalid under every COBOL implementation",
        "no claim that matching output proves equivalence outside the tested environment",
        "no claim that library/data units are executable tests",
        "no conversion of blocked units into passes"
    ]);
    r["results"] = json!({
        "units_accounted": summary.units_total,
        "by_final_classification": summary.by_final_classification,
        "raw_output_match": summary.raw_output_match,
        "canonical_output_match": summary.canonical_output_match,
        "output_mismatch": summary.output_mismatch,
        "exit_status_mismatch": summary.exit_status_mismatch,
        "generated_file_mismatch": summary.generated_file_mismatch,
        "nondeterministic": summary.nondeterministic,
        "comparison_results_sha256": meta["artifacts"]["comparison_results_json_sha256"],
        "summary_json_sha256": meta["artifacts"]["summary_json_sha256"],
        "determinism": meta["determinism"],
    });
    r["verdict"] = "pass".into();
    r
}

/// Render the receipt markdown (the repo convention: `.md` is a rendering of `receipt.json`).
pub fn render_receipt_md(r: &Value) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        "<!-- GENERATED from receipt.json by gnucobol-rs-ccvs85 — DO NOT EDIT BY HAND.\n     Regenerate: bash lab/ccvs85/run-docker.sh -->\n\
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

/// Write the three receipts under `reports/receipts/GNURUST.CCVS85.{2,3,4}/`.
pub fn write_receipts(
    receipts_dir: &Path,
    meta: &Value,
    summary: &Summary,
) -> Vec<(String, String)> {
    let gates: [(&str, Value); 3] = [
        ("GNURUST.CCVS85.2", receipt_2(meta, summary)),
        ("GNURUST.CCVS85.3", receipt_3(meta, summary)),
        ("GNURUST.CCVS85.4", receipt_4(meta, summary)),
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
