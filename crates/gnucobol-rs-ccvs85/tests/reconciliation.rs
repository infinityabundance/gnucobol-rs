//! Reconciliation / evidence-architecture tests: every unit in exactly one classification, totals
//! that reconcile, the all-512-accounted invariant, the candidate-no-oracle-delegation proof
//! schema, and receipt regeneration + freshness.

use gnucobol_rs_ccvs85::compare::summarize;
use gnucobol_rs_ccvs85::model::{
    CandidateSide, FinalClassification, MaterializedUnit, OracleSide, Summary, UnitResult,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

fn synthetic_unit(i: usize, kind: &str) -> MaterializedUnit {
    MaterializedUnit {
        unit_index: i,
        kind: kind.to_string(),
        name: format!("NC{i:03}A"),
        header_raw: "*HEADER,{}".into(),
        main_program: None,
        subprogram: None,
        source_path: format!("NC{i:03}A.cob"),
        source_sha256: format!("{:064x}", i),
        adapted_path: format!("adapted/NC{i:03}A.cob"),
        adapted_sha256: format!("{:064x}", i + 1),
        start_line: 1,
        end_line: 10,
        program_ids: vec![format!("NC{i:03}A")],
        copy_dependencies: vec![],
        missing_copybooks: vec![],
        data_dependencies: vec![],
        is_executable_candidate: kind == "COBOL",
    }
}

/// Build 512 synthetic units (459 COBOL + 51 CLBRY + 2 DATA*) with a deterministic mix of
/// classifications and assert the summary reconciles.
#[test]
fn all_512_accounted_and_totals_reconcile() {
    let mut units = Vec::new();
    let mut results = Vec::new();
    let mut n = 0usize;
    for i in 0..459 {
        let u = synthetic_unit(n, "COBOL");
        let mut oracle = OracleSide::default();
        let mut candidate = CandidateSide::default();
        let fc;
        match i % 12 {
            0 => {
                oracle.compile = "pass".into();
                oracle.run = "pass".into();
                candidate.prepare = "accepted".into();
                candidate.run = "pass".into();
                fc = FinalClassification::RawOutputMatch;
            }
            1 => {
                oracle.compile = "pass".into();
                oracle.run = "pass".into();
                candidate.prepare = "reject-unsupported".into();
                fc = FinalClassification::RustRejectUnsupported;
            }
            2 => {
                oracle.compile = "reject".into();
                fc = FinalClassification::OracleCompileReject;
            }
            3 => {
                oracle.compile = "pass".into();
                oracle.run = "fail".into();
                fc = FinalClassification::OracleRunFail;
            }
            4 => {
                oracle.compile = "pass".into();
                oracle.run = "pass".into();
                candidate.prepare = "accepted".into();
                candidate.run = "pass".into();
                fc = FinalClassification::CanonicalOutputMatch;
            }
            _ => {
                oracle.compile = "pass".into();
                oracle.run = "pass".into();
                candidate.prepare = "reject-unsupported".into();
                fc = FinalClassification::RustRejectUnsupported;
            }
        }
        results.push(UnitResult {
            unit_index: u.unit_index,
            kind: u.kind.clone(),
            name: u.name.clone(),
            source_path: u.source_path.clone(),
            source_sha256: u.source_sha256.clone(),
            oracle,
            candidate,
            comparison: Default::default(),
            final_classification: fc,
            reason_code: "TEST_REASON".into(),
            nondeterministic: false,
            determinism: None,
            first_failure_line: String::new(),
        });
        units.push(u);
        n += 1;
    }
    for _i in 0..51 {
        let u = synthetic_unit(n, "CLBRY");
        results.push(UnitResult {
            final_classification: FinalClassification::NonExecutableLibrary,
            ..unit_result_from(&u)
        });
        units.push(u);
        n += 1;
    }
    for _i in 0..2 {
        let u = synthetic_unit(n, "DATA*");
        results.push(UnitResult {
            final_classification: FinalClassification::NonExecutableData,
            ..unit_result_from(&u)
        });
        units.push(u);
        n += 1;
    }
    assert_eq!(n, 512);

    let s = summarize(&results, &units);
    assert_eq!(s.units_total, 512);
    // every unit in exactly one primary classification bucket
    let bucket_sum: usize = s.by_final_classification.values().sum();
    assert_eq!(bucket_sum, 512, "{:?}", s.by_final_classification);
    // by-kind reconciles
    assert_eq!(s.units_by_kind.get("COBOL"), Some(&459));
    assert_eq!(s.units_by_kind.get("CLBRY"), Some(&51));
    assert_eq!(s.units_by_kind.get("DATA*"), Some(&2));
    // field counters are orthogonal: every classified unit contributes to oracle or candidate
    assert_eq!(
        s.oracle_compile_pass + s.oracle_compile_reject + s.oracle_compile_error,
        459,
        "every COBOL unit has an oracle compile outcome"
    );
    // primary-classification-specific counters reconcile with the buckets
    assert_eq!(s.non_executable_library + s.non_executable_data, 53);
    assert_eq!(
        s.by_final_classification
            .get("NON_EXECUTABLE_LIBRARY")
            .copied()
            .unwrap_or(0),
        51
    );
    assert_eq!(
        s.by_final_classification
            .get("NON_EXECUTABLE_DATA")
            .copied()
            .unwrap_or(0),
        2
    );
}

fn unit_result_from(u: &MaterializedUnit) -> UnitResult {
    UnitResult {
        unit_index: u.unit_index,
        kind: u.kind.clone(),
        name: u.name.clone(),
        source_path: u.source_path.clone(),
        source_sha256: u.source_sha256.clone(),
        oracle: OracleSide::default(),
        candidate: CandidateSide::default(),
        comparison: Default::default(),
        final_classification: FinalClassification::InfrastructureError,
        reason_code: String::new(),
        nondeterministic: false,
        determinism: None,
        first_failure_line: String::new(),
    }
}

#[test]
fn summary_counts_match_classification_buckets_for_the_committed_evidence() {
    // When the committed evidence is present (a normal checkout), the summary.json totals must
    // equal a fresh recomputation from comparison-results.json — the anti-freshness invariant the
    // `gate check` enforces. Absent evidence (minimal checkout) is a no-op, not a failure.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let comp_path = root.join("reports/ccvs85/comparison-results.json");
    if !comp_path.exists() {
        return;
    }
    let comp: Value = serde_json::from_str(&std::fs::read_to_string(&comp_path).unwrap()).unwrap();
    let sum_path = root.join("reports/ccvs85/summary.json");
    let sum: Value = serde_json::from_str(&std::fs::read_to_string(&sum_path).unwrap()).unwrap();
    let units = comp["units"].as_array().unwrap();
    assert_eq!(units.len(), 512, "all-512-accounted invariant");
    let mut by_class: BTreeMap<String, usize> = BTreeMap::new();
    for u in units {
        let fc = u["final_classification"].as_str().unwrap();
        *by_class.entry(fc.to_string()).or_insert(0) += 1;
    }
    let bucket_sum: usize = by_class.values().sum();
    assert_eq!(bucket_sum, 512);
    let s = &sum["summary"];
    let declared = s["by_final_classification"].as_object().unwrap();
    for (k, v) in &by_class {
        assert_eq!(
            declared.get(k).and_then(|x| x.as_u64()).map(|x| x as usize),
            Some(*v),
            "bucket {k} disagrees with a fresh recount"
        );
    }
    assert_eq!(s["units_total"].as_u64().unwrap(), 512);
}

#[test]
fn candidate_no_oracle_delegation_proof_schema() {
    // The committed no-delegation.json (when present) must carry the two mechanical proof flags
    // the gate requires: the candidate phase ran with the oracle isolated, and cobrun links no
    // libcob. Absent evidence is a no-op.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let p = root.join("reports/ccvs85/no-delegation.json");
    if !p.exists() {
        return;
    }
    let v: Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
    assert_eq!(
        v["schema"].as_str(),
        Some("gnurust-ccvs85-no-delegation-v1")
    );
    assert!(v["candidate_phase_isolated"].as_bool().unwrap_or(false));
    assert!(v["cobrun_links_no_libcob"].as_bool().unwrap_or(false));
    assert!(v["cobc_unavailable_during_candidate_phase"]
        .as_bool()
        .unwrap_or(false));
    // the candidate binary hash is recorded
    let sha = v["candidate_binary_sha256"].as_str().unwrap_or("");
    assert_eq!(sha.len(), 64);
}

#[test]
fn receipts_regenerate_and_are_fresh() {
    let dir = tempfile::tempdir().unwrap();
    let meta = serde_json::json!({
        "generated_at": "2026-01-01T00:00:00Z",
        "git_commit": "deadbeef",
        "crate_version": "0.0.0-test",
        "oracle": {"cobc_version": "cobc (GnuCOBOL) 3.2.0", "source_sha256": "x", "built_prefix": "/p"},
        "environment": {"LC_ALL": "C.UTF-8"},
        "docker": {"isolated_daemon": true},
        "artifacts": {
            "materialized_units_json_sha256": "a".repeat(64),
            "oracle_results_json_sha256": "b".repeat(64),
            "candidate_results_json_sha256": "c".repeat(64),
            "comparison_results_json_sha256": "d".repeat(64),
            "summary_json_sha256": "e".repeat(64)
        },
        "no_delegation": {"candidate_phase_isolated": true},
        "determinism": {"identical": true}
    });
    let summary = Summary {
        units_total: 512,
        oracle_compile_pass: 370,
        candidate_unsupported: 374,
        ..Default::default()
    };
    let receipts_dir = dir.path().join("receipts");
    let written = gnucobol_rs_ccvs85::receipts::write_receipts(&receipts_dir, &meta, &summary);
    assert_eq!(written.len(), 3);
    for (gate, _sha) in &written {
        let jf = receipts_dir.join(gate).join("receipt.json");
        let mf = receipts_dir.join(gate).join("receipt.md");
        assert!(jf.exists(), "{gate} receipt.json missing");
        assert!(mf.exists(), "{gate} receipt.md missing");
        let v: Value = serde_json::from_str(&std::fs::read_to_string(&jf).unwrap()).unwrap();
        assert_eq!(v["schema"].as_str(), Some("gnurust-replay-receipt-v1"));
        assert_eq!(v["campaign"].as_str(), Some(gate.as_str()));
        assert_eq!(v["verdict"].as_str(), Some("pass"));
        assert!(v["non_claims"]
            .as_array()
            .map(|a| !a.is_empty())
            .unwrap_or(false));
        let md = std::fs::read_to_string(&mf).unwrap();
        assert!(md.contains(gate), "receipt.md missing campaign name");
        // freshness: re-writing produces identical bytes (stable hashes)
        let again = gnucobol_rs_ccvs85::receipts::write_receipts(&receipts_dir, &meta, &summary);
        assert_eq!(written, again, "receipt regeneration is not stable");
    }
}

#[test]
fn canonicalize_is_symmetric_and_documented() {
    // the canonical schema version is recorded so comparisons stay auditable
    assert_eq!(
        gnucobol_rs_ccvs85::compare::CANONICAL_SCHEMA,
        "gnurust-ccvs85-canonical-v1"
    );
    let a = b"line one  \r\nline two\n\n\n";
    let b = b"line one\nline two\n";
    let ca = gnucobol_rs_ccvs85::compare::canonicalize(a);
    let cb = gnucobol_rs_ccvs85::compare::canonicalize(b);
    assert_eq!(ca, cb);
    // byte-preserving on already-canonical input
    let c = gnucobol_rs_ccvs85::compare::canonicalize(b"x\n");
    assert_eq!(c, b"x\n");
}
