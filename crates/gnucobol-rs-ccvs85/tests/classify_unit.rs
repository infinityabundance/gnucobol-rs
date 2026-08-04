//! GNURUST.CCVS85.4 classification tests — synthetic oracle/candidate sides for every decision
//! class: timeout, oracle compile rejection, candidate unsupported rejection, raw output match,
//! canonical output match, output mismatch, exit-status mismatch, generated-file mismatch, and the
//! non-executable kinds. Each case uses real temp-dir evidence files so the comparison reads the
//! same bytes the harness would preserve.

use gnucobol_rs_ccvs85::compare::{canonicalize, classify_unit};
use gnucobol_rs_ccvs85::model::{
    CandidateSide, FinalClassification, Invocation, MaterializedUnit, OracleSide,
};
use std::path::Path;

fn unit(name: &str, kind: &str) -> MaterializedUnit {
    MaterializedUnit {
        unit_index: 0,
        kind: kind.into(),
        name: name.into(),
        header_raw: format!("*HEADER,{},{}", kind, name),
        main_program: None,
        subprogram: None,
        source_path: format!("{name}.cob"),
        source_sha256: "aa".repeat(32),
        adapted_path: format!("adapted/{name}.cob"),
        adapted_sha256: "bb".repeat(32),
        start_line: 1,
        end_line: 10,
        program_ids: vec![name.into()],
        copy_dependencies: vec![],
        missing_copybooks: vec![],
        data_dependencies: vec![],
        is_executable_candidate: kind == "COBOL",
    }
}

/// Write an invocation whose stdout lives under `<work>/<side>/u0/run/evidence/stdout` and
/// (optionally) a REPORT file next to it (the layout `oracle_primary_output` reads). The oracle
/// and candidate sides use DISTINCT dirs, mirroring the harness's `work/oracle/` vs
/// `work/candidate/` separation.
fn inv_with_output(work: &Path, side: &str, stdout: &[u8], report: Option<&[u8]>) -> Invocation {
    let ev = work.join(format!("{side}/u0/run/evidence"));
    std::fs::create_dir_all(&ev).unwrap();
    std::fs::write(ev.join("stdout"), stdout).unwrap();
    if let Some(r) = report {
        std::fs::write(ev.join("REPORT"), r).unwrap();
    }
    Invocation {
        command: vec!["x".into()],
        cwd: work.display().to_string(),
        environment: vec![],
        exit_code: Some(0),
        signal: None,
        timed_out: false,
        duration_ms: 1,
        stdout_path: Some(ev.join("stdout").display().to_string()),
        stderr_path: Some(ev.join("stderr").display().to_string()),
        stdout_sha256: String::new(),
        stderr_sha256: String::new(),
        artifacts: vec![],
        error: None,
    }
}

fn oracle_pass_run(work: &Path, out: &[u8]) -> OracleSide {
    OracleSide {
        compile: "pass".into(),
        compile_invocation: Some(Invocation::default()),
        run: "pass".into(),
        run_invocation: Some(inv_with_output(work, "oracle", out, None)),
        report_sha256: String::new(),
        verdict_counts: None,
    }
}

fn candidate_pass(work: &Path, out: &[u8]) -> CandidateSide {
    let inv = inv_with_output(work, "candidate", out, None);
    CandidateSide {
        prepare: "accepted".into(),
        prepare_invocation: Some(inv.clone()),
        prepare_invocation_rc: Some(0),
        run: "pass".into(),
        run_invocation: Some(inv),
        stdout_sha256: String::new(),
        report_sha256: String::new(),
    }
}

#[test]
fn timeout_classification_oracle_run_timeout() {
    let dir = tempfile::tempdir().unwrap();
    let work = dir.path().to_path_buf();
    let mut o = oracle_pass_run(&work, b"x");
    o.run = "timeout".into();
    let mut inv = inv_with_output(&work, "oracle", b"x", None);
    inv.timed_out = true;
    inv.exit_code = Some(124);
    o.run_invocation = Some(inv);
    let r = classify_unit(
        &unit("NC999A", "COBOL"),
        &o,
        &candidate_pass(&work, b"x"),
        &work,
    );
    assert_eq!(r.final_classification, FinalClassification::OracleTimeout);
    assert_eq!(r.reason_code, "ORACLE_RUN_TIMEOUT");
}

