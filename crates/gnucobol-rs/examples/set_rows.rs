//! Rust mirror of the `SET condition-name TO TRUE` oracle: reads `gen_set` specs, runs
//! `set_88_true`, self-checks `eval_88`, and prints `label hex`. Test infrastructure.
//! Spec: `label|pic-decl|size|88def`  (pic-decl may end in " USAGE COMP-3").

use gnucobol_rs::{build_field, eval_88, set_88_true, CondLit, CondValue, Condition, Usage};
use std::io::{self, BufRead, Write};

fn parse_def(def: &str) -> Vec<CondValue> {
    let mut out = Vec::new();
    for entry in def.split(';') {
        let p: Vec<&str> = entry.split(':').collect();
        match p.as_slice() {
            ["la", s] => out.push(CondValue::Lit(CondLit::Alpha((*s).to_string()))),
            ["ln", s] => out.push(CondValue::Lit(CondLit::Num((*s).to_string()))),
            ["ra", a, b] => out.push(CondValue::Range(
                CondLit::Alpha((*a).to_string()),
                CondLit::Alpha((*b).to_string()),
            )),
            ["rn", a, b] => out.push(CondValue::Range(
                CondLit::Num((*a).to_string()),
                CondLit::Num((*b).to_string()),
            )),
            _ => {}
        }
    }
    out
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
        let f: Vec<&str> = line.split('|').collect();
        if f.len() != 4 {
            continue;
        }
        let (label, decl, _size, def) = (f[0], f[1], f[2], f[3]);
        let (pic, usage) = match decl.strip_suffix(" USAGE COMP-3") {
            Some(p) => (p, Usage::Comp3),
            None => (decl, Usage::Display),
        };
        let pf = match build_field(pic, usage, false, false) {
            Ok(pf) => pf,
            Err(_) => {
                let _ = writeln!(out, "{label} UNSUPPORTED");
                continue;
            }
        };
        let cond = Condition {
            name: label.to_string(),
            values: parse_def(def),
            false_value: None,
        };
        match set_88_true(&pf.attr, pf.size, &cond) {
            Ok(bytes) => {
                // self-check: the constructed bytes must satisfy eval_88.
                if eval_88(&pf.attr, &bytes, &cond) != Ok(true) {
                    let _ = writeln!(out, "{label} SELFCHECK_FAIL");
                    continue;
                }
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
