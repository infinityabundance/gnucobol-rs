//! LIBCOB.PARITY — forensic 1:1 native-Rust porting parity over the admitted GnuCOBOL 3.2 libcob
//! source. For each `libcob/*.c` file it extracts the C function inventory and counts how many have a
//! named Rust counterpart in `crates/gnucobol-rs/src`. Name-level parity (the port names Rust
//! fns/docstrings after their C analog); BYTE parity is proven separately by the per-court oracle
//! sweeps in `verify-sealed-courts.sh`. `generate` writes the doc; `check` is the source-gated
//! anti-staleness gate (regenerate-and-diff when the admitted source is extracted).

use serde_json::{json, Value};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

const SRC_REL: &str = "lab/admit/gnucobol-3.2/libcob";
const JSON_REL: &str = "reports/libcob-parity.json";
const MD_REL: &str = "LIBCOB-PARITY.md";

/// The 13 libcob translation units, in port order (numeric.c first — the campaign's file 1).
const FILES: [&str; 13] = [
    "numeric.c", "move.c", "strings.c", "intrinsic.c", "cconv.c", "termio.c", "screenio.c", "call.c",
    "fileio.c", "mlio.c", "reportio.c", "common.c", "cobgetopt.c",
];

const C_KEYWORDS: [&str; 9] = ["if", "for", "while", "switch", "return", "else", "sizeof", "do", "typedef"];

/// Extract function-definition names from a libcob `.c` file: a bare return-type line (only identifier
/// chars, spaces, tabs, asterisks — no parens/semicolons) followed by a line `name (` at column 0.
/// Mirrors the awk inventory used throughout the port; the count is deterministic.
fn functions_in_c(path: &Path) -> Vec<String> {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let lines: Vec<&str> = text.lines().collect();
    let mut out: Vec<String> = Vec::new();
    let mut seen = HashSet::new();
    let is_rettype = |s: &str| {
        let t = s.trim_end();
        !t.is_empty()
            && t.chars().next().map(|c| c.is_ascii_alphabetic() || c == '_').unwrap_or(false)
            && t.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == ' ' || c == '\t' || c == '*')
    };
    for i in 1..lines.len() {
        let cur = lines[i];
        // current line must start (col 0) with an identifier immediately/space then '('
        let first = cur.chars().next();
        if !first.map(|c| c.is_ascii_alphabetic() || c == '_').unwrap_or(false) {
            continue;
        }
        let name: String = cur.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
        let rest = cur[name.len()..].trim_start();
        if !rest.starts_with('(') || name.is_empty() || C_KEYWORDS.contains(&name.as_str()) {
            continue;
        }
        if is_rettype(lines[i - 1]) && seen.insert(name.clone()) {
            out.push(name);
        }
    }
    out.sort();
    out
}

/// Every identifier token in the shipped Rust library source (`crates/gnucobol-rs/src/**/*.rs`).
fn rust_tokens(root: &str) -> HashSet<String> {
    let mut toks = HashSet::new();
    fn walk(dir: &Path, toks: &mut HashSet<String>) {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, toks);
                } else if p.extension().map(|x| x == "rs").unwrap_or(false) {
                    let s = std::fs::read_to_string(&p).unwrap_or_default();
                    let mut cur = String::new();
                    for ch in s.chars() {
                        if ch.is_ascii_alphanumeric() || ch == '_' {
                            cur.push(ch);
                        } else if !cur.is_empty() {
                            toks.insert(std::mem::take(&mut cur));
                        }
                    }
                    if !cur.is_empty() {
                        toks.insert(cur);
                    }
                }
            }
        }
    }
    walk(&Path::new(root).join("crates/gnucobol-rs/src"), &mut toks);
    toks
}

fn src_dir(root: &str) -> PathBuf {
    Path::new(root).join(SRC_REL)
}

