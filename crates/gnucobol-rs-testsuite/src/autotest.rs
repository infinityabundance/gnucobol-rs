//! Parsing of the generated GnuCOBOL Autotest `testsuite` artifacts:
//! `testsuite.log` (one status line per test group) and the per-group logs under
//! `testsuite.dir/<NN>/testsuite.log` (kept for failed groups; success groups are cleaned by the
//! harness itself). Everything here reads ONLY the raw suite output — no normalization before the
//! raw evidence is preserved.

use crate::model::{TestRecord, TestStatus};
use std::collections::BTreeMap;
use std::path::Path;

/// Parse `testsuite.log` into one [`TestRecord`] per test group. Lines not matching the status
/// shape (banners, summaries, the tail of the last failure transcript) are ignored. The record
/// order is the log order; consumers re-index by `number`.
pub fn parse_testsuite_log(path: &Path) -> Result<Vec<TestRecord>, String> {
    // Lossy read: a failing test may dump raw (non-UTF-8) bytes into the log; the status lines we
    // need are ASCII, so a lossy conversion never loses them (and the RAW bytes are preserved
    // separately in the evidence).
    let bytes = std::fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let text = String::from_utf8_lossy(&bytes);
    let mut out = Vec::new();
    for line in text.lines() {
        if let Some(rec) = parse_status_line(line) {
            out.push(rec);
        }
    }
    Ok(out)
}

/// Parse one `N. <title> (<src.at:LINE>): <msg>` status line.
pub fn parse_status_line(line: &str) -> Option<TestRecord> {
    // The title may itself contain parentheses; the AT source location is the LAST `(<file>.at:<n>)`
    // group before the `: ` status separator.
    let end_src = line.rfind("): ")?;
    let (head, msg) = line.split_at(end_src);
    let (number, rest) = head.split_once(". ")?;
    let number: usize = number.trim().parse().ok()?;
    // `rest` = `<title> (<src.at:LINE>`
    let open = rest.rfind('(')?;
    let title = rest[..open].trim().to_string();
    let at_source = rest[open + 1..].trim().to_string();
    if !at_source.contains(".at:") {
        return None;
    }
    let msg = msg[3..].to_string(); // after "): "
    let (status, detail, seconds) = parse_msg(&msg);
    Some(TestRecord {
        number,
        title,
        at_source,
        status,
        detail,
        seconds,
    })
}

fn parse_msg(msg: &str) -> (TestStatus, String, f64) {
    if let Some(rest) = msg.strip_prefix("ok") {
        // `ok (0.6s)` — timing in parens
        let secs = rest
            .trim_start()
            .strip_prefix('(')
            .and_then(|r| r.strip_suffix(')'))
            .and_then(|t| t.trim_end_matches('s').trim().parse::<f64>().ok())
            .unwrap_or(0.0);
        return (TestStatus::Pass, msg.to_string(), secs);
    }
    if msg.starts_with("FAILED") {
        return (TestStatus::Fail, msg.to_string(), 0.0);
    }
    if msg.starts_with("skipped") {
        return (TestStatus::Skip, msg.to_string(), 0.0);
    }
    if msg.starts_with("expected failure") {
        return (TestStatus::Xfail, msg.to_string(), 0.0);
    }
    if msg.starts_with("UNEXPECTED PASS") {
        return (TestStatus::Xpass, msg.to_string(), 0.0);
    }
    (TestStatus::Fail, msg.to_string(), 0.0)
}

/// Extract the numeric summary from the tail: `N tests were run,`, `M failed (K expected
/// failures).` and `S tests were skipped.` (the exact shapes written by Autotest 2.6x).
pub fn parse_summary(text: &str) -> Option<(usize, usize, usize, usize)> {
    let mut run = None;
    let mut failed = None;
    let mut xfailed = None;
    let mut skipped = None;
    for line in text.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("ERROR: ") {
            // "ERROR: 1275 tests were run,"
            if rest.contains("tests were run") {
                run = rest.split_whitespace().next().and_then(|n| n.parse().ok());
            }
        } else if let Some(rest) = t.strip_prefix("All ") {
            // "All 1346 tests passed" / "All N tests were skipped"
            if rest.contains("tests passed") {
                run = rest.split_whitespace().next().and_then(|n| n.parse().ok());
                failed = Some(0);
                skipped = Some(0);
            } else if rest.contains("tests were skipped") {
                run = rest.split_whitespace().next().and_then(|n| n.parse().ok());
                skipped = run;
            }
        }
        // "832 failed (31 expected failures).", "1 failed." and "7 tests were skipped."
        if let Some(rest) = t.strip_suffix(" expected failures).") {
            if let Some((f, x)) = rest.rsplit_once(" failed (") {
                failed = f.trim().parse().ok();
                xfailed = x.trim().parse().ok();
            }
        } else if let Some(rest) = t.strip_suffix(" failed.") {
            failed = rest.trim().parse().ok();
        } else if let Some(rest) = t.strip_suffix(" tests were skipped.") {
            skipped = rest.trim().parse().ok();
        }
    }
    match (run, failed) {
        (Some(r), Some(f)) => Some((r, f, xfailed.unwrap_or(0), skipped.unwrap_or(0))),
        _ => None,
    }
}

