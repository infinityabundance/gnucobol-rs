//! Phase 4 — official GnuCOBOL manual examples (from the texinfo sources).
//!
//! Parses the manual source (not the rendered HTML): every `@example` / `@smallexample` block is
//! extracted with its section context, classified (complete executable / compile-only /
//! copybook / partial snippet / pseudocode / command example / expected output / C-code),
//! materialized, and replayed with the documented command (or a derived minimal command when the
//! manual's is incomplete -- the derivation is recorded, never a silent repair).

use crate::extract::oracle;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// One texinfo example block with its context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TexiBlock {
    /// Nearest section heading (`@node`/`@section`/`@chapter`/...) before the block.
    pub section: String,
    pub line: usize,
    /// `example` or `smallexample`.
    pub kind: String,
    /// The raw block content (between the `@example` and `@end example` lines).
    pub content: String,
}

/// Exactly one classification per manual code block (spec 4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BlockClass {
    CompleteExecutable,
    CompleteCompileOnly,
    Copybook,
    PartialSnippet,
    Pseudocode,
    CommandExample,
    ExpectedOutput,
    CCode,
    Other,
}

/// Parse the manual texinfo into blocks with their section context.
pub fn parse_texi(path: &Path) -> Result<Vec<TexiBlock>, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let lines: Vec<&str> = text.lines().collect();
    let mut blocks = Vec::new();
    let mut section = String::new();
    let mut i = 0usize;
    while i < lines.len() {
        let l = lines[i];
        let t = l.trim();
        if t.starts_with("@node ")
            || t.starts_with("@chapter ")
            || t.starts_with("@section ")
            || t.starts_with("@subsection ")
            || t.starts_with("@subsubsection ")
            || t.starts_with("@appendix")
            || t.starts_with("@unnumbered")
        {
            section = t
                .split_once(' ')
                .map(|(_, rest)| rest.trim().to_string())
                .unwrap_or_default();
        }
        if t == "@example" || t == "@smallexample" {
            let mut content = Vec::new();
            let mut j = i + 1;
            while j < lines.len() && !lines[j].trim_start().starts_with("@end ") {
                content.push(lines[j]);
                j += 1;
            }
            blocks.push(TexiBlock {
                section: section.clone(),
                line: i + 1,
                kind: t.trim_start_matches('@').to_string(),
                content: content.join("\n"),
            });
            i = j;
        }
        i += 1;
    }
    Ok(blocks)
}

/// Resolve `@var{...}` (placeholders) in a block; presence means pseudocode, not source.
/// (`@{` is texinfo's escape for a literal `{` in C code -- NOT a placeholder.)
fn has_var_placeholder(content: &str) -> bool {
    content.contains("@var{")
}

/// The decorated filename from a `---- hello.cob ---...` line, if any.
pub fn decorated_filename(content: &str) -> Option<String> {
    for line in content.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("----") {
            let rest = rest.trim_start();
            if let Some(end) = rest.find("---") {
                let name = rest[..end].trim();
                if !name.is_empty() && name.chars().all(|c| c.is_ascii_graphic() || c == '.') {
                    return Some(name.to_string());
                }
            }
        }
    }
    None
}

/// Strip the decorative `----` border lines from a block.
pub fn strip_decorations(content: &str) -> String {
    content
        .lines()
        .filter(|l| {
            let t = l.trim();
            !(t.starts_with("----") && t.ends_with("---")) && t != "-----" && t != "-------"
        })
        .collect::<Vec<&str>>()
        .join("\n")
}

/// Whether the block is a shell session (`$ cmd` lines with optional output).
pub fn is_shell_session(content: &str) -> bool {
    content
        .lines()
        .find(|l| !l.trim().is_empty())
        .map(|l| l.trim().starts_with('$'))
        .unwrap_or(false)
}

