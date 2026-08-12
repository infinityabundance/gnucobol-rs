//! Diagnostic-unblocked GnuCOBOL Autotest lane (Phases 1–4).
//!
//! A separate, additive lane whose ONE narrow purpose: allow later semantic/runtime checks in
//! upstream Autotest groups to run even when `gnucobol-rs` does not reproduce exact compiler
//! diagnostic text. It MUST NOT weaken any other part of the testsuite.
//!
//! Three views stay separate and are NEVER conflated:
//!   A. pristine upstream testsuite — untouched, authoritative, immutable;
//!   B. diagnostic-unblocked testsuite — derived mechanically, ONLY expected compiler-diagnostic
//!      stream fields become Autotest `ignore`; commands, exit statuses, source, runtime output,
//!      generated-file expectations, environment, ordering, skip/xfail all remain identical;
//!   C. existing step/corpus phase probes — preserved, not replaced.
//!
//! The transformer decides from the upstream test structure + the nature of the expected compiler
//! diagnostic contract ONLY (never from candidate behaviour). Anything uncertain is left untouched
//! and recorded. The patch is generated deterministically and then independently re-verified by
//! `gate_patch`, which parses the actual diff and proves every hunk is legal.

use crate::extract::m4::{scan_spanned, SpannedArg, SpannedMacro};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Version of the transformer rules (recorded in every artifact so rule changes are visible).
pub const TRANSFORMER_VERSION: &str = "gnurust-diag-unblocked-transform-v1";

/// How a diagnostic stream is disposed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DiagnosticDisposition {
    /// Not a candidate: leave untouched (runtime, helper, unknown shape, empty expectation, ...).
    Preserve,
    /// Proven compiler-diagnostic stdout expectation -> `ignore`.
    IgnoreCompilerStdout,
    /// Proven compiler-diagnostic stderr expectation -> `ignore`.
    IgnoreCompilerStderr,
    /// Both streams proven compiler-diagnostic -> `ignore`.
    IgnoreCompilerBoth,
    /// Recognized compiler step but the stream cannot be *proven* diagnostic-only.
    Uncertain,
}

impl DiagnosticDisposition {
    pub fn ignores_stdout(&self) -> bool {
        matches!(self, Self::IgnoreCompilerStdout | Self::IgnoreCompilerBoth)
    }
    pub fn ignores_stderr(&self) -> bool {
        matches!(self, Self::IgnoreCompilerStderr | Self::IgnoreCompilerBoth)
    }
}

/// The command shape classification (Phase 1: only compiler-producing steps may be candidates).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CommandShape {
    /// `$COMPILE`, `$COMPILE_ONLY`, `$COMPILE_MODULE`, `$COBC` (or env-prefixed).
    Compiler,
    /// `$COMPILE_LISTING` / `$COMPILE_LISTING0`: the stdout is a generated listing artifact.
    CompilerListing,
    /// `$COBCRUN*`, `./prog`, absolute program paths: runtime execution (never a candidate).
    Runtime,
    /// `$GREP`, `$SED`, `diff`, `test`, `mkdir`, shell builtins, `|` pipelines, redirects.
    ShellHelper,
    /// Generated-file inspection / artifact checks.
    GeneratedFile,
    /// `$UNIFY_LISTING`, listings postprocessing, other unknown shapes.
    Unknown,
}

/// A single proposed (or rejected) transformation with machine-readable evidence.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Transformation {
    pub schema: String,
    pub transformer_version: String,
    /// Source `.at` file, family-relative (`testsuite.src/syn_move.at`).
    pub source_file: String,
    /// AT_SETUP title of the group.
    pub group_title: String,
    /// Group identity: source_file + group ordinal within the file.
    pub group_index: usize,
    /// AT_CHECK step index within the group (0-based).
    pub step_index: usize,
    /// Source line of the AT_CHECK macro.
    pub source_line: usize,
    /// The command argument (byte-identical in pristine and transformed).
    pub command: String,
    pub command_shape: CommandShape,
    /// Expected exit status (never changed).
    pub expected_status: String,
    /// sha256 of the pristine expected stdout ("" when empty).
    pub stdout_sha256: String,
    /// sha256 of the pristine expected stderr ("" when empty).
    pub stderr_sha256: String,
    /// Which stream is proposed for ignore.
    pub disposition: DiagnosticDisposition,
    /// Why it is classified as compiler-diagnostic-only (or why not).
    pub reason: String,
    /// Whether the original expectation was already `ignore` (nothing to change).
    pub already_ignored: bool,
    /// sha256 of the transformed expectation ("" when unchanged/empty).
    pub transformed_stdout_sha256: String,
    /// sha256 of the transformed expectation ("" when unchanged/empty).
    pub transformed_stderr_sha256: String,
    /// Original source revision (admitted git/archive identity recorded by the caller).
    pub source_revision: String,
    /// Extra evidence: group skip/xfail conditions (must never change).
    pub group_skip: Vec<String>,
    pub group_xfail: Vec<String>,
    /// Files the group captures (`AT_CAPTURE_FILE`) — never weakened.
    pub group_capture_files: Vec<String>,
}

impl Transformation {
    fn sha(s: &str) -> String {
        format!("{:x}", Sha256::digest(s.as_bytes()))
    }
}

// ---------------------------------------------------------------------------------------------
// command-shape classifier
// ---------------------------------------------------------------------------------------------

/// Leading environment assignments (`VAR=value ...`) are stripped before shape classification:
/// `DD_PRINTOUT=... ./prog` is a runtime, `COB_SWITCH_1=ON $COBCRUN_DIRECT ./prog` is a runtime.
fn strip_env_prefix(cmd: &str) -> &str {
    let mut rest = cmd.trim_start();
    loop {
        let trimmed = rest.trim_start();
        // a leading token of the form NAME=... (no spaces inside NAME)
        let end = trimmed
            .find(|c: char| c.is_whitespace())
            .unwrap_or(trimmed.len());
        let token = &trimmed[..end];
        if token.contains('=') && !token.starts_with('$') {
            rest = trimmed[end..].trim_start();
        } else {
            return rest;
        }
    }
}

/// Classify the AT_CHECK command (after env-prefix stripping). Fail-closed: unknown -> `Unknown`.
pub fn classify_command(cmd: &str) -> CommandShape {
    let cmd = strip_env_prefix(cmd);
    let first = cmd.split_whitespace().next().unwrap_or("");
    // pipelines / redirects to postprocessors are never pure compiler steps
    if cmd.contains('|') {
        return CommandShape::ShellHelper;
    }
    match first {
        "$COMPILE" | "$COMPILE_ONLY" | "$COMPILE_MODULE" | "$COBC" => CommandShape::Compiler,
        "$COMPILE_LISTING" | "$COMPILE_LISTING0" => CommandShape::CompilerListing,
        "$COBCRUN" | "$COBCRUN_DIRECT" | "./prog" => CommandShape::Runtime,
        "$GREP" | "$SED" | "diff" | "test" | "mkdir" | "rm" | "cat" | "echo" | "cp" | "mv"
        | "unset" | "export" | "true" | "false" | "printf" | "touch" | "sort" | "tail" | "head"
        | "wc" | "cmp" | "grep" => CommandShape::ShellHelper,
        "$UNIFY_LISTING" => CommandShape::GeneratedFile,
        // absolute/relative program paths and anything else are unknown -> fail closed
        _ => CommandShape::Unknown,
    }
}

/// True when the command is a *compiler-producing step* (`$COMPILE*`, `$COBC`).
/// Listing steps ARE compiler steps for the purpose of stderr diagnostics, but their stdout is a
/// generated artifact and is never a diagnostic candidate.
fn is_compiler_shape(shape: CommandShape) -> bool {
    matches!(
        shape,
        CommandShape::Compiler | CommandShape::CompilerListing
    )
}

// ---------------------------------------------------------------------------------------------
// diagnostic-content proof (Phase 2)
// ---------------------------------------------------------------------------------------------

/// A line of expected compiler-diagnostic material. Conservative: a stream is diagnostic-only when
/// EVERY non-blank line matches one of these shapes (diagnostic messages, `file:line:` references,
/// caret lines, `cobc:`/`cobcrun:` errors) and the step is a compiler invocation that does not run
/// the compiled program.
fn line_is_diagnostic(line: &str) -> bool {
    let t = line.trim();
    if t.is_empty() {
        return true;
    }
    // GCC/upstream diagnostic shapes:
    //   prog.cob:9: error: ...
    //   prog.cob:20: warning: ...
    //   sec-1.inc:2: error: ...
    //   cobc: error: ...
    //   cobc: fatal error: ...
    //   in section 'MAIN': (context header emitted by the upstream parser)
    //   caret/context lines:      ^~ , '~~~'
    //   PICTURE/definition context lines are usually preceded by file:line headers.
    if t.starts_with('^') || t.starts_with('~') || t.starts_with("'") && t.ends_with('\'') {
        return true;
    }
    // file:line: (error|warning|fatal|note) — and any message that reads as a compiler
    // diagnostic (shared predicate so `file:2: configuration file was included here` matches)
    if let Some(colon) = t.find(':') {
        let head = &t[..colon];
        let is_fileish = head.ends_with(".cob")
            || head.ends_with(".cbl")
            || head.ends_with(".cpy")
            || head.ends_with(".inc")
            || head.ends_with(".at")
            || head.ends_with(".conf")
            || head.ends_with(".words")
            || head.ends_with(".lst")
            || head.ends_with(".lis")
            || head.chars().all(|c| c.is_ascii_digit())
            || head
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.');
        if is_fileish {
            let rest = &t[colon + 1..];
            let rest = rest.trim_start();
            if let Some(n) = rest.split(':').next() {
                let after = rest[n.len()..].trim_start();
                if n.chars().all(|c| c.is_ascii_digit()) && after.starts_with(':') {
                    let msg = after[1..].trim_start();
                    return msg_looks_diagnostic(msg);
                } else if after.starts_with(':') {
                    // file: in section '...' / in paragraph '...' continuation headers: the
                    // message is the FIRST segment (before the trailing colon), e.g.
                    // `prog.cob: in paragraph 'S-03':`
                    let msg = n.trim_start();
                    if msg.starts_with("in section") || msg.starts_with("in paragraph") {
                        return true;
                    }
                }
            }
        }
    }
    // bare `error:` / `warning:` / `fatal error:` at line start, `cobc:`/`cobcrun:` prefixes,
    // `configuration error:` (cobc config diagnostics), `option requires an argument`,
    // `unrecognized option`, `missing PROGRAM name` (cobcrun CLI)
    if t.starts_with("error:")
        || t.starts_with("warning:")
        || t.starts_with("fatal error:")
        || t.starts_with("cobc: ")
        || t.starts_with("cobcrun: ")
        || t.starts_with("configuration error:")
        || t.starts_with("Error:")
        || t.starts_with("Warning:")
        || t.starts_with("invalid module argument")
        || t.starts_with("option requires an argument")
        || t.starts_with("unrecognized option")
        || t.starts_with("missing PROGRAM name")
    {
        return true;
    }
    // multi-word diagnostic-ish sentences used by the upstream suite
    msg_looks_diagnostic(t)
}