/// The parsed per-group failure log: the first failing AT_CHECK command, its source line, and the
/// full raw diff/transcript text (used for first-failure attribution).
#[derive(Debug, Clone, Default)]
pub struct GroupLog {
    /// The group's test number (dir name).
    pub number: usize,
    /// `file.at:LINE: $COMMAND` of the FIRST AT_CHECK whose outcome was not `ok`.
    pub first_failing_command: Option<String>,
    pub first_failing_source: Option<String>,
    /// The final status line (`NN. title (src): FAILED (...)`).
    pub status_line: Option<String>,
    /// Raw text of the whole group log.
    pub raw: String,
}

/// Read one failed group's log under `testsuite.dir/<NN>/testsuite.log`.
pub fn read_group_log(dir: &Path) -> Option<GroupLog> {
    let mut gl = GroupLog::default();
    if let Some(name) = dir.file_name().and_then(|n| n.to_str()) {
        gl.number = name.trim_start_matches('0').parse().unwrap_or(0);
    }
    let log = dir.join("testsuite.log");
    let bytes = std::fs::read(&log).ok()?;
    let text = String::from_utf8_lossy(&bytes).into_owned();
    gl.raw = text.clone();
    // status line: `NN. <title> (<src>): <FAILED|...>`
    for line in text.lines().rev() {
        if line.contains("): FAILED") || line.contains("): UNEXPECTED PASS") {
            gl.status_line = Some(line.to_string());
            break;
        }
    }
    // First failing AT_CHECK: the first `<file>:<line>: $cmd` line (possibly prefixed `./`) whose
    // following lines contain a diff or `exit code was`. We take the first command line after the
    // banner that is followed (within the next lines) by a diff block.
    let lines: Vec<&str> = text.lines().collect();
    // The failing AT_CHECK is the LAST command line before the FIRST diff / `exit code was` marker
    // (every check is followed by `stdout:`/`stderr:` labels, so only the diff marks the failure).
    let mut last_cmd: Option<(String, String)> = None;
    for line in &lines {
        if line.contains(": testing ...") {
            last_cmd = None; // banner: reset
            continue;
        }
        if line.starts_with("--- ") || line.contains("exit code was") {
            if let Some((src, cmd)) = last_cmd.clone() {
                gl.first_failing_command = Some(cmd);
                gl.first_failing_source = Some(src);
            }
            break;
        }
        if let Some(c) = check_command_line(line) {
            last_cmd = Some(c);
        }
    }
    Some(gl)
}

/// Match an AT_CHECK command transcript line: `./used_binaries.at:196: $COBCRUN prog`.
fn check_command_line(line: &str) -> Option<(String, String)> {
    let line = line.strip_prefix("./").unwrap_or(line);
    let (src, rest) = line.split_once(": ")?;
    if src.contains(".at:") {
        Some((src.to_string(), rest.to_string()))
    } else {
        None
    }
}

/// Index `records` by test number, in log order (later duplicates win — the log may repeat a
/// number only in abnormal cases; the summary invariants then catch it).
pub fn by_number(records: &[TestRecord]) -> BTreeMap<usize, TestRecord> {
    records.iter().cloned().map(|r| (r.number, r)).collect()
}

/// The suite's total test count as claimed by the generated `testsuite` script: the largest
/// group number in `at_help_all` (the authoritative inventory — the per-test `testsuite.log`
/// status lines cover only PASS/SKIP; FAIL/XFAIL groups keep only their `testsuite.dir/<NN>` dirs).
pub fn suite_total(testsuite_script: &str) -> Option<usize> {
    let start = testsuite_script.find("at_help_all=\"")?;
    let rest = &testsuite_script[start + "at_help_all=\"".len()..];
    let end = rest.find("\";")?;
    let mut max = 0usize;
    for part in rest[..end].split(';') {
        if let Ok(n) = part.trim().parse::<usize>() {
            max = max.max(n);
        }
    }
    if max > 0 {
        Some(max)
    } else {
        None
    }
}

