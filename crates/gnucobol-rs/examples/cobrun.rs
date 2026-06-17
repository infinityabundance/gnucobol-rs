//! `cobrun [-std=NAME] <file.cob>` -- the CLEAN-ROOM COBOL front-end: parse + EXECUTE a COBOL program on
//! the ported libcob runtime (no `cobc`, no `libcob` linked) and write its stdout. Exits 2 with the
//! reason on anything outside the sealed subset (`gnucobol_rs::frontend`). This is the turn-key form
//! of the end-to-end execution proof: feed it source, it runs it. `-std=` selects a dialect (e.g. the
//! `defaultbyte` fill of uninitialized storage), mirroring `cobc -std=`.

use gnucobol_rs::dialect::Dialect;
use std::io::Write;

fn main() {
    // cobrun [-std=NAME] [-fixed|-free] <file.cob> -- dialect selector + source format (default free).
    let mut dialect = Dialect::DEFAULT;
    let mut fixed = false;
    let mut path: Option<String> = None;
    for arg in std::env::args().skip(1) {
        if let Some(name) = arg.strip_prefix("-std=").or_else(|| arg.strip_prefix("--std=")) {
            dialect = Dialect::from_std(name);
        } else if arg == "-fixed" || arg == "--fixed" {
            fixed = true;
        } else if arg == "-free" || arg == "--free" {
            fixed = false;
        } else {
            path = Some(arg);
        }
    }
    let path = match path {
        Some(p) => p,
        None => {
            eprintln!("usage: cobrun [-std=NAME] [-fixed|-free] <file.cob>");
            std::process::exit(2);
        }
    };
    let raw = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("cobrun: cannot read {path}: {e}");
            std::process::exit(2);
        }
    };
    let src = if fixed { gnucobol_rs::frontend::fixed_to_free(&raw) } else { raw };
    match gnucobol_rs::frontend::run_program_dialect_with_rc(&src, dialect) {
        Ok((out, rc)) => {
            std::io::stdout().write_all(&out).unwrap();
            // RETURN-CODE flows to the process exit status (MOVE n TO RETURN-CODE / STOP RUN n).
            std::process::exit(rc);
        }
        Err(e) => {
            eprintln!("cobrun: {e}");
            std::process::exit(2);
        }
    }
}
