//! FUNCTION-EVIDENCE (`PORT-GOVERNANCE.2`): the per-ported-function evidence map.
//!
//! `LIBCOB-PARITY` answers "does a real Rust fn exist?". This answers the stronger question "is that
//! Rust fn *backed* by evidence?" — a unit test, a Kani proof, and/or a fuzz target. It reads the
//! committed `parity-detailed.json` (the typed C↔Rust join) and scans the Rust source for which ported
//! functions are referenced inside `#[cfg(test)]` modules, `#[cfg(kani)]` proofs, and fuzz targets,
//! then renders `FUNCTION-EVIDENCE.md` + `reports/port-index/evidence.json`. `check` regenerates and
//! diffs (an anti-staleness gate); the per-completed-file "no unevidenced active fn" rule is a future
//! Tier-2 closure gate.

use crate::paths;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

fn read_json(p: &Path) -> Value {
    std::fs::read_to_string(p).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(Value::Null)
}

/// Collect identifier-like tokens (len > 2) from `s` into `out`.
fn idents(s: &str, out: &mut HashSet<String>) {
    let mut cur = String::new();
    for c in s.chars() {
        if c.is_alphanumeric() || c == '_' {
            cur.push(c);
        } else {
            if cur.len() > 2 {
                out.insert(std::mem::take(&mut cur));
            } else {
                cur.clear();
            }
        }
    }
    if cur.len() > 2 {
        out.insert(cur);
    }
}

/// Every `.rs` file under a directory (recursive).
fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                rs_files(&p, out);
            } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
                out.push(p);
            }
        }
    }
}

/// The three evidence-token sets gathered from the Rust source tree + examples + fuzz targets.
struct Evidence {
    test: HashSet<String>,
    kani: HashSet<String>,
    fuzz: HashSet<String>,
}

fn gather_evidence(root: &Path) -> Evidence {
    let mut ev = Evidence { test: HashSet::new(), kani: HashSet::new(), fuzz: HashSet::new() };
    // library source: split each file into its body / #[cfg(test)] region / #[cfg(kani)] region.
    let mut src = Vec::new();
    rs_files(&paths::rust_src_dir(root), &mut src);
    for p in &src {
        let s = std::fs::read_to_string(p).unwrap_or_default();
        let test_at = s.find("#[cfg(test)]");
        let kani_at = s.find("#[cfg(kani)]");
        if let Some(ti) = test_at {
            // the test region runs to the kani region (if it follows) or to EOF.
            let end = kani_at.filter(|k| *k > ti).unwrap_or(s.len());
            idents(&s[ti..end], &mut ev.test);
        }
        if let Some(ki) = kani_at {
            idents(&s[ki..], &mut ev.kani);
        }
        // `__fuzz*` entry functions (in the library) name the functions they exercise.
        let mut from = 0;
        while let Some(rel) = s[from..].find("fn __fuzz") {
            let start = from + rel;
            // grab a generous window of the fuzz fn body.
            let end = (start + 1200).min(s.len());
            idents(&s[start..end], &mut ev.fuzz);
            from = start + 8;
        }
    }
    // fuzz targets reference the functions/entry points they drive.
    let mut fuzz = Vec::new();
    rs_files(&root.join("crates/gnucobol-rs/fuzz/fuzz_targets"), &mut fuzz);
    for p in &fuzz {
        idents(&std::fs::read_to_string(p).unwrap_or_default(), &mut ev.fuzz);
    }
    ev
}

/// Build the per-file evidence rows from the committed detailed parity + the gathered evidence.
fn build(root: &Path) -> Value {
    let parity = read_json(&root.join(paths::PORT_INDEX_DIR).join("parity-detailed.json"));
    let ev = gather_evidence(root);
    let empty = Vec::new();
    let rows = parity.as_array().unwrap_or(&empty);
    let mut files = Vec::new();
    let (mut tot_active, mut tot_test, mut tot_kani, mut tot_fuzz, mut tot_any) = (0u64, 0, 0, 0, 0);
    for r in rows {
        let file = r["file"].as_str().unwrap_or("");
        let fns = r["fns"].as_array().cloned().unwrap_or_default();
        let mut active = 0u64;
        let (mut wt, mut wk, mut wf, mut any) = (0u64, 0, 0, 0);
        let mut unevidenced = Vec::new();
        for f in &fns {
            if f["rust_status"].as_str() != Some("active") {
                continue;
            }
            let name = f["function"].as_str().unwrap_or("");
            active += 1;
            let t = ev.test.contains(name);
            let k = ev.kani.contains(name);
            let z = ev.fuzz.contains(name);
            if t {
                wt += 1;
            }
            if k {
                wk += 1;
            }
            if z {
                wf += 1;
            }
            if t || k || z {
                any += 1;
            } else {
                unevidenced.push(name.to_string());
            }
        }
        tot_active += active;
        tot_test += wt;
        tot_kani += wk;
        tot_fuzz += wf;
        tot_any += any;
        files.push(json!({
            "file": file,
            "active": active,
            "with_test": wt,
            "with_kani": wk,
            "with_fuzz": wf,
            "with_any": any,
            "unevidenced": unevidenced,
        }));
    }
    json!({
        "schema": "gnurust-function-evidence-v1",
        "court": "PORT-GOVERNANCE.2",
        "doctrine": "active parity (a real Rust fn exists) is upgraded to evidence parity (the fn is referenced by a unit test, a Kani proof, and/or a fuzz target). Evidence is a navigation aid, not a behaviour proof; byte parity stays the per-court oracle sweeps.",
        "totals": {"active": tot_active, "with_test": tot_test, "with_kani": tot_kani, "with_fuzz": tot_fuzz, "with_any": tot_any, "unevidenced": tot_active - tot_any},
        "files": files,
        "non_claims": ["NEG.EVIDENCE.NOT_A_BEHAVIOR_PROOF", "NEG.EVIDENCE.TOKEN_REFERENCE_ONLY"],
    })
}

