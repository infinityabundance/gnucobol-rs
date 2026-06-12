//! Rust port symbol indexer. Walks `crates/gnucobol-rs/src/**/*.rs` and records every real `fn`
//! definition with its module, visibility, and status (active / inactive mirror / test-only). It is
//! comment- and string-aware: a function name that appears only inside a doc comment or a string literal
//! is **not** a definition. The set of all identifier tokens (the "grep would match" set) is collected
//! separately so the joiner can flag doc-only false hits.

use crate::model::{RustStatus, RustSymbol};
use crate::paths;
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// The result of indexing the Rust port: real `fn` definitions (best status per name) and every
/// identifier token that appears anywhere in the source (including comments/strings).
pub struct RustIndex {
    pub defs: HashMap<String, RustSymbol>,
    pub all_tokens: HashSet<String>,
    pub symbols: Vec<RustSymbol>,
}

fn module_of(rel: &Path) -> String {
    let mut parts: Vec<String> = Vec::new();
    for c in rel.components() {
        parts.push(c.as_os_str().to_string_lossy().to_string());
    }
    if let Some(last) = parts.last_mut() {
        *last = last.trim_end_matches(".rs").to_string();
    }
    // lib.rs / mod.rs name the enclosing scope, not a child module.
    if parts.last().map(|s| s == "lib" || s == "mod").unwrap_or(false) {
        parts.pop();
    }
    if parts.is_empty() {
        "crate".to_string()
    } else {
        parts.join("::")
    }
}

/// Rank statuses so the "best" counterpart wins when a name has several definitions (active beats an
/// inactive mirror beats a test-only helper).
fn rank(s: RustStatus) -> u8 {
    match s {
        RustStatus::Active => 3,
        RustStatus::InactiveMirror => 2,
        RustStatus::TestOnly => 1,
        _ => 0,
    }
}