/// Message-level diagnostic predicate (shared by the `file:line:` branch and the bare-line tail).
fn msg_looks_diagnostic(t: &str) -> bool {
    let lower = t.to_ascii_lowercase();
    lower.starts_with("error: ")
        || lower.contains("error: ")
        || lower.contains("warning: ")
        || lower.contains("syntax error")
        || lower.contains("is not defined")
        || lower.contains("undefined symbol")
        || lower.contains("unexpected ")
        || lower.contains("expected ")
        || lower.contains("invalid ")
        || lower.contains("cannot ")
        || lower.contains("does not conform")
        || lower.contains("must be ")
        || lower.contains("not allowed")
        || lower.contains("unknown option")
        || lower.contains("ambiguous")
        || lower.contains("requires an argument")
        || lower.contains("recursive inclusion")
        || lower.contains("configuration file was included here")
        || lower.contains("included here")
        || lower.contains("configuration error")
        || lower.contains("should be one of the following values")
        || lower.contains("no such file or directory")
        || lower.contains("invalid configuration tag")
        || lower.contains("could not access word list")
        || lower.contains("could not access")
        || lower.starts_with("error")
        || lower.starts_with("warning")
        || lower.starts_with("fatal")
        || lower.starts_with("note")
        || lower.starts_with("in section")
        || lower.starts_with("in paragraph")
}

/// Prove a stream is *purely* compiler-diagnostic output (Phase 2). Requires BOTH:
///   - the command is a compiler step (never runtime/helper/unknown);
///   - every non-blank expected line is diagnostic-shaped;
///   - the expected text is non-empty and not already `ignore`;
///   - the step does not execute the compiled program (compiler shapes do not run);
///   - the stream is not a generated listing artifact (compiler-listing stdout);
///   - no `AT_CAPTURE_FILE` in the group depends on these bytes (the group's captured files are
///     never derived from an AT_CHECK expectation stream, but we still bind them as evidence).
fn prove_diagnostic_only(
    shape: CommandShape,
    stdout: &str,
    stderr: &str,
    stdout_is_listing: bool,
    stderr_is_listing: bool,
) -> DiagnosticDisposition {
    if !is_compiler_shape(shape) {
        return DiagnosticDisposition::Preserve;
    }
    let stdout_candidate = !stdout.is_empty()
        && stdout != "ignore"
        && !stdout_is_listing
        && stdout.lines().all(|l| line_is_diagnostic(l));
    let stderr_candidate = !stderr.is_empty()
        && stderr != "ignore"
        && !stderr_is_listing
        && stderr.lines().all(|l| line_is_diagnostic(l));
    match (stdout_candidate, stderr_candidate) {
        (true, true) => DiagnosticDisposition::IgnoreCompilerBoth,
        (true, false) => DiagnosticDisposition::IgnoreCompilerStdout,
        (false, true) => DiagnosticDisposition::IgnoreCompilerStderr,
        (false, false) => DiagnosticDisposition::Preserve,
    }
}

// ---------------------------------------------------------------------------------------------
// span-level extraction: walk the spanned macro stream, group by AT_SETUP..AT_CLEANUP
// ---------------------------------------------------------------------------------------------

/// A group parsed from the SPANNED stream (span-aware, for patch generation).
#[derive(Debug, Clone)]
struct SpannedGroup {
    title: String,
    line: usize,
    keywords: Vec<String>,
    skip: Vec<String>,
    xfail: Vec<String>,
    capture_files: Vec<String>,
    /// AT_CHECK macros in order, with their span records.
    checks: Vec<SpannedCheck>,
}

#[derive(Debug, Clone)]
struct SpannedCheck {
    line: usize,
    /// 0-based index of the AT_CHECK within its group.
    step_index: usize,
    macro_: SpannedMacro,
    /// Argument indices (0-based) of command/status/stdout/stderr within `macro_.args`.
    /// Autotest defaults: status=0, stdout/stderr=ignore when absent.
    command_idx: usize,
    status_idx: Option<usize>,
    stdout_idx: Option<usize>,
    stderr_idx: Option<usize>,
}

fn arg_str<'a>(args: &'a [SpannedArg], idx: Option<usize>, dflt: &'a str) -> &'a str {
    match idx {
        Some(i) => args.get(i).map(|a| a.text.as_str()).unwrap_or(dflt),
        None => dflt,
    }
}

/// Walk the spanned macro stream of one `.at` file into groups, tracking byte spans of every
/// AT_CHECK argument so a patch can replace only the stdout/stderr expectation spans.
fn spanned_groups(macros: &[SpannedMacro], source: &str) -> (Vec<SpannedGroup>, Vec<String>) {
    let mut groups: Vec<SpannedGroup> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut cur: Option<SpannedGroup> = None;
    for m in macros {
        match m.name.as_str() {
            "AT_SETUP" => {
                if let Some(g) = cur.take() {
                    groups.push(g);
                }
                cur = Some(SpannedGroup {
                    title: m.args.first().map(|a| a.text.clone()).unwrap_or_default(),
                    line: m.line,
                    keywords: Vec::new(),
                    skip: Vec::new(),
                    xfail: Vec::new(),
                    capture_files: Vec::new(),
                    checks: Vec::new(),
                });
            }
            "AT_CLEANUP" => {
                if let Some(g) = cur.take() {
                    groups.push(g);
                }
            }
            "AT_KEYWORDS" => {
                if let Some(g) = cur.as_mut() {
                    if let Some(a) = m.args.first() {
                        g.keywords = a.text.split_whitespace().map(|s| s.to_string()).collect();
                    }
                }
            }
            "AT_SKIP_IF" => {
                if let Some(g) = cur.as_mut() {
                    if let Some(a) = m.args.first() {
                        g.skip.push(a.text.clone());
                    }
                }
            }
            "AT_XFAIL_IF" => {
                if let Some(g) = cur.as_mut() {
                    if let Some(a) = m.args.first() {
                        g.xfail.push(a.text.clone());
                    }
                }
            }
            "AT_CAPTURE_FILE" => {
                if let Some(g) = cur.as_mut() {
                    if let Some(a) = m.args.first() {
                        g.capture_files.push(a.text.clone());
                    }
                }
            }
            "AT_CHECK" | "AT_CHECK_UNQUOTED" => {
                if let Some(g) = cur.as_mut() {
                    let n = g.checks.len();
                    let command_idx = 0;
                    let status_idx = if m.args.len() > 1 { Some(1) } else { None };
                    let stdout_idx = if m.args.len() > 2 { Some(2) } else { None };
                    let stderr_idx = if m.args.len() > 3 { Some(3) } else { None };
                    g.checks.push(SpannedCheck {
                        line: m.line,
                        step_index: n,
                        macro_: m.clone(),
                        command_idx,
                        status_idx,
                        stdout_idx,
                        stderr_idx,
                    });
                    let _ = source;
                }
            }
            _ => {}
        }
    }
    if let Some(g) = cur.take() {
        errors.push(format!(
            "group {:?} not terminated by AT_CLEANUP (line {})",
            g.title, g.line
        ));
        groups.push(g);
    }
    (groups, errors)
}

// ---------------------------------------------------------------------------------------------
// patch generation (Phase 3)
// ---------------------------------------------------------------------------------------------

/// One byte replacement: replace `src[span.0..span.1]` with `replacement` (e.g. `[ignore]`).
#[derive(Debug, Clone)]
pub struct SpanEdit {
    pub span: (usize, usize),
    pub replacement: String,
}

/// Apply a list of edits to a source string. Edits must be disjoint and sorted by start byte.
pub fn apply_edits(src: &str, mut edits: Vec<SpanEdit>) -> String {
    edits.sort_by_key(|e| e.span.0);
    let mut out = String::new();
    let mut pos = 0usize;
    for e in edits {
        assert!(e.span.0 >= pos && e.span.1 >= e.span.0, "overlapping edits");
        out.push_str(&src[pos..e.span.0]);
        out.push_str(&e.replacement);
        pos = e.span.1;
    }
    out.push_str(&src[pos..]);
    out
}

/// The decision for one AT_CHECK: what to ignore, and why.
#[derive(Debug, Clone)]
struct CheckDecision {
    disposition: DiagnosticDisposition,
    reason: String,
    /// The literal replacement text for each stream ("" = unchanged).
    stdout_replacement: Option<String>,
    stderr_replacement: Option<String>,
}

