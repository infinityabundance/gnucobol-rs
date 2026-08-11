//! Phase 10.5 — metamorphic testing of semantics-preserving source variants.
//!
//! Bases are ADMITTED valid programs (X-COBOL files recorded as `COMPLETE_PROGRAM` with a
//! candidate-accepted outcome, bounded sample). For each base we generate only transformations
//! that are DEFENSIBLY semantics-preserving (spec 10.5): whitespace variation, fixed-format
//! sequence-number changes, alternate literal quoting, a conservative data-name rename, and
//! copybook extraction+reinsertion. Anything we cannot prove safe is skipped with a recorded
//! reason and never claimed (spec: "Do NOT claim semantic preservation for unsafe
//! transformations").
//!
//! Each original and variant is prepared + run under a 2 s wall bound; equivalence = same exit
//! code and byte-identical stdout. Divergent variants are kept and reported honestly, never
//! hidden.

use crate::heldout::{run_bounded, truncate, XcobolRow};
use crate::store::{sha256_hex, CorpusStore};
use gnucobol_rs::copybook::{self, CopyResolver};
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Bounded sample size for the base-program selection.
pub const MAX_BASES: usize = 30;

/// Maximum variants attempted per base (one per transformation).
pub const MAX_VARIANTS_PER_BASE: usize = 5;

/// The conservative reserved-word blacklist for the identifier rename. A name declared on a level
/// line of a file the candidate accepts cannot be reserved in practice; this list is belt-and-
/// braces for the most common words (and `FILLER`, which must never be renamed).
const RESERVED: &[&str] = &[
    "FILLER",
    "VALUE",
    "PIC",
    "PICTURE",
    "OCCURS",
    "REDEFINES",
    "RENAMES",
    "USAGE",
    "COMP",
    "DISPLAY",
    "MOVE",
    "STOP",
    "RUN",
    "PERFORM",
    "GO",
    "TO",
    "CALL",
    "PROGRAM",
    "SECTION",
    "DIVISION",
    "DATA",
    "FILE",
    "WORKING-STORAGE",
    "LOCAL-STORAGE",
    "LINKAGE",
    "PROCEDURE",
    "IDENTIFICATION",
    "ENVIRONMENT",
    "CONFIGURATION",
    "INPUT-OUTPUT",
    "IF",
    "ELSE",
    "END-IF",
    "ADD",
    "SUBTRACT",
    "MULTIPLY",
    "DIVIDE",
    "COMPUTE",
    "READ",
    "WRITE",
    "OPEN",
    "CLOSE",
    "START",
    "DELETE",
    "REWRITE",
    "ACCEPT",
    "INITIALIZE",
    "RETURN",
    "EXIT",
    "GOBACK",
    "END",
    "COPY",
    "REPLACE",
    "SELECT",
    "ASSIGN",
    "STATUS",
    "ORGANIZATION",
    "ACCESS",
    "MODE",
    "DYNAMIC",
    "SEQUENTIAL",
    "INDEXED",
    "RELATIVE",
    "WHEN",
    "SEARCH",
    "SORT",
    "MERGE",
    "STRING",
    "UNSTRING",
    "INSPECT",
    "EVALUATE",
    "PERFORM",
    "UNTIL",
    "VARYING",
    "AFTER",
    "BEFORE",
    "FROM",
    "BY",
    "WITH",
    "THROUGH",
    "THRU",
    "USING",
    "GIVING",
    "RETURNING",
    "IS",
    "ARE",
    "NOT",
    "AND",
    "OR",
    "TRUE",
    "FALSE",
    "ZERO",
    "ZEROS",
    "ZEROES",
    "SPACE",
    "SPACES",
    "LOW",
    "LOW-VALUES",
    "HIGH",
    "HIGH-VALUES",
    "NULL",
    "NULLS",
    "QUOTE",
    "QUOTES",
    "ALL",
    "ANY",
];

/// One variant of one base program.
#[derive(Debug, Clone, Serialize)]
pub struct VariantRow {
    pub variant_type: String,
    pub original_sha256: String,
    pub variant_sha256: String,
    /// `true` = same exit code and byte-identical stdout; `false` = divergence or verification
    /// failure (recorded honestly, never hidden).
    pub equivalent: bool,
    /// `true` = the transformation was not attempted because it could not be proven safe.
    pub skipped: bool,
    pub skip_reason: String,
    pub original_exit: Option<i32>,
    pub variant_exit: Option<i32>,
    pub stdout_match: bool,
    pub diagnostic: String,
    pub note: String,
}