/// Whether the block is COBOL source (identification/program-id/procedure markers).
pub fn is_cobol_source(content: &str) -> bool {
    let up = content.to_ascii_uppercase();
    up.contains("IDENTIFICATION") || up.contains("PROGRAM-ID") || up.contains("PROCEDURE DIVISION")
}

/// Whether a line looks like a COBOL data-item declaration (level number + name).
fn is_data_item_line(l: &str) -> bool {
    let t = l.trim();
    let mut chars = t.chars();
    let mut level = String::new();
    for c in chars.by_ref() {
        if c.is_ascii_digit() {
            level.push(c);
        } else {
            break;
        }
    }
    if level.is_empty()
        || level
            .parse::<u16>()
            .map(|n| n > 0 && n < 100)
            .unwrap_or(false)
            == false
    {
        return false;
    }
    let rest = t[level.len()..].trim_start();
    rest.starts_with(|c: char| c.is_ascii_alphabetic())
}

/// Classify one block (exactly one class).
pub fn classify_block(content: &str) -> BlockClass {
    let stripped = strip_decorations(content);
    let trimmed = stripped.trim();
    if trimmed.is_empty() {
        return BlockClass::Other;
    }
    if is_shell_session(&trimmed) {
        return BlockClass::CommandExample;
    }
    if has_var_placeholder(&trimmed) {
        return BlockClass::Pseudocode;
    }
    // compiler listing output (the `-t`/`-T` listing examples): a page header + PG/LN ruler
    let up0 = trimmed.to_ascii_uppercase();
    if up0.contains("PG/LN") || (up0.contains("PAGE 000") && up0.contains("LINE")) {
        return BlockClass::ExpectedOutput;
    }
    // a decorated `---- name.c ----` block is C source, whatever its content markers
    if decorated_filename(content)
        .map(|f| f.ends_with(".c"))
        .unwrap_or(false)
    {
        return BlockClass::CCode;
    }
    if !is_cobol_source(&trimmed) {
        let up = trimmed.to_ascii_uppercase();
        if up.contains("#INCLUDE")
            || up.contains("COB_EXPIMP")
            || up.contains("INT (*")
            || up.contains("LIBCOB")
            || up.contains("CBL_")
        {
            return BlockClass::CCode;
        }
        // data-only blocks (copybooks): every non-empty line is a level-numbered item
        let lines: Vec<&str> = trimmed
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();
        if !lines.is_empty() && lines.iter().all(|l| is_data_item_line(l)) {
            return BlockClass::Copybook;
        }
        // COBOL statement fragments (MOVE/DISPLAY/ADD/IF/... at line starts)
        let verbs = [
            "MOVE",
            "DISPLAY",
            "ADD",
            "SUBTRACT",
            "MULTIPLY",
            "DIVIDE",
            "COMPUTE",
            "IF",
            "ELSE",
            "END-IF",
            "PERFORM",
            "ACCEPT",
            "CALL",
            "STRING",
            "UNSTRING",
            "INSPECT",
            "SET",
            "INITIALIZE",
            "EVALUATE",
            "READ",
            "WRITE",
            "OPEN",
            "CLOSE",
            "GO TO",
            "STOP",
            "GOBACK",
            "EXIT",
            "COPY",
            "INVOKE",
            "CONTINUE",
            "WHEN",
            "SORT",
            "MERGE",
            "SEARCH",
            "DELETE",
            "REWRITE",
            "START",
            "UNLOCK",
            "RETURN",
        ];
        if lines.iter().any(|l| verbs.iter().any(|v| l.starts_with(v))) {
            return BlockClass::PartialSnippet;
        }
        return BlockClass::Other;
    }
    let up = stripped.to_ascii_uppercase();
    // copybook: data-only declarations without PROGRAM-ID
    let has_program_id = up.contains("PROGRAM-ID");
    let has_procedure = up.contains("PROCEDURE DIVISION");
    let has_run_end = up.contains("STOP RUN")
        || up.contains("GOBACK")
        || up.contains("STOP")
        || up.contains("EXIT PROGRAM");
    if !has_program_id {
        if up.contains("WORKING-STORAGE")
            || up.contains("LINKAGE")
            || up.contains("SCREEN SECTION")
            || up.contains("FILE SECTION")
        {
            return BlockClass::Copybook;
        }
        return BlockClass::PartialSnippet;
    }
    if has_procedure && has_run_end {
        BlockClass::CompleteExecutable
    } else {
        BlockClass::CompleteCompileOnly
    }
}