fn decide_check(
    check: &SpannedCheck,
    source_revision: &str,
    source_file: &str,
    group: &SpannedGroup,
) -> Transformation {
    let args = &check.macro_.args;
    let command = arg_str(args, Some(check.command_idx), "");
    let status = arg_str(args, check.status_idx, "0");
    let stdout = arg_str(args, check.stdout_idx, "ignore").to_string();
    let stderr = arg_str(args, check.stderr_idx, "ignore").to_string();

    let shape = classify_command(command);
    let listing_stdout = shape == CommandShape::CompilerListing;
    let shape_for_proof = shape;
    let disposition = if !is_compiler_shape(shape_for_proof) {
        DiagnosticDisposition::Preserve
    } else if stdout == "ignore" && stderr == "ignore" {
        DiagnosticDisposition::Preserve
    } else {
        // Compiler steps never execute the compiled program; listings' stderr is still a
        // compiler stream; stdout of a listing step is a generated artifact (never ignored).
        let listing_stderr = false;
        prove_diagnostic_only(
            shape_for_proof,
            &stdout,
            &stderr,
            listing_stdout,
            listing_stderr,
        )
    };

    // reason
    let reason = match disposition {
        DiagnosticDisposition::Preserve => {
            if !is_compiler_shape(shape_for_proof) {
                format!("not a compiler step (shape {:?})", shape_for_proof)
            } else if stdout == "ignore" && stderr == "ignore" {
                "both expectations already ignore".to_string()
            } else if listing_stdout {
                "stdout is a generated listing artifact (preserved); stderr not diagnostic-only"
                    .to_string()
            } else {
                "expected stream(s) not proven purely compiler diagnostic".to_string()
            }
        }
        DiagnosticDisposition::IgnoreCompilerStdout => {
            "compiler step; stdout expectation is purely compiler diagnostic text".to_string()
        }
        DiagnosticDisposition::IgnoreCompilerStderr => {
            "compiler step; stderr expectation is purely compiler diagnostic text".to_string()
        }
        DiagnosticDisposition::IgnoreCompilerBoth => {
            "compiler step; both expectations are purely compiler diagnostic text".to_string()
        }
        DiagnosticDisposition::Uncertain => "uncertain: not transformed".to_string(),
    };

    let already_ignored = stdout == "ignore" && stderr == "ignore";

    // transformed expectations: only the proven streams become `ignore`; everything else stays.
    let stdout_rep = if disposition.ignores_stdout() && stdout != "ignore" && !stdout.is_empty() {
        Some("[ignore]".to_string())
    } else {
        None
    };
    let stderr_rep = if disposition.ignores_stderr() && stderr != "ignore" && !stderr.is_empty() {
        Some("[ignore]".to_string())
    } else {
        None
    };
    let transformed_stdout = stdout_rep
        .as_deref()
        .map(|_| "ignore")
        .unwrap_or(stdout.as_str());
    let transformed_stderr = stderr_rep
        .as_deref()
        .map(|_| "ignore")
        .unwrap_or(stderr.as_str());

    Transformation {
        schema: "gnurust-diag-unblocked-transformation-v1".to_string(),
        transformer_version: TRANSFORMER_VERSION.to_string(),
        source_file: source_file.to_string(),
        group_title: group.title.clone(),
        group_index: 0, // filled by the caller
        step_index: check.step_index,
        source_line: check.line,
        command: command.to_string(),
        command_shape: shape,
        expected_status: status.to_string(),
        stdout_sha256: if stdout.is_empty() {
            String::new()
        } else {
            Transformation::sha(&stdout)
        },
        stderr_sha256: if stderr.is_empty() {
            String::new()
        } else {
            Transformation::sha(&stderr)
        },
        disposition,
        reason,
        already_ignored,
        transformed_stdout_sha256: if transformed_stdout.is_empty() {
            String::new()
        } else {
            Transformation::sha(transformed_stdout)
        },
        transformed_stderr_sha256: if transformed_stderr.is_empty() {
            String::new()
        } else {
            Transformation::sha(transformed_stderr)
        },
        source_revision: source_revision.to_string(),
        group_skip: group.skip.clone(),
        group_xfail: group.xfail.clone(),
        group_capture_files: group.capture_files.clone(),
    }
}

/// Build the per-file transformations (records carry the disposition; edits are derived from the
/// records by [`edits_for_file`] so records and patch can never diverge).
fn transform_file(
    source: &str,
    source_file: &str,
    source_revision: &str,
) -> (Vec<Transformation>, Vec<String>) {
    let macros = match scan_spanned(source) {
        Ok(m) => m,
        Err(e) => return (Vec::new(), vec![format!("{source_file}: scan failed: {e}")]),
    };
    let (groups, group_errors) = spanned_groups(&macros, source);
    let mut trans: Vec<Transformation> = Vec::new();
    for (gi, g) in groups.iter().enumerate() {
        for check in &g.checks {
            let mut t = decide_check(check, source_revision, source_file, g);
            t.group_index = gi;
            trans.push(t);
        }
    }
    (trans, group_errors)
}

/// Derive the byte edits for one file from its transformation records (single source of truth).
fn edits_for_file(source: &str, trans: &[Transformation]) -> Result<Vec<SpanEdit>, String> {
    let macros = scan_spanned(source)?;
    let (groups, _) = spanned_groups(&macros, source);
    let mut edits = Vec::new();
    for (gi, g) in groups.iter().enumerate() {
        for check in &g.checks {
            let rec = trans
                .iter()
                .find(|t| t.group_index == gi && t.step_index == check.step_index);
            let Some(rec) = rec else { continue };
            let args = &check.macro_.args;
            if rec.disposition.ignores_stdout() {
                if let Some(a) = check.stdout_idx.and_then(|i| args.get(i)) {
                    edits.push(SpanEdit {
                        span: a.span,
                        replacement: "[ignore]".to_string(),
                    });
                }
            }
            if rec.disposition.ignores_stderr() {
                if let Some(a) = check.stderr_idx.and_then(|i| args.get(i)) {
                    edits.push(SpanEdit {
                        span: a.span,
                        replacement: "[ignore]".to_string(),
                    });
                }
            }
        }
    }
    Ok(edits)
}

/// Hash of a whole file (for pristine/transformed manifests).
pub fn file_sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// The complete transformation result for one suite source root.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransformResult {
    pub schema: String,
    pub transformer_version: String,
    pub source_revision: String,
    pub generated_at_utc: String,
    /// Per-file: pristine manifest hash (all `.at` files under the source root).
    pub pristine_manifest_sha256: String,
    /// Per-file: transformed manifest hash (only files that changed are re-hashed).
    pub transformed_manifest_sha256: String,
    pub files_scanned: usize,
    pub transformations: Vec<Transformation>,
    pub errors: Vec<String>,
}

