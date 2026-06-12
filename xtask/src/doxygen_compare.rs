//! DOXYGEN-PARITY — the C-vs-Rust function-coverage compare the port uses to *know a libcob file is
//! done*. Doxygen's preprocessed C parse (`lab/doxygen/out/xml/<file>_8c.xml`) is the authoritative
//! function inventory — it catches definitions the line-based awk inventory (`parity.rs`) can miss
//! (comment-decorated return types, macro-wrapped signatures). For each libcob `.c` file this tool lists
//! every function doxygen found, checks each has a named Rust counterpart in `crates/gnucobol-rs/src`,
//! and flags any divergence between the doxygen inventory and the awk inventory.
//!
//! `generate` writes `reports/doxygen-parity.json` + `DOXYGEN-PARITY.md`. `check` is the anti-staleness +
//! "did we miss anything" gate: regenerate-and-diff, and FAIL if any file the awk parity reports as
//! complete still has a doxygen-found function with no Rust counterpart. Source-gated on the C XML being
//! present (run `doxygen lab/doxygen/Doxyfile` first; the guard does).

use serde_json::{json, Value};
use std::collections::HashSet;
use std::path::Path;

const XML_REL: &str = "lab/doxygen/out/xml";
const JSON_REL: &str = "reports/doxygen-parity.json";
const MD_REL: &str = "DOXYGEN-PARITY.md";

/// The 13 libcob translation units, in port order (kept in sync with `parity.rs`).
const FILES: [&str; 13] = [
    "numeric.c",
    "move.c",
    "strings.c",
    "intrinsic.c",
    "cconv.c",
    "termio.c",
    "screenio.c",
    "call.c",
    "fileio.c",
    "mlio.c",
    "reportio.c",
    "common.c",
    "cobgetopt.c",
];

/// `foo.c` -> `lab/doxygen/out/xml/foo_8c.xml` (doxygen's file-id escaping).
fn xml_path(root: &str, cfile: &str) -> std::path::PathBuf {
    let base = cfile.strip_suffix(".c").unwrap_or(cfile);
    Path::new(root).join(XML_REL).join(format!("{base}_8c.xml"))
}

/// Extract the function names doxygen recorded for a `.c` file: every `<memberdef kind="function">`'s
/// `<name>`. Returns `None` if the XML is absent (the gate then skips, source-gated).
fn doxygen_functions(path: &Path) -> Option<Vec<String>> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let mut rest = text.as_str();
    let needle = "<memberdef kind=\"function\"";
    while let Some(i) = rest.find(needle) {
        rest = &rest[i + needle.len()..];
        let end = rest.find("</memberdef>").unwrap_or(rest.len());
        let blk = &rest[..end];
        if let Some(ns) = blk.find("<name>") {
            if let Some(ne) = blk[ns + 6..].find("</name>") {
                let name = blk[ns + 6..ns + 6 + ne].to_string();
                if seen.insert(name.clone()) {
                    out.push(name);
                }
            }
        }
        rest = &rest[end..];
    }
    out.sort();
    Some(out)
}

/// Every identifier token in the shipped Rust library source (same rule as `parity.rs`).
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

/// Build the doxygen-parity model. `None` if the C XML directory is absent (gate skips).
fn build(root: &str) -> Option<Value> {
    if !xml_path(root, "numeric.c").exists() {
        return None;
    }
    let toks = rust_tokens(root);
    let mut files = Vec::new();
    let (mut tot, mut tot_p) = (0u64, 0u64);
    for f in FILES {
        let Some(fns) = doxygen_functions(&xml_path(root, f)) else {
            continue;
        };
        let total = fns.len() as u64;
        let missing: Vec<&String> = fns.iter().filter(|n| !toks.contains(*n)).collect();
        let ported = total - missing.len() as u64;
        tot += total;
        tot_p += ported;
        let pct = if total > 0 {
            (ported as f64) * 1000.0 / (total as f64)
        } else {
            0.0
        };
        files.push(json!({
            "file": f,
            "doxygen_functions": total,
            "ported": ported,
            "parity_pct": pct.round() / 10.0,
            "missing": missing,
        }));
    }
    let tpct = if tot > 0 {
        (tot_p as f64) * 1000.0 / (tot as f64)
    } else {
        0.0
    };
    Some(json!({
        "schema": "libcob-doxygen-parity-v1",
        "court": "DOXYGEN.PARITY.1",
        "oracle": "GnuCOBOL 3.2.0 libcob, doxygen preprocessed C parse (lab/doxygen/Doxyfile)",
        "xml_root": XML_REL,
        "method": "Authoritative inventory: every <memberdef kind=\"function\"> doxygen recorded per libcob .c file. A function counts PORTED when its exact name appears as a whole word in crates/gnucobol-rs/src/**/*.rs. Doxygen's parse catches definitions the line-based awk inventory (LIBCOB.PARITY.1) can miss; this is the 'did we miss anything' cross-check.",
        "files": files,
        "totals": {"doxygen_functions": tot, "ported": tot_p, "parity_pct": tpct.round() / 10.0},
    }))
}