/// Derive the source filename for a COBOL example: the decorated name, else `<program-id>.cob`.
pub fn derive_filename(content: &str) -> Option<String> {
    if let Some(f) = decorated_filename(content) {
        return Some(f);
    }
    let up = content.to_ascii_uppercase();
    let mut lines = up.lines();
    let mut prog_id = None;
    for l in lines.by_ref() {
        if let Some(rest) = l.split_once("PROGRAM-ID") {
            let after = rest.1;
            if let Some(dot) = after.find('.') {
                let name: String = after[..dot]
                    .chars()
                    .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
                    .collect();
                if !name.is_empty() {
                    prog_id = Some(name.to_lowercase());
                    break;
                }
            }
        }
    }
    let _ = lines;
    prog_id.map(|p| format!("{p}.cob"))
}

/// The free/fixed format guess for a manual example (col-1 code = free, like the manual's
/// `hellonew` example).
pub fn format_of(content: &str) -> String {
    let stripped = strip_decorations(content);
    let first = stripped
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("");
    if first.trim_start().len() == first.len() && !first.starts_with("      ") {
        "free".to_string()
    } else {
        "fixed".to_string()
    }
}

/// One extracted manual example with its measured outcomes.
#[derive(Debug, Clone, Serialize)]
pub struct ManualExample {
    pub program_id: String,
    pub section: String,
    pub line: usize,
    pub classification: String,
    pub filename: String,
    pub source_format: String,
    pub content_sha256: String,
    /// The documented/deduced compile+run command (derivation recorded in `command_note`).
    pub compile_command: String,
    pub run_command: String,
    pub command_note: String,
    pub oracle_compile_exit: Option<i32>,
    pub oracle_run_exit: Option<i32>,
    pub oracle_stdout_sha256: String,
    pub oracle_stderr_sha256: String,
    pub expected_output_sha256: String,
    pub replay_verdict: String,
    pub candidate_first_failure: Option<(String, String)>,
    pub candidate_phases_ok: bool,
}

