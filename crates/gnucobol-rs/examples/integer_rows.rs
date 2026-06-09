//! Rust mirror for the INTEGER/INTEGER-PART sweep (`GNURUST.INTRINSIC.INTEGER.1`). Reads cases + the oracle
//! S9 display, parses x via intrinsic_numval, runs intrinsic_integer/intrinsic_integer_part, compares.
use gnucobol_rs::intrinsic::{intrinsic_integer, intrinsic_integer_part, intrinsic_numval};
use std::io::BufRead;
fn parse_signed(s: &str) -> i128 {
    let s = s.trim();
    let (neg, d) = match s.strip_prefix('-') { Some(d) => (true, d), None => (false, s.trim_start_matches('+')) };
    let v: i128 = d.trim_start_matches('0').parse().unwrap_or(0);
    if neg { -v } else { v }
}
fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 4 { continue; }
        let (label, op, x, oracle) = (f[0], f[1], f[2], parse_signed(f[3]));
        let nv = intrinsic_numval(x);
        let mine = if op == "INTEGER" { intrinsic_integer(nv.signed_mag(), nv.scale) } else { intrinsic_integer_part(nv.signed_mag(), nv.scale) };
        if mine == oracle { pass += 1; } else { println!("{label} FAIL {op}({x}) mine={mine} oracle={oracle}"); fail += 1; }
    }
    println!("PASS={pass} FAIL={fail}");
}