/// Index one Rust file: append real `fn` definitions and collect its identifier tokens.
fn index_file(text: &str, module: &str, file_label: &str, out: &mut Vec<RustSymbol>, toks: &mut HashSet<String>) {
    // 1. all identifier tokens over the raw text (the grep-equivalent set).
    let mut cur = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            cur.push(ch);
        } else if !cur.is_empty() {
            toks.insert(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        toks.insert(std::mem::take(&mut cur));
    }

    // 2. comment/string-aware tokenizer to find real `fn` definitions + module/test context.
    let bytes = text.as_bytes();
    let mut line = 1usize;
    let mut i = 0usize;
    // brace-scope stack: (is_test, is_cfg_gated) pushed at each `{` that opens a `mod`/`fn`/block.
    let mut test_depth = 0i32; // >0 means inside a #[cfg(test)] / `mod tests` scope
    let mut brace_stack: Vec<bool> = Vec::new(); // true = this brace opened a test scope
    // pending attributes seen since the last item.
    let mut pend_cfg_test = false;
    let mut pend_dead = false;
    let mut pend_cfg_other = false;
    // remember that the next `{` belongs to a test mod.
    let mut arm_test_brace = false;

    let n = bytes.len();
    let id_char = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    while i < n {
        let b = bytes[i];
        match b {
            b'\n' => {
                line += 1;
                i += 1;
            }
            b'/' if i + 1 < n && bytes[i + 1] == b'/' => {
                // line comment: capture attribute-like markers? no — skip to EOL.
                while i < n && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if i + 1 < n && bytes[i + 1] == b'*' => {
                i += 2;
                while i + 1 < n && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    if bytes[i] == b'\n' {
                        line += 1;
                    }
                    i += 1;
                }
                i += 2;
            }
            b'"' => {
                i += 1;
                while i < n && bytes[i] != b'"' {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    if i < n && bytes[i] == b'\n' {
                        line += 1;
                    }
                    i += 1;
                }
                i += 1;
            }
            b'#' => {
                // attribute: read to end of line, note cfg(test)/allow(dead_code)/cfg(...).
                let mut j = i;
                while j < n && bytes[j] != b'\n' {
                    j += 1;
                }
                let attr = &text[i..j];
                if attr.contains("cfg(test)") || attr.contains("cfg(all(test") {
                    pend_cfg_test = true;
                } else if attr.contains("allow(dead_code)") {
                    pend_dead = true;
                } else if attr.contains("cfg(") {
                    pend_cfg_other = true;
                }
                i = j;
            }
            b'{' => {
                brace_stack.push(arm_test_brace);
                if arm_test_brace {
                    test_depth += 1;
                }
                arm_test_brace = false;
                i += 1;
            }
            b'}' => {
                if let Some(was_test) = brace_stack.pop() {
                    if was_test {
                        test_depth -= 1;
                    }
                }
                i += 1;
            }
            _ if id_char(b) => {
                let start = i;
                while i < n && id_char(bytes[i]) {
                    i += 1;
                }
                let word = &text[start..i];
                match word {
                    "mod" => {
                        // peek the mod name; arm the next `{` as a test scope if cfg(test) or name==tests.
                        let mut k = i;
                        while k < n && (bytes[k] == b' ' || bytes[k] == b'\t') {
                            k += 1;
                        }
                        let ms = k;
                        while k < n && id_char(bytes[k]) {
                            k += 1;
                        }
                        let mname = &text[ms..k];
                        if mname == "tests" || pend_cfg_test {
                            arm_test_brace = true;
                        }
                        pend_cfg_test = false;
                        pend_dead = false;
                        pend_cfg_other = false;
                    }
                    "fn" => {
                        // next identifier is the function name.
                        let mut k = i;
                        while k < n && (bytes[k] == b' ' || bytes[k] == b'\t') {
                            k += 1;
                        }
                        let ns = k;
                        while k < n && id_char(bytes[k]) {
                            k += 1;
                        }
                        if k > ns {
                            let fname = text[ns..k].to_string();
                            // visibility: scan a little back for `pub`.
                            let back = &text[start.saturating_sub(12)..start];
                            let is_pub = back.contains("pub");
                            let status = if test_depth > 0 || pend_cfg_test {
                                RustStatus::TestOnly
                            } else if pend_dead || pend_cfg_other {
                                RustStatus::InactiveMirror
                            } else {
                                RustStatus::Active
                            };
                            out.push(RustSymbol {
                                function: fname,
                                module: module.to_string(),
                                file: file_label.to_string(),
                                line,
                                is_pub,
                                status,
                            });
                        }
                        pend_cfg_test = false;
                        pend_dead = false;
                        pend_cfg_other = false;
                        i = k;
                    }
                    _ => {
                        // a non-item identifier clears nothing; attributes persist until the item.
                    }
                }
            }
            _ => {
                i += 1;
            }
        }
    }
    let _ = file_label;
}

/// Walk `crates/gnucobol-rs/src` and build the Rust index.
pub fn index(root: &Path) -> RustIndex {
    let src = paths::rust_src_dir(root);
    let mut symbols = Vec::new();
    let mut all_tokens = HashSet::new();

    fn walk(dir: &Path, base: &Path, symbols: &mut Vec<RustSymbol>, toks: &mut HashSet<String>) {
        if let Ok(rd) = std::fs::read_dir(dir) {
            let mut entries: Vec<_> = rd.flatten().map(|e| e.path()).collect();
            entries.sort();
            for p in entries {
                if p.is_dir() {
                    walk(&p, base, symbols, toks);
                } else if p.extension().map(|x| x == "rs").unwrap_or(false) {
                    let text = std::fs::read_to_string(&p).unwrap_or_default();
                    let rel = p.strip_prefix(base).unwrap_or(&p);
                    let module = module_of(rel);
                    let label = format!("crates/gnucobol-rs/src/{}", rel.display());
                    index_file(&text, &module, &label, symbols, toks);
                }
            }
        }
    }
    walk(&src, &src, &mut symbols, &mut all_tokens);

    // best definition per name.
    let mut defs: HashMap<String, RustSymbol> = HashMap::new();
    for s in &symbols {
        match defs.get(&s.function) {
            Some(prev) if rank(prev.status) >= rank(s.status) => {}
            _ => {
                defs.insert(s.function.clone(), s.clone());
            }
        }
    }
    RustIndex { defs, all_tokens, symbols }
}
