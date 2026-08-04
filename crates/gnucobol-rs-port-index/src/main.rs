//! `gnucobol-rs-port-index` — the single port-governance tool for the gnucobol-rs libcob port. It
//! replaces grep name-matching with typed C↔Rust symbol parity: the admitted libcob C is parsed into a
//! symbol index carrying **preprocessor status** (compiled / `#if 0` / config-gated), the Rust port is
//! parsed into a **real-`fn`** index (comment/string-aware, so a doc-comment mention never counts as a
//! port), and the two are joined into the `LIBCOB-PARITY.md` scoreboard + machine indexes.
//!
//! PORT-INDEX.1 milestone subcommands: `libcob-symbols`, `rust-symbols`, `parity`, `all`, `check`.
//! Run from the repo root (cwd) or set `GNURUST_ROOT`.

#![forbid(unsafe_code)]

mod ccvs85;
mod clang_index;
mod corpus_atlas;
mod evidence;
mod libcob_symbols;
mod model;
mod parity;
mod paths;
mod rust_symbols;

use std::path::Path;

const MD_REL: &str = "LIBCOB-PARITY.md";
const LEGACY_JSON_REL: &str = "reports/libcob-parity.json";
const RECEIPT_REL: &str = "reports/provenance/port-index-1-receipt.md";

fn write_json(path: &Path, v: &serde_json::Value) {
    let _ = std::fs::create_dir_all(path.parent().unwrap());
    let _ = std::fs::write(path, serde_json::to_string_pretty(v).unwrap() + "\n");
}

/// Generate every artifact. Returns false if the admitted libcob source is absent (source-gated).
fn generate(root: &Path) -> bool {
    let Some((rows, rust)) = parity::build_rows(root) else {
        return false;
    };
    let c_syms = libcob_symbols::index_all(root).unwrap_or_default();
    let idx_dir = root.join(paths::PORT_INDEX_DIR);
    let _ = std::fs::create_dir_all(&idx_dir);

    write_json(
        &idx_dir.join("libcob-symbols.json"),
        &serde_json::to_value(&c_syms).unwrap(),
    );
    write_json(
        &idx_dir.join("rust-symbols.json"),
        &serde_json::to_value(&rust.symbols).unwrap(),
    );
    write_json(
        &idx_dir.join("parity-detailed.json"),
        &parity::detailed_json(&rows),
    );
    write_json(&root.join(LEGACY_JSON_REL), &parity::legacy_json(&rows));
    let _ = std::fs::write(root.join(MD_REL), parity::render_md(&rows));
    let _ = std::fs::write(root.join(RECEIPT_REL), parity::render_receipt(&rows));

    let tot_src: usize = rows.iter().map(|r| r.source_funcs).sum();
    let tot_compiled: usize = rows.iter().map(|r| r.compiled).sum();
    let tot_gap: usize = rows.iter().map(|r| r.gap.len()).sum();
    println!(
        "PORT-INDEX.1: {tot_src} source fns, {tot_compiled} compiled, {} compiled with a real Rust fn ({} gap)",
        tot_compiled - tot_gap,
        tot_gap
    );
    true
}

