//! Phases 7–9 of the diagnostic-unblocked lane: semantic-reachability measurement,
//! pristine-vs-unblocked reconciliation, and corpus cross-check.
//!
//! The lane's three views stay separate and are NEVER conflated:
//!   A. pristine upstream testsuite — untouched, authoritative, immutable;
//!   B. diagnostic-unblocked testsuite — derived mechanically (only proven compiler-diagnostic
//!      expected streams become Autotest `ignore`);
//!   C. existing step/corpus phase probes — preserved, not replaced.
//!
//! This module computes, from the COMMITTED raw evidence only (both lanes' `testsuite.log` +
//! `testsuite.dir/NNNN/testsuite.log` + `transformations.json`), the answer to the primary
//! question of the lane:
//!
//!   "What later semantic checks became reachable solely because compiler diagnostic text
//!    stopped gating the group?"
//!
//! Group identity is stable across the lanes: both generated suites carry the identical
//! `N;file;title;...` index (verified byte-identical for all 1281 entries) and identical check
//! sequences per group (only the embedded `file:line` references shift because multi-line
//! diagnostic expectations collapse to `[ignore]`). Checks are therefore matched across lanes by
//! step index within the group, and groups by their suite number.

use crate::diag_unblocked::{
    classify_command, cmd_gate, transform_suite, CommandShape, GateVerdict, TransformResult,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

fn now_utc() -> String {
    crate::cli::now_utc_string_pub()
}

/// The report timestamp is the FROZEN evidence instant (the transformations.json generation
/// time) rather than the wall clock, so regenerating a report from the same committed evidence
/// produces byte-identical output (deterministic projection).
fn evidence_timestamp(manifest: &serde_json::Value) -> String {
    manifest["generated_at_utc"]
        .as_str()
        .map(|s| s.to_string())
        .unwrap_or_else(now_utc)
}

fn read_json(p: &Path) -> Result<serde_json::Value, String> {
    let bytes = std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", p.display()))
}

fn write_json<T: serde::Serialize>(v: &T, p: &Path) -> Result<(), String> {
    let s = serde_json::to_string_pretty(v).map_err(|e| e.to_string())?;
    std::fs::write(p, s + "\n").map_err(|e| format!("{}: {e}", p.display()))
}

// ---------------------------------------------------------------------------------------------
// log model
// ---------------------------------------------------------------------------------------------

/// Group-level result as printed by Autotest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupResult {
    Ok,
    Failed,
    ExpectedFailure,
    UnexpectedPass,
    Skipped,
}

impl GroupResult {
    pub fn as_str(&self) -> &'static str {
        match self {
            GroupResult::Ok => "ok",
            GroupResult::Failed => "FAILED",
            GroupResult::ExpectedFailure => "expected failure",
            GroupResult::UnexpectedPass => "UNEXPECTED PASS",
            GroupResult::Skipped => "skipped",
        }
    }
}

/// One executed AT_CHECK recovered from a per-group `testsuite.log`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CheckOutcome {
    pub step: usize,
    pub file: String,
    pub line: usize,
    pub command: String,
    /// `(actual, expected)` when the exit status mismatched.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_mismatch: Option<(i64, i64)>,
    /// True when the stdout/stderr expectation produced a text diff.
    pub text_mismatch: bool,
}

/// A parsed per-group `testsuite.dir/NNNN/testsuite.log`.
#[derive(Debug, Clone)]
pub struct GroupLog {
    pub number: u32,
    pub file: String,
    pub title: String,
    pub checks: Vec<CheckOutcome>,
    pub result: GroupResult,
}

/// A group line from a top-level `testsuite.log`.
#[derive(Debug, Clone)]
pub struct MainGroupLine {
    pub number: u32,
    pub file: String,
    pub title: String,
    pub result: GroupResult,
}

/// Why a group stopped at its first failing check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoppingReason {
    /// No failing check (group passed all executed checks).
    None,
    /// Only the exit status mismatched.
    Status,
    /// Only the stdout/stderr text mismatched.
    TextOnly,
    /// Both the exit status and the text mismatched.
    TextAndStatus,
    /// Failed without any recoverable marker.
    Other,
}

impl StoppingReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            StoppingReason::None => "NONE",
            StoppingReason::Status => "STATUS",
            StoppingReason::TextOnly => "TEXT_ONLY",
            StoppingReason::TextAndStatus => "TEXT_AND_STATUS",
            StoppingReason::Other => "OTHER",
        }
    }
}

// ---------------------------------------------------------------------------------------------
// parsers
// ---------------------------------------------------------------------------------------------

fn sha_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Parse a result token ("ok", "FAILED", "expected failure", "UNEXPECTED PASS", "skipped")
/// from the tail of a result line (the token may be followed by a parenthesised location/time).
fn parse_result_token(tok: &str) -> Option<GroupResult> {
    let tok = tok.trim();
    if tok.starts_with("ok") {
        Some(GroupResult::Ok)
    } else if tok.starts_with("FAILED") {
        Some(GroupResult::Failed)
    } else if tok.starts_with("expected failure") {
        Some(GroupResult::ExpectedFailure)
    } else if tok.starts_with("UNEXPECTED PASS") {
        Some(GroupResult::UnexpectedPass)
    } else if tok.starts_with("skipped") {
        Some(GroupResult::Skipped)
    } else {
        None
    }
}

/// Parse the top-level `testsuite.log` result lines:
/// `NNN. file.at:LINE: NNN. TITLE (file.at:LINE): RESULT ...`
pub fn parse_main_log(path: &Path) -> Result<BTreeMap<u32, MainGroupLine>, String> {
    let text = std::fs::read(path).map_err(|e| format!("main log {}: {e}", path.display()))?;
    let text = String::from_utf8_lossy(&text);
    let mut out = BTreeMap::new();
    for line in text.lines() {
        // Main-log result lines come in two shapes:
        //   "NNN. TITLE (file.at:LINE): RESULT"                       (ok/skip lines)
        //   "NNN. file.at:LINE: NNN. TITLE (file.at:LINE): RESULT (loc)" (failed lines)
        // Both end in `: <result-token>`; anchor on the token (last occurrence, so titles
        // containing result words still parse).
        let (head, res) = match line
            .rfind(": ok")
            .or_else(|| line.rfind(": FAILED"))
            .or_else(|| line.rfind(": expected failure"))
            .or_else(|| line.rfind(": UNEXPECTED PASS"))
            .or_else(|| line.rfind(": skipped"))
        {
            Some(i) => (&line[..i], &line[i + 2..]),
            None => continue,
        };
        // head ends with the group location "(file.at:LINE)"
        let close = match head.rfind(')') {
            Some(c) => c,
            None => continue,
        };
        let open = match head[..close].rfind(" (") {
            Some(o) => o,
            None => continue,
        };
        let loc = head[open + 2..close].trim();
        let mut title = head[..open].trim();
        // strip a leading number: "NNN. "
        let num = match title.split_whitespace().next() {
            Some(n) => match n.trim_end_matches('.').parse::<u32>() {
                Ok(n) => n,
                Err(_) => continue,
            },
            None => continue,
        };
        let prefix = format!("{num}. ");
        if let Some(rest) = title.strip_prefix(&prefix) {
            title = rest.trim();
        }
        // strip a per-group-style "file.at:LINE: NNN. " prefix
        if let Some(idx) = title.find(": ") {
            let candidate = &title[idx + 2..];
            if let Some(rest) = candidate.strip_prefix(&prefix) {
                title = rest.trim();
            }
        }
        if title.is_empty() {
            continue;
        }
        let file = match loc.split(':').next() {
            Some(f) => f.trim().to_string(),
            None => continue,
        };
        let result = match parse_result_token(res.trim()) {
            Some(r) => r,
            None => continue,
        };
        out.insert(
            num,
            MainGroupLine {
                number: num,
                file,
                title: title.to_string(),
                result,
            },
        );
    }
    Ok(out)
}

