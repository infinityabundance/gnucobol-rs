//! Rust mirror for the LENGTH sweep (`GNURUST.INTRINSIC.LENGTH.1`). Reads cases + the oracle LENGTH, runs
//! intrinsic_length, compares. PASS=n FAIL=n.
use gnucobol_rs::intrinsic::intrinsic_length;
use gnucobol_rs::Usage;
use std::io::BufRead;
fn usage(s: &str) -> Usage {
    match s { "COMP-3" => Usage::Comp3, "COMP" => Usage::Comp, "COMP-5" => Usage::Comp5, "COMP-X" => Usage::CompX, _ => Usage::Display }
}
fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 4 { continue; }
        let (label, pic, u, oracle) = (f[0], f[1], f[2], f[3].trim_start_matches('0').parse::<usize>().unwrap_or(0));
        let mine = intrinsic_length(pic, usage(u)).unwrap_or(0);
        if mine == oracle { pass += 1; } else { println!("{label} FAIL pic={pic}/{u} mine={mine} oracle={oracle}"); fail += 1; }
    }
    println!("PASS={pass} FAIL={fail}");
}
