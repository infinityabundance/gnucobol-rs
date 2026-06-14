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

/// The evidence-token sets gathered from the Rust source tree, examples, fuzz targets, and oracle sweeps.
struct Evidence {
    test: HashSet<String>,
    kani: HashSet<String>,
    fuzz: HashSet<String>,
    /// identifiers referenced by an oracle sweep (the `examples/*_rows.rs` mirrors + `lab/oracle/*.sh`).
    oracle: HashSet<String>,
}

fn shell_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) == Some("sh") {
                out.push(p);
            }
        }
    }
}

fn gather_evidence(root: &Path) -> Evidence {
    let mut ev = Evidence { test: HashSet::new(), kani: HashSet::new(), fuzz: HashSet::new(), oracle: HashSet::new() };
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
    // ORACLE: the differential-sweep mirrors (examples/*.rs) and the sweep drivers (lab/oracle/*.sh) name
    // the Rust functions each oracle court exercises against the admitted libcob.
    let mut ex = Vec::new();
    rs_files(&root.join("crates/gnucobol-rs/examples"), &mut ex);
    let mut sh = Vec::new();
    shell_files(&root.join("lab/oracle"), &mut sh);
    for p in ex.iter().chain(sh.iter()) {
        idents(&std::fs::read_to_string(p).unwrap_or_default(), &mut ev.oracle);
    }
    ev
}

/// A name that is a deliberate lifecycle / setup / memory-management routine (RAII or no-op in the port).
fn is_lifecycle(name: &str) -> bool {
    ["init", "exit", "free", "malloc", "alloc", "tidy", "terminate", "cleanup", "_new_", "cache", "unlock"]
        .iter()
        .any(|t| name.contains(t))
}

/// Build `caller -> callees` from the clang callgraph, restricted to callees that are ported (so the
/// transitive closure stays within the port's own functions).
fn callgraph(root: &Path) -> Vec<(String, Vec<String>)> {
    let cg = read_json(&root.join(paths::PORT_INDEX_DIR).join("clang-callgraph.json"));
    let mut edges = Vec::new();
    if let Some(files) = cg["files"].as_array() {
        for f in files {
            if let Some(es) = f["edges"].as_array() {
                for e in es {
                    let caller = e["caller"].as_str().unwrap_or("").to_string();
                    let callees: Vec<String> = e["callees"].as_array().map(|a| a.iter().filter_map(|c| c.as_str().map(String::from)).collect()).unwrap_or_default();
                    if !caller.is_empty() {
                        edges.push((caller, callees));
                    }
                }
            }
        }
    }
    edges
}

/// Build the per-file classified-evidence rows from the committed detailed parity + the gathered
/// evidence + the clang callgraph (for transitive coverage).
fn build(root: &Path) -> Value {
    let parity = read_json(&root.join(paths::PORT_INDEX_DIR).join("parity-detailed.json"));
    let ev = gather_evidence(root);
    let empty = Vec::new();
    let rows = parity.as_array().unwrap_or(&empty);

    // Pass 1: the set of active fns that carry DIRECT evidence (unit/kani/fuzz/oracle token reference).
    let mut direct: HashSet<String> = HashSet::new();
    let mut active_all: HashSet<String> = HashSet::new();
    for r in rows {
        for f in r["fns"].as_array().unwrap_or(&empty) {
            if f["rust_status"].as_str() != Some("active") {
                continue;
            }
            let name = f["function"].as_str().unwrap_or("");
            active_all.insert(name.to_string());
            if ev.test.contains(name) || ev.kani.contains(name) || ev.fuzz.contains(name) || ev.oracle.contains(name) {
                direct.insert(name.to_string());
            }
        }
    }
    // TRANSITIVE: a fn called (per the clang callgraph) by a directly-covered function is covered through
    // that public oracle/test path. One hop, restricted to active ported callees.
    let mut transitive: HashSet<String> = HashSet::new();
    for (caller, callees) in callgraph(root) {
        if direct.contains(&caller) {
            for c in callees {
                if active_all.contains(&c) && !direct.contains(&c) {
                    transitive.insert(c);
                }
            }
        }
    }

    let mut files = Vec::new();
    let (mut t_active, mut t_dir, mut t_orc, mut t_tr, mut t_life, mut t_any) = (0u64, 0, 0, 0, 0, 0);
    for r in rows {
        let file = r["file"].as_str().unwrap_or("");
        let mut seen: HashSet<String> = HashSet::new();
        let (mut active, mut wdir, mut worc, mut wtr, mut wlife, mut any) = (0u64, 0, 0, 0, 0, 0);
        let mut unevidenced = Vec::new();
        for f in r["fns"].as_array().unwrap_or(&empty) {
            if f["rust_status"].as_str() != Some("active") {
                continue;
            }
            let name = f["function"].as_str().unwrap_or("");
            if !seen.insert(name.to_string()) {
                continue; // dedup repeated rows
            }
            active += 1;
            let d = direct.contains(name);
            let o = ev.oracle.contains(name);
            let tr = transitive.contains(name);
            let life = is_lifecycle(name);
            if d {
                wdir += 1;
            }
            if o {
                worc += 1;
            }
            if tr {
                wtr += 1;
            }
            if life {
                wlife += 1;
            }
            if d || tr || life {
                any += 1;
            } else {
                unevidenced.push(name.to_string());
            }
        }
        unevidenced.sort();
        unevidenced.dedup();
        t_active += active;
        t_dir += wdir;
        t_orc += worc;
        t_tr += wtr;
        t_life += wlife;
        t_any += any;
        files.push(json!({
            "file": file,
            "active": active,
            "direct": wdir,
            "oracle": worc,
            "transitive": wtr,
            "lifecycle": wlife,
            "evidenced_any": any,
            "unevidenced": unevidenced,
        }));
    }
    json!({
        "schema": "gnurust-function-evidence-v2",
        "court": "PORT-GOVERNANCE.3",
        "doctrine": "active parity (a real Rust fn exists) is upgraded to CLASSIFIED evidence parity. DIRECT = the fn name is referenced by a unit test, Kani proof, fuzz target, or oracle-sweep mirror. TRANSITIVE = it is called (per CLANG-AST-PARITY) by a directly-covered function, so it is exercised through that public oracle/test path. LIFECYCLE = an init/exit/free/alloc/cache setup routine (RAII / deliberate no-op in the port). UNEVIDENCED = none of these. Direct evidence is a token reference; court & transitive evidence are classified separately; byte parity remains the per-court oracle sweeps.",
        "totals": {"active": t_active, "direct": t_dir, "oracle": t_orc, "transitive": t_tr, "lifecycle": t_life, "evidenced_any": t_any, "unevidenced": t_active - t_any},
        "files": files,
        "non_claims": ["NEG.EVIDENCE.NOT_A_BEHAVIOR_PROOF", "NEG.EVIDENCE.TOKEN_REFERENCE_ONLY", "NEG.EVIDENCE.TRANSITIVE_IS_ONE_HOP"],
    })
}