/// Anti-staleness gate: regenerate in-memory and diff against the committed `LIBCOB-PARITY.md` +
/// `reports/libcob-parity.json`. Source-gated. Exit code 1 on drift.
fn check(root: &Path) -> i32 {
    let Some((rows, _rust)) = parity::build_rows(root) else {
        println!("PORT-INDEX.1 check: admitted libcob source absent — skipped (source-gated)");
        return 0;
    };
    let fresh_md = parity::render_md(&rows);
    let fresh_json = serde_json::to_string_pretty(&parity::legacy_json(&rows)).unwrap() + "\n";
    let fresh_receipt = parity::render_receipt(&rows);
    let cur_md = std::fs::read_to_string(root.join(MD_REL)).unwrap_or_default();
    let cur_json = std::fs::read_to_string(root.join(LEGACY_JSON_REL)).unwrap_or_default();
    let cur_receipt = std::fs::read_to_string(root.join(RECEIPT_REL)).unwrap_or_default();
    let mut bad = false;
    if cur_md != fresh_md {
        eprintln!(
            "PORT-INDEX.STALE: {MD_REL} != regeneration (run `gnucobol-rs-port-index parity`)"
        );
        bad = true;
    }
    if cur_json != fresh_json {
        eprintln!("PORT-INDEX.STALE: {LEGACY_JSON_REL} != regeneration");
        bad = true;
    }
    if cur_receipt != fresh_receipt {
        eprintln!("PORT-INDEX.STALE: {RECEIPT_REL} scoreboard != live parity (run `gnucobol-rs-port-index parity`)");
        bad = true;
    }
    if bad {
        return 1;
    }
    println!("PORT-INDEX.1 check: parity map fresh (typed C↔Rust symbol join)");
    0
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("all");
    let root = paths::root();

    let code = match cmd {
        "libcob-symbols" => {
            match libcob_symbols::index_all(&root) {
                Some(s) => {
                    let dir = root.join(paths::PORT_INDEX_DIR);
                    write_json(
                        &dir.join("libcob-symbols.json"),
                        &serde_json::to_value(&s).unwrap(),
                    );
                    println!("libcob-symbols: {} C functions indexed", s.len());
                }
                None => println!("libcob-symbols: admitted source absent — skipped"),
            }
            0
        }
        "rust-symbols" => {
            let r = rust_symbols::index(&root);
            let dir = root.join(paths::PORT_INDEX_DIR);
            write_json(
                &dir.join("rust-symbols.json"),
                &serde_json::to_value(&r.symbols).unwrap(),
            );
            println!(
                "rust-symbols: {} Rust fn definitions indexed",
                r.symbols.len()
            );
            0
        }
        "parity" | "all" | "generate" => {
            if !generate(&root) {
                println!("PORT-INDEX.1: admitted libcob source absent — generate skipped");
            }
            0
        }
        "check" => check(&root),
        "ccvs85" => {
            let sub = args.get(2).map(String::as_str).unwrap_or("");
            let flag = |name: &str| -> Option<std::path::PathBuf> {
                args.iter()
                    .position(|a| a == name)
                    .and_then(|i| args.get(i + 1))
                    .map(std::path::PathBuf::from)
            };
            match sub {
                "ingest" => ccvs85::ingest(&root, flag("--input"), flag("--out")),
                "check" => ccvs85::check(&root),
                _ => {
                    eprintln!("ccvs85: use `ccvs85 ingest [--input <.Z>] [--out <dir>]` or `ccvs85 check`");
                    2
                }
            }
        }
        "corpus-atlas" => {
            let sub = args.get(2).map(String::as_str).unwrap_or("");
            match sub {
                "generate" => corpus_atlas::generate(&root),
                "check" => corpus_atlas::check(&root),
                _ => {
                    eprintln!("corpus-atlas: use `corpus-atlas generate` or `corpus-atlas check`");
                    2
                }
            }
        }
        "evidence" => {
            let sub = args.get(2).map(String::as_str).unwrap_or("");
            match sub {
                "generate" => evidence::generate(&root),
                "check" => evidence::check(&root),
                _ => {
                    eprintln!("evidence: use `evidence generate` or `evidence check`");
                    2
                }
            }
        }
        "clang-index" => {
            let sub = args.get(2).map(String::as_str).unwrap_or("");
            match sub {
                "generate" => clang_index::generate(&root),
                "check" => clang_index::check(&root),
                _ => {
                    eprintln!("clang-index: use `clang-index generate` or `clang-index check`");
                    2
                }
            }
        }
        other => {
            eprintln!("unknown subcommand '{other}' (use: libcob-symbols | rust-symbols | parity | all | check | ccvs85)");
            2
        }
    };
    std::process::exit(code);
}