/// One base program and its variants.
#[derive(Debug, Clone, Serialize)]
pub struct BaseProgramRow {
    pub file_id: String,
    pub repo: String,
    pub bytes: usize,
    pub structural_class: String,
    pub partition: String,
    pub candidate_phases_ok: bool,
    pub variants: Vec<VariantRow>,
}

/// Summary counters.
#[derive(Debug, Clone, Serialize, Default)]
pub struct MutationSummary {
    pub total_bases: usize,
    pub total_variants: usize,
    pub equivalent: usize,
    pub divergent: usize,
    pub skipped: usize,
}

/// The `mutation-results.json` report shape.
#[derive(Debug, Clone, Serialize)]
pub struct MutationReport {
    pub timeout_seconds: u64,
    pub summary: MutationSummary,
    pub bases: Vec<BaseProgramRow>,
}

/// Select the mutation bases: COMPLETE_PROGRAM files the candidate accepted, deterministic
/// order, bounded sample.
pub fn select_bases(rows: &[XcobolRow], max: usize) -> Vec<XcobolRow> {
    let mut v: Vec<XcobolRow> = rows
        .iter()
        .filter(|r| r.structural_class == "COMPLETE_PROGRAM" && r.candidate_phases_ok)
        .cloned()
        .collect();
    v.sort_by(|a, b| a.file_id.cmp(&b.file_id));
    v.truncate(max);
    v
}

/// Expand `source` through the copybook expander with a resolver rooted at `dir` (identity when
/// the source has no COPY statements).
fn expand_with(source: &str, dir: &Path) -> Result<String, String> {
    let resolver = DirOnlyResolver {
        root: dir.to_path_buf(),
    };
    copybook::expand(source, &resolver)
        .map(|e| e.text())
        .map_err(|e| e.to_string())
}

struct DirOnlyResolver {
    root: PathBuf,
}

impl CopyResolver for DirOnlyResolver {
    fn resolve(&self, name: &str) -> Option<String> {
        for cand in [self.root.join(name), self.root.join(format!("{name}.cpy"))] {
            if let Ok(s) = std::fs::read_to_string(&cand) {
                return Some(s);
            }
        }
        None
    }
}

/// Result of one transformation attempt: `text` is `Some` only when the transformation is
/// defensible and produced a variant that differs from the original.
pub struct Transform {
    pub kind: &'static str,
    pub skip_reason: String,
    pub text: Option<String>,
}

/// 1. Whitespace variation: insert a blank line right after the `PROCEDURE DIVISION` header (a
/// literal cannot span that boundary in a file that parses, so the token stream is unchanged)
/// and ensure exactly one trailing blank line at EOF.
pub fn whitespace_variant(source: &str) -> Option<String> {
    let mut out = String::with_capacity(source.len() + 8);
    let mut inserted = false;
    for line in source.split('\n') {
        out.push_str(line);
        out.push('\n');
        if !inserted
            && line
                .trim()
                .to_ascii_uppercase()
                .starts_with("PROCEDURE DIVISION")
        {
            out.push('\n');
            inserted = true;
        }
    }
    // `split('\n')` yields a trailing "" element when the source ends with a newline, which
    // pushed one extra '\n'; drop it so the variant reproduces the source byte-faithfully.
    if source.ends_with('\n') {
        out.pop();
    }
    // exactly one trailing blank line
    if out.ends_with('\n') {
        out.push('\n');
    } else {
        out.push_str("\n\n");
    }
    if out == source {
        None
    } else {
        Some(out)
    }
}

