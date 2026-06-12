//! Rust mirror of logical_harness.c. Row: `label op v0 v1` -> `label result`.
use gnucobol_rs::logical::*;
use std::io::{self, BufRead, Write};
fn main() {
    let stdin = io::stdin(); let stdout = io::stdout(); let mut out = stdout.lock();
    for line in stdin.lock().lines().map_while(Result::ok) {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() != 4 { continue; }
        let (label, op) = (f[0], f[1]);
        let a = f[2].parse::<i64>().unwrap() as i128;
        let b = f[3].parse::<i64>().unwrap() as i128;
        let r: u64 = match op {
            "and" => logical_and(a, b), "or" => logical_or(a, b), "xor" => logical_xor(a, b),
            "not" => logical_not(b), "shl" => logical_left(a, b), "shr" => logical_right(a, b),
            _ => continue,
        };
        let _ = writeln!(out, "{label} {r}");
    }
}
