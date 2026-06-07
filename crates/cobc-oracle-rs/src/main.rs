//! `cobc-oracle` — small CLI around [`cobc_oracle_rs`]: probe the oracle, run a fixture, or write
//! a receipt. Run with the built GnuCOBOL on `PATH`/`LD_LIBRARY_PATH` and `LC_ALL=C.UTF-8`.

use std::path::Path;
use std::process::ExitCode;

fn usage() -> ExitCode {
    eprintln!(
        "usage:\n  \
         cobc-oracle oracle-smoke\n  \
         cobc-oracle run-fixture <file.cob>\n  \
         cobc-oracle write-receipt --fixture <file.cob> --out <receipt.json>"
    );
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("oracle-smoke") => match cobc_oracle_rs::probe_oracle() {
            cobc_oracle_rs::OracleAvailability::Available { cobc_version } => {
                println!("oracle available: {cobc_version}");
                ExitCode::SUCCESS
            }
            other => {
                println!("oracle not available: {other:?}");
                ExitCode::from(1)
            }
        },
        Some("run-fixture") => {
            let Some(path) = args.get(1) else {
                return usage();
            };
            match cobc_oracle_rs::run_executable_fixture(Path::new(path)) {
                Ok(r) => {
                    println!("{}", r.to_canonical_json());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("error reading fixture: {e}");
                    ExitCode::from(1)
                }
            }
        }
        Some("write-receipt") => {
            let mut fixture = None;
            let mut out = None;
            let mut it = args[1..].iter();
            while let Some(a) = it.next() {
                match a.as_str() {
                    "--fixture" => fixture = it.next().cloned(),
                    "--out" => out = it.next().cloned(),
                    _ => return usage(),
                }
            }
            let (Some(fixture), Some(out)) = (fixture, out) else {
                return usage();
            };
            match cobc_oracle_rs::run_executable_fixture(Path::new(&fixture)) {
                Ok(r) => match std::fs::write(&out, r.to_canonical_json()) {
                    Ok(()) => {
                        println!("wrote {out}");
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("error writing receipt: {e}");
                        ExitCode::from(1)
                    }
                },
                Err(e) => {
                    eprintln!("error: {e}");
                    ExitCode::from(1)
                }
            }
        }
        _ => usage(),
    }
}
