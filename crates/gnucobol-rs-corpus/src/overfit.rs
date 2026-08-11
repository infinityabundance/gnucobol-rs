//! Phase 10.4 — automated overfitting-indicator checks (read-only over the repo + reports).
//!
//! Scans the candidate source (`crates/gnucobol-rs/src/`, never modified) for the indicators in
//! the spec: hardcoded corpus program IDs / test names, hardcoded corpus source lines, absolute
//! host paths, and embedded oracle-output hashes; plus a disproportionate-success comparison of
//! the DEVELOPMENT vs HELD_OUT_EVALUATION accept rates. The `overfit` command works even when the
//! X-COBOL corpus root is absent: checks that cannot run (no corpus sources to compare against)
//! degrade to INFO with the reason recorded — they never fail the gate on missing data.

use crate::heldout::{load_xcobol_programs, XcobolRow};
use serde::Serialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// The candidate crate's source directory (scanned read-only).
pub const CANDIDATE_SRC_DIR: &str = "crates/gnucobol-rs/src";

/// Absolute-host-path prefixes flagged by the path-independence check.
pub const HOST_PATH_PREFIXES: [&str; 4] = ["/home/", "/run/media/", "/media/", "/mnt/"];

/// The SHA-256 of empty output. Excluded from the oracle-output-hash hits with a documented
/// reason: it is trivially the hash of zero bytes, appears in every corpus report (empty stdout
/// of not-run units) and in the candidate's sha256 self-test vectors; its presence is not
/// evidence of hardcoding corpus outputs.
pub const EMPTY_OUTPUT_SHA: &str =
    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

/// Max unique corpus source lines fed to the substring pass (bounds the scan cost).
pub const MAX_CORPUS_LINES_SUBSTRING: usize = 5_000;

/// Max X-COBOL rows sampled for the corpus-source-line set (deterministic: first rows by id).
pub const MAX_XCOBOL_ROWS_SAMPLED: usize = 300;

/// One check result.
#[derive(Debug, Clone, Serialize)]
pub struct OverfitCheck {
    pub name: String,
    /// `PASS` | `FAIL` | `INFO` — only `FAIL` fails the gate.
    pub result: String,
    pub details: Vec<String>,
}

/// The `overfitting.json` report shape.
#[derive(Debug, Clone, Serialize)]
pub struct OverfitReport {
    /// The gate PASSES only when no check FAILed (INFO results — e.g. disproportionate success,
    /// or a check skipped because corpus sources are unavailable — do not fail the gate).
    pub gate: bool,
    pub checks: Vec<OverfitCheck>,
    pub note: String,
}

/// One candidate source file (path + content).
#[derive(Debug, Clone)]
pub struct CandidateFile {
    pub path: PathBuf,
    pub content: String,
}

