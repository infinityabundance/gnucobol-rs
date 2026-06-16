//! `cobrun <file.cob>` -- the CLEAN-ROOM COBOL front-end: parse + EXECUTE a COBOL program on the
//! ported libcob runtime (no `cobc`, no `libcob` linked) and write its stdout. Exits 2 with the
//! reason on anything outside the sealed subset (`gnucobol_rs::frontend`). This is the turn-key form
//! of the end-to-end execution proof: feed it source, it runs it.

use std::io::Write;

fn main() {
    let path = match std::env::args().nth(1) {
        Some(p) => p,
        None => {
            eprintln!("usage: cobrun <file.cob>");
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
    match gnucobol_rs::frontend::run_program(&src) {
        Ok(out) => {
            std::io::stdout().write_all(&out).unwrap();
        }
        Err(e) => {
            eprintln!("cobrun: {e}");
            std::process::exit(2);
        }
    }
}
