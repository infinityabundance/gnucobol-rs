//! Rust mirror for the NUMVAL sweep (`GNURUST.INTRINSIC.NUMVAL.1`). Reads cases + the oracle S9(8)V9(4)
//! display, runs intrinsic_numval + numval_display, compares. PASS=n FAIL=n.
use gnucobol_rs::intrinsic::{intrinsic_numval, numval_display};
use std::io::BufRead;
fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 3 { continue; }
        let (label, input, oracle) = (f[0], f[1], f[2].trim());
        let mine = numval_display(&intrinsic_numval(input), 8, 4);
        if mine == oracle { pass += 1; } else { println!("{label} FAIL in=[{input}] mine={mine} oracle={oracle}"); fail += 1; }
    }
    println!("PASS={pass} FAIL={fail}");
}
