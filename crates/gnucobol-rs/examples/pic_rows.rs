//! Rust mirror of the PIC oracle (`lab/oracle/pic_harness.sh`): reads the same case rows and emits
//! the Rust-computed field attributes, so the PIC sweep can compare them to the compiler's.
//!
//! Input line: `label<TAB>pic<TAB>usage<TAB>sign`  (usage = ""|DISPLAY|COMP-3; sign = a SIGN clause)
//! Output line: `label<TAB>type<TAB>digits<TAB>scale<TAB>flags<TAB>size`  (type/flags as 0x hex)
//! Cases the sealed subset rejects (P, edited, …) are emitted as `label<TAB>UNSUPPORTED`.
//! Test infrastructure, not API.

use gnucobol_rs::{build_field, Usage};
use std::io::{self, BufRead, Write};

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    for line in stdin.lock().lines().map_while(Result::ok) {
        if line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        let label = f.first().copied().unwrap_or("");
        let pic = f.get(1).copied().unwrap_or("");
        let usage = match f.get(2).copied().unwrap_or("") {
            "COMP-3" | "PACKED-DECIMAL" | "COMPUTATIONAL-3" => Usage::Comp3,
            _ => Usage::Display,
        };
        let sign = f.get(3).copied().unwrap_or("").to_ascii_uppercase();
        let sep = sign.contains("SEPARATE");
        let lead = sign.contains("LEADING");
        match build_field(pic, usage, sep, lead) {
            Ok(pf) => {
                let _ = writeln!(
                    out,
                    "{label}\t0x{:02x}\t{}\t{}\t0x{:04x}\t{}",
                    pf.attr.field_type, pf.attr.digits, pf.attr.scale, pf.attr.flags, pf.size
                );
            }
            Err(_) => {
                let _ = writeln!(out, "{label}\tUNSUPPORTED");
            }
        }
    }
}