fn render_md(b: &Value) -> String {
    let t = &b["totals"];
    let mut l = vec![
        "<!-- generated by `gnucobol-rs-port-index evidence` — do not edit by hand -->".to_string(),
        String::new(),
        "# FUNCTION-EVIDENCE — does each ported function carry evidence?".to_string(),
        String::new(),
        "> `LIBCOB-PARITY` proves a real Rust `fn` exists. This map asks the stronger question: is that".to_string(),
        "> `fn` **referenced by a unit test, a Kani proof, and/or a fuzz target**? A ported-but-unevidenced".to_string(),
        "> function is a false-confidence risk. (Evidence = token reference, a navigation aid; byte parity".to_string(),
        "> stays the per-court oracle sweeps in `lab/verify-sealed-courts.sh`.)".to_string(),
        String::new(),
        format!(
            "**Active ported: {}** · with a unit test {} · with a Kani proof {} · with a fuzz target {} · **evidenced (any) {}** · **unevidenced {}**.",
            t["active"], t["with_test"], t["with_kani"], t["with_fuzz"], t["with_any"], t["unevidenced"]
        ),
        String::new(),
        "| libcob file | active | test | kani | fuzz | evidenced | unevidenced |".to_string(),
        "|---|---:|---:|---:|---:|---:|---:|".to_string(),
    ];
    for f in b["files"].as_array().unwrap_or(&Vec::new()) {
        let un = f["unevidenced"].as_array().map(|a| a.len()).unwrap_or(0);
        if f["active"].as_u64().unwrap_or(0) == 0 {
            continue;
        }
        l.push(format!(
            "| `{}` | {} | {} | {} | {} | {} | {} |",
            f["file"].as_str().unwrap_or(""),
            f["active"],
            f["with_test"],
            f["with_kani"],
            f["with_fuzz"],
            f["with_any"],
            un
        ));
    }
    l.push(String::new());
    l.push("## Ported but unevidenced (no test / Kani / fuzz reference)".to_string());
    l.push(String::new());
    let mut any_un = false;
    for f in b["files"].as_array().unwrap_or(&Vec::new()) {
        let un = f["unevidenced"].as_array().cloned().unwrap_or_default();
        if un.is_empty() {
            continue;
        }
        any_un = true;
        let names: Vec<String> = un.iter().filter_map(|x| x.as_str().map(|s| format!("`{s}`"))).collect();
        l.push(format!("- **`{}`** ({}): {}", f["file"].as_str().unwrap_or(""), un.len(), names.join(", ")));
    }
    if !any_un {
        l.push("_None — every active ported function carries at least one evidence reference._".to_string());
    }
    l.push(String::new());
    l.push("## How this is produced (reproducible)".to_string());
    l.push(String::new());
    l.push("`gnucobol-rs-port-index evidence generate` joins the committed `parity-detailed.json` with a".to_string());
    l.push("scan of the Rust source's `#[cfg(test)]` / `#[cfg(kani)]` regions and fuzz targets. `evidence".to_string());
    l.push("check` regenerates and diffs (anti-staleness).".to_string());
    l.join("\n") + "\n"
}

pub fn generate(root: &Path) -> i32 {
    let b = build(root);
    let dir = root.join(paths::PORT_INDEX_DIR);
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(dir.join("evidence.json"), serde_json::to_string_pretty(&b).unwrap_or_default() + "\n");
    let _ = std::fs::write(root.join("FUNCTION-EVIDENCE.md"), render_md(&b));
    let t = &b["totals"];
    println!(
        "FUNCTION-EVIDENCE: {} active ported, {} evidenced, {} unevidenced (test {} / kani {} / fuzz {})",
        t["active"], t["with_any"], t["unevidenced"], t["with_test"], t["with_kani"], t["with_fuzz"]
    );
    0
}

pub fn check(root: &Path) -> i32 {
    let want_md = render_md(&build(root));
    let have_md = std::fs::read_to_string(root.join("FUNCTION-EVIDENCE.md")).unwrap_or_default();
    if want_md != have_md {
        println!("EVIDENCE.DRIFT: FUNCTION-EVIDENCE.md is stale — run `gnucobol-rs-port-index evidence generate`");
        return 1;
    }
    // sanity: keep the committed totals aligned with a fresh build.
    let fresh = build(root);
    let have = read_json(&root.join(paths::PORT_INDEX_DIR).join("evidence.json"));
    if fresh["totals"] != have["totals"] {
        println!("EVIDENCE.DRIFT: evidence.json totals stale — regenerate");
        return 1;
    }
    let t = &fresh["totals"];
    println!("FUNCTION-EVIDENCE check: fresh ({} active, {} evidenced, {} unevidenced)", t["active"], t["with_any"], t["unevidenced"]);
    0
}
