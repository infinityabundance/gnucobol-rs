//! Rust mirror of the reference-modification oracle (`GNURUST.REFMOD.1`): reads `label|field|op|start|
//! length|src`, runs ref_mod/ref_mod_to_end/apply_ref_mod, prints `label hex`.
use gnucobol_rs::{apply_ref_mod, ref_mod, ref_mod_to_end};
use std::io::{self, BufRead, Write};

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    for line in stdin.lock().lines().map_while(Result::ok) {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split('|').collect();
        if f.len() != 6 {
            continue;
        }
        let (label, field, op, start, length, src) = (
            f[0], f[1].as_bytes(), f[2],
            f[3].parse::<usize>().unwrap_or(0), f[4].parse::<usize>().unwrap_or(0), f[5].as_bytes(),
        );
        let result = match op {
            "src" => ref_mod(field, start, length).map(|b| hex(b)),
            "end" => ref_mod_to_end(field, start).map(|b| hex(b)),
            "recv" => {
                let mut g = field.to_vec();
                apply_ref_mod(&mut g, start, length, src).map(|_| hex(&g))
            }
            _ => Ok(String::new()),
        };
        let _ = match result {
            Ok(h) => writeln!(out, "{label} {h}"),
            Err(_) => writeln!(out, "{label} ERR"),
        };
    }
}