#[test]
fn timeout_classification_candidate_timeout() {
    let dir = tempfile::tempdir().unwrap();
    let work = dir.path().to_path_buf();
    let mut c = candidate_pass(&work, b"x");
    c.run = "timeout".into();
    c.prepare_invocation_rc = Some(124);
    let mut inv = inv_with_output(&work, "candidate", b"", None);
    inv.timed_out = true;
    inv.exit_code = Some(124);
    c.run_invocation = Some(inv);
    let r = classify_unit(
        &unit("NC999A", "COBOL"),
        &oracle_pass_run(&work, b"x"),
        &c,
        &work,
    );
    assert_eq!(r.final_classification, FinalClassification::RustTimeout);
}

#[test]
fn oracle_compile_rejection_classified_separately() {
    let dir = tempfile::tempdir().unwrap();
    let work = dir.path().to_path_buf();
    let inv = Invocation {
        stderr_path: Some(work.join("cobc-err").display().to_string()),
        ..Default::default()
    };
    let o = OracleSide {
        compile: "reject".into(),
        compile_invocation: Some(inv),
        ..Default::default()
    };
    std::fs::write(
        work.join("cobc-err"),
        "file.cob:3: error: syntax error, unexpected ALL\n",
    )
    .unwrap();
    let r = classify_unit(
        &unit("IF999M", "COBOL"),
        &o,
        &CandidateSide::default(),
        &work,
    );
    assert_eq!(
        r.final_classification,
        FinalClassification::OracleCompileReject
    );
    assert!(r.reason_code.contains("SYNTAX_ERROR"), "{}", r.reason_code);
}

#[test]
fn candidate_unsupported_rejection_classified_separately() {
    let dir = tempfile::tempdir().unwrap();
    let work = dir.path().to_path_buf();
    let inv = Invocation {
        stderr_path: Some(work.join("cobrun-err").display().to_string()),
        ..Default::default()
    };
    let c = CandidateSide {
        prepare: "reject-unsupported".into(),
        run: "not-run".into(),
        prepare_invocation: Some(inv),
        prepare_invocation_rc: Some(2),
        ..Default::default()
    };
    std::fs::write(
        work.join("cobrun-err"),
        "cobrun: unsupported: WRITE `DUMMY-RECORD`: not an FD record\n",
    )
    .unwrap();
    let r = classify_unit(
        &unit("IC101A", "COBOL"),
        &oracle_pass_run(&work, b"r"),
        &c,
        &work,
    );
    assert_eq!(
        r.final_classification,
        FinalClassification::RustRejectUnsupported
    );
    assert!(
        r.reason_code.starts_with("COBRUN_UNSUPPORTED"),
        "{}",
        r.reason_code
    );
}

#[test]
fn raw_output_match_when_bytes_identical() {
    let dir = tempfile::tempdir().unwrap();
    let work = dir.path().to_path_buf();
    let bytes = b"THIS IS A DUMMY PROCEDURE\n";
    let r = classify_unit(
        &unit("DB301M", "COBOL"),
        &oracle_pass_run(&work, bytes),
        &candidate_pass(&work, bytes),
        &work,
    );
    assert_eq!(r.final_classification, FinalClassification::RawOutputMatch);
    assert_eq!(r.comparison.raw_stdout, "match");
    assert_eq!(r.comparison.exit_status, "match");
}

#[test]
fn raw_output_mismatch_then_canonical_match() {
    let dir = tempfile::tempdir().unwrap();
    let work = dir.path().to_path_buf();
    // raw bytes differ only by trailing whitespace -> canonical match, raw mismatch
    let oracle_out: &[u8] = b"line one  \nline two\n\n\n";
    let cand_out: &[u8] = b"line one\nline two\n";
    assert_ne!(oracle_out, cand_out);
    assert_eq!(canonicalize(oracle_out), canonicalize(cand_out));
    let r = classify_unit(
        &unit("NC999A", "COBOL"),
        &oracle_pass_run(&work, oracle_out),
        &candidate_pass(&work, cand_out),
        &work,
    );
    assert_eq!(
        r.final_classification,
        FinalClassification::CanonicalOutputMatch
    );
    assert_eq!(r.comparison.raw_stdout, "mismatch");
    assert_eq!(r.comparison.canonical_stdout, "match");
}