/// Extract + verify the manual examples for one lane. `packages_root` receives the materialized
/// sources; the reports are written under `out_dir`.
pub fn extract_manual(
    texi_path: &Path,
    lane: &str,
    revision: &str,
    packages_root: &Path,
    out_dir: &Path,
    with_candidate: bool,
) -> Result<BTreeMap<String, usize>, String> {
    let oracle = crate::extract::oracle::OracleEnv::host_default()?;
    let blocks = parse_texi(texi_path)?;
    let mut examples: Vec<ManualExample> = Vec::new();
    let mut snippets: Vec<serde_json::Value> = Vec::new();
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();

    let lane_dir = packages_root.join(format!("gnucobol-manual-{lane}"));
    std::fs::create_dir_all(&lane_dir).map_err(|e| e.to_string())?;

    // Pass 1: classify every block; collect the complete COBOL examples and every materializable
    // COBOL source (multi-file examples need their sibling blocks).
    struct CobolBlock {
        bi: usize,
        filename: String,
        source: String,
        format: String,
        class: BlockClass,
    }
    let mut cobol_blocks: Vec<CobolBlock> = Vec::new();
    let mut all_sources: BTreeMap<String, String> = BTreeMap::new();
    for (bi, block) in blocks.iter().enumerate() {
        let class = classify_block(&block.content);
        let class_name = format!("{:?}", class);
        *counts.entry(class_name.clone()).or_default() += 1;
        let stripped = strip_decorations(&block.content);
        snippets.push(serde_json::json!({
            "section": block.section,
            "line": block.line,
            "kind": block.kind,
            "classification": class_name,
            "content": block.content,
        }));
        if matches!(
            class,
            BlockClass::CompleteExecutable | BlockClass::CompleteCompileOnly
        ) {
            let filename =
                derive_filename(&block.content).unwrap_or_else(|| format!("ex{:03}.cob", bi));
            all_sources
                .entry(filename.clone())
                .or_insert_with(|| stripped.clone());
            cobol_blocks.push(CobolBlock {
                bi,
                filename,
                source: stripped,
                format: format_of(&block.content),
                class,
            });
        } else if class == BlockClass::CCode {
            // C sources referenced by compile commands (multi-language examples): materialize
            // them too, from their decorated filename, with the texinfo `@{}` brace escape
            // unescaped (`@{}` renders `{`; the raw source must contain the real brace).
            if let Some(fname) = decorated_filename(&block.content) {
                if fname.ends_with(".c") {
                    let real = stripped.replace("@{", "{").replace("@}", "}");
                    all_sources.entry(fname).or_insert_with(|| real);
                }
            }
        }
    }

    // Pass 2: materialize each complete example (with its sibling files), replay it against the
    // oracle with the documented command, and probe the candidate.
    for ex in &cobol_blocks {
        let (compile_command, run_command, command_note, expected_output) =
            derive_commands(&blocks, ex.bi, &ex.filename, &ex.format);
        // materialize every source file the documented command references (sibling blocks from
        // the same lane), plus the example itself
        let mut materialized: BTreeMap<String, String> = BTreeMap::new();
        for tok in compile_command.split_whitespace() {
            let t = tok.trim_matches(|c: char| {
                !c.is_ascii_alphanumeric() && c != '.' && c != '-' && c != '_'
            });
            if t.ends_with(".cob") || t.ends_with(".cbl") || t.ends_with(".c") {
                if let Some(src) = all_sources.get(t) {
                    materialized.insert(t.to_string(), src.clone());
                }
            }
        }
        materialized
            .entry(ex.filename.clone())
            .or_insert_with(|| ex.source.clone());
        for (name, src) in &materialized {
            let p = lane_dir.join(name);
            std::fs::write(&p, src).map_err(|e| e.to_string())?;
        }

        let sha = crate::store::sha256_hex(ex.source.as_bytes());
        let expected_sha = crate::store::sha256_hex(expected_output.as_bytes());

        // oracle replay: compile then run in the lane dir
        let compile_out = oracle::run_step(&oracle, &lane_dir, &compile_command, &[]);
        let run_out = if compile_out.exit == Some(0) && !run_command.is_empty() {
            oracle::run_step(&oracle, &lane_dir, &run_command, &[])
        } else {
            oracle::StepOutcome {
                exit: None,
                stdout: Vec::new(),
                stderr: Vec::new(),
                exec_error: None,
                skipped: false,
                skip_reason: String::new(),
                retried: false,
            }
        };
        let stdout_sha = crate::store::sha256_hex(&run_out.stdout);
        let stderr_sha = crate::store::sha256_hex(&run_out.stderr);
        let verdict = if let Some(e) = &compile_out.exec_error {
            format!("compile exec error: {e}")
        } else if compile_out.exit != Some(0) {
            format!(
                "compile exit {} (expected 0)",
                compile_out.exit.unwrap_or(-1)
            )
        } else if run_out.exit != Some(0) {
            format!("run exit {}", run_out.exit.unwrap_or(-1))
        } else if !expected_output.is_empty() && stdout_sha != expected_sha {
            // intent check: the manual's stated output is prose -- a terminal-newline-only
            // difference is a text match, recorded distinctly from byte parity
            let actual_text = String::from_utf8_lossy(&run_out.stdout);
            let expected_text = expected_output.trim_end_matches('\n');
            if actual_text.trim_end_matches('\n') == expected_text {
                "stdout matches the stated output (modulo terminal newline)".to_string()
            } else {
                "stdout differs from the manual's stated output".to_string()
            }
        } else if !expected_output.is_empty() {
            "match".to_string()
        } else {
            "executed (no stated output to compare)".to_string()
        };
        *counts.entry(format!("verified:{verdict}")).or_default() += 1;

        // candidate probe (bounded)
        let (cand_failure, cand_ok) = if with_candidate {
            let probes = crate::extract::candidate::probe_dir(&lane_dir, &ex.filename, true);
            let ff = probes.iter().find(|p| !p.ok);
            (
                ff.map(|p| (p.phase.clone(), p.diagnostic.clone())),
                ff.is_none(),
            )
        } else {
            (None, false)
        };

        examples.push(ManualExample {
            program_id: format!("gnucobol-manual-{lane}/{}", ex.filename),
            section: blocks[ex.bi].section.clone(),
            line: blocks[ex.bi].line,
            classification: format!("{:?}", ex.class),
            filename: ex.filename.clone(),
            source_format: ex.format.clone(),
            content_sha256: sha,
            compile_command,
            run_command,
            command_note,
            oracle_compile_exit: compile_out.exit,
            oracle_run_exit: run_out.exit,
            oracle_stdout_sha256: stdout_sha,
            oracle_stderr_sha256: stderr_sha,
            expected_output_sha256: expected_sha,
            replay_verdict: verdict,
            candidate_first_failure: cand_failure,
            candidate_phases_ok: cand_ok,
        });
    }

    std::fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
    let _ = revision;
    write_json(out_dir, "examples.json", &examples)?;
    write_json(out_dir, "snippets.json", &snippets)?;
    write_json(out_dir, "accuracy.json", &examples)?;
    let mut md = String::new();
    md.push_str("# GnuCOBOL manual examples (Phase 4)\n\n");
    md.push_str(&format!("source: {}\n\n", texi_path.display()));
    md.push_str("| classification | count |\n|---|---|\n");
    for (k, v) in &counts {
        if !k.starts_with("verified:") {
            md.push_str(&format!("| {k} | {v} |\n"));
        }
    }
    md.push('\n');
    md.push_str("## replay verdicts\n");
    for (k, v) in &counts {
        if k.starts_with("verified:") {
            md.push_str(&format!("- {k}: {v}\n"));
        }
    }
    std::fs::write(out_dir.join("summary.md"), md).map_err(|e| e.to_string())?;
    counts.insert("total_blocks".into(), blocks.len());
    Ok(counts)
}