/// Which files in the suite source root are `.at` sources that feed the generated testsuite.
fn suite_at_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(root) {
        for entry in rd.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("at") {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

/// Deterministic stringify of a value (sorted keys) for hashing.
fn stable_json(v: &serde_json::Value) -> String {
    serde_json::to_string(v).unwrap_or_default()
}

/// Run the transformer over a suite source root (`tests/testsuite.src/`): classify every AT_CHECK
/// and produce (a) the machine-readable transformation list, (b) the edits that turn pristine into
/// transformed, (c) pristine + transformed file bytes.
pub fn transform_suite(root: &Path, source_revision: &str) -> Result<TransformResult, String> {
    let files = suite_at_files(root);
    let mut all: Vec<Transformation> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut pristine_bytes: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let mut transformed_bytes: BTreeMap<String, Vec<u8>> = BTreeMap::new();

    for f in &files {
        let bytes = std::fs::read(f).map_err(|e| format!("{}: {e}", f.display()))?;
        let text = String::from_utf8_lossy(&bytes).into_owned();
        let name = f
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let (trans, errs) = transform_file(&text, &name, source_revision);
        errors.extend(errs);
        // edits are DERIVED from the records (single source of truth: records == patch)
        let edits = edits_for_file(&text, &trans).map_err(|e| format!("{name}: {e}"))?;
        let new_text = apply_edits(&text, edits);
        pristine_bytes.insert(name.clone(), bytes.clone());
        transformed_bytes.insert(name.clone(), new_text.into_bytes());
        all.extend(trans);
    }

    all.sort_by(|a, b| {
        (&a.source_file, a.group_index, a.step_index).cmp(&(
            &b.source_file,
            b.group_index,
            b.step_index,
        ))
    });

    let pristine_manifest = stable_json(&serde_json::json!({
        "files": pristine_bytes.iter().map(|(k, v)| (k.clone(), file_sha256(v))).collect::<BTreeMap<_,_>>(),
    }));
    let transformed_manifest = stable_json(&serde_json::json!({
        "files": transformed_bytes.iter().map(|(k, v)| (k.clone(), file_sha256(v))).collect::<BTreeMap<_,_>>(),
    }));

    Ok(TransformResult {
        schema: "gnurust-diag-unblocked-transform-result-v1".to_string(),
        transformer_version: TRANSFORMER_VERSION.to_string(),
        source_revision: source_revision.to_string(),
        generated_at_utc: crate::cli::now_utc_string_pub(),
        pristine_manifest_sha256: Transformation::sha(&pristine_manifest),
        transformed_manifest_sha256: Transformation::sha(&transformed_manifest),
        files_scanned: files.len(),
        transformations: all,
        errors,
    })
}

// ---------------------------------------------------------------------------------------------
// Phase 3 — deterministic unified-diff patch generation
// ---------------------------------------------------------------------------------------------

/// A deterministic line-based unified diff between two texts (context 3, like `diff -u`).
/// Deterministic: same inputs always produce byte-identical output.
pub fn unified_diff(orig: &str, new: &str, path: &str) -> String {
    let a: Vec<&str> = orig.split('\n').collect();
    let b: Vec<&str> = new.split('\n').collect();
    // LCS-based diff (small files; deterministic)
    let (n, m) = (a.len(), b.len());
    let mut dp = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if a[i] == b[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }
    // walk the LCS to collect ops
    #[derive(Clone, Copy, PartialEq)]
    enum Op {
        Eq,
        Del,
        Ins,
    }
    let mut ops = Vec::with_capacity(n + m);
    let (mut i, mut j) = (0usize, 0usize);
    while i < n && j < m {
        if a[i] == b[j] {
            ops.push(Op::Eq);
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            ops.push(Op::Del);
            i += 1;
        } else {
            ops.push(Op::Ins);
            j += 1;
        }
    }
    while i < n {
        ops.push(Op::Del);
        i += 1;
    }
    while j < m {
        ops.push(Op::Ins);
        j += 1;
    }

    // group into hunks with 3 lines of context
    const CTX: usize = 3;
    let mut hunks: Vec<(usize, usize, Vec<(Op, usize, usize)>)> = Vec::new(); // (a_start, b_start, ops w/ indices)
    let (mut ai, mut bi) = (0usize, 0usize);
    let mut cur: Option<(usize, usize, Vec<(Op, usize, usize)>)> = None;
    for op in &ops {
        let (a_idx, b_idx) = (ai, bi);
        match op {
            Op::Eq => {
                if let Some((_, _, inner)) = cur.as_mut() {
                    inner.push((Op::Eq, a_idx, b_idx));
                }
                ai += 1;
                bi += 1;
            }
            Op::Del => {
                let (_, _, inner) = cur.get_or_insert_with(|| (a_idx, b_idx, Vec::new()));
                inner.push((Op::Del, a_idx, b_idx));
                ai += 1;
            }
            Op::Ins => {
                let (_, _, inner) = cur.get_or_insert_with(|| (a_idx, b_idx, Vec::new()));
                inner.push((Op::Ins, a_idx, b_idx));
                bi += 1;
            }
        }
    }
    if let Some((as_, bs, inner)) = cur.take() {
        hunks.push((as_, bs, inner));
    }
    // split any run whose span exceeds 2*CTX into separate hunks (like diff)
    let mut out = String::new();
    if !hunks.is_empty() {
        out.push_str(&format!("--- a/{path}\n+++ b/{path}\n"));
    }
    for (as_, bs, inner) in hunks {
        // trim context so hunks don't overlap
        let mut lo = 0usize;
        let mut hi = inner.len();
        while lo < hi && inner[lo].0 == Op::Eq {
            lo += 1;
        }
        while hi > lo && inner[hi - 1].0 == Op::Eq {
            hi -= 1;
        }
        if lo > CTX {
            lo -= CTX;
        } else {
            lo = 0;
        }
        if hi < inner.len() && inner.len() - hi > CTX {
            hi += CTX;
        } else {
            hi = inner.len();
        }
        let start_a = if lo == 0 { as_ } else { inner[lo].1 };
        let start_b = if lo == 0 { bs } else { inner[lo].2 };
        let mut cnt_a = 0usize;
        let mut cnt_b = 0usize;
        for (op, _, _) in &inner[lo..hi] {
            match op {
                Op::Eq => {
                    cnt_a += 1;
                    cnt_b += 1;
                }
                Op::Del => cnt_a += 1,
                Op::Ins => cnt_b += 1,
            }
        }
        out.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            start_a + 1,
            cnt_a,
            start_b + 1,
            cnt_b
        ));
        for (op, a_idx, b_idx) in &inner[lo..hi] {
            match op {
                Op::Eq => out.push_str(&format!(" {}\n", a[*a_idx])),
                Op::Del => out.push_str(&format!("-{}\n", a[*a_idx])),
                Op::Ins => out.push_str(&format!("+{}\n", b[*b_idx])),
            }
        }
    }
    out
}

/// Phase 3 — write the deterministic patch + transformations.json/csv/md + pristine/transformed
/// trees into the report root. Everything is reproducible from the immutable upstream source +
/// the transformer version + the classification rules.
pub fn cmd_transform(
    suite_src_root: &Path,
    report_root: &Path,
    source_revision: &str,
) -> Result<TransformResult, String> {
    let res = transform_suite(suite_src_root, source_revision)?;
    std::fs::create_dir_all(report_root).map_err(|e| e.to_string())?;

    // write the transformed tree + pristine tree (for review + the gate)
    let pristine_dir = report_root.join("pristine");
    let transformed_dir = report_root.join("transformed");
    std::fs::create_dir_all(&pristine_dir).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&transformed_dir).map_err(|e| e.to_string())?;
    for f in suite_at_files(suite_src_root) {
        let bytes = std::fs::read(&f).map_err(|e| e.to_string())?;
        let name = f.file_name().and_then(|n| n.to_str()).unwrap_or("");
        std::fs::write(pristine_dir.join(name), &bytes).map_err(|e| e.to_string())?;
    }
    // regenerate the transformed bytes (same code path as transform_suite)
    for f in suite_at_files(suite_src_root) {
        let bytes = std::fs::read(&f).map_err(|e| e.to_string())?;
        let text = String::from_utf8_lossy(&bytes).into_owned();
        let name = f.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let (trans, _) = transform_file(&text, name, source_revision);
        let edits = edits_for_file(&text, &trans).map_err(|e| e.to_string())?;
        let new_text = apply_edits(&text, edits);
        std::fs::write(transformed_dir.join(name), new_text.as_bytes())
            .map_err(|e| e.to_string())?;
    }

    // patch = concatenated deterministic diffs (file order sorted)
    let mut patch = String::new();
    patch.push_str(&format!(
        "# diagnostic-ignore.patch — mechanically generated by {TRANSFORMER_VERSION}\n"
    ));
    patch.push_str(&format!(
        "# source revision: {source_revision} — only proven compiler-diagnostic expected streams\n"
    ));
    patch
        .push_str("# become Autotest `ignore`; commands, exit statuses, source, runtime output,\n");
    patch.push_str(
        "# generated-file expectations, environment, ordering and skip/xfail are unchanged.\n",
    );
    for f in suite_at_files(suite_src_root) {
        let name = f.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let pristine = std::fs::read_to_string(pristine_dir.join(name)).unwrap_or_default();
        let transformed = std::fs::read_to_string(transformed_dir.join(name)).unwrap_or_default();
        if pristine != transformed {
            patch.push_str(&unified_diff(&pristine, &transformed, name));
        }
    }
    std::fs::write(report_root.join("diagnostic-ignore.patch"), patch)
        .map_err(|e| e.to_string())?;

    // tree-manifest.json: per-file pristine + transformed hashes, so the trees are reproducible
    // (and verifiable) without committing the (regenerable) full copies.
    let mut pristine_hashes: BTreeMap<String, String> = BTreeMap::new();
    let mut transformed_hashes: BTreeMap<String, String> = BTreeMap::new();
    for f in suite_at_files(suite_src_root) {
        let name = f
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        pristine_hashes.insert(
            name.clone(),
            file_sha256(&std::fs::read(pristine_dir.join(&name)).map_err(|e| e.to_string())?),
        );
        transformed_hashes.insert(
            name.clone(),
            file_sha256(&std::fs::read(transformed_dir.join(&name)).map_err(|e| e.to_string())?),
        );
    }
    let tree_manifest = serde_json::json!({
        "schema": "gnurust-diag-unblocked-tree-manifest-v1",
        "source_revision": source_revision,
        "pristine": pristine_hashes,
        "transformed": transformed_hashes,
    });
    std::fs::write(
        report_root.join("tree-manifest.json"),
        serde_json::to_string_pretty(&tree_manifest).map_err(|e| e.to_string())? + "\n",
    )
    .map_err(|e| e.to_string())?;

    // transformations.json (machine-readable; the gate's manifest)
    let manifest = serde_json::json!({
        "schema": "gnurust-diag-unblocked-transformations-v1",
        "transformer_version": TRANSFORMER_VERSION,
        "source_revision": source_revision,
        "pristine_manifest_sha256": res.pristine_manifest_sha256,
        "transformed_manifest_sha256": res.transformed_manifest_sha256,
        "files_scanned": res.files_scanned,
        "generated_at_utc": res.generated_at_utc,
        "transformations": res.transformations,
    });
    std::fs::write(
        report_root.join("transformations.json"),
        serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())? + "\n",
    )
    .map_err(|e| e.to_string())?;

    // transformations.csv
    let mut csv = String::from(
        "source_file,group_index,step_index,source_line,command,expected_status,shape,stdout_sha256,stderr_sha256,disposition,reason\n",
    );
    for t in &res.transformations {
        csv.push_str(&format!(
            "{},{},{},{},{:?},{},{:?},{},{},{:?},{}\n",
            csv_escape(&t.source_file),
            t.group_index,
            t.step_index,
            t.source_line,
            t.command,
            t.expected_status,
            t.command_shape,
            t.stdout_sha256,
            t.stderr_sha256,
            t.disposition,
            csv_escape(&t.reason),
        ));
    }
    std::fs::write(report_root.join("transformations.csv"), csv).map_err(|e| e.to_string())?;

    // transformations.md (human review)
    let mut md = String::new();
    md.push_str(&format!("# Diagnostic-unblocked transformations\n\n"));
    md.push_str(&format!(
        "_transformer {TRANSFORMER_VERSION} · source revision `{source_revision}` · "
    ));
    md.push_str(&format!(
        "pristine manifest `{}` · transformed manifest `{}`_\n\n",
        res.pristine_manifest_sha256, res.transformed_manifest_sha256
    ));
    md.push_str(
        "Only expected compiler-diagnostic streams become `ignore`; commands, exit statuses,\n",
    );
    md.push_str(
        "COBOL source, runtime output, generated-file expectations, environment, ordering\n",
    );
    md.push_str(
        "and skip/xfail semantics are unchanged. Nothing else in the suite is modified.\n\n",
    );
    md.push_str("## transformations that ignore a stream\n\n");
    md.push_str("| source | group | step | line | command | status | stream | reason |\n");
    md.push_str("|---|---|---|---|---|---|---|---|\n");
    for t in &res.transformations {
        if !(t.disposition.ignores_stdout() || t.disposition.ignores_stderr()) {
            continue;
        }
        let stream = match t.disposition {
            DiagnosticDisposition::IgnoreCompilerStdout => "stdout",
            DiagnosticDisposition::IgnoreCompilerStderr => "stderr",
            DiagnosticDisposition::IgnoreCompilerBoth => "stdout+stderr",
            _ => "-",
        };
        md.push_str(&format!(
            "| {} | {} | {} | {} | `{}` | {} | {} | {} |\n",
            t.source_file,
            t.group_index,
            t.step_index,
            t.source_line,
            t.command.replace('|', "\\|"),
            t.expected_status,
            stream,
            t.reason
        ));
    }
    std::fs::write(report_root.join("transformations.md"), md).map_err(|e| e.to_string())?;

    Ok(res)
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

