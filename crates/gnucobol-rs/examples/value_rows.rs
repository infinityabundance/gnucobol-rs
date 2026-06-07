//! Rust mirror of the VALUE program-oracle: reads `gen_value` spec lines, computes `value_image`,
//! and prints `LABEL hex`. Test infrastructure.
//! Spec: `LABEL|LEVEL:NAME:PIC:USAGE:VALUE|...`  USAGE ∈ {G,D,C}  VALUE ∈ ""|N..|A..|Z|S

use gnucobol_rs::{value_image, Usage, Val, ValueItem};
use std::io::{self, BufRead, Write};

fn parse_value(tok: &str) -> Option<Val> {
    let mut ch = tok.chars();
    match ch.next() {
        None => None,
        Some('N') => Some(Val::Num(ch.collect())),
        Some('A') => Some(Val::Alpha(ch.collect())),
        Some('Z') => Some(Val::Zero),
        Some('S') => Some(Val::Space),
        _ => None,
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
        let mut parts = line.split('|');
        let label = parts.next().unwrap_or("");
        let mut items = Vec::new();
        for f in parts {
            let c: Vec<&str> = f.splitn(5, ':').collect();
            if c.len() != 5 {
                continue;
            }
            let level: u16 = c[0].parse().unwrap_or(0);
            let name = c[1].to_string();
            let pic = match c[3] {
                "G" => None,
                "D" => Some((c[2].to_string(), Usage::Display, false, false)),
                "C" => Some((c[2].to_string(), Usage::Comp3, false, false)),
                _ => None,
            };
            items.push(ValueItem {
                level,
                name,
                pic,
                value: parse_value(c[4]),
            });
        }
        match value_image(&items) {
            Ok(bytes) => {
                let mut hx = String::new();
                for b in &bytes {
                    hx.push_str(&format!("{b:02x}"));
                }
                let _ = writeln!(out, "{label} {hx}");
            }
            Err(_) => {
                let _ = writeln!(out, "{label} UNSUPPORTED");
            }
        }
    }
}