/// Derive the documented compile+run commands for a COBOL example. When the manual's next shell
/// block states the command, it is used; otherwise a minimal command is derived from the source
/// format (`cobc -x [-free] <file>`) and the derivation is recorded.
fn derive_commands(
    blocks: &[TexiBlock],
    bi: usize,
    filename: &str,
    format: &str,
) -> (String, String, String, String) {
    // look ahead at the next shell block
    for nb in blocks.iter().skip(bi + 1) {
        if !is_shell_session(&nb.content) {
            continue;
        }
        let shell = strip_decorations(&nb.content);
        let lines: Vec<&str> = shell.lines().map(|l| l.trim()).collect();
        let mut compile_cmd = String::new();
        let mut run_cmd = String::new();
        let mut output = Vec::new();
        let mut in_output = false;
        for l in &lines {
            if let Some(rest) = l.strip_prefix('$') {
                in_output = false;
                let cmd = rest.trim().to_string();
                if cmd.contains("cobc") && !cmd.contains("cob-config") {
                    if compile_cmd.is_empty() {
                        compile_cmd = cmd;
                    } else {
                        run_cmd = cmd;
                    }
                } else if !cmd.contains("cobc") {
                    // non-cobc command after the compile (e.g. ./hello, ./hello-world)
                    if run_cmd.is_empty() {
                        run_cmd = cmd;
                    }
                }
            } else if !l.is_empty() {
                in_output = true;
                output.push(*l);
            }
            let _ = in_output;
        }
        if !compile_cmd.is_empty() {
            let expected = output.join("\n");
            let note = "documented in the manual's following shell block".to_string();
            return (compile_cmd, run_cmd, note, expected);
        }
        break;
    }
    // derive a minimal command
    let free = if format == "free" { " -free" } else { "" };
    let compile_cmd = format!("cobc -x{free} {filename}");
    let run_cmd = format!(
        "./{}",
        filename.trim_end_matches(".cob").trim_end_matches(".cbl")
    );
    let note = "manual gives no explicit command; derived minimal command (recorded, never a \
                silent repair of the manual)"
        .to_string();
    (compile_cmd, run_cmd, note, String::new())
}