// ---------------------------------------------------------------------------------------------
// Phase 4 — mechanical patch policy gate (independent of the transformer)
// ---------------------------------------------------------------------------------------------

/// The gate verdict: independent verification of the actual diff.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GateVerdict {
    pub schema: String,
    pub patch_sha256: String,
    pub pristine_manifest_sha256: String,
    pub transformed_manifest_sha256: String,
    pub at_setup_pristine: usize,
    pub at_setup_transformed: usize,
    pub at_check_pristine: usize,
    pub at_check_transformed: usize,
    pub changed_files: Vec<String>,
    pub admitted_changes: usize,
    pub failures: Vec<String>,
}

/// Independent unified-diff parse: `@@ -a,c +b,d @@`, `-`/`+`/` ` lines. Returns per-hunk ops.
pub fn parse_unified_diff(patch: &str) -> Vec<(String, Vec<(usize, usize, usize)>)> {
    // (file, [(op: 0 eq 1 del 2 ins, a_line_index, b_line_index)])
    let mut out: Vec<(String, Vec<(usize, usize, usize)>)> = Vec::new();
    let mut cur_file: Option<String> = None;
    let mut cur: Vec<(usize, usize, usize)> = Vec::new();
    let (mut ai, mut bi) = (0usize, 0usize);
    for line in patch.lines() {
        if let Some(f) = line.strip_prefix("+++ b/") {
            if let Some(f0) = cur_file.take() {
                out.push((f0, std::mem::take(&mut cur)));
            }
            cur_file = Some(f.to_string());
            ai = 0;
            bi = 0;
            continue;
        }
        if let Some(_h) = line.strip_prefix("@@ -") {
            // hunk header: ai/bi continue from the previous hunk state (we track running indices)
            continue;
        }
        let cur = &mut cur;
        if let Some(_file) = cur_file.clone() {
            if let Some(rest) = line.strip_prefix('-') {
                if !rest.starts_with("-") {
                    cur.push((1, ai, bi));
                    ai += 1;
                    continue;
                }
            }
            if let Some(rest) = line.strip_prefix('+') {
                if !rest.starts_with("+") {
                    cur.push((2, ai, bi));
                    bi += 1;
                    continue;
                }
            }
            if line.starts_with(' ') {
                cur.push((0, ai, bi));
                ai += 1;
                bi += 1;
            }
        }
    }
    if let Some(f0) = cur_file.take() {
        out.push((f0, cur));
    }
    out
}