#[test]
fn output_mismatch_when_bytes_and_exit_differ() {
    let dir = tempfile::tempdir().unwrap();
    let work = dir.path().to_path_buf();
    let r = classify_unit(
        &unit("NC302M", "COBOL"),
        &oracle_pass_run(&work, b"DUMMY PROCEDURE\nFNC302\n"),
        &candidate_pass(&work, b"DUMMY PROCEDURE\n"),
        &work,
    );
    assert_eq!(r.final_classification, FinalClassification::OutputMismatch);
    assert_eq!(r.comparison.raw_stdout, "mismatch");
    assert_eq!(r.comparison.canonical_stdout, "mismatch");
}

#[test]
fn exit_status_mismatch_classified() {
    let dir = tempfile::tempdir().unwrap();
    let work = dir.path().to_path_buf();
    let o = oracle_pass_run(&work, b"out");
    let mut c = candidate_pass(&work, b"out");
    c.prepare_invocation_rc = Some(1);
    c.run = "fail".into();
    let mut inv = inv_with_output(&work, "candidate", b"out", None);
    inv.exit_code = Some(1);
    c.run_invocation = Some(inv);
    let r = classify_unit(&unit("NC999A", "COBOL"), &o, &c, &work);
    // candidate runtime failure takes precedence over the exit mismatch (deepest observable)
    assert_eq!(
        r.final_classification,
        FinalClassification::RustAcceptButRuntimeFail
    );
    // a pure exit mismatch (both ran, outputs differ, candidate rc != oracle rc) is classified
    // as EXIT_STATUS_MISMATCH — the defensive branch for an oracle rc 0 vs candidate rc non-zero
    // pair where the candidate run itself did not fail
    let o2 = oracle_pass_run(&work, b"oracle-out");
    let mut c2 = candidate_pass(&work, b"candidate-out");
    c2.run = "pass".into();
    c2.prepare_invocation_rc = Some(1);
    let mut inv2 = inv_with_output(&work, "candidate", b"candidate-out", None);
    inv2.exit_code = Some(1);
    c2.run_invocation = Some(inv2);
    let r2 = classify_unit(&unit("NC999B", "COBOL"), &o2, &c2, &work);
    assert_eq!(
        r2.final_classification,
        FinalClassification::ExitStatusMismatch
    );
    assert_eq!(r2.comparison.exit_status, "mismatch");
}

#[test]
fn generated_file_mismatch_classified() {
    let dir = tempfile::tempdir().unwrap();
    let work = dir.path().to_path_buf();
    // oracle run dir gets a generated file that the candidate run dir does not
    let run_dir = work.join("u0/run");
    std::fs::create_dir_all(&run_dir).unwrap();
    std::fs::write(run_dir.join("EXTRA-FILE"), b"x").unwrap();
    let o = oracle_pass_run(&work, b"out");
    let c = candidate_pass(&work, b"out");
    let r = classify_unit(&unit("NC999A", "COBOL"), &o, &c, &work);
    assert_eq!(
        r.final_classification,
        FinalClassification::GeneratedFileMismatch
    );
    assert_eq!(r.comparison.generated_files, "mismatch");
}

#[test]
fn non_executable_kinds_never_compared() {
    let dir = tempfile::tempdir().unwrap();
    let work = dir.path().to_path_buf();
    let r = classify_unit(
        &unit("K1FDA", "CLBRY"),
        &OracleSide::default(),
        &CandidateSide::default(),
        &work,
    );
    assert_eq!(
        r.final_classification,
        FinalClassification::NonExecutableLibrary
    );
    assert_eq!(r.comparison.raw_stdout, "not_comparable");
    let r2 = classify_unit(
        &unit("NC109M", "DATA*"),
        &OracleSide::default(),
        &CandidateSide::default(),
        &work,
    );
    assert_eq!(
        r2.final_classification,
        FinalClassification::NonExecutableData
    );
}

#[test]
fn subprogram_bound_to_main_is_library_class() {
    let dir = tempfile::tempdir().unwrap();
    let work = dir.path().to_path_buf();
    let mut u = unit("IX102A", "COBOL");
    u.subprogram = Some("IX102A".into());
    u.main_program = Some("IX101A".into());
    u.is_executable_candidate = false;
    let r = classify_unit(&u, &OracleSide::default(), &CandidateSide::default(), &work);
    assert_eq!(
        r.final_classification,
        FinalClassification::NonExecutableLibrary
    );
    assert_eq!(r.reason_code, "SUBPROGRAM_BOUND_TO_MAIN");
}