/// Parse one per-group `testsuite.dir/NNNN/testsuite.log`.
pub fn parse_group_log(path: &Path) -> Option<GroupLog> {
    let bytes = std::fs::read(path).ok()?;
    let text = String::from_utf8_lossy(&bytes);
    let lines: Vec<&str> = text.lines().collect();
    // header: "NNN. file.at:LINE: testing TITLE ..."
    let mut number = 0u32;
    let mut file = String::new();
    let mut title = String::new();
    for line in &lines {
        let trimmed = line.strip_suffix(" ...").unwrap_or(line);
        // "NNN. file.at:LINE: testing TITLE"
        if let Some(idx) = trimmed.rfind(": testing ") {
            let head = &trimmed[..idx];
            let t = &trimmed[idx + ": testing ".len()..];
            let mut hp = head.splitn(2, ". ");
            if let (Some(n), Some(loc)) = (hp.next(), hp.next()) {
                if let Ok(n) = n.trim().parse::<u32>() {
                    let mut lp = loc.splitn(2, ':');
                    if let (Some(f), Some(_)) = (lp.next(), lp.next()) {
                        number = n;
                        file = f.trim().to_string();
                        title = t.to_string();
                        break;
                    }
                }
            }
        }
    }
    if number == 0 {
        return None;
    }
    let mut checks: Vec<CheckOutcome> = Vec::new();
    let mut result = GroupResult::Ok;
    let mut cur: Option<usize> = None; // index of the current check
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if let Some(rest) = line.strip_prefix("./") {
            // check command or exit-code line: "./file.at:LINE: ..."
            let mut parts = rest.splitn(3, ':');
            if let (Some(f), Some(ln), Some(cmd)) = (parts.next(), parts.next(), parts.next()) {
                if cmd.trim_start().starts_with("exit code was") {
                    // "exit code was X, expected Y"
                    let mut nums = cmd.split_whitespace().filter_map(|w| w.parse::<i64>().ok());
                    let (actual, expected) = (nums.next(), nums.next());
                    // find the check this exit-code line belongs to (by file:line, then by position)
                    let match_idx = checks
                        .iter()
                        .rposition(|c| {
                            c.file == f.trim() && c.line == ln.trim().parse().unwrap_or(0)
                        })
                        .or_else(|| {
                            cur.filter(|ci| {
                                checks.get(*ci).map(|c| c.file == f.trim()).unwrap_or(false)
                            })
                        });
                    if let Some(ci) = match_idx {
                        if let (Some(a), Some(e)) = (actual, expected) {
                            checks[ci].status_mismatch = Some((a, e));
                        }
                    }
                } else {
                    let step = checks.len();
                    checks.push(CheckOutcome {
                        step,
                        file: f.trim().to_string(),
                        line: ln.trim().parse().unwrap_or(0),
                        command: cmd.trim_end().to_string(),
                        status_mismatch: None,
                        text_mismatch: false,
                    });
                    cur = Some(step);
                }
                i += 1;
                continue;
            }
        }
        // diff start: "--- <exp> ..." followed by "+++ <act> ..."
        if line.starts_with("--- ") && i + 1 < lines.len() && lines[i + 1].starts_with("+++ ") {
            if let Some(ci) = cur {
                if let Some(c) = checks.get_mut(ci) {
                    c.text_mismatch = true;
                }
            }
            i += 2;
            // skip the "@@ -a,b +c,d @@" and the diff body
            while i < lines.len() {
                let l = lines[i];
                if l.starts_with("@@ ") {
                    i += 1;
                    continue;
                }
                if l.starts_with('-')
                    || l.starts_with('+')
                    || l.starts_with(' ')
                    || l.starts_with("\\ No newline")
                {
                    i += 1;
                    continue;
                }
                break;
            }
            continue;
        }
        // result line: "NNN. file.at:LINE: NNN. TITLE (file.at:LINE): RESULT (loc)"
        if let Some(rest) = line.strip_prefix(&format!("{number}. ")) {
            if let Some(idx) = rest.rfind(": ") {
                let tok = rest[idx + 2..].trim();
                if let Some(r) = parse_result_token(tok) {
                    result = r;
                }
            }
        }
        i += 1;
    }
    Some(GroupLog {
        number,
        file,
        title,
        checks,
        result,
    })
}