/// Recursively list the candidate crate's `.rs` files (read-only).
pub fn scan_candidate_src(root: &Path) -> Result<Vec<CandidateFile>, String> {
    let dir = root.join(CANDIDATE_SRC_DIR);
    if !dir.is_dir() {
        return Err(format!(
            "candidate source directory {} does not exist",
            dir.display()
        ));
    }
    let mut out = Vec::new();
    let mut stack = vec![dir];
    while let Some(d) = stack.pop() {
        let rd = std::fs::read_dir(&d).map_err(|e| e.to_string())?;
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            if p.extension().and_then(|x| x.to_str()) == Some("rs") {
                let content = std::fs::read_to_string(&p).map_err(|e| e.to_string())?;
                // report paths relative to the repo root (portable reports)
                let rel = p.strip_prefix(root).unwrap_or(&p).to_path_buf();
                out.push(CandidateFile { path: rel, content });
            }
        }
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

/// Extract Rust string-literal values (with 1-based line numbers). Handles `"..."` with backslash
/// escapes and `r#"..."#` raw strings; `'x'` char literals hold a single char and are skipped.
pub fn rust_string_literals(content: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let chars: Vec<char> = content.chars().collect();
    let mut i = 0usize;
    let mut line = 1usize;
    while i < chars.len() {
        let c = chars[i];
        if c == '\n' {
            line += 1;
            i += 1;
            continue;
        }
        // raw string: r"..." or r#"..."# with any number of '#'s
        if c == 'r' {
            let mut j = i + 1;
            while j < chars.len() && chars[j] == '#' {
                j += 1;
            }
            if j < chars.len() && chars[j] == '"' {
                let hashes = j - (i + 1);
                let start = j + 1;
                let start_line = line;
                let mut k = start;
                while k < chars.len() {
                    if chars[k] == '\n' {
                        line += 1;
                    }
                    if chars[k] == '"' {
                        let close = k + 1;
                        let mut h = close;
                        while h < chars.len() && chars[h] == '#' {
                            h += 1;
                        }
                        if h - close == hashes {
                            let lit: String = chars[start..k].iter().collect();
                            out.push((start_line, lit));
                            i = h;
                            break;
                        }
                    }
                    k += 1;
                }
                if k >= chars.len() {
                    i = k;
                }
                continue;
            }
        }
        if c == '"' {
            let start = i + 1;
            let start_line = line;
            let mut j = start;
            let mut closed = false;
            while j < chars.len() {
                let d = chars[j];
                if d == '\n' {
                    line += 1;
                }
                if d == '\\' {
                    j += 2; // skip the escaped char (kept verbatim)
                    continue;
                }
                if d == '"' {
                    closed = true;
                    break;
                }
                j += 1;
            }
            if closed {
                let lit: String = chars[start..j].iter().collect();
                out.push((start_line, lit));
                i = j + 1;
            } else {
                i = j;
            }
            continue;
        }
        i += 1;
    }
    out
}

/// Collect known corpus program IDs / test names from the committed reports (best-effort; a
/// missing report contributes nothing and never errors).
pub fn collect_program_ids(root: &Path) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();
    let mut add = |v: &str| {
        let t = v.trim();
        if t.len() >= 3 && !ids.iter().any(|x| x == t) {
            ids.push(t.to_string());
        }
    };
    // X-COBOL file ids (`xcobol/<repo>/<file>`).
    if let Ok(rows) = load_xcobol_programs(root) {
        for r in rows {
            add(&r.file_id);
        }
    }
    // CCVS85 program ids + bare unit names (e.g. `ccvs85/NC107A`, `NC107A`).
    let ccvs = root
        .join("reports")
        .join("valid-corpus")
        .join("ccvs85")
        .join("programs.json");
    if let Ok(bytes) = std::fs::read(&ccvs) {
        if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            if let Some(arr) = v.as_array() {
                for e in arr {
                    if let Some(id) = e.get("program_id").and_then(|x| x.as_str()) {
                        add(id);
                    }
                    if let Some(n) = e.get("name").and_then(|x| x.as_str()) {
                        add(n);
                    }
                }
            }
        }
    }
    // GnuCOBOL manual examples.
    for lane in ["stable-3.2", "current"] {
        let p = root
            .join("reports")
            .join("valid-corpus")
            .join("gnucobol-manual")
            .join(lane)
            .join("examples.json");
        if let Ok(bytes) = std::fs::read(&p) {
            if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                if let Some(arr) = v.as_array() {
                    for e in arr {
                        if let Some(id) = e.get("program_id").and_then(|x| x.as_str()) {
                            add(id);
                        }
                        if let Some(f) = e.get("filename").and_then(|x| x.as_str()) {
                            add(f);
                        }
                    }
                }
            }
        }
    }
    // Testsuit test groups (`run_move`, `syn_copy`, ... from the `.at` basenames).
    let ts = root
        .join("reports")
        .join("valid-corpus")
        .join("gnucobol-testsuite")
        .join("valid-programs.json");
    if let Ok(bytes) = std::fs::read(&ts) {
        if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            if let Some(arr) = v.as_array() {
                for e in arr {
                    if let Some(src) = e.get("source_file").and_then(|x| x.as_str()) {
                        let base = src.rsplit('/').next().unwrap_or(src);
                        if let Some(stripped) = base.strip_suffix(".at") {
                            add(stripped);
                        }
                    }
                }
            }
        }
    }
    // OMP course programs.
    let omp = root
        .join("reports")
        .join("valid-corpus")
        .join("omp")
        .join("programs.json");
    if let Ok(bytes) = std::fs::read(&omp) {
        if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            if let Some(arr) = v.as_array() {
                for e in arr {
                    if let Some(id) = e.get("program_id").and_then(|x| x.as_str()) {
                        add(id);
                    }
                }
            }
        }
    }
    ids.sort();
    ids.dedup();
    ids
}

