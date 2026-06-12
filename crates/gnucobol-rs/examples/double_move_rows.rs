//! Rust mirror of the float->decimal oracle: reads the same rows and runs cob_decimal_setget_fld
//! (set_field -> set_double -> get_field), the path libcob's cob_move(COMP-2 -> DISPLAY) takes. Emits
//! the DISPLAY result bytes as hex. Test infrastructure, not API.

use gnucobol_rs::cob_decimal::{cob_decimal_get_double, cob_decimal_set_field, cob_decimal_setget_fld};
use gnucobol_rs::{FieldAttr, Round};
use std::io::{self, BufRead, Write};

const DOUBLE: i64 = 0x14;

fn parse_hex(s: &str, size: usize) -> Option<Vec<u8>> {
    if s.len() < size * 2 {
        return None;
    }
    let b = s.as_bytes();
    (0..size)
        .map(|i| {
            let hi = (b[2 * i] as char).to_digit(16)?;
            let lo = (b[2 * i + 1] as char).to_digit(16)?;
            Some(((hi << 4) | lo) as u8)
        })
        .collect()
}

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    for line in stdin.lock().lines().map_while(Result::ok) {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() != 12 {
            continue;
        }
        let n = |i: usize| f[i].parse::<i64>().ok();
        let label = f[0];
        let (Some(st), Some(sd), Some(ssc), Some(sf), Some(ssz)) = (n(1), n(2), n(3), n(4), n(5)) else {
            continue;
        };
        let (Some(dt), Some(dd), Some(dsc), Some(df), Some(dsz)) = (n(7), n(8), n(9), n(10), n(11)) else {
            continue;
        };
        let Some(src) = parse_hex(f[6], ssz as usize) else {
            continue;
        };
        let s_attr = FieldAttr { field_type: st as u16, digits: sd as u16, scale: ssc as i16, flags: sf as u16 };
        let d_attr = FieldAttr { field_type: dt as u16, digits: dd as u16, scale: dsc as i16, flags: df as u16 };

        let bytes: Option<Vec<u8>> = if dt == DOUBLE {
            // reverse: decimal -> double via cob_decimal_get_double, emit 8-byte LE image
            let v = cob_decimal_get_double(&cob_decimal_set_field(&src, &s_attr));
            Some(v.to_le_bytes().to_vec())
        } else {
            cob_decimal_setget_fld(&src, &s_attr, &d_attr, dsz as usize, Round::Truncate).ok()
        };
        match bytes {
            Some(r) => {
                let mut hx = String::new();
                for byte in &r {
                    hx.push_str(&format!("{byte:02x}"));
                }
                let _ = writeln!(out, "{label} {hx}");
            }
            None => {
                let _ = writeln!(out, "{label} UNSUPPORTED");
            }
        }
    }
}
