//! Rust evaluator for the cobgetopt.c differential. Reads the same scenarios `gen_getopt` emits (and the
//! libcob oracle `getopt_harness.c` consumes), drives [`CobGetopt`], and prints one token per call —
//! `LABEL  r:optarg:optind:optopt ...` — identical to the C harness so the two streams diff byte-for-byte.
//! `opterr` is forced to 0 (compare parse semantics, not stderr text).

use gnucobol_rs::cobgetopt::{CobGetopt, OptionDef};
use std::io::{self, BufRead, Write};

fn main() {
    let stdin = io::stdin();
    let mut out = io::BufWriter::new(io::stdout());
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut fields: Vec<&str> = line.splitn(5, '\t').collect();
        while fields.len() < 5 {
            fields.push("");
        }
        let label = fields[0];
        let long_only: i32 = fields[1].parse().unwrap_or(0);
        let optstring = fields[2].as_bytes();
        let longspec = fields[3];
        let argspec = fields[4];

        // argv: "prog" + space-separated args
        let mut argv: Vec<Vec<u8>> = vec![b"prog".to_vec()];
        for a in argspec.split(' ').filter(|s| !s.is_empty()) {
            argv.push(a.as_bytes().to_vec());
        }

        // longopts
        let mut longopts: Vec<OptionDef> = Vec::new();
        if longspec != "-" {
            for spec in longspec.split('|') {
                let parts: Vec<&str> = spec.split(':').collect();
                if parts.len() == 3 {
                    longopts.push(OptionDef {
                        name: parts[0].as_bytes().to_vec(),
                        has_arg: parts[1].parse().unwrap_or(0),
                        flag: None,
                        val: parts[2].parse().unwrap_or(0),
                    });
                }
            }
        }

        let mut g = CobGetopt::new(argv, optstring, longopts, long_only);
        g.opterr = 0;

        write!(out, "{label}").unwrap();
        let mut guard = 0;
        loop {
            let r = g.cob_getopt_long_long();
            let oa = match &g.optarg {
                Some(v) => String::from_utf8_lossy(v).into_owned(),
                None => "-".to_string(),
            };
            write!(out, " {r}:{oa}:{}:{}", g.optind, g.optopt).unwrap();
            guard += 1;
            if r == -1 || guard > 50 {
                break;
            }
        }
        writeln!(out).unwrap();
    }
    out.flush().unwrap();
}