/// Phase 4 — independently verify the patch: parse the actual diff, re-parse pristine and
/// transformed trees, and prove every change is legal (15 rules). The gate does NOT trust the
/// transformer: it verifies the bytes.
pub fn cmd_gate(
    patch_path: &Path,
    pristine_root: &Path,
    transformed_root: &Path,
    manifest_path: &Path,
) -> Result<GateVerdict, String> {
    let mut failures: Vec<String> = Vec::new();
    let patch_bytes = std::fs::read(patch_path).map_err(|e| format!("patch: {e}"))?;
    let patch_text = String::from_utf8_lossy(&patch_bytes).into_owned();
    let patch_sha = file_sha256(&patch_bytes);

    let manifest: serde_json::Value = read_json_file(manifest_path)
        .ok_or_else(|| format!("manifest unreadable: {}", manifest_path.display()))?;
    let manifest_trans: Vec<serde_json::Value> = manifest["transformations"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    // rule 20: the set of expectations the manifest ADMITS for ignore
    let admitted: std::collections::BTreeSet<String> = manifest_trans
        .iter()
        .filter(|t| {
            let d = t["disposition"].as_str().unwrap_or("");
            d.contains("IGNORE")
        })
        .map(|t| {
            format!(
                "{}#{}#{}",
                t["source_file"].as_str().unwrap_or(""),
                t["group_index"].as_u64().unwrap_or(0),
                t["step_index"].as_u64().unwrap_or(0)
            )
        })
        .collect();

    // rule 1+2: only approved `.at` files may change
    let pristine_files = suite_at_files(pristine_root);
    let transformed_files = suite_at_files(transformed_root);
    let mut changed: Vec<String> = Vec::new();
    for f in &pristine_files {
        let name = f
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let pb = std::fs::read(f).map_err(|e| e.to_string())?;
        let tb = std::fs::read(transformed_root.join(&name)).map_err(|e| e.to_string())?;
        if pb != tb {
            changed.push(name.clone());
        }
    }
    for f in &transformed_files {
        let name = f.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !name.ends_with(".at") {
            failures.push(format!("rule1: non-.at file in transformed tree: {name}"));
        }
    }
    if changed.is_empty() {
        failures.push("rule3: no AT_CHECK hunks changed".to_string());
    }

    // rule 15 counts + structural compare per file (rules 4-14, 20)
    let mut setup_p = 0usize;
    let mut setup_t = 0usize;
    let mut check_p = 0usize;
    let mut check_t = 0usize;
    let mut admitted_hits = 0usize;

    for f in &pristine_files {
        let name = f
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let p_text = std::fs::read_to_string(f).unwrap_or_default();
        let t_text = std::fs::read_to_string(transformed_root.join(&name)).unwrap_or_default();
        let pm = scan_spanned(&p_text).map_err(|e| format!("{name} pristine: {e}"))?;
        let tm = scan_spanned(&t_text).map_err(|e| format!("{name} transformed: {e}"))?;
        // group index bookkeeping: AT_SETUP ordinal per file (manifest uses 0-based group
        // ordinals, so the key subtracts one from the running count)
        let mut group_index: usize = 0;
        if pm.len() != tm.len() {
            failures.push(format!(
                "rule14: {name} macro count changed ({} -> {})",
                pm.len(),
                tm.len()
            ));
            continue;
        }
        let mut check_step = 0usize;
        for (pi, ti) in pm.iter().zip(tm.iter()) {
            if pi.name != ti.name {
                failures.push(format!(
                    "rule13/14: {name} macro {} became {}",
                    pi.name, ti.name
                ));
                continue;
            }
            if pi.name == "AT_SETUP" {
                group_index += 1;
                check_step = 0; // step_index is per-group
            }
            if pi.name == "AT_CHECK" || pi.name == "AT_CHECK_UNQUOTED" {
                check_p += 1;
                check_t += 1;
                let step = check_step;
                check_step += 1;
                let group_0based = group_index.saturating_sub(1);
                // rules 4-10, 20: command + status byte-identical; only stdout/stderr
                // expectation args may change, and only exact-diagnostic -> ignore
                if pi.args.len() != ti.args.len() {
                    failures.push(format!(
                        "rule6: {name}:{} AT_CHECK arg count changed",
                        pi.line
                    ));
                    continue;
                }
                for (ai, (pa, ta)) in pi.args.iter().zip(ti.args.iter()).enumerate() {
                    match ai {
                        0 => {
                            if pa.text != ta.text {
                                failures.push(format!("rule4: {name}:{} command changed", pi.line));
                            }
                        }
                        1 => {
                            if pa.text != ta.text {
                                failures.push(format!(
                                    "rule5: {name}:{} expected status changed",
                                    pi.line
                                ));
                            }
                        }
                        2 | 3 => {
                            if pa.text != ta.text {
                                // rule 7: only exact diagnostic expectation -> ignore
                                let legal = pa.text != "ignore"
                                    && !pa.text.is_empty()
                                    && ta.text == "ignore";
                                if !legal {
                                    failures.push(format!(
                                        "rule7: {name}:{} arg{} changed non-legally: {:?} -> {:?}",
                                        pi.line, ai, pa.text, ta.text
                                    ));
                                } else {
                                    // rule: the changed step must be a COMPILER step (the gate
                                    // classifies the command independently of the manifest)
                                    let cmd_shape = classify_command(&pi.args[0].text);
                                    if !is_compiler_shape(cmd_shape) {
                                        failures.push(format!(
                                            "rule9: {name}:{} non-compiler step {cmd_shape:?} had an expectation ignored",
                                            pi.line
                                        ));
                                    }
                                    // rule: listing stdout is a generated artifact, never ignored
                                    if ai == 2 && cmd_shape == CommandShape::CompilerListing {
                                        failures.push(format!(
                                            "rule10: {name}:{} listing stdout expectation ignored (generated artifact)",
                                            pi.line
                                        ));
                                    }
                                    // rule 20: this exact change must be admitted
                                    let key = format!("{name}#{group_0based}#{step}");
                                    if admitted.contains(&key) {
                                        admitted_hits += 1;
                                    } else {
                                        failures.push(format!(
                                            "rule20: {name}:{} change not admitted in transformations.json (key {key})",
                                            pi.line
                                        ));
                                    }
                                }
                            } else if pa.text == "ignore" {
                                // rule 8: existing ignore stays ignore (already enforced by ==)
                            } else if pa.text.is_empty() {
                                // empty expectation stays empty (never weakened)
                            }
                        }
                        _ => {
                            if pa.text != ta.text {
                                failures.push(format!(
                                    "rule11: {name}:{} extra arg {} changed",
                                    pi.line, ai
                                ));
                            }
                        }
                    }
                }
            } else {
                // rules 12/13/14: every non-AT_CHECK macro must be byte-identical
                if pi.args.len() != ti.args.len() {
                    failures.push(format!(
                        "rule12: {name}:{} {} arg count changed",
                        pi.line, pi.name
                    ));
                    continue;
                }
                for (pa, ta) in pi.args.iter().zip(ti.args.iter()) {
                    if pa.text != ta.text {
                        failures.push(format!(
                            "rule12: {name}:{} {} argument changed",
                            pi.line, pi.name
                        ));
                    }
                }
            }
        }
        if pi_setup(&pm) != pi_setup(&tm) {
            // unreachable: pm.len()==tm.len() and every name matched above
        }
        setup_p += pi_setup(&pm);
        setup_t += pi_setup(&tm);
    }

    // rule 15: counts reconcile exactly
    if setup_p != setup_t {
        failures.push(format!("rule15: AT_SETUP count {} != {}", setup_p, setup_t));
    }
    if check_p != check_t {
        failures.push(format!("rule15: AT_CHECK count {} != {}", check_p, check_t));
    }
    // rule 20: every admitted IGNORE disposition must correspond to an actual changed
    // expectation (the manifest cannot admit changes the patch does not make)
    if admitted_hits == 0 && !admitted.is_empty() {
        failures.push(
            "rule20: no admitted change matched an actual patch change (manifest/patch divergence)"
                .to_string(),
        );
    }
    if manifest_trans.is_empty() {
        failures.push("manifest: no transformation records".to_string());
    }

    // cross-check the patch text itself: the patch must only touch the changed files, and its
    // `+++ b/` headers must exactly match the changed set (rule 2 + patch integrity)
    let parsed = parse_unified_diff(&patch_text);
    let patch_files: std::collections::BTreeSet<String> =
        parsed.iter().map(|(f, _)| f.clone()).collect();
    let changed_set: std::collections::BTreeSet<String> = changed.iter().cloned().collect();
    if patch_files != changed_set {
        failures.push(format!(
            "rule2: patch files {:?} != changed files {:?}",
            patch_files, changed_set
        ));
    }

    let pristine_manifest_sha = {
        let files: BTreeMap<String, String> = pristine_files
            .iter()
            .map(|f| {
                let n = f.file_name().and_then(|x| x.to_str()).unwrap_or("");
                (
                    n.to_string(),
                    file_sha256(&std::fs::read(f).unwrap_or_default()),
                )
            })
            .collect();
        file_sha256(&stable_json_bytes(&serde_json::json!({ "files": files })))
    };
    let transformed_manifest_sha = {
        let files: BTreeMap<String, String> = transformed_files
            .iter()
            .map(|f| {
                let n = f.file_name().and_then(|x| x.to_str()).unwrap_or("");
                (
                    n.to_string(),
                    file_sha256(&std::fs::read(f).unwrap_or_default()),
                )
            })
            .collect();
        file_sha256(&stable_json_bytes(&serde_json::json!({ "files": files })))
    };

    Ok(GateVerdict {
        schema: "gnurust-diag-unblocked-gate-v1".to_string(),
        patch_sha256: patch_sha,
        pristine_manifest_sha256: pristine_manifest_sha,
        transformed_manifest_sha256: transformed_manifest_sha,
        at_setup_pristine: setup_p,
        at_setup_transformed: setup_t,
        at_check_pristine: check_p,
        at_check_transformed: check_t,
        changed_files: changed,
        admitted_changes: manifest_trans.len(),
        failures,
    })
}

fn pi_setup(ms: &[SpannedMacro]) -> usize {
    ms.iter().filter(|m| m.name == "AT_SETUP").count()
}

fn read_json_file(p: &Path) -> Option<serde_json::Value> {
    let text = std::fs::read_to_string(p).ok()?;
    serde_json::from_str(&text).ok()
}

fn stable_json_bytes(v: &serde_json::Value) -> Vec<u8> {
    serde_json::to_vec(v).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r##"
AT_SETUP([MOVE SPACE TO numeric item])
AT_KEYWORDS([move editing])

AT_DATA([prog.cob], [
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.
       PROCEDURE        DIVISION.
           MOVE SPACE TO X.
           STOP RUN.
])

AT_CHECK([$COMPILE_ONLY prog.cob], [1], [],
[prog.cob:9: error: MOVE of figurative constant SPACE to numeric item used
])

AT_CLEANUP

AT_SETUP([MOVE ZERO TO alphabetic item])
AT_DATA([prog.cob], [
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.
       PROCEDURE        DIVISION.
           MOVE ZERO TO A.
           STOP RUN.
])

AT_CHECK([$COMPILE prog.cob], [0], [], [])
AT_CHECK([./prog], [0], [OK
], [])
AT_CLEANUP
"##;

    #[test]
    fn compiler_diagnostic_stderr_is_ignored_but_runtime_is_preserved() {
        let (trans, errs) = transform_file(SAMPLE, "syn_move.at", "rev");
        assert!(errs.is_empty(), "{errs:?}");
        assert_eq!(trans.len(), 3);
        // step 0: $COMPILE_ONLY status 1 with diagnostic stderr -> IgnoreCompilerStderr
        let t0 = &trans[0];
        assert_eq!(t0.command_shape, CommandShape::Compiler);
        assert_eq!(t0.disposition, DiagnosticDisposition::IgnoreCompilerStderr);
        assert!(t0.disposition.ignores_stderr());
        // step 1: $COMPILE with empty expectations -> Preserve (nothing to ignore)
        let t1 = &trans[1];
        assert_eq!(t1.disposition, DiagnosticDisposition::Preserve);
        // step 2: ./prog runtime -> Preserve (never a diagnostic candidate)
        let t2 = &trans[2];
        assert_eq!(t2.command_shape, CommandShape::Runtime);
        assert_eq!(t2.disposition, DiagnosticDisposition::Preserve);
        // exactly one edit: the stderr span of step 0 becomes [ignore]
        let edits = edits_for_file(SAMPLE, &trans).unwrap();
        assert_eq!(edits.len(), 1);
        let transformed = apply_edits(SAMPLE, edits);
        assert!(transformed.contains("AT_CHECK([$COMPILE_ONLY prog.cob], [1], [],\n[ignore])"));
        assert!(transformed.contains("AT_CHECK([./prog], [0], [OK"));
        // byte-identical outside the edited span: only the expectation text shrank
        assert_eq!(
            transformed.len(),
            SAMPLE.len()
                - ("prog.cob:9: error: MOVE of figurative constant SPACE to numeric item used\n")
                    .len()
                + "ignore".len()
        );
    }

    #[test]
    fn listing_stdout_is_never_ignored() {
        let src = "AT_SETUP([listing])\nAT_CHECK([$COMPILE_LISTING prog.cob], [0], [listing content\n], [])\nAT_CLEANUP\n";
        let (trans, _) = transform_file(src, "listings.at", "rev");
        assert_eq!(trans[0].command_shape, CommandShape::CompilerListing);
        assert_eq!(trans[0].disposition, DiagnosticDisposition::Preserve);
        let edits = edits_for_file(src, &trans).unwrap();
        assert!(edits.is_empty());
    }

    #[test]
    fn grep_postprocessing_is_preserved() {
        let src = "AT_SETUP([grep])\nAT_CHECK([$COMPILE_ONLY prog.cob 2>&1 | $GREP error], [0], [], [])\nAT_CLEANUP\n";
        let (trans, _) = transform_file(src, "syn_occurs.at", "rev");
        assert_eq!(trans[0].command_shape, CommandShape::ShellHelper);
        assert_eq!(trans[0].disposition, DiagnosticDisposition::Preserve);
        let edits = edits_for_file(src, &trans).unwrap();
        assert!(edits.is_empty());
    }

    #[test]
    fn non_diagnostic_application_stderr_is_preserved() {
        let src = "AT_SETUP([app])\nAT_CHECK([$COMPILE_ONLY prog.cob], [1], [], [This is sent to SYSERR  PLAIN\n])\nAT_CLEANUP\n";
        let (trans, _) = transform_file(src, "run_misc.at", "rev");
        assert_eq!(trans[0].disposition, DiagnosticDisposition::Preserve);
        let edits = edits_for_file(src, &trans).unwrap();
        assert!(edits.is_empty());
    }

    #[test]
    fn env_prefixed_runtime_is_preserved() {
        let src = "AT_SETUP([env])\nAT_CHECK([COB_SWITCH_1=ON $COBCRUN_DIRECT ./prog], [0], [OK\n], [])\nAT_CLEANUP\n";
        let (trans, _) = transform_file(src, "run_misc.at", "rev");
        assert_eq!(trans[0].command_shape, CommandShape::Runtime);
        assert_eq!(trans[0].disposition, DiagnosticDisposition::Preserve);
    }

    #[test]
    fn warning_stream_on_status_zero_is_ignored() {
        let src = "AT_SETUP([warn])\nAT_CHECK([$COMPILE prog.cob], [0], [], [prog.cob:11: warning: no CORRESPONDING items found\n])\nAT_CLEANUP\n";
        let (trans, _) = transform_file(src, "syn_move.at", "rev");
        assert_eq!(
            trans[0].disposition,
            DiagnosticDisposition::IgnoreCompilerStderr
        );
        let edits = edits_for_file(src, &trans).unwrap();
        assert_eq!(edits.len(), 1);
    }

    /// Integration: run the transformer over the admitted stable suite source (present in the
    /// working tree at `lab/admit/gnucobol-3.2/tests/testsuite.src`) and check the census is sane.
    #[test]
    fn admitted_suite_census_is_sane() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .unwrap()
            .join("lab/admit/gnucobol-3.2/tests/testsuite.src");
        if !root.is_dir() {
            eprintln!(
                "admitted suite source absent ({}) — skipping census",
                root.display()
            );
            return;
        }
        let res = transform_suite(&root, "stable-3.2").unwrap();
        assert!(res.errors.is_empty(), "errors: {:?}", res.errors);
        assert!(
            res.files_scanned >= 30,
            "files_scanned={}",
            res.files_scanned
        );
        let total = res.transformations.len();
        let ignored = res
            .transformations
            .iter()
            .filter(|t| t.disposition.ignores_stdout() || t.disposition.ignores_stderr())
            .count();
        let preserved = total - ignored;
        // sanity: the majority of compiler steps with non-empty diagnostic expectations are
        // candidates; runtime/helper steps dominate the preserved side. Exact numbers are not
        // pinned (upstream may drift); the census is recorded in the committed transformations.json.
        assert!(ignored > 300, "ignored={ignored}");
        assert!(
            preserved > ignored / 3,
            "preserved={preserved} ignored={ignored}"
        );
        // determinism: running twice yields identical records (stable_json sorted maps)
        let res2 = transform_suite(&root, "stable-3.2").unwrap();
        assert_eq!(res.pristine_manifest_sha256, res2.pristine_manifest_sha256);
        assert_eq!(
            res.transformed_manifest_sha256,
            res2.transformed_manifest_sha256
        );
        assert_eq!(res.transformations.len(), res2.transformations.len());
        // every ignored stream has a non-empty original expectation and a transformed `ignore`
        for t in res
            .transformations
            .iter()
            .filter(|t| t.disposition.ignores_stdout() || t.disposition.ignores_stderr())
        {
            if t.disposition.ignores_stdout() {
                assert!(
                    !t.stdout_sha256.is_empty(),
                    "empty stdout ignored in {}",
                    t.source_file
                );
                assert_eq!(t.transformed_stdout_sha256, Transformation::sha("ignore"));
            }
            if t.disposition.ignores_stderr() {
                assert!(
                    !t.stderr_sha256.is_empty(),
                    "empty stderr ignored in {}",
                    t.source_file
                );
                assert_eq!(t.transformed_stderr_sha256, Transformation::sha("ignore"));
            }
        }
        eprintln!(
            "admitted census: {} checks, {} ignored, {} preserved across {} files",
            total, ignored, preserved, res.files_scanned
        );
        // audit: every preserved COMPILER step must have a reason (never silent)
        let mut preserved_compiler = 0;
        let mut by_reason: std::collections::BTreeMap<&str, usize> = Default::default();
        for t in res.transformations.iter() {
            if matches!(
                t.command_shape,
                CommandShape::Compiler | CommandShape::CompilerListing
            ) && !(t.disposition.ignores_stdout() || t.disposition.ignores_stderr())
            {
                preserved_compiler += 1;
                let reason = if t.stdout_sha256.is_empty() && t.stderr_sha256.is_empty() {
                    "both expectations empty/ignore"
                } else if t.reason.contains("already ignore") {
                    "already ignore"
                } else if t.reason.contains("listing artifact") {
                    "listing stdout"
                } else if t.reason.contains("not proven") {
                    "not proven diagnostic"
                } else {
                    "other"
                };
                *by_reason.entry(reason).or_default() += 1;
            }
        }
        eprintln!(
            "preserved compiler steps: {} ({} both-empty, {} already-ignore, {} listing, {} not-proven, {} other)",
            preserved_compiler,
            by_reason.get("both expectations empty/ignore").copied().unwrap_or(0),
            by_reason.get("already ignore").copied().unwrap_or(0),
            by_reason.get("listing stdout").copied().unwrap_or(0),
            by_reason.get("not proven diagnostic").copied().unwrap_or(0),
            by_reason.get("other").copied().unwrap_or(0),
        );
        // the by_reason "other" bucket must be empty: show any stragglers' actual reasons
        for t in res.transformations.iter() {
            if matches!(
                t.command_shape,
                CommandShape::Compiler | CommandShape::CompilerListing
            ) && !(t.disposition.ignores_stdout() || t.disposition.ignores_stderr())
                && !(t.stdout_sha256.is_empty() && t.stderr_sha256.is_empty())
                && !t.reason.contains("listing artifact")
                && !t.reason.contains("not proven")
                && !t.reason.contains("already ignore")
            {
                eprintln!(
                    "  OTHER {}:{} cmd={:?} reason={}",
                    t.source_file,
                    t.source_line,
                    t.command.chars().take(40).collect::<String>(),
                    t.reason
                );
            }
        }
        // sample the not-proven compiler steps (evidence of conservatism)
        let mut samples: Vec<&Transformation> = Vec::new();
        for t in res.transformations.iter() {
            if matches!(
                t.command_shape,
                CommandShape::Compiler | CommandShape::CompilerListing
            ) && !(t.disposition.ignores_stdout() || t.disposition.ignores_stderr())
                && !(t.stdout_sha256.is_empty() && t.stderr_sha256.is_empty())
                && !t.reason.contains("listing artifact")
            {
                samples.push(t);
            }
        }
        samples.sort_by(|a, b| {
            (a.source_file.as_str(), a.source_line).cmp(&(b.source_file.as_str(), b.source_line))
        });
        for t in samples.iter().take(12) {
            eprintln!(
                "  preserved {}:{} cmd={:?} reason={}",
                t.source_file,
                t.source_line,
                t.command.chars().take(44).collect::<String>(),
                t.reason
            );
        }
        // the "other" bucket must not exist: every preserved compiler step has a typed reason
        let other: Vec<&Transformation> = samples
            .iter()
            .filter(|t| {
                !t.reason.contains("listing artifact")
                    && !t.reason.contains("not proven")
                    && !t.reason.contains("already ignore")
            })
            .cloned()
            .collect();
        for t in &other {
            eprintln!(
                "  OTHER {}:{} cmd={:?} reason={}",
                t.source_file,
                t.source_line,
                t.command.chars().take(40).collect::<String>(),
                t.reason
            );
        }
        assert!(
            other.is_empty(),
            "preserved compiler steps without a typed reason: {}",
            other.len()
        );
    }

    // -----------------------------------------------------------------------------------------
    // Phase 4 gate + Phase 11 adversarial tests
    // -----------------------------------------------------------------------------------------

    /// Build a scratch pristine/transformed/manifest/patch set; returns the dir.
    fn scratch_gate_inputs(
        pristine_src: &str,
        transform: impl Fn(&str) -> String,
    ) -> (tempfile::TempDir, String) {
        let td = tempfile::tempdir().unwrap();
        let pristine_dir = td.path().join("pristine");
        let transformed_dir = td.path().join("transformed");
        std::fs::create_dir_all(&pristine_dir).unwrap();
        std::fs::create_dir_all(&transformed_dir).unwrap();
        std::fs::write(pristine_dir.join("syn_move.at"), pristine_src).unwrap();
        std::fs::write(transformed_dir.join("syn_move.at"), transform(pristine_src)).unwrap();
        // manifest admitting the single diagnostic->ignore change at group 0 step 0
        let manifest = serde_json::json!({
            "schema": "gnurust-diag-unblocked-transformations-v1",
            "transformer_version": TRANSFORMER_VERSION,
            "transformations": [{
                "source_file": "syn_move.at",
                "group_index": 0,
                "step_index": 0,
                "disposition": "IGNORE_COMPILER_STDERR",
                "reason": "test",
                "command": "$COMPILE_ONLY prog.cob",
                "expected_status": "1",
                "stdout_sha256": "",
                "stderr_sha256": "x",
                "transformed_stderr_sha256": "y",
                "transformed_stdout_sha256": "",
                "already_ignored": false,
                "source_revision": "rev"
            }]
        });
        std::fs::write(
            td.path().join("transformations.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        (td, "syn_move.at".to_string())
    }

    const LEGAL_SRC: &str = "AT_SETUP([t])\nAT_CHECK([$COMPILE_ONLY prog.cob], [1], [], [prog.cob:9: error: bad\n])\nAT_CLEANUP\n";

    #[test]
    fn gate_accepts_legal_diagnostic_ignore() {
        let (td, _) = scratch_gate_inputs(LEGAL_SRC, |s| {
            s.replace("prog.cob:9: error: bad\n", "ignore")
        });
        let patch = unified_diff(
            LEGAL_SRC,
            &LEGAL_SRC.replace("prog.cob:9: error: bad\n", "ignore"),
            "syn_move.at",
        );
        let patch_path = td.path().join("p.patch");
        std::fs::write(&patch_path, patch).unwrap();
        let v = cmd_gate(
            &patch_path,
            &td.path().join("pristine"),
            &td.path().join("transformed"),
            &td.path().join("transformations.json"),
        )
        .unwrap();
        assert!(v.failures.is_empty(), "{:?}", v.failures);
        assert_eq!(v.at_setup_pristine, 1);
        assert_eq!(v.at_check_pristine, 1);
        assert_eq!(v.at_setup_transformed, 1);
        assert_eq!(v.at_check_transformed, 1);
    }

    #[test]
    fn gate_rejects_status_change() {
        let (td, _) = scratch_gate_inputs(LEGAL_SRC, |s| {
            // expected status 1 -> 0 (illegal)
            s.replace(
                "AT_CHECK([$COMPILE_ONLY prog.cob], [1]",
                "AT_CHECK([$COMPILE_ONLY prog.cob], [0]",
            )
        });
        let transformed = LEGAL_SRC.replace(
            "AT_CHECK([$COMPILE_ONLY prog.cob], [1]",
            "AT_CHECK([$COMPILE_ONLY prog.cob], [0]",
        );
        let patch_path = td.path().join("p.patch");
        std::fs::write(
            &patch_path,
            unified_diff(LEGAL_SRC, &transformed, "syn_move.at"),
        )
        .unwrap();
        let v = cmd_gate(
            &patch_path,
            &td.path().join("pristine"),
            &td.path().join("transformed"),
            &td.path().join("transformations.json"),
        )
        .unwrap();
        assert!(
            v.failures.iter().any(|f| f.contains("rule5")),
            "{:?}",
            v.failures
        );
    }

    #[test]
    fn gate_rejects_command_change() {
        let (td, _) = scratch_gate_inputs(LEGAL_SRC, |s| {
            s.replace(
                "$COMPILE_ONLY prog.cob",
                "$COMPILE_ONLY -std=cobol85 prog.cob",
            )
        });
        let transformed = LEGAL_SRC.replace(
            "$COMPILE_ONLY prog.cob",
            "$COMPILE_ONLY -std=cobol85 prog.cob",
        );
        let patch_path = td.path().join("p.patch");
        std::fs::write(
            &patch_path,
            unified_diff(LEGAL_SRC, &transformed, "syn_move.at"),
        )
        .unwrap();
        let v = cmd_gate(
            &patch_path,
            &td.path().join("pristine"),
            &td.path().join("transformed"),
            &td.path().join("transformations.json"),
        )
        .unwrap();
        assert!(
            v.failures.iter().any(|f| f.contains("rule4")),
            "{:?}",
            v.failures
        );
    }

    #[test]
    fn gate_rejects_runtime_stdout_ignore() {
        // runtime ./prog stdout must NEVER become ignore
        let src = "AT_SETUP([t])\nAT_CHECK([./prog], [0], [OK\n], [])\nAT_CLEANUP\n";
        let (td, _) = scratch_gate_inputs(src, |s| s.replace("OK\n", "ignore"));
        let transformed = src.replace("OK\n", "ignore");
        let patch_path = td.path().join("p.patch");
        std::fs::write(&patch_path, unified_diff(src, &transformed, "syn_move.at")).unwrap();
        let v = cmd_gate(
            &patch_path,
            &td.path().join("pristine"),
            &td.path().join("transformed"),
            &td.path().join("transformations.json"),
        )
        .unwrap();
        assert!(
            !v.failures.is_empty(),
            "gate must reject runtime stdout ignore"
        );
    }

    #[test]
    fn gate_rejects_at_data_change() {
        let src = "AT_SETUP([t])\nAT_DATA([prog.cob], [IDENTIFICATION DIVISION.\n])\nAT_CHECK([$COMPILE_ONLY prog.cob], [1], [], [prog.cob:9: error: bad\n])\nAT_CLEANUP\n";
        let (td, _) = scratch_gate_inputs(src, |s| {
            s.replace(
                "IDENTIFICATION DIVISION.",
                "IDENTIFICATION DIVISION. CHANGED",
            )
        });
        let transformed = src.replace(
            "IDENTIFICATION DIVISION.",
            "IDENTIFICATION DIVISION. CHANGED",
        );
        let patch_path = td.path().join("p.patch");
        std::fs::write(&patch_path, unified_diff(src, &transformed, "syn_move.at")).unwrap();
        let v = cmd_gate(
            &patch_path,
            &td.path().join("pristine"),
            &td.path().join("transformed"),
            &td.path().join("transformations.json"),
        )
        .unwrap();
        assert!(
            v.failures.iter().any(|f| f.contains("rule12")),
            "{:?}",
            v.failures
        );
    }

    #[test]
    fn gate_rejects_group_deletion() {
        let src = "AT_SETUP([t1])\nAT_CHECK([$COMPILE_ONLY a.cob], [1], [], [a.cob:1: error: x\n])\nAT_CLEANUP\nAT_SETUP([t2])\nAT_CHECK([$COMPILE_ONLY b.cob], [1], [], [b.cob:1: error: y\n])\nAT_CLEANUP\n";
        let (td, _) = scratch_gate_inputs(src, |s| {
            // delete the second group
            s.replace(
                "AT_CLEANUP\nAT_SETUP([t2])\nAT_CHECK([$COMPILE_ONLY b.cob], [1], [], [b.cob:1: error: y\n])\nAT_CLEANUP\n",
                "AT_CLEANUP\n",
            )
        });
        let transformed = src.replace(
            "AT_CLEANUP\nAT_SETUP([t2])\nAT_CHECK([$COMPILE_ONLY b.cob], [1], [], [b.cob:1: error: y\n])\nAT_CLEANUP\n",
            "AT_CLEANUP\n",
        );
        let patch_path = td.path().join("p.patch");
        std::fs::write(&patch_path, unified_diff(src, &transformed, "syn_move.at")).unwrap();
        let v = cmd_gate(
            &patch_path,
            &td.path().join("pristine"),
            &td.path().join("transformed"),
            &td.path().join("transformations.json"),
        )
        .unwrap();
        assert!(
            v.failures
                .iter()
                .any(|f| f.contains("rule14") || f.contains("rule13")),
            "{:?}",
            v.failures
        );
    }

    #[test]
    fn gate_rejects_skip_if_insertion() {
        let src = "AT_SETUP([t])\nAT_CHECK([$COMPILE_ONLY prog.cob], [1], [], [prog.cob:9: error: bad\n])\nAT_CLEANUP\n";
        let (td, _) = scratch_gate_inputs(src, |s| {
            s.replace("AT_SETUP([t])\n", "AT_SETUP([t])\nAT_SKIP_IF([true])\n")
        });
        let transformed = src.replace("AT_SETUP([t])\n", "AT_SETUP([t])\nAT_SKIP_IF([true])\n");
        let patch_path = td.path().join("p.patch");
        std::fs::write(&patch_path, unified_diff(src, &transformed, "syn_move.at")).unwrap();
        let v = cmd_gate(
            &patch_path,
            &td.path().join("pristine"),
            &td.path().join("transformed"),
            &td.path().join("transformations.json"),
        )
        .unwrap();
        assert!(
            v.failures
                .iter()
                .any(|f| f.contains("rule12") || f.contains("rule14")),
            "{:?}",
            v.failures
        );
    }

    #[test]
    fn gate_rejects_entire_check_replacement() {
        // replacing an entire AT_CHECK with a permissive form (status -> 0) must fail
        let src = "AT_SETUP([t])\nAT_CHECK([$COMPILE_ONLY prog.cob], [1], [], [prog.cob:9: error: bad\n])\nAT_CLEANUP\n";
        let (td, _) = scratch_gate_inputs(src, |s| {
            s.replace(
                "AT_CHECK([$COMPILE_ONLY prog.cob], [1], [], [prog.cob:9: error: bad\n])",
                "AT_CHECK([$COMPILE_ONLY prog.cob], [0], [], [])",
            )
        });
        let transformed = src.replace(
            "AT_CHECK([$COMPILE_ONLY prog.cob], [1], [], [prog.cob:9: error: bad\n])",
            "AT_CHECK([$COMPILE_ONLY prog.cob], [0], [], [])",
        );
        let patch_path = td.path().join("p.patch");
        std::fs::write(&patch_path, unified_diff(src, &transformed, "syn_move.at")).unwrap();
        let v = cmd_gate(
            &patch_path,
            &td.path().join("pristine"),
            &td.path().join("transformed"),
            &td.path().join("transformations.json"),
        )
        .unwrap();
        assert!(
            v.failures.iter().any(|f| f.contains("rule5")),
            "{:?}",
            v.failures
        );
    }

    #[test]
    fn unified_diff_is_deterministic_and_roundtrips() {
        let a = "line1\nline2\nline3\nAT_CHECK([$COMPILE_ONLY p.cob], [1], [], [p.cob:1: error: x\n])\nline5\n";
        let b = a.replace("p.cob:1: error: x\n", "ignore");
        let d1 = unified_diff(a, &b, "t.at");
        let d2 = unified_diff(a, &b, "t.at");
        assert_eq!(d1, d2, "diff must be deterministic");
        assert!(d1.contains("--- a/t.at"));
        assert!(d1.contains("+++ b/t.at"));
        assert!(d1.contains("@@ "));
        assert!(d1.contains("-AT_CHECK([$COMPILE_ONLY p.cob], [1], [], [p.cob:1: error: x"));
        assert!(d1.contains("+AT_CHECK([$COMPILE_ONLY p.cob], [1], [], [ignore])"));
        // the parsed hunks must reference the file
        let parsed = parse_unified_diff(&d1);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].0, "t.at");
        assert!(!parsed[0].1.is_empty());
    }

    #[test]
    fn cmd_transform_is_reproducible() {
        // transform a scratch tree twice; the patch + manifests must be byte-identical
        let td = tempfile::tempdir().unwrap();
        let src = td.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(
            src.join("a.at"),
            "AT_SETUP([t])\nAT_CHECK([$COMPILE_ONLY prog.cob], [1], [], [prog.cob:9: error: bad\n])\nAT_CLEANUP\n",
        )
        .unwrap();
        let out1 = td.path().join("out1");
        let out2 = td.path().join("out2");
        cmd_transform(&src, &out1, "rev").unwrap();
        cmd_transform(&src, &out2, "rev").unwrap();
        let p1 = std::fs::read(out1.join("diagnostic-ignore.patch")).unwrap();
        let p2 = std::fs::read(out2.join("diagnostic-ignore.patch")).unwrap();
        assert_eq!(p1, p2, "patch must be byte-identical across runs");
        let m1 = std::fs::read(out1.join("transformations.json")).unwrap();
        let m2 = std::fs::read(out2.join("transformations.json")).unwrap();
        assert_eq!(
            m1, m2,
            "transformations.json must be byte-identical across runs"
        );
        // the gate must pass on the freshly generated output
        let v = cmd_gate(
            &out1.join("diagnostic-ignore.patch"),
            &out1.join("pristine"),
            &out1.join("transformed"),
            &out1.join("transformations.json"),
        )
        .unwrap();
        assert!(v.failures.is_empty(), "{:?}", v.failures);
    }
}
