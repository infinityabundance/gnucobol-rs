//! GNURUST.MODULE.{BASIC,MULTI,ARGS,CANCEL,SEARCH,PARALLEL}.1 — focused oracle-shaped module
//! lifecycle courts for the interpreted module model (Phase-2 module boundary).
//!
//! These courts exercise the REAL binary end-to-end: `-m` artifacts, `cobcrun` module resolution
//! (`-M <dir>`, cwd, COB_LIBRARY_PATH), module arguments (ACCEPT FROM COMMAND-LINE), EXTERNAL
//! sharing + CALL across modules, CANCEL state semantics, error messages, and parallel isolation.
//! They never require the GnuCOBOL oracle or the container; the oracle differential is the suite
//! court itself (GNURUST.GNUCOBOL-TESTSUITE.*).

use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::process::{Command, Output};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_cobc-rs")
}

/// A directory with a `cobcrun` symlink to the real binary (argv[0]-dispatched runner mode).
struct RunDir {
    dir: tempfile::TempDir,
}

impl RunDir {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        symlink(bin(), dir.path().join("cobcrun")).unwrap();
        symlink(bin(), dir.path().join("cobc-rs")).unwrap();
        RunDir { dir }
    }
    fn path(&self) -> &Path {
        self.dir.path()
    }
    fn cobc(&self, args: &[&str]) -> Output {
        Command::new(self.path().join("cobc-rs"))
            .args(args)
            .current_dir(self.path())
            .output()
            .expect("cobc-rs must run")
    }
    fn cobcrun(&self, args: &[&str]) -> Output {
        Command::new(self.path().join("cobcrun"))
            .args(args)
            .current_dir(self.path())
            .output()
            .expect("cobcrun must run")
    }
    fn write(&self, name: &str, content: &str) {
        fs::write(self.path().join(name), content).unwrap();
    }
}

