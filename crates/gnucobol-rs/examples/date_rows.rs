//! Rust mirror for the date-intrinsic sweep (`GNURUST.INTRINSIC.DATE.1`).
use gnucobol_rs::intrinsic::{intrinsic_date_of_integer, intrinsic_day_of_integer, intrinsic_integer_of_date, intrinsic_integer_of_day};
use std::io::BufRead;
fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 4 { continue; }
        let (label, op, arg, oracle) = (f[0], f[1], f[2], f[3].trim().trim_start_matches('0'));
        let oracle: i64 = if oracle.is_empty() { 0 } else { oracle.parse().unwrap_or(-1) };
        let mine: i64 = match op {
            "IOD"  => intrinsic_integer_of_date(arg.parse().unwrap_or(0)),
            "DOI"  => intrinsic_date_of_integer(arg.parse().unwrap_or(0)) as i64,
            "IODY" => intrinsic_integer_of_day(arg.parse().unwrap_or(0)),
            "DYOI" => intrinsic_day_of_integer(arg.parse().unwrap_or(0)) as i64,
            _ => -1,
        };
        if mine == oracle { pass += 1; } else { println!("{label} FAIL {op}({arg}) mine={mine} oracle={oracle}"); fail += 1; }
    }
    println!("PASS={pass} FAIL={fail}");
}
