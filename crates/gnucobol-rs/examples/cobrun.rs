//! `cobrun [-std=NAME] <file.cob>` -- the CLEAN-ROOM COBOL front-end: parse + EXECUTE a COBOL program on
//! the ported libcob runtime (no `cobc`, no `libcob` linked) and write its stdout. Exits 2 with the
//! reason on anything outside the sealed subset (`gnucobol_rs::frontend`). This is the turn-key form
//! of the end-to-end execution proof: feed it source, it runs it. `-std=` selects a dialect (e.g. the
//! `defaultbyte` fill of uninitialized storage), mirroring `cobc -std=`.

use gnucobol_rs::dialect::Dialect;
use std::io::Write;

fn main() {
    // cobrun [-std=NAME] <file.cob> -- an optional dialect selector (default: the default dialect).
    let mut dialect = Dialect::DEFAULT;
    let mut path: Option<String> = None;
    for arg in std::env::args().skip(1) {
        if let Some(name) = arg.strip_prefix("-std=").or_else(|| arg.strip_prefix("--std=")) {
            dialect = Dialect::from_std(name);
        } else {
            path = Some(arg);
        }
    }
    let path = match path {
        Some(p) => p,
        None => {
            eprintln!("usage: cobrun [-std=NAME] <file.cob>");
            std::process::exit(2);
        }
    };
    let src = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("cobrun: cannot read {path}: {e}");
            std::process::exit(2);
        }
    };
    match gnucobol_rs::frontend::run_program_dialect(&src, dialect) {
        Ok(out) => {
            std::io::stdout().write_all(&out).unwrap();
        }
        Err(e) => {
            eprintln!("cobrun: {e}");
            std::process::exit(2);
        }
    }
}