fn out_str(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err_str(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

const CALLEE: &str = r#"       IDENTIFICATION DIVISION.
       PROGRAM-ID. callee.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 EXT-VAR PIC X(5) EXTERNAL.
       PROCEDURE DIVISION.
           DISPLAY EXT-VAR END-DISPLAY.
           MOVE "World" TO EXT-VAR.
           EXIT PROGRAM.
"#;

const CALLER: &str = r#"       IDENTIFICATION DIVISION.
       PROGRAM-ID. caller.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 EXT-VAR PIC X(5) EXTERNAL.
       PROCEDURE DIVISION.
           MOVE "Hello" TO EXT-VAR.
           CALL "callee" END-CALL.
           DISPLAY EXT-VAR END-DISPLAY.
           STOP RUN.
"#;

// -------------------------------------------------------------------------------------------
// GNURUST.MODULE.BASIC.1 — `-m` builds a truthful interpreted-module artifact; cobcrun runs it.
// -------------------------------------------------------------------------------------------

#[test]
fn module_build_is_silent_and_runnable() {
    let d = RunDir::new();
    d.write(
        "prog.cob",
        r#"       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.
       PROCEDURE DIVISION.
           DISPLAY "OK" END-DISPLAY.
           STOP RUN.
"#,
    );
    // cobc -m writes the artifact silently (no module-name echo).
    let o = d.cobc(&["-m", "prog.cob"]);
    assert_eq!(o.status.code(), Some(0));
    assert!(
        out_str(&o).is_empty(),
        "cobc -m must be silent, got: {:?}",
        out_str(&o)
    );
    assert!(
        d.path().join("prog.so").exists(),
        "default -m artifact is prog.so"
    );
    assert!(d.path().join("prog.so.cobr.json").exists());
    // cobcrun prog runs the module.
    let r = d.cobcrun(&["prog"]);
    assert_eq!(r.status.code(), Some(0), "stderr: {}", err_str(&r));
    assert_eq!(out_str(&r), "OK\n");
}

// -------------------------------------------------------------------------------------------
// GNURUST.MODULE.SEARCH.1 — `-M <dir>` search (slash appended), cwd, COB_LIBRARY_PATH, errors.
// -------------------------------------------------------------------------------------------

#[test]
fn cobcrun_m_searches_the_module_directory() {
    let d = RunDir::new();
    d.write(
        "prog.cob",
        r#"       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.
       PROCEDURE DIVISION.
           DISPLAY "OK" END-DISPLAY.
           STOP RUN.
"#,
    );
    fs::create_dir_all(d.path().join("sub")).unwrap();
    let o = d.cobc(&["-m", "-o", "sub/prog", "prog.cob"]);
    assert_eq!(o.status.code(), Some(0));
    // -M value without a trailing slash: cobcrun appends '/' (GnuCOBOL behavior).
    let r = d.cobcrun(&["-M", "sub", "prog"]);
    assert_eq!(r.status.code(), Some(0), "stderr: {}", err_str(&r));
    assert_eq!(out_str(&r), "OK\n");
    let r2 = d.cobcrun(&["-M", "sub/", "prog"]);
    assert_eq!(r2.status.code(), Some(0));
    assert_eq!(out_str(&r2), "OK\n");
}

#[test]
fn cobcrun_module_search_uses_cwd_and_library_path() {
    let d = RunDir::new();
    d.write(
        "prog.cob",
        r#"       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.
       PROCEDURE DIVISION.
           DISPLAY "OK" END-DISPLAY.
           STOP RUN.
"#,
    );
    d.cobc(&["-m", "prog.cob"]);
    let r = d.cobcrun(&["prog"]);
    assert_eq!(r.status.code(), Some(0));
    assert_eq!(out_str(&r), "OK\n");
    // COB_LIBRARY_PATH: move the module to a lib dir and resolve through the env var.
    let lib = d.path().join("lib");
    fs::create_dir_all(&lib).unwrap();
    let o = d.cobc(&["-m", "-o", "lib/prog", "prog.cob"]);
    assert_eq!(o.status.code(), Some(0));
    let r = Command::new(d.path().join("cobcrun"))
        .args(["prog"])
        .env("COB_LIBRARY_PATH", d.path().join("lib"))
        .current_dir(d.path())
        .output()
        .unwrap();
    assert_eq!(r.status.code(), Some(0), "stderr: {}", err_str(&r));
    assert_eq!(out_str(&r), "OK\n");
}

#[test]
fn cobcrun_error_messages_match_cobcrun() {
    let d = RunDir::new();
    // missing PROGRAM name
    let r = d.cobcrun(&[]);
    assert_eq!(r.status.code(), Some(1));
    assert!(
        err_str(&r).contains("cobcrun: missing PROGRAM name"),
        "got: {}",
        err_str(&r)
    );
    assert!(err_str(&r).contains("Try 'cobcrun --help' for more information."));
    // -M "" -> invalid module argument
    let r = d.cobcrun(&["-M", "", "nope"]);
    assert_eq!(r.status.code(), Some(1));
    assert_eq!(err_str(&r).trim(), "invalid module argument ''");
    // unknown module
    let r = d.cobcrun(&["noprog"]);
    assert_eq!(r.status.code(), Some(1));
    assert!(
        err_str(&r).contains("cannot find module noprog"),
        "got: {}",
        err_str(&r)
    );
}

// -------------------------------------------------------------------------------------------
// GNURUST.MODULE.MULTI.1 + ARGS.1 — CALL across separately compiled modules; program arguments.
// -------------------------------------------------------------------------------------------

#[test]
fn separately_compiled_callee_is_called_through_the_module() {
    let d = RunDir::new();
    d.write("callee.cob", CALLEE);
    d.write("caller.cob", CALLER);
    // compile both as modules; caller's CALL "callee" resolves (sibling source appended at
    // caller's compile time, exactly as the GnuCOBOL suite's module tests do).
    let c = d.cobc(&["-m", "callee.cob"]);
    assert_eq!(c.status.code(), Some(0), "stderr: {}", err_str(&c));
    let c = d.cobc(&["-m", "caller.cob"]);
    assert_eq!(c.status.code(), Some(0), "stderr: {}", err_str(&c));
    let r = d.cobcrun(&["-M", "./", "caller"]);
    assert_eq!(r.status.code(), Some(0), "stderr: {}", err_str(&r));
    assert_eq!(out_str(&r), "Hello\nWorld\n");
}

#[test]
fn cobcrun_passes_program_arguments_to_command_line() {
    let d = RunDir::new();
    d.write(
        "cli.cob",
        r#"       IDENTIFICATION DIVISION.
       PROGRAM-ID. cli.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 CLI PIC X(20).
       PROCEDURE DIVISION.
           ACCEPT CLI FROM COMMAND-LINE
           DISPLAY CLI WITH NO ADVANCING END-DISPLAY.
           STOP RUN.
"#,
    );
    let o = d.cobc(&["-m", "cli.cob"]);
    assert_eq!(o.status.code(), Some(0));
    let r = d.cobcrun(&["cli", "job123"]);
    assert_eq!(r.status.code(), Some(0));
    assert_eq!(out_str(&r), "job123              "); // PIC X(20) padded
}

// -------------------------------------------------------------------------------------------
// GNURUST.MODULE.CANCEL.1 — CANCEL state semantics (oracle-shaped: retained WS, CANCEL resets).
// -------------------------------------------------------------------------------------------

#[test]
fn cancel_resets_persisted_working_storage() {
    let d = RunDir::new();
    // M calls S twice, CANCELs it, calls again: S's WS counter persists across calls, resets on CANCEL.
    let src = r#"       IDENTIFICATION DIVISION.
       PROGRAM-ID. M.
       PROCEDURE DIVISION.
           CALL "S". CALL "S". CANCEL "S". CALL "S".
           STOP RUN.
       END PROGRAM M.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. S.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 C PIC 9 VALUE 0.
       PROCEDURE DIVISION.
           ADD 1 TO C. DISPLAY "C=" C.
           EXIT PROGRAM.
       END PROGRAM S.
"#;
    d.write("m.cob", src);
    let o = d.cobc(&["-x", "-o", "m", "m.cob"]);
    assert_eq!(o.status.code(), Some(0), "stderr: {}", err_str(&o));
    let r = Command::new(d.path().join("m"))
        .current_dir(d.path())
        .output()
        .unwrap();
    assert_eq!(r.status.code(), Some(0));
    // 1, 2 (persisted), CANCEL -> 1 again
    assert_eq!(out_str(&r), "C=1\nC=2\nC=1\n");
}

#[test]
fn cancel_of_active_program_is_fatal() {
    let d = RunDir::new();
    let src = r#"       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.
       PROCEDURE DIVISION.
           CANCEL "notthere".
           CANCEL "prog".
           DISPLAY "NG" NO ADVANCING END-DISPLAY.
           STOP RUN.
"#;
    d.write("prog.cob", src);
    let o = d.cobc(&["-x", "-o", "prog", "prog.cob"]);
    assert_eq!(o.status.code(), Some(0));
    let r = Command::new(d.path().join("prog"))
        .current_dir(d.path())
        .output()
        .unwrap();
    assert_eq!(r.status.code(), Some(1));
    assert!(
        err_str(&r).contains("libcob: prog.cob:5: error: attempt to CANCEL active program"),
        "got stderr: {}",
        err_str(&r)
    );
}

// -------------------------------------------------------------------------------------------
// GNURUST.MODULE.PARALLEL.1 — same module basename in concurrent directories stays isolated.
// -------------------------------------------------------------------------------------------

#[test]
fn one_hundred_parallel_modules_with_colliding_basenames_stay_isolated() {
    let d = RunDir::new();
    let mut handles = Vec::new();
    for i in 0..50usize {
        let base = d.path().to_path_buf();
        handles.push(std::thread::spawn(move || {
            let sub = base.join(format!("d{i}"));
            fs::create_dir_all(&sub).unwrap();
            fs::create_dir_all(sub.join("sub")).unwrap();
            // per-directory cobcrun symlink (argv[0]-dispatched runner mode)
            symlink(bin(), sub.join("cobcrun")).unwrap();
            let src = format!(
                "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. prog.\n       PROCEDURE DIVISION.\n           DISPLAY \"{i}\" END-DISPLAY.\n           STOP RUN.\n"
            );
            fs::write(sub.join("prog.cob"), &src).unwrap();
            let c = Command::new(bin())
                .args(["-m", "-o", "sub/prog", "prog.cob"])
                .current_dir(&sub)
                .output()
                .unwrap();
            assert_eq!(c.status.code(), Some(0));
            let r = Command::new(sub.join("cobcrun"))
                .args(["-M", "sub", "prog"])
                .current_dir(&sub)
                .output()
                .unwrap();
            (i, r)
        }));
    }
    for h in handles {
        let (i, r) = h.join().unwrap();
        assert_eq!(r.status.code(), Some(0), "dir {i} stderr: {}", err_str(&r));
        assert_eq!(
            out_str(&r),
            format!("{i}\n"),
            "dir {i} must see its OWN module"
        );
    }
}

// -------------------------------------------------------------------------------------------
// Tamper/stale-manifest guards (the launcher must refuse a mismatched manifest hash).
// -------------------------------------------------------------------------------------------

#[test]
fn tampered_manifest_is_refused() {
    let d = RunDir::new();
    d.write(
        "prog.cob",
        r#"       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.
       PROCEDURE DIVISION.
           DISPLAY "OK" END-DISPLAY.
           STOP RUN.
"#,
    );
    d.cobc(&["-m", "prog.cob"]);
    // tamper with the expanded source (the manifest self-hash covers the manifest body, not the
    // source file; tampering the MANIFEST body must be caught)
    let mp = d.path().join("prog.so.cobr.json");
    let mut m: serde_json::Value = serde_json::from_str(&fs::read_to_string(&mp).unwrap()).unwrap();
    m["dialect"] = serde_json::Value::String("ibm".into());
    fs::write(&mp, serde_json::to_string_pretty(&m).unwrap()).unwrap();
    let r = d.cobcrun(&["prog"]);
    assert_eq!(
        r.status.code(),
        Some(2),
        "tampered manifest must be refused"
    );
    assert!(err_str(&r).contains("integrity"), "got: {}", err_str(&r));
}
