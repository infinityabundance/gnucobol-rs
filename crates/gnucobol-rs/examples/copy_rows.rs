//! Rust mirror of the COPY oracle (`cobc -P`): reads a COBOL program from stdin, expands its `COPY`
//! statements using copybooks from a directory (argv[1]), and prints the expanded source. The copy
//! sweep tokenizes this and `cobc -P`'s output into text-words and compares them. Not API.

use gnucobol_rs::copybook::{expand, CopyResolver};
use std::io::{self, Read};

struct DirResolver {
    dir: String,
}

impl CopyResolver for DirResolver {
    fn resolve(&self, name: &str) -> Option<String> {
        // Mirror cobc's copybook search: try the name and common extensions, case as given and
        // lowercased (the oracle resolves COPY CUSTREC -> CUSTREC.cpy under -I dir).
        for base in [name.to_string(), name.to_ascii_lowercase()] {
            for ext in ["", ".cpy", ".CPY", ".cbl", ".cob"] {
                let path = format!("{}/{}{}", self.dir, base, ext);
                if let Ok(s) = std::fs::read_to_string(&path) {
                    return Some(s);
                }
            }
        }
        None
    }
}

fn main() {
    let dir = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let mut src = String::new();
    let _ = io::stdin().read_to_string(&mut src);
    let resolver = DirResolver { dir };
    match expand(&src, &resolver) {
        Ok(e) => println!("{}", e.text()),
        Err(e) => eprintln!("COPY_ERROR {e}"),
    }
}