/// Collect known non-empty expected-output hashes from the corpus reports (best-effort).
pub fn collect_known_output_hashes(root: &Path) -> Vec<String> {
    let mut hashes: Vec<String> = Vec::new();
    let mut add = |h: &str| {
        let t = h.trim();
        if t.len() == 64
            && t.chars().all(|c| c.is_ascii_hexdigit())
            && t != EMPTY_OUTPUT_SHA
            && !hashes.iter().any(|x| x == t)
        {
            hashes.push(t.to_string());
        }
    };
    for lane in ["stable-3.2", "current"] {
        let p = root
            .join("reports")
            .join("valid-corpus")
            .join("gnucobol-manual")
            .join(lane)
            .join("examples.json");
        if let Ok(bytes) = std::fs::read(&p) {
            if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                if let Some(arr) = v.as_array() {
                    for e in arr {
                        for k in [
                            "oracle_stdout_sha256",
                            "oracle_stderr_sha256",
                            "expected_output_sha256",
                        ] {
                            if let Some(h) = e.get(k).and_then(|x| x.as_str()) {
                                add(h);
                            }
                        }
                    }
                }
            }
        }
    }
    let ccvs = root
        .join("reports")
        .join("valid-corpus")
        .join("ccvs85")
        .join("programs.json");
    if let Ok(bytes) = std::fs::read(&ccvs) {
        if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            if let Some(arr) = v.as_array() {
                for e in arr {
                    for k in [
                        "oracle_report_sha256",
                        "candidate_stdout_sha256",
                        "candidate_report_sha256",
                    ] {
                        if let Some(h) = e.get(k).and_then(|x| x.as_str()) {
                            add(h);
                        }
                    }
                }
            }
        }
    }
    hashes.sort();
    hashes.dedup();
    hashes
}

/// Collect corpus source long-lines for the hardcoded-source-strings check. Uses a bounded,
/// deterministic sample of the X-COBOL rows (best-effort: unavailable sources are skipped so the
/// overfit command works without the corpus root), plus the admitted testsuite `.at` sources when
/// present. Lines are trimmed, restricted to 40..=200 chars and must carry at least two distinct
/// alphabetic tokens (pure punctuation/separator lines are not evidence of leakage).
pub fn collect_corpus_lines(root: &Path) -> Vec<String> {
    let mut lines: HashSet<String> = HashSet::new();
    let mut add = |l: &str| {
        let t = l.trim();
        let len = t.chars().count();
        if (40..=200).contains(&len) {
            let words: HashSet<&str> = t
                .split(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
                .filter(|w| !w.is_empty() && w.chars().any(|c| c.is_ascii_alphabetic()))
                .collect();
            if words.len() >= 2 {
                lines.insert(t.to_string());
            }
        }
    };
    if let Ok(rows) = load_xcobol_programs(root) {
        let mut rows: Vec<XcobolRow> = rows;
        rows.sort_by(|a, b| a.file_id.cmp(&b.file_id));
        for row in rows.iter().take(MAX_XCOBOL_ROWS_SAMPLED) {
            if row.exact_sha256.is_empty() {
                continue;
            }
            // store blob first; otherwise the admitted extraction tree / package work dir
            let bytes: Option<Vec<u8>> = std::fs::read(
                root.join("lab/corpus/x-cobol/extracted/X-COBOL/COBOL_Files")
                    .join(&row.path),
            )
            .ok();
            if let Some(b) = bytes {
                if let Ok(text) = String::from_utf8(b) {
                    for l in text.split('\n') {
                        add(l);
                    }
                }
            }
        }
    }
    // admitted testsuite sources on disk (both lanes)
    for dir in [
        "lab/admit/gnucobol-3.2/tests/testsuite.src",
        "lab/admit/gnucobol-upstream-current/tests/testsuite.src",
    ] {
        let d = root.join(dir);
        if let Ok(rd) = std::fs::read_dir(&d) {
            let mut at_files: Vec<PathBuf> = rd
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("at"))
                .collect();
            at_files.sort();
            for p in at_files.iter().take(30) {
                if let Ok(text) = std::fs::read_to_string(p) {
                    for l in text.split('\n') {
                        add(l);
                    }
                }
            }
        }
    }
    let mut v: Vec<String> = lines.into_iter().collect();
    v.sort();
    v
}