fn render_md(b: &Value) -> String {
    let t = &b["totals"];
    let mut l = vec![
        "<!-- generated by `gnucobol-rs-port-index evidence` — do not edit by hand -->".to_string(),
        String::new(),
        "# FUNCTION-EVIDENCE — does each ported function carry direct or classified evidence?".to_string(),
        String::new(),
        "> `LIBCOB-PARITY` proves a real Rust `fn` exists. This map classifies the evidence behind it:".to_string(),
        "> **DIRECT** = the fn name is referenced by a unit test, a Kani proof, a fuzz target, or an".to_string(),
        "> oracle-sweep mirror. **TRANSITIVE** = it is *called* (per `CLANG-AST-PARITY`) by a directly-".to_string(),
        "> covered function, so it is exercised through that public oracle/test path. **LIFECYCLE** = an".to_string(),
        "> init/exit/free/alloc/cache routine (RAII / deliberate no-op). **UNEVIDENCED** = none of these — a".to_string(),
        "> false-confidence risk. (Direct evidence is a token reference; court & transitive evidence are".to_string(),
        "> classified separately; byte parity remains the per-court oracle sweeps in `lab/verify-sealed-courts.sh`.)".to_string(),
        String::new(),
        format!(
            "**Active ported: {}** · direct {} (incl. oracle-sweep {}) · transitive {} · lifecycle {} · **evidenced (any) {}** · **unevidenced {}**.",
            t["active"], t["direct"], t["oracle"], t["transitive"], t["lifecycle"], t["evidenced_any"], t["unevidenced"]
        ),
        String::new(),
        "| libcob file | active | direct | oracle | transitive | lifecycle | evidenced | unevidenced |".to_string(),
        "|---|---:|---:|---:|---:|---:|---:|---:|".to_string(),
    ];
    for f in b["files"].as_array().unwrap_or(&Vec::new()) {
        let un = f["unevidenced"].as_array().map(|a| a.len()).unwrap_or(0);
        if f["active"].as_u64().unwrap_or(0) == 0 {
            continue;
        }
        l.push(format!(
            "| `{}` | {} | {} | {} | {} | {} | {} | {} |",
            f["file"].as_str().unwrap_or(""),
            f["active"],
            f["direct"],
            f["oracle"],
            f["transitive"],
            f["lifecycle"],
            f["evidenced_any"],
            un
        ));
    }
    l.push(String::new());
    l.push("## Ported but unevidenced (no direct / transitive / lifecycle coverage)".to_string());
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
    l.push("scan of the Rust `#[cfg(test)]` / `#[cfg(kani)]` regions, the fuzz targets, the oracle-sweep".to_string());
    l.push("mirrors (`examples/*.rs` + `lab/oracle/*.sh`), and the `clang-callgraph.json` (for transitive".to_string());
    l.push("coverage). `evidence check` regenerates and diffs (anti-staleness). PORT-GOVERNANCE.3 closure".to_string());
    l.push("rule (future Tier-2 gate): a 100%-closed file must have every active fn direct / transitive /".to_string());
    l.push("lifecycle / court covered — no unclassified unevidenced active functions.".to_string());
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
        "FUNCTION-EVIDENCE: {} active, {} evidenced ({} unevidenced) — direct {} / oracle {} / transitive {} / lifecycle {}",
        t["active"], t["evidenced_any"], t["unevidenced"], t["direct"], t["oracle"], t["transitive"], t["lifecycle"]
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
    println!("FUNCTION-EVIDENCE check: fresh ({} active, {} evidenced, {} unevidenced)", t["active"], t["evidenced_any"], t["unevidenced"]);
    0
}
