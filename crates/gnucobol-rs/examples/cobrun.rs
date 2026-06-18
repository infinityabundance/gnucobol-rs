//! `cobrun [-std=NAME] <file.cob>` -- the CLEAN-ROOM COBOL front-end: parse + EXECUTE a COBOL program on
//! the ported libcob runtime (no `cobc`, no `libcob` linked) and write its stdout. Exits 2 with the
//! reason on anything outside the sealed subset (`gnucobol_rs::frontend`). This is the turn-key form
//! of the end-to-end execution proof: feed it source, it runs it. `-std=` selects a dialect (e.g. the
//! `defaultbyte` fill of uninitialized storage), mirroring `cobc -std=`.

use gnucobol_rs::dialect::Dialect;
use std::io::Write;

/// The GnuCOBOL version this front-end targets (the ported `libcob` version constants), e.g. `3.2.0`.
fn target_version() -> String {
    use gnucobol_rs::common::{LIBCOB_VERSION, LIBCOB_VERSION_MINOR, LIBCOB_VERSION_PATCHLEVEL};
    format!("{LIBCOB_VERSION}.{LIBCOB_VERSION_MINOR}.{LIBCOB_VERSION_PATCHLEVEL}")
}

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
        } else if arg == "-x" || arg == "-j" || arg == "--job" {
            // cobc-compatibility: `cobc -x`/`-j` build-and-run an executable; for the interpreter, running
            // the source IS that step, so these are accepted as no-ops (cobrun <file> == cobc -x <file>).
        } else if arg.starts_with('-') && arg != "-" {
            // an unknown leading-dash token is a flag, not the source file (avoid mis-reading it as <file>).
            eprintln!("cobrun: unknown option `{arg}` (try --help)");
            std::process::exit(2);
        } else if arg == "-dumpversion" || arg == "--dumpversion" {
            // the targeted GnuCOBOL version, byte-identical to `cobc -dumpversion` / `cobcrun -dumpversion`.
            println!("{}", target_version());
            return;
        } else if arg == "-V" || arg == "--version" {
            // Identify honestly as the gnucobol-rs port (NOT a masquerade of GnuCOBOL), reporting the
            // GnuCOBOL version it reproduces, in the GnuCOBOL --version block format.
            print!(
                "cobrun (gnucobol-rs, reproducing GnuCOBOL) {ver}\n\
                 A clean-room native-Rust reimplementation of the GnuCOBOL {ver} runtime + a COBOL\n\
                 interpreter front-end, proven byte-identical to the admitted cobc oracle.\n\
                 License LGPL-3.0-or-later. Not GnuCOBOL; not affiliated with the GNU project.\n",
                ver = target_version(),
            );
            return;
        } else if arg == "-h" || arg == "--help" {
            print!(
                "cobrun -- run a COBOL program on the native-Rust GnuCOBOL runtime (no cobc/libcob linked)\n\n\
                 Usage: cobrun [options] <file.cob>\n\n\
                 Options:\n  \
                 -free | -fixed              source format (default: free)\n  \
                 -std=<name>                 dialect (default | ibm | mvs | mf | cobol2014 | ...)\n  \
                 -V, --version               version information and exit\n  \
                 -dumpversion                the targeted GnuCOBOL version (e.g. {}) and exit\n  \
                 -h, --help                  this help and exit\n\n\
                 The program's RETURN-CODE becomes the process exit status.\n",
                target_version(),
            );
            return;
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
    // Record the source path for FUNCTION MODULE-SOURCE (cobc embeds the source name it was given).
    gnucobol_rs::frontend::set_source_file(&path);
    // DISPLAY ... UPON PRINTER redirect: when COB_DISPLAY_PRINT_FILE is set, libcob diverts UPON PRINTER
    // to that file (appending) instead of stdout. cobrun is the host that owns this env+file boundary; the
    // interpreter only separates the printer stream. (COB_DISPLAY_PRINT_PIPE -- a spawned pipe -- is a
    // boundary cobrun does not implement.)
    let print_file = std::env::var_os("COB_DISPLAY_PRINT_FILE");
    let redirect = print_file.is_some();
    match gnucobol_rs::frontend::run_program_redirected(&src, dialect, redirect) {
        Ok((out, printer, rc)) => {
            std::io::stdout().write_all(&out).unwrap();
            if let Some(path) = print_file {
                if !printer.is_empty() {
                    use std::io::Write as _;
                    if let Ok(mut f) =
                        std::fs::OpenOptions::new().create(true).append(true).open(&path)
                    {
                        let _ = f.write_all(&printer);
                    }
                }
            }
            // RETURN-CODE flows to the process exit status (MOVE n TO RETURN-CODE / STOP RUN n).
            std::process::exit(rc);
        }
        Err(e) => {
            eprintln!("cobrun: {e}");
            std::process::exit(2);
        }
    }
}