/// 1. Hardcoded test names / program IDs: Rust string literals equal to a known corpus ID.
pub fn hardcoded_test_names(files: &[CandidateFile], known_ids: &[String]) -> OverfitCheck {
    let id_set: HashSet<&str> = known_ids.iter().map(String::as_str).collect();
    let mut details = Vec::new();
    for f in files {
        for (line, lit) in rust_string_literals(&f.content) {
            if lit.len() >= 3 && id_set.contains(lit.as_str()) {
                details.push(format!(
                    "{}:{line}: string literal {:?} equals a known corpus program id / test name",
                    f.path.display(),
                    lit
                ));
            }
        }
    }
    let (result, mut det) = if details.is_empty() {
        (
            "PASS".to_string(),
            vec![format!(
                "scanned {} candidate file(s) for string literals equal to any of {} known corpus \
                 ids; no hits",
                files.len(),
                known_ids.len()
            )],
        )
    } else {
        ("FAIL".to_string(), Vec::new())
    };
    det.append(&mut details);
    OverfitCheck {
        name: "hardcoded_test_names".to_string(),
        result,
        details: det,
    }
}

/// 2. Hardcoded corpus source strings: 40+ char corpus source lines appearing inside candidate
/// code. Two passes: (a) exact candidate-line membership (fast, all corpus lines), (b) substring
/// search over candidate files that carry COBOL markers, capped at `MAX_CORPUS_LINES_SUBSTRING`
/// lines to bound the scan.
pub fn hardcoded_source_strings(files: &[CandidateFile], corpus_lines: &[String]) -> OverfitCheck {
    if corpus_lines.is_empty() {
        return OverfitCheck {
            name: "hardcoded_source_strings".to_string(),
            result: "INFO".to_string(),
            details: vec![
                "no corpus source lines available (X-COBOL dataset not admitted and no testsuite \
                 sources on disk); the check was skipped, not failed"
                    .to_string(),
            ],
        };
    }
    let set: HashSet<&str> = corpus_lines.iter().map(String::as_str).collect();
    let mut details = Vec::new();
    // pass (a): candidate lines equal to a corpus line
    for f in files {
        for (idx, line) in f.content.split('\n').enumerate() {
            let t = line.trim();
            if (40..=200).contains(&t.chars().count()) && set.contains(t) {
                details.push(format!(
                    "{}:{}: candidate line exactly matches a corpus source line",
                    f.path.display(),
                    idx + 1
                ));
            }
        }
    }
    // pass (b): substring search over candidate files that contain COBOL markers
    let long: Vec<&str> = corpus_lines
        .iter()
        .filter(|l| l.chars().count() >= 60)
        .take(MAX_CORPUS_LINES_SUBSTRING)
        .map(String::as_str)
        .collect();
    for f in files {
        let up = f.content.to_ascii_uppercase();
        let is_cobol = ["IDENTIFICATION", "PROGRAM-ID", "PROCEDURE DIVISION", "PIC "]
            .iter()
            .filter(|m| up.contains(**m))
            .count()
            >= 2;
        if !is_cobol {
            continue;
        }
        for line in &long {
            if f.content.contains(line) {
                details.push(format!(
                    "{}: contains the corpus source line {:?}",
                    f.path.display(),
                    truncate(line, 90)
                ));
            }
        }
    }
    let (result, mut det) = if details.is_empty() {
        (
            "PASS".to_string(),
            vec![format!(
                "compared {} candidate file(s) against {} unique corpus source lines (40-200 \
                 chars, content-bearing); no exact or substring matches",
                files.len(),
                corpus_lines.len()
            )],
        )
    } else {
        ("FAIL".to_string(), Vec::new())
    };
    det.append(&mut details);
    OverfitCheck {
        name: "hardcoded_source_strings".to_string(),
        result,
        details: det,
    }
}

