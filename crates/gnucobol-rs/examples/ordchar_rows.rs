//! Rust mirror for the ORD/CHAR sweep (`GNURUST.INTRINSIC.ORD-CHAR.1`). ORD cases compare the integer; CHAR
//! cases compare the byte (hex). PASS=n FAIL=n.
use gnucobol_rs::intrinsic::{intrinsic_char, intrinsic_ord};
use std::io::BufRead;
fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 4 { continue; }
        let (label, op, arg, oracle) = (f[0], f[1], f[2], f[3].trim());
        let ok = if op == "ORD" {
            let oracle_val: u32 = oracle.parse().unwrap_or(0);
            intrinsic_ord(arg.as_bytes()[0]) == oracle_val
        } else {
            format!("{:02x}", intrinsic_char(arg.parse().unwrap_or(0))) == oracle
        };
        if ok { pass += 1; } else { println!("{label} FAIL {op}({arg}) oracle={oracle}"); fail += 1; }
    }
    println!("PASS={pass} FAIL={fail}");
}
