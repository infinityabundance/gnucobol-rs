//! Rust mirror of the arithmetic oracle (`lab/oracle/arith_harness.c`): reads the same rows, runs
//! `cob_arith` (f1 := f1 op f2), and emits the result field bytes as hex. Test infrastructure.
//!
//! Input row: `label op a_type a_dig a_scale a_flags a_size a_hex b_type b_dig b_scale b_flags b_size b_hex opt`

use gnucobol_rs::{cob_arith, FieldAttr, Op, Round};
use std::io::{self, BufRead, Write};

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
        if f.len() != 15 {
            eprintln!("bad row ({}): {line}", f.len());
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
        let Some(a) = parse_hex(f[7], asz as usize) else {
            continue;
        };
        let Some(b) = parse_hex(f[13], bsz as usize) else {
            continue;
        };
        let a_attr = FieldAttr {
            field_type: at as u16,
            digits: ad as u16,
            scale: asc as i16,
            flags: af as u16,
        };
        let b_attr = FieldAttr {
            field_type: bt as u16,
            digits: bd as u16,
            scale: bsc as i16,
            flags: bf as u16,
        };
        let op = match op {
            1 => Op::Add,
            2 => Op::Subtract,
            _ => Op::Multiply,
        };
        let round = if opt == 1 {
            Round::NearAwayFromZero
        } else {
            Round::Truncate
        };
        match cob_arith(op, &a, &a_attr, &b, &b_attr, round) {
            Ok(r) => {
                let mut hx = String::new();
                for byte in &r {
                    hx.push_str(&format!("{byte:02x}"));
                }
                let _ = writeln!(out, "{label} {hx}");
            }
            Err(_) => {
                let _ = writeln!(out, "{label} UNSUPPORTED");
            }
        }
    }
}