/// Brace-delta of one Rust line, ignoring string/char literals, raw strings and comments.
/// Used to delimit `#[cfg(test)]` modules (heuristic; good enough to classify hit lines).
fn brace_delta(line: &str) -> i32 {
    let chars: Vec<char> = line.chars().collect();
    let mut d = 0i32;
    let mut i = 0usize;
    let mut in_str = false;
    let mut in_char = false;
    while i < chars.len() {
        let c = chars[i];
        if in_str {
            if c == '\\' {
                i += 2;
                continue;
            }
            if c == '"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if in_char {
            if c == '\\' {
                i += 2;
                continue;
            }
            if c == '\'' {
                in_char = false;
            }
            i += 1;
            continue;
        }
        // raw string r#"..."#
        if c == 'r' {
            let mut j = i + 1;
            while j < chars.len() && chars[j] == '#' {
                j += 1;
            }
            if j < chars.len() && chars[j] == '"' {
                let hashes = j - i - 1;
                let mut k = j + 1;
                while k < chars.len() {
                    if chars[k] == '"' {
                        let mut h = k + 1;
                        while h < chars.len() && chars[h] == '#' {
                            h += 1;
                        }
                        if h - (k + 1) == hashes {
                            break;
                        }
                    }
                    k += 1;
                }
                i = k + 1;
                continue;
            }
        }
        match c {
            '"' => in_str = true,
            '\'' => in_char = true,
            '{' => d += 1,
            '}' => d -= 1,
            '/' => {
                if chars.get(i + 1) == Some(&'/') {
                    break; // line comment to EOL
                }
                if chars.get(i + 1) == Some(&'*') {
                    let mut j = i + 2;
                    while j + 1 < chars.len() && !(chars[j] == '*' && chars[j + 1] == '/') {
                        j += 1;
                    }
                    i = j + 2;
                    continue;
                }
            }
            _ => {}
        }
        i += 1;
    }
    d
}

/// 1-based inclusive line ranges of `#[cfg(test)]`-guarded modules (brace-delimited).
fn test_module_ranges(content: &str) -> Vec<(usize, usize)> {
    let lines: Vec<&str> = content.split('\n').collect();
    let n = lines.len();
    let mut ranges = Vec::new();
    let mut i = 0usize;
    while i < n {
        if lines[i].trim().contains("#[cfg(test)]") {
            let mut j = i + 1;
            while j < n {
                let m = lines[j].trim();
                if m.is_empty() || m.starts_with("#[") {
                    j += 1;
                    continue;
                }
                if m.starts_with("mod ") && m.contains('{') {
                    let mut depth = 0i32;
                    let mut k = j;
                    while k < n {
                        depth += brace_delta(lines[k]);
                        if depth <= 0 {
                            break;
                        }
                        k += 1;
                    }
                    if depth <= 0 {
                        ranges.push((j + 1, k + 1));
                        i = k;
                    } else {
                        ranges.push((j + 1, n)); // unbalanced; assume to EOF
                        i = n;
                    }
                }
                break;
            }
        }
        i += 1;
    }
    ranges
}

/// 3. Absolute host paths in the candidate source. A path token inside a `#[cfg(test)]` module is
/// a test fixture (synthetic value), not source-path-dependent production behavior: it is
/// reported as INFO context and does not fail the gate. A token in non-test code is a FAIL.
pub fn absolute_host_paths(files: &[CandidateFile]) -> OverfitCheck {
    let mut prod_hits: Vec<String> = Vec::new();
    let mut test_hits: Vec<String> = Vec::new();
    for f in files {
        let ranges = test_module_ranges(&f.content);
        let in_test = |line: usize| ranges.iter().any(|(a, b)| *a <= line && line <= *b);
        for (idx, line) in f.content.split('\n').enumerate() {
            let ln = idx + 1;
            for p in HOST_PATH_PREFIXES {
                if line.contains(p) {
                    let msg = format!("{}:{ln}: contains host path prefix {p}", f.path.display());
                    if in_test(ln) {
                        test_hits.push(msg);
                    } else {
                        prod_hits.push(msg);
                    }
                }
            }
        }
    }
    let (result, mut det) = if prod_hits.is_empty() && test_hits.is_empty() {
        (
            "PASS".to_string(),
            vec![format!(
                "no absolute host paths ({}) in {} candidate file(s)",
                HOST_PATH_PREFIXES.join(", "),
                files.len()
            )],
        )
    } else if prod_hits.is_empty() {
        (
            "INFO".to_string(),
            vec![format!(
                "{} host-path token(s) found, all inside #[cfg(test)] modules (test fixtures with \
                 synthetic values, not source-path-dependent production behavior)",
                test_hits.len()
            )],
        )
    } else {
        ("FAIL".to_string(), Vec::new())
    };
    det.extend(prod_hits);
    det.extend(test_hits);
    OverfitCheck {
        name: "absolute_host_paths".to_string(),
        result,
        details: det,
    }
}

