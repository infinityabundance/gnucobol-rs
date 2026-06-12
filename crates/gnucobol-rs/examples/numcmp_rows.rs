//! Rust mirror for the numeric comparison sweep.
use gnucobol_rs::cob_decimal::cob_numeric_cmp;
use gnucobol_rs::FieldAttr;
use std::io::{self, BufRead, Write};
fn ph(s: &str, n: usize) -> Vec<u8> { (0..n).map(|i| u8::from_str_radix(&s[2*i..2*i+2], 16).unwrap()).collect() }
fn main() {
    let si = io::stdin(); let so = io::stdout(); let mut o = so.lock();
    for line in si.lock().lines().map_while(Result::ok) {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() != 13 { continue; }
        let g = |i: usize| f[i].parse::<i64>().unwrap();
        let a1 = FieldAttr { field_type: g(1) as u16, digits: g(2) as u16, scale: g(3) as i16, flags: g(4) as u16 };
        let b1 = ph(f[6], g(5) as usize);
        let a2 = FieldAttr { field_type: g(7) as u16, digits: g(8) as u16, scale: g(9) as i16, flags: g(10) as u16 };
        let b2 = ph(f[12], g(11) as usize);
        let _ = writeln!(o, "{} {}", f[0], cob_numeric_cmp(&b1, &a1, &b2, &a2));
    }
}