fn write_json<T: Serialize>(dir: &Path, name: &str, v: &T) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(v).map_err(|e| e.to_string())?;
    std::fs::write(dir.join(name), json).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const HELLO: &str = "---- hello.cob -------------------------\n      * Sample COBOL program\n       IDENTIFICATION DIVISION.\n       PROGRAM-ID. hello.\n       PROCEDURE DIVISION.\n           DISPLAY \"Hello, world!\".\n           STOP RUN.\n----------------------------------------\n";
    const HELLONEW: &str = "---- hellonew.cob ----------------\n*> Sample GnuCOBOL program\nidentification division.\nprogram-id. hellonew.\nprocedure division.\ndisplay\n   \"Hello, new world!\"\nend-display\ngoback.\n----------------------------------\n";
    const PSEUDO: &str = "Select @var{file} assign to \"/tmp/myfile\".\n";

    #[test]
    fn classifies_complete_examples() {
        assert_eq!(classify_block(HELLO), BlockClass::CompleteExecutable);
        assert_eq!(classify_block(HELLONEW), BlockClass::CompleteExecutable);
        assert_eq!(classify_block(PSEUDO), BlockClass::Pseudocode);
    }

    #[test]
    fn decoration_and_filename() {
        assert_eq!(decorated_filename(HELLO).as_deref(), Some("hello.cob"));
        assert_eq!(
            decorated_filename(HELLONEW).as_deref(),
            Some("hellonew.cob")
        );
        assert!(!strip_decorations(HELLO).contains("----"));
        assert_eq!(format_of(HELLO), "fixed");
        assert_eq!(format_of(HELLONEW), "free");
        assert_eq!(derive_filename(HELLO).as_deref(), Some("hello.cob"));
        assert_eq!(derive_filename(HELLONEW).as_deref(), Some("hellonew.cob"));
    }

    #[test]
    fn shell_session_detected() {
        let shell = "$ cobc -x hello.cob\n$ ./hello\nHello, world!\n";
        assert!(is_shell_session(shell));
        assert!(!is_shell_session(HELLO));
        assert_eq!(classify_block(shell), BlockClass::CommandExample);
    }

    #[test]
    fn copybook_and_partial() {
        let cb = "       01  EMPLOYEE-RECORD.\n           05  EMP-NAME PIC X(30).\n";
        assert_eq!(classify_block(cb), BlockClass::Copybook);
        let partial = "           MOVE A TO B.\n";
        assert_eq!(classify_block(partial), BlockClass::PartialSnippet);
    }

    #[test]
    fn parses_texi_blocks() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("t.texi");
        std::fs::write(
            &p,
            "@section Getting started\n\n@example\n---- hi.cob ---\n       IDENTIFICATION DIVISION.\n@end example\n\n@example\n$ cobc -x hi.cob\n@end example\n",
        )
        .unwrap();
        let blocks = parse_texi(&p).unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].section, "Getting started");
        assert_eq!(blocks[1].content.trim(), "$ cobc -x hi.cob");
    }
}