/// 4. Oracle-output tables: 64-hex SHA-256 literals in the candidate source that equal a known
/// non-empty corpus expected-output hash. All 64-hex tokens are reported as INFO context; the
/// empty-output hash is documented as excluded (hash of zero bytes, present in the candidate's
/// sha256 self-test vectors, not evidence of hardcoding).
pub fn oracle_output_tables(files: &[CandidateFile], known_hashes: &[String]) -> OverfitCheck {
    // The empty-output hash is excluded here (not just at collection time): it is the SHA-256 of
    // zero bytes and its presence in the candidate's sha256 self-test vectors is not evidence of
    // hardcoding corpus outputs.
    let known: HashSet<&str> = known_hashes
        .iter()
        .map(String::as_str)
        .filter(|h| *h != EMPTY_OUTPUT_SHA)
        .collect();
    let mut hits = Vec::new();
    let mut tokens = Vec::new();
    for f in files {
        for (idx, line) in f.content.split('\n').enumerate() {
            let mut rest = line;
            while let Some(pos) = rest.find(|c: char| c.is_ascii_hexdigit()) {
                // collect a maximal hex run
                let tail = &rest[pos..];
                let run: String = tail.chars().take_while(|c| c.is_ascii_hexdigit()).collect();
                if run.len() == 64 {
                    let tok = run.clone();
                    tokens.push(format!("{}:{}: {tok}", f.path.display(), idx + 1));
                    if known.contains(tok.as_str()) {
                        hits.push(format!(
                            "{}:{}: 64-hex literal {tok} equals a known corpus expected-output hash",
                            f.path.display(),
                            idx + 1
                        ));
                    }
                }
                rest = &tail[run.len().max(1)..];
            }
        }
    }
    let mut details = Vec::new();
    details.push(format!(
        "scanned {} candidate file(s); {} 64-hex token(s) found, {} equal a known non-empty \
         corpus expected-output hash",
        files.len(),
        tokens.len(),
        hits.len()
    ));
    details.push(format!(
        "the empty-output hash {EMPTY_OUTPUT_SHA} is excluded: it is the SHA-256 of zero bytes, \
         appears as the recorded stdout of not-run corpus units and in the candidate's sha256 \
         self-test vectors; not evidence of hardcoding"
    ));
    details.push(format!(
        "{} known non-empty corpus expected-output hash(es) were compared against",
        known_hashes.len()
    ));
    for t in tokens.iter().take(10) {
        details.push(format!("token: {t}"));
    }
    if tokens.len() > 10 {
        details.push(format!("... and {} more token(s)", tokens.len() - 10));
    }
    if hits.is_empty() {
        details.push("no hardcoded oracle-output hashes".to_string());
        OverfitCheck {
            name: "oracle_output_tables".to_string(),
            result: "PASS".to_string(),
            details,
        }
    } else {
        details.extend(hits);
        OverfitCheck {
            name: "oracle_output_tables".to_string(),
            result: "FAIL".to_string(),
            details,
        }
    }
}