/// Build the parity model. Returns `None` if the admitted libcob source is not extracted (gate skips).
fn build(root: &str) -> Option<Value> {
    let dir = src_dir(root);
    if !dir.join("numeric.c").exists() {
        return None;
    }
    let toks = rust_tokens(root);
    let mut files = Vec::new();
    let (mut tot, mut tot_p) = (0u64, 0u64);
    for f in FILES {
        let fns = functions_in_c(&dir.join(f));
        let total = fns.len() as u64;
        let missing: Vec<&String> = fns.iter().filter(|n| !toks.contains(*n)).collect();
        let ported = total - missing.len() as u64;
        tot += total;
        tot_p += ported;
        let pct = if total > 0 { (ported as f64) * 1000.0 / (total as f64) } else { 0.0 };
        files.push(json!({
            "file": f,
            "functions": total,
            "ported": ported,
            "parity_pct": (pct).round() / 10.0,
            "missing": missing,
        }));
    }
    let tpct = if tot > 0 { (tot_p as f64) * 1000.0 / (tot as f64) } else { 0.0 };
    Some(json!({
        "schema": "libcob-porting-parity-v1",
        "court": "LIBCOB.PARITY.1",
        "oracle": "GnuCOBOL 3.2.0 (admitted; see reports/admission/RECEIPT-ADMISSION.md)",
        "source_root": SRC_REL,
        "method": "A libcob function counts as PORTED when its exact name appears as a whole word in crates/gnucobol-rs/src/**/*.rs (the port names Rust functions/docstrings after their C analog). This is NAME-level parity; BYTE parity is proven separately by the per-court oracle sweeps (lab/verify-sealed-courts.sh).",
        "files": files,
        "totals": {"functions": tot, "ported": tot_p, "parity_pct": (tpct).round() / 10.0},
        "non_claims": [
            "Name-level parity is not byte-parity (see the oracle sweeps).",
            "#if 0-disabled functions ported for completeness count as ported; their behaviour is not oracle-verifiable.",
        ],
    }))
}

fn render_md(b: &Value) -> String {
    let mut l = vec![
        "<!-- generated by `cargo run -p xtask -- parity generate` — do not edit by hand -->".to_string(),
        String::new(),
        "# LIBCOB porting parity — 1:1 native Rust".to_string(),
        String::new(),
        "> Forensic name-match over the admitted GnuCOBOL 3.2 `libcob/*.c` source. **Name-level** parity".to_string(),
        "> (a function counts as ported when its name appears in `crates/gnucobol-rs/src`); **byte** parity".to_string(),
        "> is proven separately by the per-court oracle sweeps in `lab/verify-sealed-courts.sh`.".to_string(),
        String::new(),
        "| libcob file | functions | ported | parity |".to_string(),
        "|---|---:|---:|---:|".to_string(),
    ];
    for f in b["files"].as_array().unwrap() {
        l.push(format!(
            "| `{}` | {} | {} | {:.1}% |",
            f["file"].as_str().unwrap(),
            f["functions"],
            f["ported"],
            f["parity_pct"].as_f64().unwrap_or(0.0)
        ));
    }
    let t = &b["totals"];
    l.push(format!("| **total** | **{}** | **{}** | **{:.1}%** |", t["functions"], t["ported"], t["parity_pct"].as_f64().unwrap_or(0.0)));
    l.extend([
        String::new(),
        format!("_Method: {}_", b["method"].as_str().unwrap()),
        String::new(),
        "## Non-claims".to_string(),
        "- Name-level parity is **not** byte-parity (that is the oracle sweeps).".to_string(),
        "- `#if 0`-disabled functions ported for literal completeness count as ported; their behaviour is not oracle-verifiable.".to_string(),
        String::new(),
    ]);
    l.join("\n") + "\n"
}

pub fn run(cmd: &str, root: &str) -> i32 {
    match cmd {
        "generate" => match build(root) {
            Some(b) => {
                let _ = std::fs::write(Path::new(root).join(JSON_REL), serde_json::to_vec_pretty(&b).unwrap_or_default());
                let _ = std::fs::write(Path::new(root).join(MD_REL), render_md(&b));
                let t = &b["totals"];
                println!("LIBCOB.PARITY: {}/{} functions ({:.1}%) across 13 files", t["ported"], t["functions"], t["parity_pct"].as_f64().unwrap_or(0.0));
                0
            }
            None => {
                println!("LIBCOB.PARITY: admitted libcob source not extracted ({SRC_REL}) — generate skipped");
                0
            }
        },
        "check" => match build(root) {
            None => {
                println!("LIBCOB.PARITY: source not extracted — gate skipped (committed doc stands)");
                0
            }
            Some(fresh) => {
                let mut bad = 0;
                let cur_json = std::fs::read_to_string(Path::new(root).join(JSON_REL)).unwrap_or_default();
                let cur: Value = serde_json::from_str(&cur_json).unwrap_or(Value::Null);
                if cur != fresh {
                    println!("NEG.PARITY.STALE: {JSON_REL} != regenerated (re-run: cargo run -p xtask -- parity generate)");
                    bad += 1;
                }
                let cur_md = std::fs::read_to_string(Path::new(root).join(MD_REL)).unwrap_or_default();
                if cur_md != render_md(&fresh) {
                    println!("NEG.PARITY.STALE: {MD_REL} != render(parity.json)");
                    bad += 1;
                }
                if bad > 0 {
                    return 1;
                }
                let t = &fresh["totals"];
                println!("LIBCOB.PARITY: fresh — {}/{} functions ({:.1}%)", t["ported"], t["functions"], t["parity_pct"].as_f64().unwrap_or(0.0));
                0
            }
        },
        _ => {
            eprintln!("usage: parity generate|check");
            2
        }
    }
}
