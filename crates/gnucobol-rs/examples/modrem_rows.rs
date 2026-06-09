//! Rust mirror for the MOD/REM sweep (`GNURUST.INTRINSIC.MOD-REM.1`). Reads cases + the oracle S9(4) display,
//! runs intrinsic_mod/intrinsic_rem, compares the integer value. PASS=n FAIL=n.
use gnucobol_rs::intrinsic::{intrinsic_mod, intrinsic_rem};
use std::io::BufRead;
fn parse_signed(s: &str) -> i128 {
    let s = s.trim();
    let (neg, digits) = match s.strip_prefix('-') { Some(d) => (true, d), None => (false, s.trim_start_matches('+')) };
    let v: i128 = digits.trim_start_matches('0').parse().unwrap_or(0);
    if neg { -v } else { v }
}
fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 5 { continue; }
        let (label, op, a, b, oracle) = (f[0], f[1], f[2].parse::<i128>().unwrap(), f[3].parse::<i128>().unwrap(), parse_signed(f[4]));
        let mine = if op == "MOD" { intrinsic_mod(a, b) } else { intrinsic_rem(a, b) };
        if mine == oracle { pass += 1; } else { println!("{label} FAIL {op}({a},{b}) mine={mine} oracle={oracle}"); fail += 1; }
    }
    println!("PASS={pass} FAIL={fail}");
}
