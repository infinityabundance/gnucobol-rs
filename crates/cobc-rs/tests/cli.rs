//! End-to-end integration tests for the `cobc-rs` compatibility driver (prompt Phase 10).
//!
//! These tests exercise the REAL binary (`CARGO_BIN_EXE_cobc-rs`) through its CLI: argument
//! parsing, translation, artifact generation, launcher execution, parallelism, and candidate
//! isolation. They never require the oracle or the container.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_cobc-rs")
}

/// A minimal free-format program that terminates with a known stdout.
const HELLO: &str = r#"IDENTIFICATION DIVISION.
       PROGRAM-ID. P.
       PROCEDURE DIVISION.
           DISPLAY "hello cobc-rs".
           STOP RUN.
"#;

/// A program whose RETURN-CODE should become the exit status.
const RC_PROG: &str = r#"IDENTIFICATION DIVISION.
       PROGRAM-ID. P.
       PROCEDURE DIVISION.
           MOVE 7 TO RETURN-CODE.
           STOP RUN.
"#;

fn run_in(dir: &Path, args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .current_dir(dir)
        .output()
        .expect("cobc-rs must run")
}

fn write(dir: &Path, name: &str, content: &str) {
    fs::write(dir.join(name), content).unwrap();
}

fn out_str(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err_str(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

// -------------------------------------------------------------------------------------------
// argument parsing
// -------------------------------------------------------------------------------------------

#[test]
fn unknown_option_is_rejected_not_ignored() {
    let d = tempfile::tempdir().unwrap();
    let o = run_in(d.path(), &["-x", "--thisoptiondoesntexist", "prog.cob"]);
    assert_ne!(o.status.code(), Some(0), "unknown option must fail closed");
    assert!(
        err_str(&o).contains("unknown option") || err_str(&o).contains("unsupported option"),
        "expected an honest rejection diagnostic, got: {}",
        err_str(&o)
    );
}

#[test]
fn semantic_unsupported_option_is_rejected_honestly() {
    let d = tempfile::tempdir().unwrap();
    write(d.path(), "prog.cob", HELLO);
    let o = run_in(d.path(), &["-x", "-fec", "prog.cob"]);
    assert_ne!(
        o.status.code(),
        Some(0),
        "-fec (exception checking) must be rejected honestly"
    );
    assert!(err_str(&o).contains("unsupported option") || err_str(&o).contains("reject"));
}

#[test]
fn version_and_dumpversion_shapes() {
    let d = tempfile::tempdir().unwrap();
    let o = run_in(d.path(), &["--version"]);
    assert_eq!(o.status.code(), Some(0));
    let s = out_str(&o);
    assert!(
        s.contains("cobc-rs"),
        "--version must identify as cobc-rs: {s}"
    );
    let o = run_in(d.path(), &["--dumpversion"]);
    assert_eq!(o.status.code(), Some(0));
    assert!(
        out_str(&o).trim().starts_with("3.2"),
        "dumpversion = reproduced GnuCOBOL version"
    );
}

#[test]
fn runtime_conf_and_info_modes_succeed() {
    let d = tempfile::tempdir().unwrap();
    for arg in ["--runtime-conf", "--info", "--help"] {
        let o = run_in(d.path(), &[arg]);
        assert_eq!(o.status.code(), Some(0), "{arg} should exit 0");
        assert!(
            !out_str(&o).is_empty() || !err_str(&o).is_empty(),
            "{arg} should print something"
        );
    }
}

#[test]
fn source_reading_stdin_dash() {
    let d = tempfile::tempdir().unwrap();
    use std::io::Write;
    let mut child = Command::new(bin())
        .args(["-x", "-o", "prog", "-"])
        .current_dir(d.path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(HELLO.as_bytes())
        .unwrap();
    let o = child.wait_with_output().unwrap();
    assert_eq!(
        o.status.code(),
        Some(0),
        "stdin source compile failed: {}",
        err_str(&o)
    );
    assert!(
        d.path().join("prog.cobr.json").exists(),
        "stdin compile must produce a manifest"
    );
}

#[test]
fn filename_with_spaces_via_output_and_source_paths() {
    let d = tempfile::tempdir().unwrap();
    let src = d.path().join("my prog.cob");
    fs::write(&src, HELLO).unwrap();
    let o = Command::new(bin())
        .args(["-x", "-o", "out prog"])
        .arg(&src)
        .current_dir(d.path())
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(0), "compile failed: {}", err_str(&o));
    let launcher = d.path().join("out prog");
    assert!(launcher.exists(), "space-containing output path must exist");
    let run = Command::new(&launcher)
        .current_dir(d.path())
        .output()
        .unwrap();
    assert_eq!(run.status.code(), Some(0));
    assert_eq!(out_str(&run).trim(), "hello cobc-rs");
}

// -------------------------------------------------------------------------------------------
// translation
// -------------------------------------------------------------------------------------------

#[test]
fn dialect_translation_std_and_format() {
    let d = tempfile::tempdir().unwrap();
    write(d.path(), "prog.cob", HELLO);
    for args in [
        vec!["-x", "-std=cobol85", "prog.cob"],
        vec!["-x", "-std=mf", "prog.cob"],
        vec!["-x", "-std=ibm", "prog.cob"],
        vec!["-x", "-free", "prog.cob"],
        vec!["-x", "-fixed", "prog.cob"],
    ] {
        let o = run_in(d.path(), &args);
        assert_eq!(
            o.status.code(),
            Some(0),
            "args {args:?} failed: {}",
            err_str(&o)
        );
    }
}

#[test]
fn defines_are_applied_as_preprocessor_symbols() {
    let d = tempfile::tempdir().unwrap();
    write(
        d.path(),
        "prog.cob",
        r#"IDENTIFICATION DIVISION.
       PROGRAM-ID. P.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 X PIC 9(4) VALUE 0.
       PROCEDURE DIVISION.
       >>IF FLAG IS DEFINED
           MOVE 42 TO X.
       >>ELSE
           MOVE 1 TO X.
       >>END-IF
           DISPLAY X.
           STOP RUN.
"#,
    );
    let o = run_in(d.path(), &["-x", "-DFLAG", "-o", "prog", "prog.cob"]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "define compile failed: {}",
        err_str(&o)
    );
    let run = Command::new(d.path().join("prog"))
        .current_dir(d.path())
        .output()
        .unwrap();
    assert_eq!(out_str(&run).trim(), "0042");
}

#[test]
fn copybook_resolution_via_include_path() {
    let d = tempfile::tempdir().unwrap();
    let inc = d.path().join("cpy");
    fs::create_dir(&inc).unwrap();
    fs::write(
        inc.join("MYCOPY.cpy"),
        "       01 COPYVAL PIC X(4) VALUE 'OK'.\n",
    )
    .unwrap();
    write(
        d.path(),
        "prog.cob",
        r#"IDENTIFICATION DIVISION.
       PROGRAM-ID. P.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
           COPY MYCOPY.
       PROCEDURE DIVISION.
           DISPLAY COPYVAL.
           STOP RUN.
"#,
    );
    let o = run_in(d.path(), &["-x", "-I", "cpy", "-o", "prog", "prog.cob"]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "copybook compile failed: {}",
        err_str(&o)
    );
    let run = Command::new(d.path().join("prog"))
        .current_dir(d.path())
        .output()
        .unwrap();
    assert_eq!(
        run.status.code(),
        Some(0),
        "copybook run failed: {}",
        err_str(&run)
    );
    assert_eq!(out_str(&run).trim(), "OK");
}

#[test]
fn syntax_only_runs_the_check_pipeline_without_artifacts() {
    let d = tempfile::tempdir().unwrap();
    write(d.path(), "good.cob", HELLO);
    let o = run_in(d.path(), &["-fsyntax-only", "good.cob"]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "syntax-only on good program failed: {}",
        err_str(&o)
    );
    assert!(
        !d.path().join("good.cobr.json").exists(),
        "syntax-only must not emit artifacts"
    );
    write(
        d.path(),
        "bad.cob",
        "IDENTIFICATION DIVISION.\n       PROGRAM-ID. P.\n       PROCEDURE DIVISION.\n           NOT A STATEMENT.\n",
    );
    let o = run_in(d.path(), &["-fsyntax-only", "bad.cob"]);
    assert_ne!(
        o.status.code(),
        Some(0),
        "syntax-only must fail closed on unsupported syntax"
    );
}

#[test]
fn preprocess_mode_emits_expanded_source() {
    let d = tempfile::tempdir().unwrap();
    write(d.path(), "prog.cob", HELLO);
    let o = run_in(d.path(), &["-E", "prog.cob"]);
    assert_eq!(o.status.code(), Some(0));
    let s = out_str(&o);
    assert!(s.contains("#line 1"), "-E must emit the #line header");
    assert!(s.contains("DISPLAY"), "-E must contain the expanded body");
}

#[test]
fn accepted_noop_flags_are_ignored_with_success() {
    let d = tempfile::tempdir().unwrap();
    write(d.path(), "prog.cob", HELLO);
    let o = run_in(
        d.path(),
        &[
            "--compat=gnucobol-testsuite",
            "-x",
            "-Wall",
            "-Wextra",
            "-w",
            "-debug",
            "-O2",
            "-g",
            "-fdiagnostics-plain-output",
            "-fno-diagnostics-show-option",
            "-o",
            "prog",
            "prog.cob",
        ],
    );
    assert_eq!(
        o.status.code(),
        Some(0),
        "benign suite flags must be accepted: {}",
        err_str(&o)
    );
    let run = Command::new(d.path().join("prog"))
        .current_dir(d.path())
        .output()
        .unwrap();
    assert_eq!(out_str(&run).trim(), "hello cobc-rs");
}

#[test]
fn unsupported_native_codegen_modes_are_rejected_honestly() {
    let d = tempfile::tempdir().unwrap();
    write(d.path(), "prog.cob", HELLO);
    for flag in ["-c", "-S", "-C", "-Xref", "-t-", "-b", "-r"] {
        let o = run_in(d.path(), &[flag, "prog.cob"]);
        assert_ne!(
            o.status.code(),
            Some(0),
            "{flag} must be rejected (no native model)"
        );
        assert!(
            err_str(&o).contains("unsupported"),
            "{flag} diagnostic: {}",
            err_str(&o)
        );
    }
}

// -------------------------------------------------------------------------------------------
// artifact generation + launcher execution
// -------------------------------------------------------------------------------------------

#[test]
fn launcher_runs_program_and_propagates_exit_status() {
    let d = tempfile::tempdir().unwrap();
    write(d.path(), "prog.cob", HELLO);
    let o = run_in(d.path(), &["-x", "-o", "prog", "prog.cob"]);
    assert_eq!(o.status.code(), Some(0));
    // artifacts: launcher (symlink), manifest, expanded source
    assert!(
        d.path().join("prog").is_symlink(),
        "launcher must be a symlink to cobc-rs"
    );
    let run = Command::new(d.path().join("prog"))
        .current_dir(d.path())
        .output()
        .unwrap();
    assert_eq!(run.status.code(), Some(0));
    assert_eq!(out_str(&run).trim(), "hello cobc-rs");

    // RETURN-CODE -> exit status
    write(d.path(), "rc.cob", RC_PROG);
    let o = run_in(d.path(), &["-x", "-o", "rc", "rc.cob"]);
    assert_eq!(o.status.code(), Some(0));
    let run = Command::new(d.path().join("rc"))
        .current_dir(d.path())
        .output()
        .unwrap();
    assert_eq!(
        run.status.code(),
        Some(7),
        "RETURN-CODE 7 must become exit status 7"
    );
}

#[test]
fn manifest_self_hash_refuses_tampering() {
    let d = tempfile::tempdir().unwrap();
    write(d.path(), "prog.cob", HELLO);
    let o = run_in(d.path(), &["-x", "-o", "prog", "prog.cob"]);
    assert_eq!(o.status.code(), Some(0));
    let mp = d.path().join("prog.cobr.json");
    let text = fs::read_to_string(&mp).unwrap();
    fs::write(&mp, text.replace("\"schema\"", "\"schemax\"")).unwrap();
    let run = Command::new(d.path().join("prog"))
        .current_dir(d.path())
        .output()
        .unwrap();
    assert_eq!(
        run.status.code(),
        Some(2),
        "tampered manifest must refuse to run"
    );
    let e = err_str(&run);
    assert!(
        e.contains("integrity") || e.contains("tampered"),
        "diagnostic: {e}"
    );
}

#[test]
fn manifest_hash_is_self_consistent_and_stable() {
    let d = tempfile::tempdir().unwrap();
    write(d.path(), "prog.cob", HELLO);
    let o = run_in(d.path(), &["-x", "-o", "prog", "prog.cob"]);
    assert_eq!(o.status.code(), Some(0));
    let m1 = fs::read_to_string(d.path().join("prog.cobr.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&m1).unwrap();
    let sha = v["manifest_sha256"].as_str().unwrap().to_string();
    let mut body = v.clone();
    body.as_object_mut().unwrap().remove("manifest_sha256");
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(serde_json::to_vec(&body).unwrap());
    assert_eq!(
        format!("{:x}", h.finalize()),
        sha,
        "manifest self-hash must be self-consistent"
    );
    // a fresh compile of the same source produces an identical manifest (deterministic artifacts)
    let _ = fs::remove_file(d.path().join("prog.cobr.json"));
    let _ = fs::remove_file(d.path().join("prog"));
    let _ = fs::remove_file(d.path().join("prog.cobr-src"));
    let o = run_in(d.path(), &["-x", "-o", "prog", "prog.cob"]);
    assert_eq!(o.status.code(), Some(0));
    let m2 = fs::read_to_string(d.path().join("prog.cobr.json")).unwrap();
    assert_eq!(
        m1, m2,
        "rebuilding the same source must produce a byte-identical manifest"
    );
}

#[test]
fn launcher_tolerates_program_arguments() {
    let d = tempfile::tempdir().unwrap();
    write(d.path(), "prog.cob", HELLO);
    let o = run_in(d.path(), &["-x", "-o", "prog", "prog.cob"]);
    assert_eq!(o.status.code(), Some(0));
    let run = Command::new(d.path().join("prog"))
        .arg("--some-flag")
        .arg("with space")
        .current_dir(d.path())
        .output()
        .unwrap();
    assert_eq!(
        run.status.code(),
        Some(0),
        "launcher must tolerate arbitrary program args"
    );
    assert_eq!(out_str(&run).trim(), "hello cobc-rs");
}

// -------------------------------------------------------------------------------------------
// parallelism
// -------------------------------------------------------------------------------------------

#[test]
fn one_hundred_concurrent_invocations_colliding_basenames() {
    let root = tempfile::tempdir().unwrap();
    let mut pids = Vec::new();
    for i in 0..100 {
        let dir = root.path().join(format!("d{i:03}"));
        fs::create_dir(&dir).unwrap();
        let src = format!(
            "IDENTIFICATION DIVISION.\n       PROGRAM-ID. P.\n       PROCEDURE DIVISION.\n           DISPLAY \"job {i:03}\".\n           STOP RUN.\n"
        );
        fs::write(dir.join("prog.cob"), &src).unwrap();
        let c = Command::new(bin())
            .args(["-x", "-o", "prog", "prog.cob"])
            .current_dir(&dir)
            .spawn()
            .unwrap();
        pids.push((i, c, dir));
    }
    for (i, mut c, dir) in pids {
        let st = c.wait().unwrap();
        assert!(st.success(), "concurrent compile {i} failed");
        let run = Command::new(dir.join("prog"))
            .current_dir(&dir)
            .output()
            .unwrap();
        assert_eq!(
            out_str(&run).trim(),
            format!("job {i:03}"),
            "cross-test leakage in dir {i}"
        );
    }
}

// -------------------------------------------------------------------------------------------
// candidate isolation
// -------------------------------------------------------------------------------------------

#[test]
fn driver_does_not_require_oracle_on_path() {
    // With a PATH that contains no cobc/cobcrun at all, the full compile+run cycle must still work
    // (the candidate never delegates).
    let d = tempfile::tempdir().unwrap();
    write(d.path(), "prog.cob", HELLO);
    let o = Command::new(bin())
        .args(["-x", "-o", "prog", "prog.cob"])
        .current_dir(d.path())
        .env("PATH", "/nonexistent-dir-only")
        .output()
        .unwrap();
    assert_eq!(
        o.status.code(),
        Some(0),
        "compile must not need cobc on PATH: {}",
        err_str(&o)
    );
    let run = Command::new(d.path().join("prog"))
        .current_dir(d.path())
        .env("PATH", "/nonexistent-dir-only")
        .output()
        .unwrap();
    assert_eq!(run.status.code(), Some(0));
    assert_eq!(out_str(&run).trim(), "hello cobc-rs");
}

#[test]
fn candidate_binary_has_no_libcob_dynamic_dependency() {
    // The mechanical no-delegation property at the binary level: the shipped cobc-rs must not link
    // libcob (ldd output must contain no libcob/gnucobol hit). This mirrors the container-side check.
    let ldd = Command::new("ldd").arg(bin()).output();
    if let Ok(o) = ldd {
        let text = String::from_utf8_lossy(&o.stdout);
        let hits: Vec<&str> = text
            .lines()
            .filter(|l| l.to_ascii_lowercase().contains("cob") && !l.contains("cobc-rs"))
            .collect();
        assert!(
            hits.is_empty(),
            "cobc-rs must not dynamically link libcob: {hits:?}"
        );
    }
    // readelf -d equivalent lives in the container-side no-delegation record (no-delegation.json).
}