fn render_md(b: &Value) -> String {
    let mut l = vec![
        "<!-- generated by `cargo run -p xtask -- doxygen-compare generate` — do not edit by hand -->".to_string(),
        String::new(),
        "# LIBCOB doxygen parity — C-vs-Rust function coverage".to_string(),
        String::new(),
        "> Authoritative function inventory from **doxygen's preprocessed C parse** of the admitted".to_string(),
        "> GnuCOBOL 3.2 `libcob/*.c` (`lab/doxygen/out/xml`), cross-checked against the Rust port. This is".to_string(),
        "> the \"did we miss anything\" view: doxygen catches definitions the line-based awk inventory".to_string(),
        "> (`LIBCOB-PARITY.md`) can miss. A file is **done** when its `missing` list is empty.".to_string(),
        String::new(),
        "> **Counting note.** Doxygen counts functions that survive the C preprocessor, so its per-file".to_string(),
        "> totals are *lower* than `LIBCOB-PARITY.md` for files with `#if 0` / `COB_EXPERIMENTAL` blocks".to_string(),
        "> (e.g. `numeric.c`: 94 here vs 104 there). Those disabled functions are still ported 1:1 and".to_string(),
        "> covered; this view simply does not see them. The two are complementary, not contradictory.".to_string(),
        String::new(),
        "| libcob file | doxygen fns | ported | coverage | missing |".to_string(),
        "|---|---:|---:|---:|---|".to_string(),
    ];
    for f in b["files"].as_array().unwrap() {
        let missing: Vec<&str> = f["missing"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|m| m.as_str())
            .collect();
        let miss = if missing.is_empty() {
            "—".to_string()
        } else {
            format!("`{}`", missing.join("`, `"))
        };
        l.push(format!(
            "| `{}` | {} | {} | {:.1}% | {} |",
            f["file"].as_str().unwrap(),
            f["doxygen_functions"],
            f["ported"],
            f["parity_pct"].as_f64().unwrap_or(0.0),
            miss,
        ));
    }
    let t = &b["totals"];
    l.push(format!(
        "| **total** | **{}** | **{}** | **{:.1}%** | |",
        t["doxygen_functions"],
        t["ported"],
        t["parity_pct"].as_f64().unwrap_or(0.0)
    ));
    l.push(String::new());
    l.push("## How this is produced (reproducible)".to_string());
    l.push(String::new());
    l.push(
        "The C-vs-Rust function diff is generated end-to-end in Rust — no Python, no hand edits:"
            .to_string(),
    );
    l.push(String::new());
    l.push(
        "1. **Doxygen parses the pinned libcob C** into a machine-readable inventory:".to_string(),
    );
    l.push("   `doxygen lab/doxygen/Doxyfile-c-xml` → `lab/doxygen/out/xml/*_8c.xml` (XML-only, ~0.2 s,".to_string());
    l.push("   `EXTRACT_ALL`/`EXTRACT_STATIC` so every static helper is listed; the GMP path is predefined".to_string());
    l.push(
        "   so the real code is seen). The browsable HTML + call graphs are the separate"
            .to_string(),
    );
    l.push(
        "   `lab/doxygen/Doxyfile`; the native-Rust-port doxygen is `lab/doxygen/Doxyfile-rust`."
            .to_string(),
    );
    l.push(
        "2. **`xtask doxygen-compare generate`** (Rust: `xtask/src/doxygen_compare.rs`) reads each"
            .to_string(),
    );
    l.push("   `<memberdef kind=\"function\">` doxygen recorded, checks each name appears as a whole word in".to_string());
    l.push("   `crates/gnucobol-rs/src/**/*.rs`, and writes `reports/doxygen-parity.json` + this file.".to_string());
    l.push("3. **`xtask doxygen-compare check`** is the gate: it regenerates and diffs (anti-staleness) and".to_string());
    l.push(
        "   FAILs if any file the awk parity (`LIBCOB-PARITY.md`) reports complete still has a"
            .to_string(),
    );
    l.push("   doxygen-found function with no Rust counterpart (the \"did we miss anything\" guarantee).".to_string());
    l.push("   It runs in both `lab/verify-sealed-courts.sh` and the docs staleness gate `lab/check-docs.sh`.".to_string());
    l.push(String::new());
    l.push("Regenerate everything with `bash lab/doxygen/refresh-rust.sh`.".to_string());
    l.push(String::new());
    l.join("\n") + "\n"
}

