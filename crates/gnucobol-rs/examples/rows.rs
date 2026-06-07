//! Row-mirror of the libcob `decimal_harness`: reads the **identical** input rows and emits the
//! destination field bytes after `cob_move`, so the differential sweep can pipe the same lines to
//! both the C oracle and this Rust port and compare full byte streams (`GNURUST.BYTESTREAM.0`).
//!
//! Input line: `label s_type s_digits s_scale s_flags s_size s_hex d_type d_digits d_scale d_flags d_size`
//! Output line: `label <dst_hex>` (lowercase hex of the destination field after the move).
//!
//! This example is test infrastructure, not part of the crate's API.

use gnucobol_rs::{cob_move, FieldAttr};
use std::io::{self, BufRead, Write};

fn parse_hex(s: &str, size: usize) -> Option<Vec<u8>> {
    if s.len() < size * 2 {
        return None;
    }
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(size);
    for i in 0..size {
        let hi = (bytes[2 * i] as char).to_digit(16)?;
        let lo = (bytes[2 * i + 1] as char).to_digit(16)?;
        out.push(((hi << 4) | lo) as u8);
    }
    Some(out)
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
            eprintln!("bad line ({} fields): {line}", f.len());
            continue;
        }
        let p = |i: usize| f[i].parse::<i64>().ok();
        let (Some(s_type), Some(s_digits), Some(s_scale), Some(s_flags), Some(s_size)) =
            (p(1), p(2), p(3), p(4), p(5))
        else {
            eprintln!("bad numbers: {line}");
            continue;
        };
        let (Some(d_type), Some(d_digits), Some(d_scale), Some(d_flags), Some(d_size)) =
            (p(7), p(8), p(9), p(10), p(11))
        else {
            eprintln!("bad numbers: {line}");
            continue;
        };
        let Some(src) = parse_hex(f[6], s_size as usize) else {
            eprintln!("bad hex: {line}");
            continue;
        };
        let src_attr = FieldAttr {
            field_type: s_type as u16,
            digits: s_digits as u16,
            scale: s_scale as i16,
            flags: s_flags as u16,
        };
        let dst_attr = FieldAttr {
            field_type: d_type as u16,
            digits: d_digits as u16,
            scale: d_scale as i16,
            flags: d_flags as u16,
        };
        let mut dst = vec![0u8; d_size as usize];
        // Ignore unsupported-pair errors here: the sweep only feeds the sealed pairs.
        let _ = cob_move(&src, &src_attr, &mut dst, &dst_attr);
        let mut hex = String::with_capacity(dst.len() * 2);
        for b in &dst {
            hex.push_str(&format!("{b:02x}"));
        }
        let _ = writeln!(out, "{} {}", f[0], hex);
    }
}
