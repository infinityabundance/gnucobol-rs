//! Rust mirror routing MULTIPLY through the 1:1 cob_decimal path (cob_mul on Mpz). Same arith row
//! format as arith_harness; only op=3 (MUL) is evaluated here (the general cob_decimal path).
use gnucobol_rs::cob_decimal::cob_mul;
use gnucobol_rs::{FieldAttr, Round};
use std::io::{self, BufRead, Write};
fn ph(s: &str, n: usize) -> Option<Vec<u8>> {
    if s.len() < n * 2 { return None; }
    (0..n).map(|i| u8::from_str_radix(&s[2*i..2*i+2], 16).ok()).collect()
}
fn main() {
    let si = io::stdin(); let so = io::stdout(); let mut o = so.lock();
    for line in si.lock().lines().map_while(Result::ok) {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() != 15 { continue; }
        let g = |i: usize| f[i].parse::<i64>().unwrap();
        let label = f[0];
        if g(1) != 3 { let _ = writeln!(o, "{label} UNSUPPORTED"); continue; } // MUL only
        let a1 = FieldAttr { field_type: g(2) as u16, digits: g(3) as u16, scale: g(4) as i16, flags: g(5) as u16 };
        let a2 = FieldAttr { field_type: g(8) as u16, digits: g(9) as u16, scale: g(10) as i16, flags: g(11) as u16 };
        let (Some(a), Some(b)) = (ph(f[7], g(6) as usize), ph(f[13], g(12) as usize)) else { continue; };
        let opt = g(14);
        let round = if opt & 1 == 0 { Round::Truncate }
            else if opt & 16 != 0 { Round::AwayFromZero }
            else if opt & 64 != 0 { Round::NearEven }
            else if opt & 128 != 0 { Round::NearTowardZero }
            else if opt & 256 != 0 { Round::Prohibited }
            else if opt & 512 != 0 { Round::TowardGreater }
            else if opt & 1024 != 0 { Round::TowardLesser }
            else if opt & 2048 != 0 { Round::Truncate }
            else { Round::NearAwayFromZero };
        match cob_mul(&a, &a1, &b, &a2, round) {
            Ok(r) => { let hx: String = r.iter().map(|x| format!("{x:02x}")).collect(); let _ = writeln!(o, "{label} {hx}"); }
            Err(_) => { let _ = writeln!(o, "{label} UNSUPPORTED"); }
        }
    }
}
