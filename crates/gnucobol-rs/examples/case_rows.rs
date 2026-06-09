//! Rust mirror for the CASE/REVERSE sweep (`GNURUST.INTRINSIC.CASE.1`). Reads cases + the oracle X(8) bytes,
//! runs intrinsic_upper_case/lower_case/reverse (padded to 8), compares.
use gnucobol_rs::intrinsic::{intrinsic_lower_case, intrinsic_reverse, intrinsic_upper_case};
use std::io::BufRead;
fn unhex(s: &str) -> Vec<u8> { (0..s.len()/2).map(|k| u8::from_str_radix(&s[k*2..k*2+2],16).unwrap_or(0)).collect() }
fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 4 { continue; }
        let (label, op, input, oracle) = (f[0], f[1], f[2].as_bytes(), unhex(f[3].trim()));
        let mut mine = match op { "UPPER-CASE" => intrinsic_upper_case(input), "LOWER-CASE" => intrinsic_lower_case(input), _ => intrinsic_reverse(input) };
        mine.resize(8, b' '); // MOVE into X(8)
        if mine == oracle { pass += 1; } else { println!("{label} FAIL {op} mine={} oracle={}", String::from_utf8_lossy(&mine), f[3]); fail += 1; }
    }
    println!("PASS={pass} FAIL={fail}");
}
