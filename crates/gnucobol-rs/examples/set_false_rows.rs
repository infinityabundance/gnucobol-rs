//! Rust mirror of the `SET condition-name TO FALSE` oracle (`GNURUST.12B`): reads `gen_set_false` specs,
//! runs `set_88_false`, self-checks that `eval_88` of the result is **false** (SET TO FALSE makes the 88
//! false), and prints `label hex`.  Spec: `label|pic-decl|size|88def|falselit`.
use gnucobol_rs::{build_field, eval_88, set_88_false, CondLit, CondValue, Condition, Usage};
use std::io::{self, BufRead, Write};

fn parse_def(def: &str) -> Vec<CondValue> {
    let mut out = Vec::new();
    for entry in def.split(';') {
        let p: Vec<&str> = entry.split(':').collect();
        match p.as_slice() {
            ["la", s] => out.push(CondValue::Lit(CondLit::Alpha((*s).to_string()))),
            ["ln", s] => out.push(CondValue::Lit(CondLit::Num((*s).to_string()))),
            ["ra", a, b] => out.push(CondValue::Range(CondLit::Alpha((*a).into()), CondLit::Alpha((*b).into()))),
            ["rn", a, b] => out.push(CondValue::Range(CondLit::Num((*a).into()), CondLit::Num((*b).into()))),
            _ => {}
        }
    }
    out
}

fn parse_false(fls: &str) -> Option<CondLit> {
    let p: Vec<&str> = fls.split(':').collect();
    match p.as_slice() {
        ["fa", s] => Some(CondLit::Alpha((*s).to_string())),
        ["fn", s] => Some(CondLit::Num((*s).to_string())),
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
        let f: Vec<&str> = line.split('|').collect();
        if f.len() != 5 {
            continue;
        }
        let (label, decl, _size, def, fls) = (f[0], f[1], f[2], f[3], f[4]);
        let (pic, usage) = match decl.strip_suffix(" USAGE COMP-3") {
            Some(p) => (p, Usage::Comp3),
            None => (decl, Usage::Display),
        };
        let pf = match build_field(pic, usage, false, false) {
            Ok(pf) => pf,
            Err(_) => { let _ = writeln!(out, "{label} UNSUPPORTED"); continue; }
        };
        let cond = Condition {
            name: label.to_string(),
            values: parse_def(def),
            false_value: parse_false(fls),
        };
        match set_88_false(&pf.attr, pf.size, &cond) {
            Ok(bytes) => {
                if eval_88(&pf.attr, &bytes, &cond) == Ok(true) {
                    let _ = writeln!(out, "{label} SELFCHECK_FAIL"); // SET TO FALSE must NOT leave it true
                    continue;
                }
                let mut hx = String::new();
                for b in &bytes { hx.push_str(&format!("{b:02x}")); }
                let _ = writeln!(out, "{label} {hx}");
            }
            Err(_) => { let _ = writeln!(out, "{label} UNSUPPORTED"); }
        }
    }
}
