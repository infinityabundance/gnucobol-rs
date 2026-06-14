//! CLANG-AST-PARITY (`PORT-GOVERNANCE.2`): an independent C-side inventory from clang's AST.
//!
//! Doxygen says *which* functions exist; the typed port-index joins them to Rust. Clang's AST adds the
//! third view — for each libcob translation unit it records every function **definition**'s storage
//! class, signature, and the functions it **calls** (the C callgraph). That turns "did we miss a fn?"
//! into "what does this fn depend on?" — the map needed to port `fileio.c`/`common.c`/`call.c` safely.
//!
//! `clang-index generate` runs `clang -Xclang -ast-dump=json` per file (source- and clang-gated), writes
//! `reports/port-index/clang-functions.json` + `clang-callgraph.json`, and renders `CLANG-AST-PARITY.md`
//! (cross-referenced with the Rust port). `check` regenerates the rendered map and diffs it.

use crate::paths;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

fn clang_available() -> bool {
    Command::new("clang").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

/// Dump the clang JSON AST for one libcob `.c` file (with the admitted source's include roots).
fn dump_ast(root: &Path, file: &str) -> Option<Value> {
    let l = paths::libcob_dir(root);
    let base = l.parent()?; // lab/admit/gnucobol-3.2
    let out = Command::new("clang")
        .args(["-Xclang", "-ast-dump=json", "-fsyntax-only"])
        .arg("-I").arg(base)
        .arg("-I").arg(&l)
        .arg(l.join(file))
        .output()
        .ok()?;
    serde_json::from_slice(&out.stdout).ok()
}

/// Does a node's `loc`/`range` carry a `file:` that ends with `target`? (clang omits `file` when it is
/// unchanged from the previous node in source order, so we update a running current-file in pre-order.)
fn node_file(n: &Value) -> Option<&str> {
    n.get("loc")
        .and_then(|l| l.get("file"))
        .or_else(|| n.get("range").and_then(|r| r.get("begin")).and_then(|b| b.get("file")))
        .and_then(Value::as_str)
}

/// Collect the names of functions CALLed within a node's subtree (the callees of one definition).
fn callees(n: &Value, out: &mut BTreeSet<String>) {
    let mut stack = vec![n];
    while let Some(node) = stack.pop() {
        if node.get("kind").and_then(Value::as_str) == Some("DeclRefExpr") {
            if let Some(rd) = node.get("referencedDecl") {
                if rd.get("kind").and_then(Value::as_str) == Some("FunctionDecl") {
                    if let Some(name) = rd.get("name").and_then(Value::as_str) {
                        out.insert(name.to_string());
                    }
                }
            }
        }
        if let Some(inner) = node.get("inner").and_then(Value::as_array) {
            for c in inner {
                stack.push(c);
            }
        }
    }
}

/// One function definition extracted from a file's AST.
fn extract(ast: &Value, target: &str) -> Vec<Value> {
    let mut funcs = Vec::new();
    // pre-order DFS over the top-level TranslationUnitDecl children, tracking the current source file.
    let mut cur = String::new();
    // stack holds children in reverse so the leftmost is processed first (true pre-order).
    let empty = Vec::new();
    let top = ast.get("inner").and_then(Value::as_array).unwrap_or(&empty);
    let mut stack: Vec<&Value> = top.iter().rev().collect();
    while let Some(node) = stack.pop() {
        if let Some(f) = node_file(node) {
            cur = f.to_string();
        }
        if node.get("kind").and_then(Value::as_str) == Some("FunctionDecl") && cur.ends_with(target) {
            // a definition has a CompoundStmt body among its inner nodes.
            let has_body = node
                .get("inner")
                .and_then(Value::as_array)
                .map(|a| a.iter().any(|c| c.get("kind").and_then(Value::as_str) == Some("CompoundStmt")))
                .unwrap_or(false);
            if has_body {
                if let Some(name) = node.get("name").and_then(Value::as_str) {
                    let storage = node.get("storageClass").and_then(Value::as_str).unwrap_or("extern");
                    let sig = node.get("type").and_then(|t| t.get("qualType")).and_then(Value::as_str).unwrap_or("");
                    let mut cs = BTreeSet::new();
                    callees(node, &mut cs);
                    cs.remove(name); // drop self-recursion noise
                    funcs.push(json!({
                        "function": name,
                        "storage": storage,
                        "signature": sig,
                        "callees": cs.into_iter().collect::<Vec<_>>(),
                    }));
                }
            }
            // do not descend into a definition's body for *file* tracking (its callees are scanned above)
            continue;
        }
        if let Some(inner) = node.get("inner").and_then(Value::as_array) {
            for c in inner.iter().rev() {
                stack.push(c);
            }
        }
    }
    funcs.sort_by(|a, b| a["function"].as_str().unwrap_or("").cmp(b["function"].as_str().unwrap_or("")));
    funcs.dedup_by(|a, b| a["function"] == b["function"]);
    funcs
}

/// Build the full clang index across the libcob files. Returns None when clang or the source is absent.
fn build(root: &Path) -> Option<Value> {
    if !paths::libcob_present(root) || !clang_available() {
        return None;
    }
    // the Rust port's ported C-function names (to cross-reference: which clang fns have a Rust fn).
    let parity = std::fs::read_to_string(root.join(paths::PORT_INDEX_DIR).join("parity-detailed.json"))
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .unwrap_or(Value::Null);
    let mut files = Vec::new();
    let (mut tot_fns, mut tot_static, mut tot_edges, mut tot_ported) = (0u64, 0u64, 0u64, 0u64);
    for file in paths::FILES {
        let Some(ast) = dump_ast(root, file) else { continue };
        let fns = extract(&ast, file);
        // ported set for this file from parity-detailed.
        let ported: BTreeSet<String> = parity
            .as_array()
            .and_then(|rows| rows.iter().find(|r| r["file"].as_str().map(|s| s.ends_with(file)).unwrap_or(false)))
            .and_then(|r| r["fns"].as_array())
            .map(|a| {
                a.iter()
                    .filter(|f| matches!(f["rust_status"].as_str(), Some("active") | Some("inactive_mirror")))
                    .filter_map(|f| f["function"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        let nstatic = fns.iter().filter(|f| f["storage"].as_str() == Some("static")).count() as u64;
        let edges: u64 = fns.iter().map(|f| f["callees"].as_array().map(|a| a.len()).unwrap_or(0) as u64).sum();
        let nported = fns.iter().filter(|f| ported.contains(f["function"].as_str().unwrap_or(""))).count() as u64;
        tot_fns += fns.len() as u64;
        tot_static += nstatic;
        tot_edges += edges;
        tot_ported += nported;
        files.push(json!({
            "file": file,
            "definitions": fns.len(),
            "static": nstatic,
            "call_edges": edges,
            "ported": nported,
            "functions": fns,
        }));
    }
    Some(json!({
        "schema": "gnurust-clang-ast-index-v1",
        "court": "PORT-GOVERNANCE.2",
        "doctrine": "an independent clang AST view of the admitted libcob: every function DEFINITION's storage class, signature, and callees (the C callgraph). Cross-referenced with the typed Rust port. Structure/dependency map, not a behaviour proof.",
        "totals": {"definitions": tot_fns, "static": tot_static, "call_edges": tot_edges, "ported": tot_ported},
        "files": files,
    }))
}

fn render_md(b: &Value) -> String {
    let t = &b["totals"];
    let mut l = vec![
        "<!-- generated by `gnucobol-rs-port-index clang-index` — do not edit by hand -->".to_string(),
        String::new(),
        "# CLANG-AST-PARITY — the C structure/dependency view".to_string(),
        String::new(),
        "> Doxygen says which functions exist; the typed port-index joins them to Rust. This third view is".to_string(),
        "> clang's AST: per file, every function **definition**'s storage class, signature, and the functions".to_string(),
        "> it **calls** (the C callgraph). It answers \"what does this function depend on?\" — the map for".to_string(),
        "> porting the entangled files. Structure/dependency only; behaviour stays the per-court oracle sweeps.".to_string(),
        String::new(),
        format!(
            "**Definitions indexed: {}** (static {}) · call edges {} · with a Rust port {}.",
            t["definitions"], t["static"], t["call_edges"], t["ported"]
        ),
        String::new(),
        "| libcob file | definitions | static | call edges | ported |".to_string(),
        "|---|---:|---:|---:|---:|".to_string(),
    ];
    for f in b["files"].as_array().unwrap_or(&Vec::new()) {
        l.push(format!(
            "| `{}` | {} | {} | {} | {} |",
            f["file"].as_str().unwrap_or(""),
            f["definitions"],
            f["static"],
            f["call_edges"],
            f["ported"]
        ));
    }
    l.push(String::new());
    l.push("## How this is produced (reproducible)".to_string());
    l.push(String::new());
    l.push("`gnucobol-rs-port-index clang-index generate` runs `clang -Xclang -ast-dump=json -fsyntax-only`".to_string());
    l.push("per libcob `.c` (include roots = the admitted source tree), extracts each file's own function".to_string());
    l.push("definitions + callees, and writes `reports/port-index/clang-functions.json` +".to_string());
    l.push("`clang-callgraph.json`. `clang-index check` regenerates the rendered map and diffs it. Source-".to_string());
    l.push("and clang-gated: absent clang or libcob source skips cleanly.".to_string());
    l.join("\n") + "\n"
}

pub fn generate(root: &Path) -> i32 {
    let Some(b) = build(root) else {
        println!("CLANG-AST: clang or admitted libcob source absent — generate skipped");
        return 0;
    };
    let dir = root.join(paths::PORT_INDEX_DIR);
    let _ = std::fs::create_dir_all(&dir);
    // functions.json = the full per-file definitions; callgraph.json = just the edges.
    let _ = std::fs::write(dir.join("clang-functions.json"), serde_json::to_string_pretty(&b).unwrap_or_default() + "\n");
    let cg: Value = json!({
        "schema": "gnurust-clang-callgraph-v1",
        "files": b["files"].as_array().unwrap_or(&Vec::new()).iter().map(|f| json!({
            "file": f["file"],
            "edges": f["functions"].as_array().unwrap_or(&Vec::new()).iter().map(|fn_| json!({
                "caller": fn_["function"], "callees": fn_["callees"]
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    });
    let _ = std::fs::write(dir.join("clang-callgraph.json"), serde_json::to_string_pretty(&cg).unwrap_or_default() + "\n");
    let _ = std::fs::write(root.join("CLANG-AST-PARITY.md"), render_md(&b));
    let t = &b["totals"];
    println!("CLANG-AST-PARITY: {} definitions, {} static, {} call edges, {} ported", t["definitions"], t["static"], t["call_edges"], t["ported"]);
    0
}

pub fn check(root: &Path) -> i32 {
    let Some(b) = build(root) else {
        println!("CLANG-AST check: clang/source absent — skipped");
        return 0;
    };
    let want = render_md(&b);
    let have = std::fs::read_to_string(root.join("CLANG-AST-PARITY.md")).unwrap_or_default();
    if want != have {
        println!("CLANG-AST.DRIFT: CLANG-AST-PARITY.md is stale — run `gnucobol-rs-port-index clang-index generate`");
        return 1;
    }
    println!("CLANG-AST check: fresh ({} definitions)", b["totals"]["definitions"]);
    0
}