/// Read one kept group dir (`testsuite.dir/<NN>/testsuite.log`) and derive its final status from
/// the trailing status line: `...: FAILED (...)` / `...: expected failure (...)` /
/// `...: UNEXPECTED PASS (...)`.
pub fn group_dir_status(dir: &Path) -> Option<TestRecord> {
    let name = dir.file_name().and_then(|n| n.to_str())?;
    let number = name.trim_start_matches('0').parse::<usize>().ok()?;
    let log = dir.join("testsuite.log");
    let text = String::from_utf8_lossy(&std::fs::read(&log).ok()?).into_owned();
    for line in text.lines().rev() {
        if let Some(rec) = parse_status_line(line) {
            return Some(rec);
        }
    }
    // Fallback: no parseable status line; the dir exists -> treat as a failure with no detail.
    Some(TestRecord {
        number,
        title: "(no status line in group log)".to_string(),
        at_source: String::new(),
        status: TestStatus::Fail,
        detail: "group dir kept but no status line parsed".to_string(),
        seconds: 0.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pass_line_with_timing() {
        let r =
            parse_status_line("1564. MULTIPLY DISPLAY basic (run_fundamental.at:1564): ok (0.42s)")
                .expect("parsed");
        assert_eq!(r.number, 1564);
        assert_eq!(r.title, "MULTIPLY DISPLAY basic");
        assert_eq!(r.at_source, "run_fundamental.at:1564");
        assert_eq!(r.status, TestStatus::Pass);
        assert!((r.seconds - 0.42).abs() < 1e-9);
    }

    #[test]
    fn parses_fail_line_with_parens_in_title() {
        let r = parse_status_line(
            "3. compiler outputs (general) (used_binaries.at:179): FAILED (used_binaries.at:196)",
        )
        .expect("parsed");
        assert_eq!(r.title, "compiler outputs (general)");
        assert_eq!(r.status, TestStatus::Fail);
    }

    #[test]
    fn parses_skip_and_xfail() {
        let s = parse_status_line("77. needs screen (run_misc.at:77): skipped (run_misc.at:78)")
            .unwrap();
        assert_eq!(s.status, TestStatus::Skip);
        let x =
            parse_status_line("10. old syntax (syn_misc.at:10): expected failure (syn_misc.at:11)")
                .unwrap();
        assert_eq!(x.status, TestStatus::Xfail);
        let xp =
            parse_status_line("12. now works (syn_misc.at:12): UNEXPECTED PASS (syn_misc.at:13)")
                .unwrap();
        assert_eq!(xp.status, TestStatus::Xpass);
    }

    #[test]
    fn ignores_banner_lines() {
        assert!(parse_status_line("## ---------------- ##").is_none());
        assert!(parse_status_line("ERROR: 1275 tests were run,").is_none());
    }

    #[test]
    fn parses_summary_block() {
        let text = "## ------------- ##\n## Test results. ##\n## ------------- ##\n\nERROR: 1275 tests were run,\n832 failed (31 expected failures).\n7 tests were skipped.\n";
        let (run, failed, xfailed, skipped) = parse_summary(text).unwrap();
        assert_eq!((run, failed, xfailed, skipped), (1275, 832, 31, 7));
    }

    #[test]
    fn parses_all_passed_summary() {
        let text = "All 1346 tests passed\n";
        let (run, failed, _, skipped) = parse_summary(text).unwrap();
        assert_eq!((run, failed, skipped), (1346, 0, 0));
    }

    #[test]
    fn group_log_finds_failing_check() {
        let dir = tempfile::tempdir().unwrap();
        let gd = dir.path().join("0003");
        std::fs::create_dir_all(&gd).unwrap();
        std::fs::write(
            gd.join("testsuite.log"),
            "# -*- compilation -*-\n3. used_binaries.at:179: testing ... \n./used_binaries.at:193: $COBC -C prog.cob\nstderr:\nstdout:\n./used_binaries.at:196: $COBCRUN prog\n--- /dev/null\t2026-01-01\n+++ stderr\n@@\n+Traceback\n./used_binaries.at:196: exit code was 1, expected 0\n3. used_binaries.at:179: 3. compiler outputs (general) (used_binaries.at:179): FAILED (used_binaries.at:196)\n",
        )
        .unwrap();
        let gl = read_group_log(&gd).expect("group log read");
        assert_eq!(gl.number, 3);
        assert_eq!(gl.first_failing_command.as_deref(), Some("$COBCRUN prog"));
        assert_eq!(
            gl.first_failing_source.as_deref(),
            Some("used_binaries.at:196")
        );
        assert!(gl.status_line.as_deref().unwrap().contains("FAILED"));
    }
}