/// 2. Fixed-format sequence-number changes: lines whose columns 1-6 are digits and column 7 is
/// blank get a fresh 6-digit sequence number. The sequence area is not program text (the corpus
/// structural hash strips it), so this is defensibly semantics-preserving.
pub fn fixed_sequence_variant(source: &str) -> Option<String> {
    let mut changed = false;
    let mut counter = 1u64;
    let out = source
        .split('\n')
        .map(|line| {
            let b = line.as_bytes();
            if b.len() >= 7 && b[..6].iter().all(|c| c.is_ascii_digit()) && b[6] == b' ' {
                changed = true;
                let seq = format!("{counter:06}");
                counter += 1;
                // columns 1-7 are all ASCII here, so byte slicing is char-safe; the blank at
                // column 7 is preserved between the new sequence and the text area
                format!("{seq} {}", &line[7..])
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n");
    if changed && out != source {
        Some(out)
    } else {
        None
    }
}

/// 3. Alternate literal quoting (`'` <-> `"`), only when provably safe: every line must carry an
/// even count of both quote characters (so no literal spans lines and no apostrophe hides inside
/// a double-quoted literal) and no `'` appears in fixed-format column 7. Under those conditions
/// the swap is a pure renaming of literal delimiters and preserves every literal value exactly.
pub fn quote_style_variant(source: &str) -> Option<String> {
    for line in source.split('\n') {
        let q = line.chars().filter(|&c| c == '\'').count();
        let d = line.chars().filter(|&c| c == '"').count();
        if q % 2 != 0 || d % 2 != 0 {
            return None; // cannot prove safe: skip
        }
        if matches!(line.chars().nth(6), Some('\'')) {
            return None; // fixed-format indicator area with a quote: skip
        }
    }
    let out: String = source
        .chars()
        .map(|c| match c {
            '\'' => '"',
            '"' => '\'',
            other => other,
        })
        .collect();
    if out == source {
        None
    } else {
        Some(out)
    }
}

/// Count case-insensitive occurrences of `name` as a whole word, outside quoted literals.
fn count_word(source: &str, name: &str) -> usize {
    let mut count = 0;
    let mut in_quote: Option<char> = None;
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if let Some(q) = in_quote {
            if c == q {
                in_quote = None;
            }
            i += 1;
            continue;
        }
        if c == '\'' || c == '"' {
            in_quote = Some(c);
            i += 1;
            continue;
        }
        if is_word_char(c) {
            let mut j = i;
            while j < chars.len() && is_word_char(chars[j]) {
                j += 1;
            }
            let word: String = chars[i..j].iter().collect();
            let is_match = word.len() == name.len()
                && word
                    .chars()
                    .zip(name.chars())
                    .all(|(a, b)| a.eq_ignore_ascii_case(&b));
            if is_match {
                count += 1;
            }
            i = j;
            continue;
        }
        i += 1;
    }
    count
}

fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-'
}

/// Replace every case-insensitive whole-word occurrence of `old` with `new`, outside quoted
/// literals. Used only when `new` has been proven absent from the file.
fn replace_word(source: &str, old: &str, new: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut in_quote: Option<char> = None;
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if let Some(q) = in_quote {
            out.push(c);
            if c == q {
                in_quote = None;
            }
            i += 1;
            continue;
        }
        if c == '\'' || c == '"' {
            in_quote = Some(c);
            out.push(c);
            i += 1;
            continue;
        }
        if is_word_char(c) {
            let mut j = i;
            while j < chars.len() && is_word_char(chars[j]) {
                j += 1;
            }
            let word: String = chars[i..j].iter().collect();
            let is_match = word.len() == old.len()
                && word
                    .chars()
                    .zip(old.chars())
                    .all(|(a, b)| a.eq_ignore_ascii_case(&b));
            if is_match {
                out.push_str(new);
            } else {
                out.push_str(&word);
            }
            i = j;
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

/// Collect candidate data names: the second token of level-number lines (01-49, 66, 77, 88) in
/// the DATA DIVISION, with the declaration casing. Names that live in another COBOL namespace
/// (SELECT file names, `INDEXED BY` index names, COPY targets) are excluded: a consistent
/// rename of a data name must never touch a file or index name that shares its spelling.
fn data_names(source: &str) -> Vec<String> {
    let mut in_data = false;
    let mut names: Vec<String> = Vec::new();
    let mut other_ns: Vec<String> = Vec::new(); // SELECT names + INDEXED BY names + COPY targets
    for line in source.split('\n') {
        let lu = line.trim().to_ascii_uppercase();
        if !in_data && lu.starts_with("DATA DIVISION") {
            in_data = true;
            continue;
        }
        if in_data && lu.starts_with("PROCEDURE DIVISION") {
            break;
        }
        if !in_data {
            if let Some(rest) = lu.trim_start().strip_prefix("SELECT") {
                if let Some(n) = rest.split_whitespace().next() {
                    other_ns.push(n.trim_end_matches('.').to_string());
                }
            }
            continue;
        }
        if lu.contains("INDEXED BY") {
            let after = lu.split("INDEXED BY").nth(1).unwrap_or("");
            for n in after.split_whitespace() {
                let n = n.trim_end_matches('.');
                if !n.is_empty() {
                    other_ns.push(n.to_string());
                }
            }
        }
        let t = line.trim_start();
        let mut toks = t.split_whitespace();
        let level = toks.next().unwrap_or("");
        let name = toks.next().unwrap_or("");
        let level_num: u8 = match level.trim_end_matches('.').parse() {
            Ok(n) => n,
            Err(_) => continue,
        };
        let valid_level = (1..=49).contains(&level_num) || matches!(level_num, 66 | 77 | 88);
        if !valid_level || name.is_empty() {
            continue;
        }
        let clean = name.trim_end_matches('.');
        if clean.is_empty() {
            continue;
        }
        let upper = clean.to_ascii_uppercase();
        if RESERVED.contains(&upper.as_str()) || other_ns.iter().any(|n| n == &upper) {
            continue;
        }
        // a comment/continuation marker in the indicator area (fixed format, col 7) or a
        // line-start comment disqualifies the declaration line
        let col7 = line.chars().nth(6);
        if t.starts_with('*') || matches!(col7, Some('*') | Some('-') | Some('/')) {
            continue;
        }
        if !names.iter().any(|n| n.eq_ignore_ascii_case(clean)) {
            names.push(clean.to_string());
        }
    }
    names
}

/// 4. Identifier renaming: rename ONE simple user data name consistently through the file.
/// Gates (all must hold, else `None`): the name is declared on a level-number line; it occurs
/// >= 2 times outside quotes; the new name (`<name>-R`, <= 30 chars) is absent from the file;
/// the name is not the target of a COPY statement. Case-insensitive whole-word replacement keeps
/// literals and comments untouched.
pub fn identifier_rename_variant(source: &str) -> Option<String> {
    for name in data_names(source) {
        let upper = name.to_ascii_uppercase();
        if count_word(source, &upper) < 2 {
            continue;
        }
        if name.len() > 28 {
            continue; // "<name>-R" must stay within the 30-char user-word limit
        }
        let new = format!("{name}-R");
        let new_up = new.to_ascii_uppercase();
        if count_word(source, &new_up) > 0 {
            continue; // collision with an existing identifier
        }
        let out = replace_word(source, &upper, &new);
        if out != source {
            return Some(out);
        }
    }
    None
}

/// 5. Copybook extraction + reinsertion: extract a self-contained WORKING-STORAGE chunk into a
/// copybook file (`WSCHUNK.cpy` written under `workdir`) and reinsert `COPY WSCHUNK.`. Only when
/// the file has no COPY/REPLACE/EXEC/`>>` constructs and the chunk consists solely of
/// level-numbered data lines with per-line balanced quotes — anything else is skipped (never
/// claimed).
pub fn copybook_variant(source: &str, workdir: &Path) -> Option<String> {
    let up = source.to_ascii_uppercase();
    let has_copy = source
        .split('\n')
        .any(|l| l.trim().to_ascii_uppercase().starts_with("COPY "));
    if has_copy || up.contains("REPLACE") || up.contains("EXEC ") || up.contains(">>") {
        return None;
    }
    let lines: Vec<&str> = source.split('\n').collect();
    let mut ws_start: Option<usize> = None;
    let mut proc_start: Option<usize> = None;
    for (i, l) in lines.iter().enumerate() {
        let t = l.trim().to_ascii_uppercase();
        if ws_start.is_none() && t.starts_with("WORKING-STORAGE SECTION") {
            ws_start = Some(i);
        }
        if t.starts_with("PROCEDURE DIVISION") {
            proc_start = Some(i);
            break;
        }
    }
    let (a, b) = match (ws_start, proc_start) {
        (Some(a), Some(b)) if b > a + 1 => (a, b),
        _ => return None, // no WS section or empty: not defensible
    };
    let chunk: Vec<&str> = lines[a + 1..b].to_vec();
    for l in &chunk {
        let t = l.trim_start();
        if t.is_empty() {
            continue;
        }
        let first = t.split_whitespace().next().unwrap_or("");
        let level: u8 = match first.trim_end_matches('.').parse() {
            Ok(n) => n,
            Err(_) => return None,
        };
        if !(1..=49).contains(&level) && !matches!(level, 66 | 77 | 88) {
            return None;
        }
        let q = l.chars().filter(|&c| c == '\'' || c == '"').count();
        if q % 2 != 0 {
            return None; // a literal may span lines: cannot prove the chunk self-contained
        }
        let lu = t.to_ascii_uppercase();
        for bad in ["OCCURS", "REDEFINES", "EXTERNAL", "GLOBAL", "EXEC", ">>"] {
            if lu.contains(bad) {
                return None;
            }
        }
    }
    let copybook = chunk.join("\n");
    let copy_name = "WSCHUNK";
    std::fs::write(workdir.join(format!("{copy_name}.cpy")), &copybook).ok()?;
    let mut out_lines: Vec<String> = Vec::with_capacity(lines.len() - chunk.len() + 1);
    out_lines.extend(lines[..a + 1].iter().map(|s| s.to_string()));
    out_lines.push(format!("       COPY {copy_name}."));
    out_lines.extend(lines[b..].iter().map(|s| s.to_string()));
    let out = out_lines.join("\n");
    if out == source {
        None
    } else {
        Some(out)
    }
}

/// The ordered transformations. The copybook transform needs a scratch directory.
fn transforms(source: &str, scratch: &Path) -> Vec<Transform> {
    let mut v: Vec<Transform> = Vec::with_capacity(MAX_VARIANTS_PER_BASE);
    let mut push = |kind: &'static str, skip_reason: String, text: Option<String>| {
        v.push(Transform {
            kind,
            skip_reason,
            text,
        });
    };
    match whitespace_variant(source) {
        Some(t) => push("whitespace", String::new(), Some(t)),
        None => push(
            "whitespace",
            "no distinct whitespace-only variant is defensible".into(),
            None,
        ),
    }
    match fixed_sequence_variant(source) {
        Some(t) => push("fixed_sequence", String::new(), Some(t)),
        None => push(
            "fixed_sequence",
            "no fixed-format sequence-number lines (cols 1-6 digits, col 7 blank); not applicable"
                .into(),
            None,
        ),
    }
    match quote_style_variant(source) {
        Some(t) => push("quote_style", String::new(), Some(t)),
        None => push(
            "quote_style",
            "quote characters are not per-line balanced (a literal may span lines or an \
             apostrophe hides inside a literal); swap cannot be proven safe"
                .into(),
            None,
        ),
    }
    match identifier_rename_variant(source) {
        Some(t) => push("identifier_rename", String::new(), Some(t)),
        None => push(
            "identifier_rename",
            "no simple data name passes the rename gates (declared level line, >= 2 unquoted \
             occurrences, collision-free <name>-R, not a COPY target)"
                .into(),
            None,
        ),
    }
    match copybook_variant(source, scratch) {
        Some(t) => push("copybook", String::new(), Some(t)),
        None => push(
            "copybook",
            "no self-contained WORKING-STORAGE chunk is defensibly extractable (file has \
             COPY/REPLACE/EXEC/>> constructs, no WS section, or the chunk is not purely \
             level-numbered data lines with balanced quotes)"
                .into(),
            None,
        ),
    }
    v
}

fn equivalent(a: &crate::heldout::BoundedRun, b: &crate::heldout::BoundedRun) -> bool {
    a.exit.is_some() && a.exit == b.exit && a.stdout == b.stdout
}

/// Run the mutation harness over a bounded sample of admitted valid programs.
pub fn run_mutation(root: &Path, store: &CorpusStore) -> Result<MutationReport, String> {
    let rows = crate::heldout::load_xcobol_programs(root)?;
    let bases = select_bases(&rows, MAX_BASES);
    if bases.is_empty() {
        return Err(
            "no mutation bases: no X-COBOL COMPLETE_PROGRAM file has a candidate-accepted \
             record in programs.json (run extract-xcobol with the candidate first)"
                .to_string(),
        );
    }
    let scratch = std::env::temp_dir().join(format!(
        "gnucobol-rs-corpus-mutation-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&scratch).map_err(|e| e.to_string())?;
    let mut rep = MutationReport {
        timeout_seconds: crate::heldout::TIMEOUT_SECS,
        summary: MutationSummary::default(),
        bases: Vec::with_capacity(bases.len()),
    };
    for row in &bases {
        let src = crate::heldout::resolve_source(store, root, row)?;
        let text = String::from_utf8_lossy(&src).into_owned();
        let original_sha = sha256_hex(&src);
        // The base must be a complete program we can prepare; verify once (bounded).
        let mut base_row = BaseProgramRow {
            file_id: row.file_id.clone(),
            repo: row.repo.clone(),
            bytes: row.bytes,
            structural_class: row.structural_class.clone(),
            partition: row.partition.clone(),
            candidate_phases_ok: row.candidate_phases_ok,
            variants: Vec::with_capacity(MAX_VARIANTS_PER_BASE),
        };
        // run the raw original once and cache it (the run path the simple variants use)
        let raw_original = run_bounded(&text);
        let mut verifiable = raw_original.exit.is_some();
        for tr in transforms(&text, &scratch) {
            if !verifiable {
                base_row.variants.push(VariantRow {
                    variant_type: tr.kind.to_string(),
                    original_sha256: original_sha.clone(),
                    variant_sha256: String::new(),
                    equivalent: false,
                    skipped: true,
                    skip_reason: format!(
                        "base program does not terminate within the {}s wall bound; metamorphic \
                         comparison is not verifiable",
                        crate::heldout::TIMEOUT_SECS
                    ),
                    original_exit: raw_original.exit,
                    variant_exit: None,
                    stdout_match: false,
                    diagnostic: raw_original
                        .error
                        .clone()
                        .unwrap_or_else(|| "base run did not complete".to_string()),
                    note: String::new(),
                });
                rep.summary.skipped += 1;
                continue;
            }
            let Some(variant) = tr.text else {
                base_row.variants.push(VariantRow {
                    variant_type: tr.kind.to_string(),
                    original_sha256: original_sha.clone(),
                    variant_sha256: String::new(),
                    equivalent: false,
                    skipped: true,
                    skip_reason: tr.skip_reason,
                    original_exit: raw_original.exit,
                    variant_exit: None,
                    stdout_match: false,
                    diagnostic: String::new(),
                    note: String::new(),
                });
                rep.summary.skipped += 1;
                continue;
            };
            // The copybook variant must be expanded (its COPY must resolve); run both sides
            // through the same pipeline.
            let (orig_run, var_run) = if tr.kind == "copybook" {
                let o = match expand_with(&text, &scratch) {
                    Ok(e) => run_bounded(&e),
                    Err(e) => {
                        base_row.variants.push(VariantRow {
                            variant_type: tr.kind.to_string(),
                            original_sha256: original_sha.clone(),
                            variant_sha256: sha256_hex(variant.as_bytes()),
                            equivalent: false,
                            skipped: true,
                            skip_reason: format!("copybook expansion failed: {e}"),
                            original_exit: raw_original.exit,
                            variant_exit: None,
                            stdout_match: false,
                            diagnostic: String::new(),
                            note: String::new(),
                        });
                        rep.summary.skipped += 1;
                        continue;
                    }
                };
                let v = match expand_with(&variant, &scratch) {
                    Ok(e) => run_bounded(&e),
                    Err(e) => {
                        base_row.variants.push(VariantRow {
                            variant_type: tr.kind.to_string(),
                            original_sha256: original_sha.clone(),
                            variant_sha256: sha256_hex(variant.as_bytes()),
                            equivalent: false,
                            skipped: true,
                            skip_reason: format!("copybook expansion failed: {e}"),
                            original_exit: o.exit,
                            variant_exit: None,
                            stdout_match: false,
                            diagnostic: String::new(),
                            note: String::new(),
                        });
                        rep.summary.skipped += 1;
                        continue;
                    }
                };
                (o, v)
            } else {
                (raw_original.clone(), run_bounded(&variant))
            };
            if var_run.timed_out || var_run.crashed {
                // the variant itself does not terminate: recorded as a divergence (honest)
                verifiable = false;
            }
            let equiv = equivalent(&orig_run, &var_run);
            let diagnostic = if equiv {
                String::new()
            } else {
                match (&orig_run.error, &var_run.error) {
                    (Some(e), _) => format!("original: {}", truncate(e, 200)),
                    (None, Some(e)) => format!("variant: {}", truncate(e, 200)),
                    (None, None) if orig_run.exit != var_run.exit => format!(
                        "exit mismatch: original {} vs variant {}",
                        orig_run.exit.unwrap_or(-1),
                        var_run.exit.unwrap_or(-1)
                    ),
                    _ => "stdout mismatch (bytes differ)".to_string(),
                }
            };
            base_row.variants.push(VariantRow {
                variant_type: tr.kind.to_string(),
                original_sha256: original_sha.clone(),
                variant_sha256: sha256_hex(variant.as_bytes()),
                equivalent: equiv,
                skipped: false,
                skip_reason: String::new(),
                original_exit: orig_run.exit,
                variant_exit: var_run.exit,
                stdout_match: orig_run.stdout == var_run.stdout,
                diagnostic,
                note: if tr.kind == "copybook" {
                    "expanded through copybook::expand (resolver rooted at a scratch dir); the \
                     original is expanded through the same pipeline"
                        .to_string()
                } else {
                    String::new()
                },
            });
            if equiv {
                rep.summary.equivalent += 1;
            } else {
                rep.summary.divergent += 1;
            }
            rep.summary.total_variants += 1;
        }
        rep.summary.total_bases += 1;
        rep.bases.push(base_row);
    }
    let _ = std::fs::remove_dir_all(&scratch);
    Ok(rep)
}

impl MutationReport {
    /// A compact human-readable markdown summary of the metamorphic results.
    pub fn summary_md(&self) -> String {
        let mut md = String::new();
        md.push_str("# Mutation / metamorphic testing (Phase 10.5)\n\n");
        md.push_str(&format!(
            "{} bases, {} variants ({} equivalent, {} divergent, {} skipped); every run bounded \
             at {}s.\n\n",
            self.summary.total_bases,
            self.summary.total_variants,
            self.summary.equivalent,
            self.summary.divergent,
            self.summary.skipped,
            self.timeout_seconds
        ));
        md.push_str("Only defensible transformations are claimed; anything not provably safe is\n");
        md.push_str("skipped with a recorded reason. Divergent variants are reported honestly.\n");
        md.push_str("See `mutation-results.json` for the per-base variant rows.\n");
        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heldout::run_bounded;

    const HELLO: &str = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       DATA DIVISION.\n       WORKING-STORAGE SECTION.\n       01 WS-X PIC 9(4) VALUE 7.\n       PROCEDURE DIVISION.\n           DISPLAY WS-X.\n           STOP RUN.\n";

    fn assert_equiv(a: &str, b: &str) {
        let ra = run_bounded(a);
        let rb = run_bounded(b);
        assert!(ra.exit.is_some(), "original must run: {:?}", ra.error);
        assert!(rb.exit.is_some(), "variant must run: {:?}", rb.error);
        assert_eq!(ra.exit, rb.exit);
        assert_eq!(ra.stdout, rb.stdout);
    }

    #[test]
    fn whitespace_variant_is_equivalent() {
        let v = whitespace_variant(HELLO).expect("whitespace variant");
        assert_ne!(v, HELLO);
        assert_equiv(HELLO, &v);
    }

    #[test]
    fn whitespace_variant_keeps_token_stream() {
        let v = whitespace_variant(HELLO).unwrap();
        // exactly the same non-blank lines in the same order
        let orig_lines: Vec<&str> = HELLO.lines().filter(|l| !l.trim().is_empty()).collect();
        let var_lines: Vec<&str> = v.lines().filter(|l| !l.trim().is_empty()).collect();
        assert_eq!(orig_lines, var_lines);
    }

    #[test]
    fn fixed_sequence_variant_changes_sequence_area() {
        let fixed = "000100 IDENTIFICATION DIVISION.\n000200 PROGRAM-ID. T.\n000300 PROCEDURE DIVISION.\n000400     DISPLAY \"OK\".\n000500     STOP RUN.\n";
        let v = fixed_sequence_variant(fixed).expect("fixed-format sequence variant");
        assert_ne!(v, fixed);
        assert!(v.starts_with("000001 "));
        // under the fixed-format interpretation (the -fixed pipeline) both compile to the same
        // free-form text: the sequence area is not program text
        let f1 = gnucobol_rs::frontend::fixed_to_free(fixed);
        let f2 = gnucobol_rs::frontend::fixed_to_free(&v);
        assert_eq!(f1, f2);
        assert_equiv(&f1, &f2);
        // free-format source without sequence lines is skipped
        assert!(fixed_sequence_variant(HELLO).is_none());
    }

    #[test]
    fn quote_style_variant_swaps_only_balanced_files() {
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       PROCEDURE DIVISION.\n           DISPLAY \"HI\".\n           STOP RUN.\n";
        let v = quote_style_variant(src).expect("quote variant");
        assert!(v.contains("DISPLAY 'HI'"));
        assert_equiv(src, &v);
        // an apostrophe inside a double-quoted literal makes the swap unprovable -> skipped
        let bad = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       PROCEDURE DIVISION.\n           DISPLAY \"IT'S\".\n           STOP RUN.\n";
        assert!(quote_style_variant(bad).is_none());
    }

    #[test]
    fn identifier_rename_is_equivalent() {
        let v = identifier_rename_variant(HELLO).expect("rename variant");
        assert!(v.contains("WS-X-R"));
        assert!(!v.contains("DISPLAY WS-X."));
        assert_equiv(HELLO, &v);
    }

    #[test]
    fn identifier_rename_skips_reserved_and_single_use_names() {
        // a single-occurrence name is not renamed
        let single = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       DATA DIVISION.\n       WORKING-STORAGE SECTION.\n       01 LONELY PIC 9.\n       PROCEDURE DIVISION.\n           DISPLAY 1.\n           STOP RUN.\n";
        assert!(identifier_rename_variant(single).is_none());
        // FILLER is never a rename candidate (it is reserved)
        let filler = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       DATA DIVISION.\n       WORKING-STORAGE SECTION.\n       01 FILLER PIC X(2) VALUE \"AA\".\n       01 FILLER PIC X(2) VALUE \"BB\".\n       PROCEDURE DIVISION.\n           DISPLAY \"X\".\n           STOP RUN.\n";
        assert!(identifier_rename_variant(filler).is_none());
    }

    #[test]
    fn copybook_variant_is_equivalent() {
        let dir = tempfile::tempdir().unwrap();
        let v = copybook_variant(HELLO, dir.path()).expect("copybook variant");
        assert!(v.contains("COPY WSCHUNK."));
        assert!(dir.path().join("WSCHUNK.cpy").exists());
        // both sides run through the same expansion pipeline
        let o = expand_with(HELLO, dir.path()).unwrap();
        let x = expand_with(&v, dir.path()).unwrap();
        assert_eq!(o, x); // expansion restores the original text exactly
        assert_equiv(&o, &x);
        // files with COPY statements are never restructured
        let with_copy = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       DATA DIVISION.\n       WORKING-STORAGE SECTION.\n       COPY BOOK1.\n       PROCEDURE DIVISION.\n           STOP RUN.\n";
        assert!(copybook_variant(with_copy, dir.path()).is_none());
    }

    #[test]
    fn select_bases_bounds_and_filters() {
        let rows = vec![
            crate::heldout::XcobolRow {
                file_id: "xcobol/r/a.cob".into(),
                repo: "r".into(),
                path: "r/a.cob".into(),
                bytes: 1,
                extension: "cob".into(),
                structural_class: "COMPLETE_PROGRAM".into(),
                encoding: "UTF-8/ASCII".into(),
                dialect_accepted: None,
                candidate_first_failure: None,
                candidate_phases_ok: true,
                partition: "DEVELOPMENT".into(),
                exact_sha256: String::new(),
            },
            crate::heldout::XcobolRow {
                file_id: "xcobol/r/b.cob".into(),
                repo: "r".into(),
                path: "r/b.cob".into(),
                bytes: 1,
                extension: "cob".into(),
                structural_class: "COMPLETE_PROGRAM".into(),
                encoding: "UTF-8/ASCII".into(),
                dialect_accepted: None,
                candidate_first_failure: None,
                candidate_phases_ok: false, // rejected: not a base
                partition: "DEVELOPMENT".into(),
                exact_sha256: String::new(),
            },
            crate::heldout::XcobolRow {
                file_id: "xcobol/r/c.cpy".into(),
                repo: "r".into(),
                path: "r/c.cpy".into(),
                bytes: 1,
                extension: "cpy".into(),
                structural_class: "COPYBOOK_OR_DATA".into(),
                encoding: "UTF-8/ASCII".into(),
                dialect_accepted: None,
                candidate_first_failure: None,
                candidate_phases_ok: true,
                partition: "DEVELOPMENT".into(),
                exact_sha256: String::new(),
            },
        ];
        let bases = select_bases(&rows, 10);
        assert_eq!(bases.len(), 1);
        assert_eq!(bases[0].file_id, "xcobol/r/a.cob");
        assert!(select_bases(&rows, 0).is_empty());
    }
}
