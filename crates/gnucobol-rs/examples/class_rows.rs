//! Rust mirror of the class-condition oracle (`GNURUST.CLASS.1`): reads `label|len|test|hex`, runs the
//! predicate, prints `label Y` / `label N`.
use gnucobol_rs::{is_alphabetic, is_alphabetic_lower, is_alphabetic_upper, is_numeric};
use std::io::{self, BufRead, Write};

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
        if f.len() != 4 {
            continue;
        }
        let (label, _len, test, hex) = (f[0], f[1], f[2], f[3]);
        let bytes: Vec<u8> = (0..hex.len() / 2)
            .map(|k| u8::from_str_radix(&hex[k * 2..k * 2 + 2], 16).unwrap_or(0))
            .collect();
        let r = match test {
            "num" => is_numeric(&bytes),
            "alp" => is_alphabetic(&bytes),
            "upr" => is_alphabetic_upper(&bytes),
            "lwr" => is_alphabetic_lower(&bytes),
            _ => false,
        };
        let _ = writeln!(out, "{label} {}", if r { "Y" } else { "N" });
    }
}