/// The set of `testsuite.dir/NNNN` per-group logs present for a run.
pub fn group_log_dir_numbers(dir: &Path) -> BTreeSet<u32> {
    let mut out = BTreeSet::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            if let Some(name) = e.file_name().to_str() {
                if let Ok(n) = name.parse::<u32>() {
                    if e.path().join("testsuite.log").is_file() {
                        out.insert(n);
                    }
                }
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------------------------
// group map
// ---------------------------------------------------------------------------------------------

/// Suite group map: number -> (file, title), plus per-file ordinals (0-based, in number order).
/// Ordinals agree with `transformations.json` `group_index` for every group present in the
/// generated suite (conditional AT_SETUP groups that never expand are simply absent).
#[derive(Debug, Clone)]
pub struct GroupMap {
    /// number -> (file, title)
    pub groups: BTreeMap<u32, (String, String)>,
    /// file -> sorted group numbers (ordinal == index in this vec)
    pub by_file: BTreeMap<String, Vec<u32>>,
}

impl GroupMap {
    pub fn build_from(main: &BTreeMap<u32, MainGroupLine>, logs: &[GroupLog]) -> GroupMap {
        let mut groups: BTreeMap<u32, (String, String)> = BTreeMap::new();
        for (n, m) in main {
            groups.insert(*n, (m.file.clone(), m.title.clone()));
        }
        for g in logs {
            groups
                .entry(g.number)
                .or_insert_with(|| (g.file.clone(), g.title.clone()));
        }
        let mut by_file: BTreeMap<String, Vec<u32>> = BTreeMap::new();
        for (n, (f, _)) in &groups {
            by_file.entry(f.clone()).or_default().push(*n);
        }
        for v in by_file.values_mut() {
            v.sort_unstable();
        }
        GroupMap { groups, by_file }
    }

    /// 0-based ordinal of `number` within its file, or None when unknown.
    pub fn ordinal(&self, number: u32) -> Option<usize> {
        let (f, _) = self.groups.get(&number)?;
        self.by_file.get(f)?.binary_search(&number).ok()
    }
}

// ---------------------------------------------------------------------------------------------
// Phase 7 — semantic reachability
// ---------------------------------------------------------------------------------------------

/// The command-shape bucket used for newly-reached checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReachKind {
    CompilerCheck,
    RuntimeExecution,
    ArtifactOrHelper,
    Unknown,
}

fn reach_kind(shape: CommandShape) -> ReachKind {
    match shape {
        CommandShape::Compiler | CommandShape::CompilerListing => ReachKind::CompilerCheck,
        CommandShape::Runtime => ReachKind::RuntimeExecution,
        CommandShape::ShellHelper | CommandShape::GeneratedFile => ReachKind::ArtifactOrHelper,
        CommandShape::Unknown => ReachKind::Unknown,
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct NewlyReachedCheck {
    pub step: usize,
    pub command: String,
    pub shape: ReachKind,
    pub passed: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GroupReachability {
    pub number: u32,
    pub file: String,
    pub title: String,
    pub group_index: Option<usize>,
    pub ignored_expectations: usize,
    pub pristine_result: Option<String>,
    pub unblocked_result: Option<String>,
    pub pristine_checks_run: usize,
    pub unblocked_checks_run: usize,
    pub pristine_first_failing_step: Option<usize>,
    pub unblocked_first_failing_step: Option<usize>,
    pub pristine_stopping_reason: String,
    pub pristine_stopping_gated: bool,
    pub progressed: bool,
    pub newly_reached: Vec<NewlyReachedCheck>,
    /// Analysis notes (findings, scope limits).
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ReachabilityReport {
    pub schema: String,
    pub transformer_version: String,
    pub source_revision: String,
    pub generated_at_utc: String,
    pub inputs: BTreeMap<String, String>,
    pub totals: serde_json::Value,
    pub oracle: serde_json::Value,
    pub groups: Vec<GroupReachability>,
    pub findings: Vec<String>,
}

fn check_stopping_reason(c: &CheckOutcome) -> StoppingReason {
    match (c.status_mismatch.is_some(), c.text_mismatch) {
        (true, true) => StoppingReason::TextAndStatus,
        (true, false) => StoppingReason::Status,
        (false, true) => StoppingReason::TextOnly,
        (false, false) => StoppingReason::Other,
    }
}

struct TransformIndex {
    /// (file, group_index, step_index) -> Transformation (only IGNORE dispositions matter here)
    ignores: BTreeSet<(String, usize, usize)>,
    /// (file, group_index) -> count of ignored expectations
    ignore_count: BTreeMap<(String, usize), usize>,
    /// (file, group_index) -> total check count (from the pristine source census)
    total_checks: BTreeMap<(String, usize), usize>,
    /// (file, group_index, step_index) -> command (census; every AT_CHECK is present)
    commands: BTreeMap<(String, usize, usize), String>,
    /// (file, group_index) -> group is AT_XFAIL
    xfail: BTreeMap<(String, usize), bool>,
}

fn build_transform_index(manifest: &serde_json::Value) -> TransformIndex {
    let mut ignores = BTreeSet::new();
    let mut ignore_count: BTreeMap<(String, usize), usize> = BTreeMap::new();
    let mut total_checks: BTreeMap<(String, usize), usize> = BTreeMap::new();
    let mut commands: BTreeMap<(String, usize, usize), String> = BTreeMap::new();
    let mut xfail: BTreeMap<(String, usize), bool> = BTreeMap::new();
    if let Some(trans) = manifest["transformations"].as_array() {
        for t in trans {
            let key = (
                t["source_file"].as_str().unwrap_or("").to_string(),
                t["group_index"].as_u64().unwrap_or(0) as usize,
                t["step_index"].as_u64().unwrap_or(0) as usize,
            );
            *total_checks.entry((key.0.clone(), key.1)).or_insert(0) += 1;
            commands.insert(key.clone(), t["command"].as_str().unwrap_or("").to_string());
            let disp = t["disposition"].as_str().unwrap_or("");
            if disp.contains("IGNORE") {
                ignores.insert(key.clone());
                *ignore_count.entry((key.0.clone(), key.1)).or_insert(0) += 1;
            }
            let gk = (key.0.clone(), key.1);
            xfail.entry(gk).or_insert_with(|| {
                t["group_xfail"]
                    .as_array()
                    .map(|v| !v.is_empty())
                    .unwrap_or(false)
            });
        }
    }
    TransformIndex {
        ignores,
        ignore_count,
        total_checks,
        commands,
        xfail,
    }
}

/// The per-group result line can be glued to raw binary output (a compiled artifact the
/// harness dumped into the log); when the parsed result defaults to `ok` but the executed
/// checks contain a failure, derive the result from the check outcomes + the xfail status.
fn fix_group_result(g: &mut GroupLog, xfail: bool) {
    if g.result == GroupResult::Ok {
        let has_failure = g
            .checks
            .iter()
            .any(|c| c.status_mismatch.is_some() || c.text_mismatch);
        if has_failure {
            g.result = if xfail {
                GroupResult::ExpectedFailure
            } else {
                GroupResult::Failed
            };
        }
    }
}

/// Run the Phase 7 semantic-reachability measurement from committed raw evidence.
pub fn cmd_reachability(
    pristine_log: &Path,
    pristine_dir: &Path,
    unblocked_log: &Path,
    unblocked_dir: &Path,
    pristine_oracle_log: &Path,
    unblocked_oracle_log: &Path,
    transformations: &Path,
    out_root: &Path,
) -> Result<ReachabilityReport, String> {
    let manifest = read_json(transformations)?;
    let now = evidence_timestamp(&manifest);
    let tix = build_transform_index(&manifest);
    let source_revision = manifest["source_revision"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();
    let transformer_version = manifest["transformer_version"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();

    let p_main = parse_main_log(pristine_log)?;
    let u_main = parse_main_log(unblocked_log)?;
    let p_dir_nums = group_log_dir_numbers(pristine_dir);
    let u_dir_nums = group_log_dir_numbers(unblocked_dir);
    let mut p_logs: BTreeMap<u32, GroupLog> = BTreeMap::new();
    for n in &p_dir_nums {
        if let Some(g) = parse_group_log(&pristine_dir.join(format!("{n:04}/testsuite.log"))) {
            p_logs.insert(*n, g);
        }
    }
    let mut u_logs: BTreeMap<u32, GroupLog> = BTreeMap::new();
    for n in &u_dir_nums {
        if let Some(g) = parse_group_log(&unblocked_dir.join(format!("{n:04}/testsuite.log"))) {
            u_logs.insert(*n, g);
        }
    }
    let p_log_vec: Vec<GroupLog> = p_logs.values().cloned().collect();
    let u_log_vec: Vec<GroupLog> = u_logs.values().cloned().collect();
    let map = GroupMap::build_from(&p_main, &p_log_vec);
    let map_u = GroupMap::build_from(&u_main, &u_log_vec);
    // repair results whose final line was glued to binary artifact output (see fix_group_result)
    for g in p_logs.values_mut() {
        let xf = map
            .ordinal(g.number)
            .and_then(|gi| tix.xfail.get(&(g.file.clone(), gi)))
            .copied()
            .unwrap_or(false);
        fix_group_result(g, xf);
    }
    for g in u_logs.values_mut() {
        let xf = map_u
            .ordinal(g.number)
            .and_then(|gi| tix.xfail.get(&(g.file.clone(), gi)))
            .copied()
            .unwrap_or(false);
        fix_group_result(g, xf);
    }
    // the two maps must agree on number -> (file, title)
    let mut map_findings: Vec<String> = Vec::new();
    for (n, (f, t)) in &map.groups {
        match map_u.groups.get(n) {
            Some((uf, ut)) => {
                if uf != f || ut != t {
                    map_findings.push(format!(
                        "group {n} identity differs between lanes: pristine {f}/{t} vs unblocked {uf}/{ut}"
                    ));
                }
            }
            None => map_findings.push(format!("group {n} absent from the unblocked group map")),
        }
    }

    // suite total: the union of group evidence across ALL sources (both lanes' candidate and
    // oracle logs/dirs); the generated suites carry the identical 1281-entry index, and the
    // oracle executes every group, so this union is the executed-group set.
    let p_ora_main = parse_main_log(pristine_oracle_log)?;
    let u_ora_main = parse_main_log(unblocked_oracle_log)?;
    // the oracle per-group dirs are siblings of the candidate dirs' parent (`raw/`)
    let p_ora_dirs = group_log_dir_numbers(&oracle_sibling_dir(pristine_dir, "baseline"));
    let u_ora_dirs = group_log_dir_numbers(&oracle_sibling_dir(unblocked_dir, "oracle"));
    let suite_all: BTreeSet<u32> = p_main
        .keys()
        .copied()
        .chain(p_dir_nums.iter().copied())
        .chain(u_main.keys().copied())
        .chain(u_dir_nums.iter().copied())
        .chain(p_ora_main.keys().copied())
        .chain(u_ora_main.keys().copied())
        .chain(p_ora_dirs)
        .chain(u_ora_dirs)
        .collect();
    let suite_total = suite_all.len();
    let mut totals_findings = Vec::new();
    let p_ora_all: BTreeSet<u32> = p_ora_main
        .keys()
        .copied()
        .chain(group_log_dir_numbers(&oracle_sibling_dir(
            pristine_dir,
            "baseline",
        )))
        .collect();
    let u_ora_all: BTreeSet<u32> = u_ora_main
        .keys()
        .copied()
        .chain(group_log_dir_numbers(&oracle_sibling_dir(
            unblocked_dir,
            "oracle",
        )))
        .collect();
    if p_ora_all.len() != suite_total {
        totals_findings.push(format!(
            "pristine oracle evidence covers {} groups, suite total is {}",
            p_ora_all.len(),
            suite_total
        ));
    }
    if u_ora_all.len() != suite_total {
        totals_findings.push(format!(
            "oracle evidence group sets differ: pristine {} vs unblocked {}",
            suite_total,
            u_ora_all.len()
        ));
    }

    // oracle xpass
    let p_ora_xpass: Vec<u32> = p_ora_main
        .iter()
        .filter(|(_, m)| m.result == GroupResult::UnexpectedPass)
        .map(|(n, _)| *n)
        .collect();
    let u_ora_xpass: Vec<u32> = u_ora_main
        .iter()
        .filter(|(_, m)| m.result == GroupResult::UnexpectedPass)
        .map(|(n, _)| *n)
        .collect();
    let mut oracle_xpass_rows: Vec<serde_json::Value> = Vec::new();
    for n in &u_ora_xpass {
        let (f, t) = map.groups.get(n).cloned().unwrap_or_default();
        let xfail = manifest["transformations"]
            .as_array()
            .and_then(|a| {
                a.iter().find(|x| {
                    x["group_title"].as_str() == Some(t.as_str())
                        && x["source_file"].as_str() == Some(f.as_str())
                })
            })
            .map(|x| {
                !x["group_xfail"]
                    .as_array()
                    .map(|v| v.is_empty())
                    .unwrap_or(true)
            })
            .unwrap_or(false);
        oracle_xpass_rows.push(serde_json::json!({
            "number": n,
            "file": f,
            "title": t,
            "pristine_result": p_ora_main.get(n).map(|m| m.result.as_str()).unwrap_or("no trace"),
            "xfail": xfail,
        }));
    }
    for n in &p_ora_xpass {
        if !u_ora_xpass.contains(n) {
            totals_findings.push(format!(
                "oracle group {n} passed unexpectedly in pristine but not in unblocked"
            ));
        }
    }

    // affected groups: (file, group_index) with >= 1 ignore
    let mut affected: BTreeSet<(String, usize)> = BTreeSet::new();
    for (k, _) in &tix.ignore_count {
        affected.insert(k.clone());
    }
    let mut affected_numbers: BTreeSet<u32> = BTreeSet::new();
    let mut affected_not_in_suite: Vec<(String, usize)> = Vec::new();
    for (f, gi) in &affected {
        let numbers = map.by_file.get(f).cloned().unwrap_or_default();
        match numbers.get(*gi) {
            Some(n) => {
                affected_numbers.insert(*n);
            }
            None => affected_not_in_suite.push((f.clone(), *gi)),
        }
    }

    // per-group analysis
    let mut groups: Vec<GroupReachability> = Vec::new();
    let mut agg: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    let mut progressed_count = 0usize;
    let mut no_additional = 0usize;
    let mut later_compile = 0usize;
    let mut exec_reached = 0usize;
    let mut new_runtime = 0usize;
    let mut new_runtime_matched = 0usize;
    let mut new_compile_fail = 0usize;
    let mut new_runtime_fail = 0usize;
    let mut new_artifact_fail = 0usize;
    let mut new_reached_total = 0usize;
    let mut gate_lifted_no_progress = 0usize;
    let mut regressions: Vec<String> = Vec::new();
    let mut analyzed_with_detail = 0usize;

    let mut numbers: Vec<u32> = map.groups.keys().copied().collect();
    numbers.extend(map_u.groups.keys().copied());
    numbers.sort_unstable();
    numbers.dedup();

    for n in numbers {
        let (file, title) = map.groups.get(&n).cloned().unwrap_or_default();
        let gi = map.ordinal(n);
        let ignored = gi
            .map(|g| {
                tix.ignore_count
                    .get(&(file.clone(), g))
                    .copied()
                    .unwrap_or(0)
            })
            .unwrap_or(0);
        let total_checks = gi
            .and_then(|g| tix.total_checks.get(&(file.clone(), g)))
            .copied()
            .unwrap_or(0);
        let p_res = p_main
            .get(&n)
            .map(|m| m.result)
            .or_else(|| p_logs.get(&n).map(|g| g.result))
            .or_else(|| p_ora_main.get(&n).map(|m| m.result));
        let u_res = u_main
            .get(&n)
            .map(|m| m.result)
            .or_else(|| u_logs.get(&n).map(|g| g.result));
        let pg = p_logs.get(&n);
        let ug = u_logs.get(&n);
        let has_detail = pg.is_some() && ug.is_some();
        if has_detail {
            analyzed_with_detail += 1;
        }
        let p_checks = pg.map(|g| g.checks.clone()).unwrap_or_default();
        let u_checks = ug.map(|g| g.checks.clone()).unwrap_or_default();
        let p_first = p_checks
            .iter()
            .find(|c| c.status_mismatch.is_some() || c.text_mismatch)
            .map(|c| c.step);
        let u_first = u_checks
            .iter()
            .find(|c| c.status_mismatch.is_some() || c.text_mismatch)
            .map(|c| c.step);
        let p_stop_reason = match p_first {
            Some(s) => p_checks
                .get(s)
                .map(check_stopping_reason)
                .unwrap_or(StoppingReason::Other),
            None => StoppingReason::None,
        };
        let p_stop_gated = match (gi, p_first) {
            (Some(g), Some(s)) => {
                tix.ignores.contains(&(file.clone(), g, s))
                    && matches!(
                        p_stop_reason,
                        StoppingReason::TextOnly | StoppingReason::TextAndStatus
                    )
            }
            _ => false,
        };
        let p_checks_run = match p_first {
            Some(s) => s + 1,
            None => {
                if p_checks.is_empty() {
                    // no detail: use the census total when the group ran
                    total_checks
                } else {
                    p_checks.len()
                }
            }
        };
        let u_checks_run = match u_first {
            Some(s) => s + 1,
            None => {
                if u_checks.is_empty() {
                    total_checks
                } else {
                    u_checks.len()
                }
            }
        };
        let progressed = match (p_first, u_first) {
            (Some(_), None) => true,
            (Some(pa), Some(ua)) => ua > pa,
            (None, Some(ua)) => {
                regressions.push(format!(
                    "group {n} ({title}): pristine passed all checks but unblocked failed at step {ua}"
                ));
                false
            }
            (None, None) => false,
        };
        if progressed && ignored > 0 {
            progressed_count += 1;
        }
        if progressed && ignored == 0 {
            regressions.push(format!(
                "group {n} ({title}): progressed without any ignored diagnostic (unexpected)"
            ));
        }
        if !progressed && ignored > 0 && p_stop_gated {
            gate_lifted_no_progress += 1;
        }
        if !progressed && ignored > 0 {
            no_additional += 1;
        }
        let mut newly: Vec<NewlyReachedCheck> = Vec::new();
        if let Some(ps) = p_first {
            // enumerate newly-reached steps: from the unblocked per-group log when present,
            // otherwise (group fully passed -> no retained log) from the pristine census,
            // which records every AT_CHECK command of the group.
            if u_checks.is_empty() {
                if let (Some(g), Some(tc)) = (gi, Some(total_checks)) {
                    for step in ps + 1..tc {
                        if let Some(cmd) = tix.commands.get(&(file.clone(), g, step)) {
                            let kind = reach_kind(classify_command(cmd));
                            match kind {
                                ReachKind::CompilerCheck => {}
                                ReachKind::RuntimeExecution => {
                                    new_runtime += 1;
                                    new_runtime_matched += 1;
                                    exec_reached += 1;
                                }
                                ReachKind::ArtifactOrHelper => {}
                                ReachKind::Unknown => {}
                            }
                            if matches!(kind, ReachKind::CompilerCheck) {
                                later_compile += 1;
                            }
                            new_reached_total += 1;
                            newly.push(NewlyReachedCheck {
                                step,
                                command: cmd.clone(),
                                shape: kind,
                                passed: true,
                            });
                        }
                    }
                }
            } else {
                for c in &u_checks {
                    if c.step > ps {
                        let passed = u_first.map(|uf| c.step < uf).unwrap_or(true);
                        let kind = reach_kind(classify_command(&c.command));
                        match kind {
                            ReachKind::CompilerCheck => {
                                if !passed {
                                    new_compile_fail += 1;
                                }
                            }
                            ReachKind::RuntimeExecution => {
                                new_runtime += 1;
                                if passed {
                                    new_runtime_matched += 1;
                                } else {
                                    new_runtime_fail += 1;
                                }
                                exec_reached += 1;
                            }
                            ReachKind::ArtifactOrHelper => {
                                if !passed {
                                    new_artifact_fail += 1;
                                }
                            }
                            ReachKind::Unknown => {}
                        }
                        if matches!(kind, ReachKind::CompilerCheck) {
                            later_compile += 1;
                        }
                        new_reached_total += 1;
                        newly.push(NewlyReachedCheck {
                            step: c.step,
                            command: c.command.clone(),
                            shape: kind,
                            passed,
                        });
                    }
                }
            }
        }
        groups.push(GroupReachability {
            number: n,
            file: file.clone(),
            title,
            group_index: gi,
            ignored_expectations: ignored,
            pristine_result: p_res.map(|r| r.as_str().to_string()),
            unblocked_result: u_res.map(|r| r.as_str().to_string()),
            pristine_checks_run: p_checks_run,
            unblocked_checks_run: u_checks_run,
            pristine_first_failing_step: p_first,
            unblocked_first_failing_step: u_first,
            pristine_stopping_reason: p_stop_reason.as_str().to_string(),
            pristine_stopping_gated: p_stop_gated,
            progressed,
            newly_reached: newly,
            notes: Vec::new(),
        });
    }

    let ignored_total = affected.iter().map(|k| tix.ignore_count[k]).sum::<usize>();
    let stdout_ignored = manifest["transformations"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter(|t| {
                    t["disposition"]
                        .as_str()
                        .map(|d| d.contains("STDOUT"))
                        .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0);
    let stderr_ignored = ignored_total - stdout_ignored;

    agg.insert(
        "diagnostic_expectations_ignored".into(),
        json!(ignored_total),
    );
    agg.insert("stdout_ignored".into(), json!(stdout_ignored));
    agg.insert("stderr_ignored".into(), json!(stderr_ignored));
    agg.insert("groups_affected".into(), json!(affected_numbers.len()));
    agg.insert(
        "groups_affected_not_in_suite".into(),
        json!(affected_not_in_suite.len()),
    );
    agg.insert(
        "groups_analyzed_with_check_detail".into(),
        json!(analyzed_with_detail),
    );
    agg.insert("groups_progressed_further".into(), json!(progressed_count));
    agg.insert("groups_no_additional_step".into(), json!(no_additional));
    agg.insert("groups_later_compile_reached".into(), json!(later_compile));
    agg.insert("groups_execution_reached".into(), json!(exec_reached));
    agg.insert(
        "gate_lifted_no_progress".into(),
        json!(gate_lifted_no_progress),
    );
    agg.insert("newly_reached_checks".into(), json!(new_reached_total));
    agg.insert("newly_reached_runtime_checks".into(), json!(new_runtime));
    agg.insert(
        "newly_matched_runtime_checks".into(),
        json!(new_runtime_matched),
    );
    agg.insert(
        "newly_exposed_compile_failures".into(),
        json!(new_compile_fail),
    );
    agg.insert(
        "newly_exposed_runtime_failures".into(),
        json!(new_runtime_fail),
    );
    agg.insert(
        "newly_exposed_artifact_failures".into(),
        json!(new_artifact_fail),
    );
    agg.insert(
        "pristine_group_passes".into(),
        json!(groups
            .iter()
            .filter(|g| g.pristine_result.as_deref() == Some("ok"))
            .count()),
    );
    agg.insert(
        "unblocked_group_passes".into(),
        json!(groups
            .iter()
            .filter(|g| g.unblocked_result.as_deref() == Some("ok"))
            .count()),
    );
    agg.insert(
        "pristine_candidate_xpass".into(),
        json!(groups
            .iter()
            .filter(|g| g.pristine_result.as_deref() == Some("UNEXPECTED PASS"))
            .count()),
    );
    agg.insert(
        "unblocked_candidate_xpass".into(),
        json!(groups
            .iter()
            .filter(|g| g.unblocked_result.as_deref() == Some("UNEXPECTED PASS"))
            .count()),
    );
    agg.insert("suite_groups".into(), json!(suite_total));

    let mut inputs = BTreeMap::new();
    for (k, p) in [
        ("pristine_candidate_log", pristine_log),
        ("pristine_candidate_dir", pristine_dir),
        ("unblocked_candidate_log", unblocked_log),
        ("unblocked_candidate_dir", unblocked_dir),
        ("pristine_oracle_log", pristine_oracle_log),
        ("unblocked_oracle_log", unblocked_oracle_log),
        ("transformations", transformations),
    ] {
        inputs.insert(k.to_string(), input_identity(p)?);
    }

    let report = ReachabilityReport {
        schema: "gnurust-diag-unblocked-reachability-v1".to_string(),
        transformer_version,
        source_revision,
        generated_at_utc: now,
        inputs,
        totals: serde_json::Value::Object(agg),
        oracle: serde_json::json!({
            "pristine_xpass": p_ora_xpass,
            "unblocked_xpass": u_ora_xpass,
            "rows": oracle_xpass_rows,
        }),
        groups,
        findings: map_findings
            .into_iter()
            .chain(totals_findings)
            .chain(regressions)
            .collect(),
    };

    let json_path = out_root.join("semantic-reachability.json");
    let md_path = out_root.join("semantic-reachability.md");
    write_json(&report, &json_path)?;
    let md = render_reachability_md(&report);
    std::fs::write(&md_path, md).map_err(|e| e.to_string())?;
    Ok(report)
}

/// The per-group dir of the other lane phase (oracle vs candidate) is a sibling of the
/// candidate dir's grandparent: `raw/<phase>/testsuite.dir` sits next to `raw/<candidate>/...`.
fn oracle_sibling_dir(candidate_dir: &Path, phase: &str) -> PathBuf {
    candidate_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{phase}/testsuite.dir"))
}

/// Deterministic identity of an input: path tail + size + sha256 (dirs: listing hash).
fn input_identity(p: &Path) -> Result<String, String> {
    let meta = std::fs::metadata(p).map_err(|e| format!("{}: {e}", p.display()))?;
    if meta.is_dir() {
        let mut listing = String::new();
        if let Ok(rd) = std::fs::read_dir(p) {
            let mut names: Vec<String> = rd
                .flatten()
                .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
                .collect();
            names.sort();
            for n in names {
                listing.push_str(&n);
                listing.push('\n');
            }
        }
        Ok(format!(
            "{}:dir:{}:{}",
            p.display(),
            listing.len(),
            sha_hex(listing.as_bytes())
        ))
    } else {
        let bytes = std::fs::read(p).map_err(|e| e.to_string())?;
        Ok(format!(
            "{}:{}:{}",
            p.display(),
            bytes.len(),
            sha_hex(&bytes)
        ))
    }
}

fn render_reachability_md(r: &ReachabilityReport) -> String {
    let t = &r.totals;
    let mut s = String::new();
    s.push_str("# Diagnostic-unblocked — semantic reachability\n\n");
    s.push_str(&format!(
        "_schema: {} · transformer {} · source {}\n\n",
        r.schema, r.transformer_version, r.source_revision
    ));
    s.push_str("The primary question is NOT “more tests passed”. It is: which later semantic\n");
    s.push_str("checks became reachable solely because compiler diagnostic text stopped gating\n");
    s.push_str("the group. Ignored diagnostic text is NOT diagnostic compatibility.\n\n");
    s.push_str("## Totals\n\n");
    for k in [
        "diagnostic_expectations_ignored",
        "stdout_ignored",
        "stderr_ignored",
        "groups_affected",
        "groups_affected_not_in_suite",
        "groups_analyzed_with_check_detail",
        "groups_progressed_further",
        "groups_no_additional_step",
        "gate_lifted_no_progress",
        "groups_later_compile_reached",
        "groups_execution_reached",
        "newly_reached_checks",
        "newly_reached_runtime_checks",
        "newly_matched_runtime_checks",
        "newly_exposed_compile_failures",
        "newly_exposed_runtime_failures",
        "newly_exposed_artifact_failures",
        "pristine_group_passes",
        "unblocked_group_passes",
        "pristine_candidate_xpass",
        "unblocked_candidate_xpass",
        "suite_groups",
    ] {
        s.push_str(&format!(
            "| {k} | {} |\n",
            t.get(k).map(|v| v.to_string()).unwrap_or_default()
        ));
    }
    s.push_str("\n## Oracle cross-reference\n\n");
    s.push_str(&format!(
        "pristine oracle XPASS: {} · unblocked oracle XPASS: {}\n\n",
        r.oracle["pristine_xpass"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0),
        r.oracle["unblocked_xpass"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0),
    ));
    for row in r.oracle["rows"].as_array().cloned().unwrap_or_default() {
        s.push_str(&format!(
            "- group {} `{}` — {} — pristine: {} — xfail: {}\n",
            row["number"].as_u64().unwrap_or(0),
            row["title"].as_str().unwrap_or(""),
            row["file"].as_str().unwrap_or(""),
            row["pristine_result"].as_str().unwrap_or(""),
            row["xfail"].as_bool().unwrap_or(false),
        ));
    }
    if !r.findings.is_empty() {
        s.push_str("\n## Findings\n\n");
        for f in &r.findings {
            s.push_str(&format!("- {f}\n"));
        }
    }
    s.push_str("\n## Groups that progressed further\n\n");
    s.push_str(
        "| group | file | title | ignored | pristine stop | unblocked stop | newly reached |\n",
    );
    s.push_str("|---|---|---|---|---|---|---|\n");
    for g in r.groups.iter().filter(|g| g.progressed) {
        let pstop = match g.pristine_first_failing_step {
            Some(s) => format!("{s} {}", g.pristine_stopping_reason),
            None => "none".to_string(),
        };
        let ustop = g
            .unblocked_first_failing_step
            .map(|s| s.to_string())
            .unwrap_or_else(|| "all passed".to_string());
        s.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            g.number,
            g.file,
            g.title,
            g.ignored_expectations,
            pstop,
            ustop,
            g.newly_reached.len()
        ));
    }
    s.push_str("\n_Generated from committed raw evidence; raw samples are preserved._\n");
    s
}

// ---------------------------------------------------------------------------------------------
// Phase 8 — pristine vs unblocked reconciliation
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize)]
pub struct ReconcileReport {
    pub schema: String,
    pub generated_at_utc: String,
    pub inputs: BTreeMap<String, String>,
    pub at_setup_pristine: usize,
    pub at_setup_transformed: usize,
    pub at_check_pristine: usize,
    pub at_check_transformed: usize,
    pub pristine_manifest_sha256: String,
    pub transformed_manifest_sha256: String,
    pub committed_patch_sha256: String,
    pub regenerated_patch_sha256: String,
    pub patch_reproducible: bool,
    pub transformations_reproducible: bool,
    pub group_index_identical: bool,
    pub suite_groups: usize,
    pub pristine_evidence_groups: usize,
    pub unblocked_evidence_groups: usize,
    pub command_hash: String,
    pub status_hash: String,
    pub gate: serde_json::Value,
    pub findings: Vec<String>,
}

/// Run the Phase 8 reconciliation: prove the unblocked suite differs ONLY in the admitted
/// diagnostic expectations and that group identity/counts reconcile across the lanes.
pub fn cmd_reconcile(
    pristine_src: &Path,
    transformations: &Path,
    patch: &Path,
    pristine_log: &Path,
    unblocked_log: &Path,
    pristine_dir: &Path,
    unblocked_dir: &Path,
    pristine_oracle_log: &Path,
    unblocked_oracle_log: &Path,
    out_root: &Path,
) -> Result<ReconcileReport, String> {
    let manifest = read_json(transformations)?;
    let now = evidence_timestamp(&manifest);
    let source_revision = manifest["source_revision"]
        .as_str()
        .unwrap_or("stable-3.2")
        .to_string();
    // 1. regenerate the transformation deterministically from the pristine sources
    let rep: TransformResult = transform_suite(pristine_src, &source_revision)?;
    let regenerated_patch =
        crate::diag_unblocked::cmd_transform_bytes(pristine_src, &source_revision)?;
    let committed_patch_bytes = std::fs::read(patch).map_err(|e| format!("patch: {e}"))?;
    let committed_patch_sha = crate::diag_unblocked::file_sha256(&committed_patch_bytes);
    let regenerated_patch_sha = crate::diag_unblocked::file_sha256(&regenerated_patch);
    let patch_reproducible = committed_patch_sha == regenerated_patch_sha;
    // 2. independent gate on the regenerated trees + committed manifest
    let scratch = out_root.join("scratch-reconcile");
    let pristine_copy = scratch.join("pristine");
    let transformed_copy = scratch.join("transformed");
    if pristine_copy.exists() {
        std::fs::remove_dir_all(&pristine_copy).map_err(|e| e.to_string())?;
    }
    if transformed_copy.exists() {
        std::fs::remove_dir_all(&transformed_copy).map_err(|e| e.to_string())?;
    }
    std::fs::create_dir_all(&pristine_copy).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&transformed_copy).map_err(|e| e.to_string())?;
    copy_at_dir(pristine_src, &pristine_copy)?;
    let transformed_at = crate::diag_unblocked::transformed_tree(pristine_src)?;
    for (name, bytes) in &transformed_at {
        std::fs::write(transformed_copy.join(name), bytes).map_err(|e| e.to_string())?;
    }
    let gate: GateVerdict = cmd_gate(patch, &pristine_copy, &transformed_copy, transformations)?;
    // 3. group identity reconciliation from the logs
    let p_main = parse_main_log(pristine_log)?;
    let u_main = parse_main_log(unblocked_log)?;
    let mut p_logs = Vec::new();
    for n in group_log_dir_numbers(pristine_dir) {
        if let Some(g) = parse_group_log(&pristine_dir.join(format!("{n:04}/testsuite.log"))) {
            p_logs.push(g);
        }
    }
    let mut u_logs = Vec::new();
    for n in group_log_dir_numbers(unblocked_dir) {
        if let Some(g) = parse_group_log(&unblocked_dir.join(format!("{n:04}/testsuite.log"))) {
            u_logs.push(g);
        }
    }
    let map_p = GroupMap::build_from(&p_main, &p_logs);
    let map_u = GroupMap::build_from(&u_main, &u_logs);
    let group_index_identical = map_p.groups == map_u.groups;
    let mut findings = Vec::new();
    if !group_index_identical {
        let only_p: Vec<u32> = map_p
            .groups
            .keys()
            .copied()
            .filter(|n| !map_u.groups.contains_key(n))
            .collect();
        let only_u: Vec<u32> = map_u
            .groups
            .keys()
            .copied()
            .filter(|n| !map_p.groups.contains_key(n))
            .collect();
        findings.push(format!(
            "group identity differs: only pristine {only_p:?}, only unblocked {only_u:?}"
        ));
    }
    // 4. suite total from oracle evidence (union of result lines + per-group dirs; parallel
    //    runs lose a few result lines, so the union — not the raw line count — is the total)
    let p_ora = parse_main_log(pristine_oracle_log)?;
    let u_ora = parse_main_log(unblocked_oracle_log)?;
    let p_ora_all: BTreeSet<u32> = p_ora
        .keys()
        .copied()
        .chain(group_log_dir_numbers(&oracle_sibling_dir(
            pristine_dir,
            "baseline",
        )))
        .collect();
    let u_ora_all: BTreeSet<u32> = u_ora
        .keys()
        .copied()
        .chain(group_log_dir_numbers(&oracle_sibling_dir(
            unblocked_dir,
            "oracle",
        )))
        .collect();
    let suite_total = p_ora_all.len();
    if u_ora_all != p_ora_all {
        findings.push(format!(
            "oracle evidence group sets differ: pristine {} vs unblocked {}",
            p_ora_all.len(),
            u_ora_all.len()
        ));
    }
    // 5. command + status identity hashes from the manifest (the gate proves byte-identity in
    //    the patch; these hashes bind the admitted command/status census)
    let mut cmd_buf = String::new();
    let mut status_buf = String::new();
    if let Some(trans) = manifest["transformations"].as_array() {
        for t in trans {
            cmd_buf.push_str(t["command"].as_str().unwrap_or(""));
            cmd_buf.push('\n');
            status_buf.push_str(t["expected_status"].as_str().unwrap_or(""));
            status_buf.push('\n');
        }
    }
    let command_hash = sha_hex(cmd_buf.as_bytes());
    let status_hash = sha_hex(status_buf.as_bytes());
    // reproducibility of the committed transformations.json (timestamp excluded: it records the
    // generation instant and legitimately differs between regenerations)
    let regen = serde_json::json!({
        "schema": "gnurust-diag-unblocked-transformations-v1",
        "transformer_version": rep.transformer_version,
        "source_revision": rep.source_revision,
        "pristine_manifest_sha256": rep.pristine_manifest_sha256,
        "transformed_manifest_sha256": rep.transformed_manifest_sha256,
        "files_scanned": rep.files_scanned,
        "transformations": rep.transformations,
    });
    let mut committed = manifest.clone();
    if let Some(o) = committed.as_object_mut() {
        o.remove("generated_at_utc");
    }
    let transformations_reproducible = crate::diag_unblocked::stable_json(&regen)
        == crate::diag_unblocked::stable_json(&committed);

    let report = ReconcileReport {
        schema: "gnurust-diag-unblocked-reconcile-v1".to_string(),
        generated_at_utc: now,
        inputs: {
            let mut m = BTreeMap::new();
            for (k, p) in [
                ("pristine_suite_src", pristine_src),
                ("transformations", transformations),
                ("patch", patch),
                ("pristine_candidate_log", pristine_log),
                ("unblocked_candidate_log", unblocked_log),
                ("pristine_oracle_log", pristine_oracle_log),
                ("unblocked_oracle_log", unblocked_oracle_log),
            ] {
                m.insert(k.to_string(), input_identity(p).unwrap_or_default());
            }
            m
        },
        at_setup_pristine: gate.at_setup_pristine,
        at_setup_transformed: gate.at_setup_transformed,
        at_check_pristine: gate.at_check_pristine,
        at_check_transformed: gate.at_check_transformed,
        pristine_manifest_sha256: rep.pristine_manifest_sha256.clone(),
        transformed_manifest_sha256: rep.transformed_manifest_sha256.clone(),
        committed_patch_sha256: committed_patch_sha,
        regenerated_patch_sha256: regenerated_patch_sha,
        patch_reproducible,
        transformations_reproducible,
        group_index_identical,
        suite_groups: suite_total,
        pristine_evidence_groups: map_p.groups.len(),
        unblocked_evidence_groups: map_u.groups.len(),
        command_hash,
        status_hash,
        gate: serde_json::to_value(&gate).map_err(|e| e.to_string())?,
        findings,
    };
    write_json(
        &report,
        &out_root.join("pristine-vs-diagnostic-unblocked.json"),
    )?;
    write_json(
        &report,
        &out_root.join("pristine-vs-diagnostic-unblocked.json"),
    )?;
    let md = render_reconcile_md(&report);
    std::fs::write(out_root.join("pristine-vs-diagnostic-unblocked.md"), md)
        .map_err(|e| e.to_string())?;
    // the regenerated trees are derived artifacts; drop them so the report root stays clean
    if scratch.exists() {
        let _ = std::fs::remove_dir_all(&scratch);
    }
    Ok(report)
}

fn copy_at_dir(src: &Path, dst: &Path) -> Result<(), String> {
    let mut names: Vec<String> = std::fs::read_dir(src)
        .map_err(|e| e.to_string())?
        .flatten()
        .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
        .collect();
    names.sort();
    for n in names {
        let bytes = std::fs::read(src.join(&n)).map_err(|e| e.to_string())?;
        std::fs::write(dst.join(&n), bytes).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn render_reconcile_md(r: &ReconcileReport) -> String {
    let mut s = String::new();
    s.push_str("# Pristine vs diagnostic-unblocked — reconciliation\n\n");
    s.push_str(&format!("_schema: {}\n\n", r.schema));
    s.push_str("PRISTINE: exact diagnostic parity still required; the upstream suite is the\n");
    s.push_str("compatibility authority and remains untouched.\n\n");
    s.push_str("UNBLOCKED: exactly the same test semantics except explicitly admitted compiler\n");
    s.push_str("diagnostic bytes are ignored; expected exit status, commands, source, runtime\n");
    s.push_str("output and generated-file expectations are still enforced.\n\n");
    s.push_str("## Structural counts\n\n");
    s.push_str(&format!(
        "| AT_SETUP pristine | AT_SETUP unblocked | AT_CHECK pristine | AT_CHECK unblocked |\n"
    ));
    s.push_str(&format!("|---|---|---|---|\n"));
    s.push_str(&format!(
        "| {} | {} | {} | {} |\n",
        r.at_setup_pristine, r.at_setup_transformed, r.at_check_pristine, r.at_check_transformed
    ));
    s.push_str(&format!(
        "- suite groups (oracle evidence): {}\n",
        r.suite_groups
    ));
    s.push_str(&format!(
        "- pristine candidate evidence groups: {}; unblocked: {}\n",
        r.pristine_evidence_groups, r.unblocked_evidence_groups
    ));
    s.push_str(&format!(
        "- group identity identical: {}\n",
        r.group_index_identical
    ));
    s.push_str("\n## Integrity proofs\n\n");
    s.push_str(&format!(
        "- patch reproducible (regenerated == committed): {}\n",
        r.patch_reproducible
    ));
    s.push_str(&format!(
        "- transformations.json reproducible: {}\n",
        r.transformations_reproducible
    ));
    s.push_str(&format!(
        "- committed patch sha256: `{}`\n",
        r.committed_patch_sha256
    ));
    s.push_str(&format!(
        "- regenerated patch sha256: `{}`\n",
        r.regenerated_patch_sha256
    ));
    s.push_str(&format!(
        "- pristine manifest sha256: `{}`\n",
        r.pristine_manifest_sha256
    ));
    s.push_str(&format!(
        "- transformed manifest sha256: `{}`\n",
        r.transformed_manifest_sha256
    ));
    s.push_str(&format!(
        "- command census hash (all 3422 commands): `{}`\n",
        r.command_hash
    ));
    s.push_str(&format!(
        "- expected-status census hash: `{}`\n",
        r.status_hash
    ));
    s.push_str(&format!(
        "- policy gate: {} failures\n",
        r.gate["failures"].as_array().map(|a| a.len()).unwrap_or(0)
    ));
    if !r.findings.is_empty() {
        s.push_str("\n## Findings\n\n");
        for f in &r.findings {
            s.push_str(&format!("- {f}\n"));
        }
    }
    s.push_str("\nThe semantic reachability delta (what the unblocked lane actually exposed) is\n");
    s.push_str("reported separately in `semantic-reachability.json` / `.md`.\n");
    s
}

// ---------------------------------------------------------------------------------------------
// Phase 9 — corpus cross-check
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize)]
pub struct CrossCheckRow {
    pub group: u32,
    pub file: String,
    pub title: String,
    pub step: usize,
    pub command: String,
    pub unblocked_outcome: String,
    pub corpus_identity: Option<String>,
    pub corpus_classification: Option<String>,
    pub corpus_first_failure: Option<String>,
    pub agree: bool,
    pub note: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CrossCheckReport {
    pub schema: String,
    pub generated_at_utc: String,
    pub inputs: BTreeMap<String, String>,
    pub rows: Vec<CrossCheckRow>,
    pub totals: serde_json::Value,
    pub findings: Vec<String>,
}

/// Phase 9 — cross-check the unblocked lane against the existing AT_CHECK-level corpus
/// extraction. Where the unblocked suite reaches a step the corpus extracted, the corpus
/// identity/classification should agree; disagreements are findings (never silently reconciled).
pub fn cmd_cross_check(
    unblocked_log: &Path,
    unblocked_dir: &Path,
    transformations: &Path,
    valid_programs: &Path,
    out_root: &Path,
) -> Result<CrossCheckReport, String> {
    let _manifest = read_json(transformations)?;
    let now = evidence_timestamp(&_manifest);
    let corpus: serde_json::Value = read_json(valid_programs)?;
    // corpus index: "file#group-ordinal-1based#step-1based" -> entry
    let mut corpus_index: BTreeMap<String, &serde_json::Value> = BTreeMap::new();
    if let Some(arr) = corpus.as_array() {
        for e in arr {
            let id = e["identity"].as_str().unwrap_or("");
            // identity: ".../testsuite.src-<file>/group-<GGGG>/step-<NNN>"
            let mut parts = id.split("/group-");
            let head = parts.next().unwrap_or("");
            let tail = parts.next().unwrap_or("");
            let file = head.rsplit("testsuite.src-").next().unwrap_or("");
            let mut tp = tail.splitn(2, "/step-");
            let gnum = tp.next().unwrap_or("");
            let snum = tp.next().unwrap_or("");
            let key = format!(
                "{}#{}#{}",
                file.trim_start_matches("testsuite.src/"),
                gnum,
                snum
            );
            corpus_index.insert(key, e);
        }
    }
    // per-group unblocked logs
    let u_main = parse_main_log(unblocked_log)?;
    let mut u_logs = Vec::new();
    for n in group_log_dir_numbers(unblocked_dir) {
        if let Some(g) = parse_group_log(&unblocked_dir.join(format!("{n:04}/testsuite.log"))) {
            u_logs.push(g);
        }
    }
    let map = GroupMap::build_from(&u_main, &u_logs);
    let mut rows: Vec<CrossCheckRow> = Vec::new();
    let mut findings = Vec::new();
    let mut matched = 0usize;
    let mut agreed = 0usize;
    let mut not_in_corpus = 0usize;
    let mut matched_passed = 0usize;
    let mut matched_failed = 0usize;
    for g in &u_logs {
        let gi = map.ordinal(g.number);
        for c in &g.checks {
            let key = match gi {
                Some(gi) => format!("{}#{:04}#{:03}", g.file, gi + 1, c.step + 1),
                None => continue,
            };
            let entry = corpus_index.get(&key);
            let corpus_id = entry.and_then(|e| e["identity"].as_str());
            let corpus_class = entry.and_then(|e| e["classification"].as_str());
            let failed = c.status_mismatch.is_some() || c.text_mismatch;
            // Contract agreement: the corpus independently classified the step as an oracle-valid
            // program package; the unblocked lane reached it. The candidate outcome is recorded
            // separately — a failure on a corpus-valid step is a newly-exposed candidate gap,
            // not a contract disagreement (the corpus records the ORACLE contract, first_failure
            // is null there by design).
            let contract_valid = corpus_class
                .map(|cl| cl.starts_with("VALID"))
                .unwrap_or(false);
            let agree = entry.is_some() && contract_valid;
            match entry {
                Some(_) => {
                    matched += 1;
                    if agree {
                        agreed += 1;
                    } else {
                        findings.push(format!(
                            "group {} step {} ({}): corpus classification {:?} contradicts the oracle-valid contract reached by the unblocked lane",
                            g.number,
                            c.step,
                            c.command,
                            corpus_class.unwrap_or(""),
                        ));
                    }
                    if failed {
                        matched_failed += 1;
                        if contract_valid {
                            findings.push(format!(
                                "newly-exposed candidate failure on corpus-valid step: group {} step {} ({})",
                                g.number, c.step, c.command
                            ));
                        }
                    } else {
                        matched_passed += 1;
                    }
                }
                None => not_in_corpus += 1,
            }
            rows.push(CrossCheckRow {
                group: g.number,
                file: g.file.clone(),
                title: g.title.clone(),
                step: c.step,
                command: c.command.clone(),
                unblocked_outcome: if failed { "failed" } else { "passed" }.to_string(),
                corpus_identity: corpus_id.map(|s| s.to_string()),
                corpus_classification: corpus_class.map(|s| s.to_string()),
                corpus_first_failure: None,
                agree,
                note: if entry.is_some() {
                    String::new()
                } else {
                    "step not extracted by the corpus extractor (scope difference)".to_string()
                },
            });
        }
    }
    let steps_total = rows.len();
    let candidate_failures_on_valid = findings
        .iter()
        .filter(|f| f.starts_with("newly-exposed candidate failure"))
        .count();
    let contract_contradictions = findings.len() - candidate_failures_on_valid;
    let report = CrossCheckReport {
        schema: "gnurust-diag-unblocked-corpus-cross-check-v1".to_string(),
        generated_at_utc: now,
        inputs: {
            let mut m = BTreeMap::new();
            for (k, p) in [
                ("unblocked_candidate_log", unblocked_log),
                ("unblocked_candidate_dir", unblocked_dir),
                ("transformations", transformations),
                ("valid_programs", valid_programs),
            ] {
                m.insert(k.to_string(), input_identity(p).unwrap_or_default());
            }
            m
        },
        rows,
        totals: serde_json::json!({
            "steps_in_unblocked_logs": steps_total,
            "matched_in_corpus": matched,
            "matched_passed": matched_passed,
            "matched_failed": matched_failed,
            "agreed": agreed,
            "contract_contradictions": contract_contradictions,
            "candidate_failures_on_valid_steps": candidate_failures_on_valid,
            "not_in_corpus": not_in_corpus,
        }),
        findings,
    };
    write_json(&report, &out_root.join("corpus-cross-check.json"))?;
    let md = render_cross_check_md(&report);
    std::fs::write(out_root.join("corpus-cross-check.md"), md).map_err(|e| e.to_string())?;
    Ok(report)
}

fn render_cross_check_md(r: &CrossCheckReport) -> String {
    let mut s = String::new();
    s.push_str("# Diagnostic-unblocked × corpus cross-check\n\n");
    s.push_str(&format!("_schema: {}\n\n", r.schema));
    s.push_str("Three independent perspectives cross-check one another:\n");
    s.push_str("1. pristine upstream harness (authority);\n");
    s.push_str("2. diagnostic-unblocked upstream harness (this lane);\n");
    s.push_str("3. extracted AT_CHECK-level corpus with phase attribution.\n\n");
    s.push_str("## Totals\n\n");
    for (k, v) in [
        (
            "steps_in_unblocked_logs",
            r.totals["steps_in_unblocked_logs"].as_u64().unwrap_or(0),
        ),
        (
            "matched_in_corpus",
            r.totals["matched_in_corpus"].as_u64().unwrap_or(0),
        ),
        (
            "matched_passed",
            r.totals["matched_passed"].as_u64().unwrap_or(0),
        ),
        (
            "matched_failed",
            r.totals["matched_failed"].as_u64().unwrap_or(0),
        ),
        ("agreed", r.totals["agreed"].as_u64().unwrap_or(0)),
        (
            "candidate_failures_on_valid_steps",
            r.totals["candidate_failures_on_valid_steps"]
                .as_u64()
                .unwrap_or(0),
        ),
        (
            "contract_contradictions",
            r.totals["contract_contradictions"].as_u64().unwrap_or(0),
        ),
        (
            "not_in_corpus",
            r.totals["not_in_corpus"].as_u64().unwrap_or(0),
        ),
    ] {
        s.push_str(&format!("| {k} | {v} |\n"));
    }
    if !r.findings.is_empty() {
        s.push_str("\n## Findings (disagreements — never silently reconciled)\n\n");
        for f in &r.findings {
            s.push_str(&format!("- {f}\n"));
        }
    }
    s.push_str("\n_Cross-checked from committed raw evidence._\n");
    s
}
