//! Rust mirror of the LEVEL-88 oracle: reads `gen_cond` specs, encodes the parent value via the
//! sealed `value_image`, runs `eval_88`, and prints `label T|F`. Test infrastructure.
//! Spec: `label|pic|mvkind:mv|88def`.

use gnucobol_rs::{
    build_field, eval_88, value_image, CondLit, CondValue, Condition, Usage, Val, ValueItem,
};
use std::io::{self, BufRead, Write};

fn parse_lit(kind: char, s: &str) -> CondLit {
    if kind == 'a' {
        CondLit::Alpha(s.to_string())
    } else {
        CondLit::Num(s.to_string())
    }
}

fn parse_def(def: &str) -> Vec<CondValue> {
    let mut out = Vec::new();
    for entry in def.split(';') {
        let parts: Vec<&str> = entry.split(':').collect();
        match parts.as_slice() {
            ["la", s] => out.push(CondValue::Lit(CondLit::Alpha((*s).to_string()))),
            ["ln", s] => out.push(CondValue::Lit(CondLit::Num((*s).to_string()))),
            ["ra", a, b] => out.push(CondValue::Range(parse_lit('a', a), parse_lit('a', b))),
            ["rn", a, b] => out.push(CondValue::Range(parse_lit('n', a), parse_lit('n', b))),
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
        let (label, pic, mvspec, def) = (f[0], f[1], f[2], f[3]);
        let (mvkind, mv) = mvspec.split_once(':').unwrap_or(("N", "0"));
        let value = if mvkind == "A" {
            Val::Alpha(mv.to_string())
        } else {
            Val::Num(mv.to_string())
        };
        // Encode the parent bytes via the sealed value_image (a single `01` field with this VALUE).
        let item = ValueItem {
            level: 1,
            name: "P".into(),
            pic: Some((pic.to_string(), Usage::Display, false, false)),
            value: Some(value),
        };
        let bytes = match value_image(&[item]) {
            Ok(b) => b,
            Err(_) => {
                let _ = writeln!(out, "{label} UNSUPPORTED");
                continue;
            }
        };
        let attr = match build_field(pic, Usage::Display, false, false) {
            Ok(pf) => pf.attr,
            Err(_) => {
                let _ = writeln!(out, "{label} UNSUPPORTED");
                continue;
            }
        };
        let cond = Condition {
            name: label.to_string(),
            values: parse_def(def),
        };
        match eval_88(&attr, &bytes, &cond) {
            Ok(true) => {
                let _ = writeln!(out, "{label} T");
            }
            Ok(false) => {
                let _ = writeln!(out, "{label} F");
            }
            Err(_) => {
                let _ = writeln!(out, "{label} UNSUPPORTED");
            }
        }
    }
}
