//! Mirror for the edited-picture decode sweep (`GNURUST.16a`). Reads `label<TAB>pic<TAB>value<TAB>hex`
//! (hex = the oracle's edited field bytes), decodes via `gnucobol_rs::decode_edited`, and checks the
//! recovered numeric value equals the moved-in value and the size matches. Prints `PASS=n FAIL=n`.

use gnucobol_rs::{decode_edited, edited_size};
use std::io::BufRead;

fn signed_int(neg: bool, digits: &[u8]) -> i128 {
    let mut s: i128 = 0;
    for &d in digits {
        s = s * 10 + d as i128;
    }
    if neg {
        -s
    } else {
        s
    }
}

/// Parse a decimal literal like `-1234.56` into (signed integer, scale).
fn parse(value: &str) -> (i128, i32) {
    let neg = value.starts_with('-');
    let t = value.trim_start_matches(['-', '+']);
    let (i, f) = t.split_once('.').unwrap_or((t, ""));
    let digits: Vec<u8> = i.bytes().chain(f.bytes()).map(|b| b - b'0').collect();
    (signed_int(neg, &digits), f.len() as i32)
}

fn main() {
    let mut pass = 0u32;
    let mut fail = 0u32;
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 4 {
            continue;
        }
        let (label, pic, value, hex) = (f[0], f[1], f[2], f[3]);
        let bytes: Vec<u8> = (0..hex.len() / 2)
            .map(|k| u8::from_str_radix(&hex[k * 2..k * 2 + 2], 16).unwrap_or(0))
            .collect();
        let decoded = match decode_edited(pic, &bytes) {
            Ok(d) => d,
            Err(e) => {
                println!("{label} FAIL decode-error {e} (pic={pic} hex={hex})");
                fail += 1;
                continue;
            }
        };
        let n = decoded.numeric_value.unwrap();
        let (got_int, got_sc) = (signed_int(n.negative, &n.digits), n.scale as i32);
        let (want_int, want_sc) = parse(value);
        // Compare scale-normalized: bring both to the larger scale and compare signed integers.
        let common = got_sc.max(want_sc);
        let a = got_int * 10i128.pow((common - got_sc).max(0) as u32);
        let b = want_int * 10i128.pow((common - want_sc).max(0) as u32);
        let size_ok = edited_size(pic).map(|s| s == bytes.len()).unwrap_or(false);
        if a == b && size_ok {
            pass += 1;
        } else {
            println!(
                "{label} FAIL pic={pic} bytes='{}' want={value} got={a}@{common} size_ok={size_ok}",
                String::from_utf8_lossy(&bytes)
            );
            fail += 1;
        }
    }
    println!("PASS={pass} FAIL={fail}");
}