/// xtask dispatch: `doxygen-compare {generate|check}`.
pub fn run(cmd: &str, root: &str) -> i32 {
    match cmd {
        "generate" => {
            generate(root);
            0
        }
        _ => check(root),
    }
}

/// `doxygen-compare generate` — write the JSON + Markdown coverage report.
pub fn generate(root: &str) {
    let Some(b) = build(root) else {
        eprintln!(
            "DOXYGEN.PARITY: C XML absent ({XML_REL}); run `doxygen lab/doxygen/Doxyfile` first"
        );
        return;
    };
    std::fs::write(
        Path::new(root).join(JSON_REL),
        serde_json::to_string_pretty(&b).unwrap() + "\n",
    )
    .unwrap();
    std::fs::write(Path::new(root).join(MD_REL), render_md(&b)).unwrap();
    let t = &b["totals"];
    println!(
        "DOXYGEN.PARITY: {}/{} functions ({:.1}%) — authoritative C parse",
        t["ported"],
        t["doxygen_functions"],
        t["parity_pct"].as_f64().unwrap_or(0.0)
    );
}

/// `doxygen-compare check` — anti-staleness (regenerate-and-diff) + "did we miss anything". Source-gated:
/// skips cleanly when the C XML is absent. FAILS (exit 1) on a stale committed doc, or when any libcob
/// file the awk parity reports complete still has a doxygen-found function with no Rust counterpart.
pub fn check(root: &str) -> i32 {
    let Some(b) = build(root) else {
        println!("DOXYGEN.PARITY check: C XML absent — skipped (source-gated)");
        return 0;
    };
    // 1. anti-staleness: the committed JSON/MD must equal a fresh regeneration.
    let fresh_json = serde_json::to_string_pretty(&b).unwrap() + "\n";
    let fresh_md = render_md(&b);
    let cur_json = std::fs::read_to_string(Path::new(root).join(JSON_REL)).unwrap_or_default();
    let cur_md = std::fs::read_to_string(Path::new(root).join(MD_REL)).unwrap_or_default();
    let mut bad = false;
    if cur_json != fresh_json {
        eprintln!("DOXYGEN.PARITY STALE: {JSON_REL} != regeneration (run `xtask doxygen-compare generate`)");
        bad = true;
    }
    if cur_md != fresh_md {
        eprintln!("DOXYGEN.PARITY STALE: {MD_REL} != regeneration");
        bad = true;
    }

    // 2. "did we miss anything": cross-check against the awk parity. For every file the awk parity reports
    // as complete (missing == []), the authoritative doxygen inventory must also be fully covered.
    if let Ok(pj) = std::fs::read_to_string(Path::new(root).join("reports/libcob-parity.json")) {
        if let Ok(p) = serde_json::from_str::<Value>(&pj) {
            let awk_complete: HashSet<&str> = p["files"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter(|f| {
                            f["missing"]
                                .as_array()
                                .map(|m| m.is_empty())
                                .unwrap_or(false)
                        })
                        .filter_map(|f| f["file"].as_str())
                        .collect()
                })
                .unwrap_or_default();
            for f in b["files"].as_array().unwrap() {
                let name = f["file"].as_str().unwrap();
                let missing: Vec<&str> = f["missing"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|m| m.as_str())
                    .collect();
                if awk_complete.contains(name) && !missing.is_empty() {
                    eprintln!(
                        "DOXYGEN.PARITY MISS: {name} is awk-complete but doxygen finds un-ported fns: {missing:?}"
                    );
                    bad = true;
                }
            }
        }
    }

    if bad {
        return 1;
    }
    println!("DOXYGEN.PARITY check: fresh; no awk-missed functions in completed files");
    0
}
