//! Rust mirror for the PACKED in-place arithmetic oracle: reads the arith row format and, for a
//! PACKED receiver with op ADD/SUBTRACT, routes through `packed::cob_addsub_optimized` (the real
//! `cob_add_bcd` fast path) exactly as libcob's `cob_add`/`cob_sub` do; everything else falls back to
//! `cob_arith`. Emits the result field bytes as hex. Test infrastructure, not API.
//!
//! Input row: `label op a_type a_dig a_scale a_flags a_size a_hex b_type b_dig b_scale b_flags b_size b_hex opt`

use gnucobol_rs::packed::cob_addsub_optimized;
use gnucobol_rs::{cob_arith, FieldAttr, Op, Round};
use std::io::{self, BufRead, Write};

const PACKED: i64 = 18;

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

fn round_of(opt: i64) -> Round {
    if opt & 1 == 0 {
        Round::Truncate
    } else if opt & 16 != 0 {
        Round::AwayFromZero
    } else if opt & 64 != 0 {
        Round::NearEven
    } else if opt & 128 != 0 {
        Round::NearTowardZero
    } else if opt & 256 != 0 {
        Round::Prohibited
    } else if opt & 512 != 0 {
        Round::TowardGreater
    } else if opt & 1024 != 0 {
        Round::TowardLesser
    } else if opt & 2048 != 0 {
        Round::Truncate
    } else {
        Round::NearAwayFromZero
    }
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
        if f.len() != 15 {
            continue;
        }
        let n = |i: usize| f[i].parse::<i64>().ok();
        let label = f[0];
        let (Some(op), Some(at), Some(ad), Some(asc), Some(af), Some(asz)) =
            (n(1), n(2), n(3), n(4), n(5), n(6))
        else {
            continue;
        };
        let (Some(bt), Some(bd), Some(bsc), Some(bf), Some(bsz), Some(opt)) =
            (n(8), n(9), n(10), n(11), n(12), n(14))
        else {
            continue;
        };
        let (Some(a), Some(b)) = (parse_hex(f[7], asz as usize), parse_hex(f[13], bsz as usize)) else {
            continue;
        };
        let a_attr = FieldAttr { field_type: at as u16, digits: ad as u16, scale: asc as i16, flags: af as u16 };
        let b_attr = FieldAttr { field_type: bt as u16, digits: bd as u16, scale: bsc as i16, flags: bf as u16 };

        // The path under test: PACKED receiver, ADD or SUBTRACT, via the in-place cob_add_bcd fast path.
        let result: Option<Vec<u8>> = if at == PACKED && (op == 1 || op == 2) {
            let mut acc = a.clone();
            match cob_addsub_optimized(&mut acc, &a_attr, &b, &b_attr, opt as i32, op == 2) {
                Some(_) => Some(acc), // status reflects size error; libcob leaves f1 as written/unchanged
                None => None,
            }
        } else {
            None
        };

        let bytes = match result {
            Some(r) => Some(r),
            None => {
                let o = match op {
                    1 => Op::Add,
                    2 => Op::Subtract,
                    _ => Op::Multiply,
                };
                cob_arith(o, &a, &a_attr, &b, &b_attr, round_of(opt)).ok()
            }
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
