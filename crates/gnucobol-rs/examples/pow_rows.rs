//! Rust mirror of pow_harness.c: reads `label width base power`, runs cob_s32_pow/cob_s64_pow.
use gnucobol_rs::int_pow::{cob_s32_pow, cob_s64_pow};
use std::io::{self, BufRead, Write};
fn main() {
    let stdin = io::stdin(); let stdout = io::stdout(); let mut out = stdout.lock();
    for line in stdin.lock().lines().map_while(Result::ok) {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() != 4 { continue; }
        let (label, w, b, p) = (f[0], f[1].parse::<u32>().unwrap(), f[2].parse::<i64>().unwrap(), f[3].parse::<i64>().unwrap());
        let res = if w == 32 { cob_s32_pow(b as i32, p as i32).map(|r| r as i64) } else { cob_s64_pow(b, p) };
        match res { Ok(r) => { let _ = writeln!(out, "{label} {r}"); }, Err(_) => { let _ = writeln!(out, "{label} SIGFPE"); } }
    }
}
