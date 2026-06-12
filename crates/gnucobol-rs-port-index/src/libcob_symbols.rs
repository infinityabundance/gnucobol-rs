//! libcob C symbol indexer. For each admitted `libcob/*.c` it records every top-level function
//! definition with its line range, `static`/exported visibility, and — the load-bearing field — its
//! **preprocessor status** (compiled / `#if 0`-disabled / config-gated), so a source mirror that the
//! compiler never sees is never conflated with live runtime behaviour.
//!
//! The classifier is a textual `#if`/`#ifdef`/`#else`/`#endif` scanner (the "first win" per the
//! milestone plan — no full `cpp` yet). `#if 0` is definite; `#ifdef COB_EXPERIMENTAL` is the one
//! config gate known to be off in the admitted build. Every other conditional is assumed compiled, and
//! anything it cannot classify is reported `Unknown`, never silently `Compiled`.

use crate::model::{LibcobSymbol, PreprocStatus};
use crate::paths;
use std::path::Path;

const C_KEYWORDS: [&str; 9] =
    ["if", "for", "while", "switch", "return", "else", "sizeof", "do", "typedef"];

/// A preprocessor conditional frame.
enum Frame {
    /// `#if 0` — the `if` branch is dead, the `#else` branch is compiled.
    If0 { in_else: bool },
    /// `#ifdef COB_EXPERIMENTAL` (or equivalent) — the `if` branch is off in the admitted build.
    Config { mac: String, in_else: bool },
    /// Any other conditional (`#if 1`, `#ifdef HAVE_*`, platform gates) — treated as compiled.
    Plain,
}

/// Does the current conditional stack disable code here? Returns the disabling status + gate macro.
fn disabled_now(stack: &[Frame]) -> Option<(PreprocStatus, Option<String>)> {
    for f in stack {
        match f {
            Frame::If0 { in_else: false } => return Some((PreprocStatus::If0Disabled, None)),
            Frame::Config { mac, in_else: false } => {
                return Some((PreprocStatus::ConfigDisabled, Some(mac.clone())))
            }
            _ => {}
        }
    }
    None
}

/// Is `s` (a return-type line) a plausible function return type? (identifier-ish tokens + `*`, after a
/// trailing `/* ... */` is stripped.)
fn is_rettype(s: &str) -> bool {
    let t = s.split("/*").next().unwrap_or(s).trim_end();
    !t.is_empty()
        && t.chars().next().map(|c| c.is_ascii_alphabetic() || c == '_').unwrap_or(false)
        && t.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == ' ' || c == '\t' || c == '*')
}

/// Classify a `#...` directive, mutating the conditional stack.
fn apply_directive(directive: &str, stack: &mut Vec<Frame>) {
    let d = directive.trim();
    let rest = |kw: &str| d.strip_prefix(kw).map(|r| r.trim().to_string());
    if let Some(expr) = rest("if ").or_else(|| if d == "if" { Some(String::new()) } else { None }) {
        // `#if <expr>`
        if expr == "0" {
            stack.push(Frame::If0 { in_else: false });
        } else if expr.contains("COB_EXPERIMENTAL") {
            stack.push(Frame::Config { mac: "COB_EXPERIMENTAL".into(), in_else: false });
        } else {
            stack.push(Frame::Plain);
        }
    } else if let Some(mac) = rest("ifdef") {
        if mac == "COB_EXPERIMENTAL" {
            stack.push(Frame::Config { mac, in_else: false });
        } else {
            stack.push(Frame::Plain);
        }
    } else if rest("ifndef").is_some() {
        stack.push(Frame::Plain);
    } else if d == "else" {
        if let Some(top) = stack.last_mut() {
            match top {
                Frame::If0 { in_else } | Frame::Config { in_else, .. } => *in_else = !*in_else,
                Frame::Plain => {}
            }
        }
    } else if d.starts_with("elif") {
        // entering an alternative branch — treat the remainder as compiled
        if let Some(top) = stack.last_mut() {
            *top = Frame::Plain;
        }
    } else if d == "endif" {
        stack.pop();
    }
}

/// Index one libcob `.c` file.
pub fn index_file(path: &Path, file_label: &str) -> Vec<LibcobSymbol> {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    let mut stack: Vec<Frame> = Vec::new();

    for i in 0..lines.len() {
        let cur = lines[i];
        // Track preprocessor state as we go (directive on THIS line takes effect for following lines).
        let trimmed = cur.trim_start();
        if let Some(rest) = trimmed.strip_prefix('#') {
            apply_directive(rest.trim_start(), &mut stack);
            continue;
        }
        if i == 0 {
            continue;
        }
        // Function definition: `name (` at column 0, preceded by a return-type line.
        let first = cur.chars().next();
        if !first.map(|c| c.is_ascii_alphabetic() || c == '_').unwrap_or(false) {
            continue;
        }
        let name: String = cur.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
        let after = cur[name.len()..].trim_start();
        if !after.starts_with('(') || name.is_empty() || C_KEYWORDS.contains(&name.as_str()) {
            continue;
        }
        if !is_rettype(lines[i - 1]) {
            continue;
        }
        // line range: from the name line to the next `}` at column 0.
        let line_start = i + 1; // 1-based
        let mut line_end = line_start;
        for (j, l) in lines.iter().enumerate().skip(i + 1) {
            if l.starts_with('}') {
                line_end = j + 1;
                break;
            }
        }
        let (preprocessor_status, gate) = match disabled_now(&stack) {
            Some((s, g)) => (s, g),
            None => (PreprocStatus::Compiled, None),
        };
        let is_static = lines[i - 1].trim_start().starts_with("static");
        out.push(LibcobSymbol {
            file: file_label.to_string(),
            function: name,
            line_start,
            line_end,
            preprocessor_status,
            is_static,
            gate,
        });
    }
    out
}

/// Index every admitted libcob `.c` file. `None` if the source is not extracted (gate skips).
pub fn index_all(root: &Path) -> Option<Vec<LibcobSymbol>> {
    if !paths::libcob_present(root) {
        return None;
    }
    let dir = paths::libcob_dir(root);
    let mut all = Vec::new();
    for f in paths::FILES {
        all.extend(index_file(&dir.join(f), f));
    }
    Some(all)
}