/// 5. Disproportionate success: candidate accept rate on DEVELOPMENT vs HELD_OUT_EVALUATION.
/// A large gap is flagged as an overfitting indicator (INFO), never a gate failure.
pub fn disproportionate_success(rows: &[XcobolRow]) -> OverfitCheck {
    let dev = accept_rate(rows, "DEVELOPMENT");
    let held = accept_rate(rows, "HELD_OUT_EVALUATION");
    let ratio = if held.rate > 0.0 {
        dev.rate / held.rate
    } else if dev.rate > 0.0 {
        f64::INFINITY
    } else {
        1.0
    };
    let mut details = vec![
        format!(
            "DEVELOPMENT accept rate {:.3} ({}/{}), HELD_OUT_EVALUATION accept rate {:.3} ({}/{})",
            dev.rate, dev.ok, dev.total, held.rate, held.ok, held.total
        ),
        format!(
            "ratio dev/held: {}",
            if ratio.is_finite() {
                format!("{ratio:.3}")
            } else {
                "infinity (held-out accept rate is zero)".to_string()
            }
        ),
    ];
    let flag = ratio.is_finite() && ratio > 2.0 || ratio.is_infinite();
    details.push(if flag {
        "flagged: the held-out accept rate is substantially below the development rate; this \
             is an overfitting INDICATOR (a candidate that tuned to development quirks), not a \
             gate failure"
            .to_string()
    } else {
        "no large gap between development and held-out accept rates".to_string()
    });
    OverfitCheck {
        name: "disproportionate_success".to_string(),
        result: "INFO".to_string(),
        details,
    }
}

struct Rate {
    total: usize,
    ok: usize,
    rate: f64,
}

fn accept_rate(rows: &[XcobolRow], partition: &str) -> Rate {
    let total = rows.iter().filter(|r| r.partition == partition).count();
    let ok = rows
        .iter()
        .filter(|r| r.partition == partition && r.candidate_phases_ok)
        .count();
    Rate {
        total,
        ok,
        rate: if total == 0 {
            0.0
        } else {
            ok as f64 / total as f64
        },
    }
}

/// Run all overfit checks over the repo + committed reports. Works without the corpus root:
/// `hardcoded_source_strings` degrades to INFO when no corpus sources are available.
pub fn run_checks(root: &Path) -> Result<OverfitReport, String> {
    let files = scan_candidate_src(root)?;
    let ids = collect_program_ids(root);
    let corpus_lines = collect_corpus_lines(root);
    let hashes = collect_known_output_hashes(root);
    let rows = load_xcobol_programs(root).unwrap_or_default();
    let checks = vec![
        hardcoded_test_names(&files, &ids),
        hardcoded_source_strings(&files, &corpus_lines),
        absolute_host_paths(&files),
        oracle_output_tables(&files, &hashes),
        disproportionate_success(&rows),
    ];
    let gate = !checks.iter().any(|c| c.result == "FAIL");
    Ok(OverfitReport {
        gate,
        note: "The gate PASSES only when no check FAILed: no hardcoded test names, no hardcoded \
               source strings, no absolute host paths, no oracle-output tables. \
               Disproportionate success is INFO only. All scans are read-only over \
               crates/gnucobol-rs/src/ and the committed reports."
            .to_string(),
        checks,
    })
}

fn truncate(s: &str, n: usize) -> String {
    let mut out: String = s.chars().take(n).collect();
    if s.chars().count() > n {
        out.push('…');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(name: &str, content: &str) -> CandidateFile {
        CandidateFile {
            path: PathBuf::from(name),
            content: content.to_string(),
        }
    }

    #[test]
    fn literals_extract_quoted_and_raw_strings() {
        let src = "let a = \"hi\";\nlet b = r#\"multi\nline\"#;\nlet c = 'x';\n";
        let lits = rust_string_literals(src);
        assert_eq!(lits.len(), 2);
        assert_eq!(lits[0], (1, "hi".to_string()));
        assert_eq!(lits[1], (2, "multi\nline".to_string()));
    }

    #[test]
    fn hardcoded_test_names_detects_synthetic_hit() {
        let f = file(
            "crates/gnucobol-rs/src/x.rs",
            "fn t() { let id = \"ccvs85/NC107A\"; }\n",
        );
        let check = hardcoded_test_names(&[f], &["ccvs85/NC107A".to_string()]);
        assert_eq!(check.result, "FAIL");
        assert!(check.details[0].contains("ccvs85/NC107A"));
    }

    #[test]
    fn hardcoded_test_names_passes_without_hits() {
        let f = file(
            "crates/gnucobol-rs/src/x.rs",
            "fn t() { let s = \"hello world\"; }\n",
        );
        let check = hardcoded_test_names(&[f], &["ccvs85/NC107A".to_string()]);
        assert_eq!(check.result, "PASS");
    }

    #[test]
    fn hardcoded_source_strings_detects_embedded_line() {
        let line =
            "       IDENTIFICATION DIVISION AND PROCEDURE DIVISION ARE REALLY LONG TEXT HERE.";
        let f = file(
            "crates/gnucobol-rs/src/x.rs",
            &format!("const SRC: &str = \"{line}\\n\";\n"),
        );
        let check = hardcoded_source_strings(&[f], &[line.to_string()]);
        assert_eq!(check.result, "FAIL");
    }

    #[test]
    fn hardcoded_source_strings_skips_when_no_corpus_lines() {
        let f = file("crates/gnucobol-rs/src/x.rs", "fn t() {}\n");
        let check = hardcoded_source_strings(&[f], &[]);
        assert_eq!(check.result, "INFO");
    }

    #[test]
    fn absolute_host_paths_detected_and_clean() {
        let bad = file(
            "crates/gnucobol-rs/src/x.rs",
            "let p = \"/home/one/secret\";\n",
        );
        assert_eq!(absolute_host_paths(&[bad]).result, "FAIL");
        let good = file("crates/gnucobol-rs/src/x.rs", "let p = \"./relative\";\n");
        assert_eq!(absolute_host_paths(&[good]).result, "PASS");
        // a host path inside a #[cfg(test)] module is a test fixture: INFO, not FAIL
        let fixture = file(
            "crates/gnucobol-rs/src/x.rs",
            "pub fn f() -> i32 { 1 }\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {\n        \
             let home = \"/home/u\".to_string();\n    }\n}\n",
        );
        let check = absolute_host_paths(&[fixture]);
        assert_eq!(check.result, "INFO");
        assert!(check.details.iter().any(|d| d.contains("cfg(test)")));
    }

    #[test]
    fn oracle_output_tables_ignores_self_test_vectors() {
        // the sha256 self-test vectors (incl. the empty-output hash) are not hits
        let f = file(
            "crates/gnucobol-rs/src/sha256.rs",
            "assert_eq!(hex, \"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\");\n",
        );
        let known = vec![
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
            "d9014c4624844aa5bac314773d6b689ad467fa4e1d1a50a1b8a99d5a95f72ff5".to_string(),
        ];
        let check = oracle_output_tables(&[f], &known);
        assert_eq!(check.result, "PASS"); // empty hash excluded, other vector unknown
        assert!(check.details.iter().any(|d| d.contains("excluded")));
    }

    #[test]
    fn oracle_output_tables_flags_known_hash() {
        let f = file(
            "crates/gnucobol-rs/src/x.rs",
            "let exp = \"d9014c4624844aa5bac314773d6b689ad467fa4e1d1a50a1b8a99d5a95f72ff5\";\n",
        );
        let known =
            vec!["d9014c4624844aa5bac314773d6b689ad467fa4e1d1a50a1b8a99d5a95f72ff5".to_string()];
        assert_eq!(oracle_output_tables(&[f], &known).result, "FAIL");
    }

    #[test]
    fn disproportionate_success_is_info_and_reports_ratio() {
        let row = |part: &str, ok: bool| XcobolRow {
            file_id: String::new(),
            repo: String::new(),
            path: String::new(),
            bytes: 0,
            extension: String::new(),
            structural_class: String::new(),
            encoding: String::new(),
            dialect_accepted: None,
            candidate_first_failure: None,
            candidate_phases_ok: ok,
            partition: part.to_string(),
            exact_sha256: String::new(),
        };
        let rows = vec![
            row("DEVELOPMENT", true),
            row("DEVELOPMENT", true),
            row("DEVELOPMENT", false),
            row("HELD_OUT_EVALUATION", false),
            row("HELD_OUT_EVALUATION", false),
            row("HELD_OUT_EVALUATION", false),
            row("VALIDATION", true),
        ];
        let check = disproportionate_success(&rows);
        assert_eq!(check.result, "INFO");
        assert!(check.details[1].contains("infinity"));
    }

    #[test]
    fn gate_passes_without_corpus_root() {
        // a temp "repo" with a candidate src dir but no reports at all: every check must either
        // pass or degrade to INFO, so the gate passes
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("crates/gnucobol-rs/src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("lib.rs"), "pub fn answer() -> i32 { 42 }\n").unwrap();
        let rep = run_checks(dir.path()).unwrap();
        assert!(rep.gate, "{rep:?}");
        assert!(!rep.checks.is_empty());
    }
}
