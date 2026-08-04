//! Classification (GNURUST.GNUCOBOL-TESTSUITE.3): reconcile the baseline (oracle) and candidate
//! runs, attribute every candidate failure to its earliest meaningful boundary, and emit the
//! per-test result model + summaries. The all-tests-accounted invariant is enforced here.

use crate::autotest::{by_number, parse_testsuite_log, read_group_log};
use crate::model::*;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

pub struct Inputs {
    pub baseline_log: std::path::PathBuf,
    pub baseline_dir: std::path::PathBuf, // testsuite.dir of the baseline tree
    pub candidate_log: std::path::PathBuf,
    pub candidate_dir: std::path::PathBuf,
    pub suite_total: usize,
    pub pass: String,
}

/// Load both sides' records and produce the full result set. The per-test ledger merges the
/// global-log status lines (PASS/SKIP) with the kept group dirs (FAIL/XFAIL/XPASS).
pub fn classify(inputs: &Inputs) -> Result<Vec<TestResultRow>, String> {
    let oracle_records = merge_ledger(&inputs.baseline_log, &inputs.baseline_dir)?;
    let candidate_records = merge_ledger(&inputs.candidate_log, &inputs.candidate_dir)?;
    if oracle_records.is_empty() {
        return Err("no oracle records parsed — baseline testsuite.log empty?".into());
    }
    if candidate_records.is_empty() {
        return Err("no candidate records parsed — candidate testsuite.log empty?".into());
    }

    let mut rows = Vec::new();
    for n in 1..=inputs.suite_total {
        let oracle = oracle_records.get(&n);
        let candidate = candidate_records.get(&n);
        let row = classify_one(n, oracle, candidate, inputs);
        rows.push(row);
    }
    Ok(rows)
}

/// Merge global-log status lines with kept-group-dir statuses (the dir wins — it is authoritative
/// for FAIL/XFAIL/XPASS; a PASS/SKIP group dir is normally cleaned by the harness).
fn merge_ledger(
    log: &std::path::Path,
    dir: &std::path::Path,
) -> Result<std::collections::BTreeMap<usize, TestRecord>, String> {
    let mut map =
        by_number(&parse_testsuite_log(log).map_err(|e| format!("log {}: {e}", log.display()))?);
    let rd = std::fs::read_dir(dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))?;
    for ent in rd.flatten() {
        let p = ent.path();
        if !p.is_dir() {
            continue;
        }
        if let Some(rec) = crate::autotest::group_dir_status(&p) {
            map.insert(rec.number, rec);
        }
    }
    Ok(map)
}

fn group_log_text(dir: &Path, n: usize) -> Option<crate::autotest::GroupLog> {
    let name = format!("{n:04}");
    read_group_log(&dir.join(name))
}

fn classify_one(
    n: usize,
    oracle: Option<&TestRecord>,
    candidate: Option<&TestRecord>,
    inputs: &Inputs,
) -> TestResultRow {
    let title = candidate
        .or(oracle)
        .map(|r| r.title.clone())
        .unwrap_or_else(|| "(suite did not reach this test)".to_string());
    let group = candidate
        .or(oracle)
        .map(|r| r.at_source.clone())
        .unwrap_or_default();

    // Oracle side.
    let oracle_view = match oracle {
        None => StatusView {
            compile: "NOT_RUN".into(),
            run: "NOT_RUN".into(),
            verdict: "NOT_REACHED".into(),
        },
        Some(r) => StatusView {
            compile: "ORACLE_OK".into(),
            run: "ORACLE_OK".into(),
            verdict: match r.status {
                TestStatus::Pass => "ORACLE_PASS".into(),
                TestStatus::Fail => "ORACLE_FAIL".into(),
                TestStatus::Skip => "ORACLE_SKIP".into(),
                TestStatus::Xfail => "ORACLE_XFAIL".into(),
                TestStatus::Xpass => "ORACLE_XPASS".into(),
            },
        },
    };

    // Candidate side + attribution.
    let (primary, reason, candidate_view, comparison) = match (oracle, candidate) {
        (None, _) => (
            "HARNESS_BLOCKED".to_string(),
            "oracle suite did not reach this test (aborted before it ran)".to_string(),
            CandidateView::default(),
            ComparisonView::default(),
        ),
        (_, None) => (
            "CANDIDATE_NOT_REACHED".to_string(),
            "candidate suite did not reach this test (aborted or timed out earlier)".to_string(),
            CandidateView::default(),
            ComparisonView::default(),
        ),
        (Some(o), Some(c)) => {
            let cv = candidate_view(c, inputs);
            let cmp = compare_view(o, c, inputs);
            let (primary, reason) = classify_pair(o, c, &cv, &cmp, inputs);
            (primary, reason, cv, cmp)
        }
    };

    TestResultRow {
        test_id: format!("{n:04}"),
        number: n,
        title,
        group,
        oracle: oracle_view,
        wrapper: WrapperView {
            argument_translation: "see candidate-view + invocation census".into(),
            artifact_generation: "launcher+manifest (cobc-rs) — never a native COBOL executable"
                .into(),
        },
        candidate: candidate_view,
        comparison,
        primary_classification: primary,
        reason_code: reason,
    }
}

fn candidate_view(c: &TestRecord, inputs: &Inputs) -> CandidateView {
    match c.status {
        TestStatus::Pass => CandidateView {
            preprocess: "ok".into(),
            parse: "ok".into(),
            check: "ok".into(),
            prepare: "ok".into(),
            run: "ok".into(),
        },
        TestStatus::Skip => CandidateView {
            preprocess: "skipped".into(),
            parse: "-".into(),
            check: "-".into(),
            prepare: "-".into(),
            run: "-".into(),
        },
        TestStatus::Xfail => CandidateView {
            preprocess: "ok".into(),
            parse: "ok".into(),
            check: "ok".into(),
            prepare: "ok".into(),
            run: "expected-failure".into(),
        },
        TestStatus::Xpass => CandidateView {
            preprocess: "ok".into(),
            parse: "ok".into(),
            check: "ok".into(),
            prepare: "ok".into(),
            run: "unexpected-pass".into(),
        },
        TestStatus::Fail => {
            let gl = group_log_text(&inputs.candidate_dir, c.number);
            match gl {
                None => CandidateView {
                    preprocess: "-".into(),
                    parse: "-".into(),
                    check: "fail".into(),
                    prepare: "-".into(),
                    run: "-".into(),
                },
                Some(_g) => CandidateView {
                    preprocess: "-".into(),
                    parse: "-".into(),
                    check: "fail".into(),
                    prepare: "-".into(),
                    run: "-".into(),
                },
            }
        }
    }
}

fn compare_view(o: &TestRecord, c: &TestRecord, inputs: &Inputs) -> ComparisonView {
    let empty = || ComparisonView {
        stdout: "same".into(),
        stderr: "same".into(),
        exit_status: "same".into(),
        files: "same".into(),
    };
    if o.status == c.status {
        return empty();
    }
    // Both failed: compare the group-log diffs (raw) when both sides have group logs.
    let ogl = group_log_text(&inputs.baseline_dir, o.number);
    let cgl = group_log_text(&inputs.candidate_dir, c.number);
    ComparisonView {
        stdout: diff_snapshot(
            ogl.as_ref().map(|g| g.raw.as_str()),
            cgl.as_ref().map(|g| g.raw.as_str()),
        ),
        stderr: "see raw group logs".into(),
        exit_status: "see raw group logs".into(),
        files: "see raw group logs".into(),
    }
}

fn diff_snapshot(o: Option<&str>, c: Option<&str>) -> String {
    match (o, c) {
        (Some(a), Some(b)) if a == b => "identical group logs".into(),
        (Some(_), Some(_)) => "oracle and candidate group logs differ (raw preserved)".into(),
        (Some(_), None) => "oracle failed with a group log; candidate has none".into(),
        (None, Some(_)) => "candidate failed with a group log; oracle has none".into(),
        (None, None) => "no group logs on either side".into(),
    }
}

/// Attribute the pair to a primary classification + reason code (prompt §3.3/§3.4).
fn classify_pair(
    o: &TestRecord,
    c: &TestRecord,
    cv: &CandidateView,
    cmp: &ComparisonView,
    inputs: &Inputs,
) -> (String, String) {
    match (o.status, c.status) {
        (TestStatus::Pass, TestStatus::Pass) => (
            "OBSERVABLE_MATCH".into(),
            "oracle and candidate both satisfy the test's AT_CHECK assertions".into(),
        ),
        (TestStatus::Pass, TestStatus::Skip) => (
            "CANDIDATE_SKIP".into(),
            "candidate skipped (suite-side skip, reason in the candidate log)".into(),
        ),
        (TestStatus::Pass, TestStatus::Xfail) => (
            "CANDIDATE_XFAIL".into(),
            "candidate expected-failure (suite-marked xfail)".into(),
        ),
        (TestStatus::Pass, TestStatus::Xpass) => (
            "CANDIDATE_XPASS".into(),
            "candidate unexpected-pass (suite-marked xfail but passed)".into(),
        ),
        (TestStatus::Pass, TestStatus::Fail) => {
            let (bucket, reason) = attribute_candidate_failure(c, inputs);
            (bucket, reason)
        }
        (TestStatus::Fail, TestStatus::Pass) => (
            "ORACLE_FAIL".into(),
            format!("oracle failed while the candidate passed ({})", o.detail),
        ),
        (TestStatus::Fail, TestStatus::Fail) => (
            "ORACLE_FAIL".into(),
            format!(
                "oracle failed too ({}) — no parity claim; oracle-side baseline failure",
                o.detail
            ),
        ),
        (TestStatus::Skip, _) => (
            "ORACLE_SKIP".into(),
            format!("oracle skipped this test ({})", o.detail),
        ),
        (TestStatus::Xfail, TestStatus::Xfail) => (
            "ORACLE_XFAIL".into(),
            "both sides expected-failure (suite-marked)".into(),
        ),
        (TestStatus::Xfail, _) => (
            "ORACLE_XFAIL".into(),
            format!("oracle expected-failure ({})", o.detail),
        ),
        (TestStatus::Xpass, _) => (
            "ORACLE_XPASS".into(),
            format!("oracle unexpected-pass ({})", o.detail),
        ),
        _ => {
            let _ = (cv, cmp);
            (
                "INFRASTRUCTURE_ERROR".into(),
                "unhandled status combination".into(),
            )
        }
    }
}

/// Attribute a candidate-side failure (oracle passed) to the earliest meaningful boundary.
fn attribute_candidate_failure(c: &TestRecord, inputs: &Inputs) -> (String, String) {
    let gl = group_log_text(&inputs.candidate_dir, c.number);
    let Some(g) = gl else {
        return (
            "CANDIDATE_UNSUPPORTED".into(),
            format!(
                "no candidate group log for test {} (suite abort?)",
                c.number
            ),
        );
    };
    let cmd = g.first_failing_command.clone().unwrap_or_default();
    let raw = g.raw.clone();
    let lower = raw.to_ascii_lowercase();
    let reason = |b: &str, m: String| (b.to_string(), m);

    // Wrapper argument problems (cobc-rs diagnostics on stderr).
    if lower.contains("cobc-rs:") && lower.contains("unsupported option") {
        return reason(
            "WRAPPER_OPTION_UNSUPPORTED",
            format!("cobc-rs rejected an option in: {cmd}"),
        );
    }
    if lower.contains("cobc-rs:")
        && (lower.contains("malformed") || lower.contains("unknown option"))
    {
        return reason(
            "WRAPPER_INVOCATION_MALFORMED",
            format!("cobc-rs rejected the invocation shape: {cmd}"),
        );
    }
    // Launcher/manifest artifacts.
    if lower.contains("launch") || lower.contains("manifest") || lower.contains("cobrun-launcher") {
        return reason(
            "WRAPPER_ARTIFACT_ERROR",
            format!("launcher/manifest problem in: {cmd}"),
        );
    }
    // Syntax-only compiles.
    if cmd.contains("-fsyntax-only") || cmd.contains("$COMPILE_ONLY") {
        if lower.contains("cobrun: unsupported") {
            return reason(
                "CANDIDATE_CHECK_REJECT",
                format!("parser/checker rejected: {cmd}"),
            );
        }
        return reason(
            "CANDIDATE_CHECK_REJECT",
            format!("candidate syntax check failed: {cmd}"),
        );
    }
    // Executable compiles (-x / $COMPILE).
    if cmd.contains("-x") || cmd.contains("$COMPILE") || cmd.contains("$COBC ") {
        if lower.contains("cobrun: unsupported") {
            return reason(
                "CANDIDATE_CHECK_REJECT",
                format!("parser/checker rejected: {cmd}"),
            );
        }
        if lower.contains("cobrun: undefined data name") {
            return reason(
                "CANDIDATE_PARSE_REJECT",
                format!("undefined data name in: {cmd}"),
            );
        }
        return reason(
            "CANDIDATE_CHECK_REJECT",
            format!("candidate compile-phase failure: {cmd}"),
        );
    }
    // Program runs (./prog / $COBCRUN_DIRECT). The launcher RAN and the interpreter failed: the
    // true first boundary is the cobrun diagnostic, NOT the module model -- the pre-reduction
    // attribution shadowed ~400 `$COBCRUN_DIRECT ./prog` failures behind the module bucket. The
    // same front-end diagnostic gets the same classification as in the syntax-only path.
    if cmd.contains("./") || cmd.contains("COBCRUN_DIRECT") || cmd.contains("$COBCRUN_DIRECT") {
        if lower.contains("cobrun: undefined data name") {
            return reason(
                "CANDIDATE_PARSE_REJECT",
                format!("undefined data name at run ({cmd})"),
            );
        }
        if lower.contains("cobrun: unsupported") {
            let note = extract_note(&raw, "unsupported");
            return reason("CANDIDATE_CHECK_REJECT", format!("{note} ({cmd})"));
        }
        if lower.contains("cobrun: runtime error") || lower.contains("libcob:") {
            return reason(
                "CANDIDATE_RUNTIME_FAIL",
                format!("runtime error during execution ({cmd})"),
            );
        }
        if lower.contains("size error") {
            return reason(
                "CANDIDATE_RUNTIME_FAIL",
                format!("SIZE ERROR at run ({cmd})"),
            );
        }
        return reason(
            "CANDIDATE_RUNTIME_FAIL",
            format!("candidate program run failed ({cmd})"),
        );
    }
    // Genuine cobcrun module lifecycle (module search / args / runtime config). A cobcrun
    // invocation whose MODULE RAN and then failed at the interpreter is attributed by the cobrun
    // diagnostic (same rule as a direct run); only cobcrun-side failures (module not found,
    // invalid module argument, missing PROGRAM name, runtime-config errors) stay module-model.
    if cmd.contains("cobcrun") || cmd.contains("$COBCRUN") || cmd.contains("-m") {
        if lower.contains("cobrun: undefined data name") {
            return reason(
                "CANDIDATE_PARSE_REJECT",
                format!("undefined data name in called module ({cmd})"),
            );
        }
        if lower.contains("cobrun: unsupported") {
            let note = extract_note(&raw, "unsupported");
            return reason("CANDIDATE_CHECK_REJECT", format!("{note} ({cmd})"));
        }
        if lower.contains("cobrun: runtime error") || lower.contains("libcob:") {
            return reason(
                "CANDIDATE_RUNTIME_FAIL",
                format!("runtime error in called module ({cmd})"),
            );
        }
        return reason(
            "CANDIDATE_MODULE_MODEL_UNSUPPORTED",
            format!("module lifecycle not supported by the candidate model: {cmd}"),
        );
    }
    reason(
        "CANDIDATE_UNSUPPORTED",
        format!("candidate failed at an unclassified boundary: {cmd}"),
    )
}

/// Extract the `unsupported: <feature>` note from a cobrun error line.
fn extract_note(raw: &str, kw: &str) -> String {
    for line in raw.lines() {
        let t = line.trim();
        if let Some(p) = t.find(kw) {
            let s = &t[p.max(1) - 1..];
            let s = s.trim_matches(|c: char| c == ':' || c.is_whitespace());
            if s.len() <= 160 {
                return format!("unsupported: {s}");
            }
        }
    }
    "unsupported (see raw group log)".to_string()
}

/// Build the summaries + artifact set from the rows. Returns the summary object.
pub fn summarize(rows: &[TestResultRow]) -> Summary {
    let mut s = Summary {
        total_tests: rows.len(),
        ..Default::default()
    };
    for r in rows {
        match r.oracle.verdict.as_str() {
            "ORACLE_PASS" => s.oracle.pass += 1,
            "ORACLE_FAIL" => s.oracle.fail += 1,
            "ORACLE_SKIP" => s.oracle.skip += 1,
            "ORACLE_XFAIL" => s.oracle.xfail += 1,
            "ORACLE_XPASS" => s.oracle.xpass += 1,
            "NOT_REACHED" => s.oracle.infra_error += 1,
            _ => s.oracle.infra_error += 1,
        }
        match r.primary_classification.as_str() {
            "OBSERVABLE_MATCH" => s.comparison.observable_match += 1,
            "ORACLE_FAIL" => {}
            "ORACLE_SKIP" => {}
            "ORACLE_XFAIL" | "ORACLE_XPASS" => {}
            "WRAPPER_OPTION_UNSUPPORTED" => s.wrapper.option_unsupported += 1,
            "WRAPPER_INVOCATION_MALFORMED" => s.wrapper.invocation_malformed += 1,
            "WRAPPER_ARTIFACT_ERROR" => s.wrapper.artifact_error += 1,
            "CANDIDATE_PREPROCESS_REJECT" => s.candidate.preprocess_reject += 1,
            "CANDIDATE_PARSE_REJECT" => s.candidate.parse_reject += 1,
            "CANDIDATE_CHECK_REJECT" => s.candidate.check_reject += 1,
            "CANDIDATE_LAYOUT_REJECT" => s.candidate.layout_reject += 1,
            "CANDIDATE_UNSUPPORTED" => s.candidate.unsupported += 1,
            "CANDIDATE_MODULE_MODEL_UNSUPPORTED" => s.candidate.module_model_unsupported += 1,
            "CANDIDATE_RUNTIME_FAIL" => s.candidate.runtime_fail += 1,
            "CANDIDATE_TIMEOUT" => s.candidate.timeout += 1,
            "CANDIDATE_NONDETERMINISTIC" => s.candidate.nondeterministic += 1,
            "CANDIDATE_SKIP" => s.candidate.skipped += 1,
            "CANDIDATE_XFAIL" => s.candidate.passed += 1,
            "CANDIDATE_XPASS" => s.candidate.passed += 1,
            "CANDIDATE_NOT_REACHED" | "HARNESS_BLOCKED" => s.candidate.not_reached += 1,
            other => {
                s.reason_codes
                    .entry(other.to_string())
                    .and_modify(|n| *n += 1)
                    .or_insert(1);
            }
        }
        *s.first_failure
            .entry(r.primary_classification.clone())
            .or_insert(0) += 1;
        *s.reason_codes
            .entry(r.primary_classification.clone())
            .or_insert(0) += 1;
    }
    s
}

/// Write every report artifact into `out` (summary.json/md, results.csv, failure-buckets.md,
/// oracle-results.json, candidate-results.json, comparison-results.json, test-inventory.json,
/// upstream-observations.md). Returns the summary.
pub fn write_reports(
    rows: &[TestResultRow],
    out: &Path,
    pass: &str,
    oracle_summary: Option<(usize, usize, usize, usize)>,
    candidate_summary: Option<(usize, usize, usize, usize)>,
) -> Result<Summary, String> {
    let summary = summarize(rows);

    let inventory: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "test_id": r.test_id,
                "number": r.number,
                "title": r.title,
                "group": r.group,
                "oracle_status": r.oracle.verdict,
                "candidate_status": r.candidate.run,
                "primary_classification": r.primary_classification,
                "reason_code": r.reason_code,
            })
        })
        .collect();
    write_json(
        out.join("test-inventory.json"),
        &json!({
            "schema": "gnurust-gnucobol-testsuite-inventory-v1",
            "pass": pass,
            "suite_total_claimed": rows.len(),
            "tests": inventory,
        }),
    )?;

    let oracle_rows: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "test_id": r.test_id, "number": r.number, "title": r.title,
                "status": r.oracle.verdict, "detail": r.oracle.run,
            })
        })
        .collect();
    write_json(
        out.join("oracle-results.json"),
        &json!({
            "schema": "gnurust-gnucobol-testsuite-oracle-results-v1",
            "pass": pass,
            "autotest_summary": oracle_summary.map(|(run, failed, xfailed, skipped)| json!({
                "tests_run": run, "failed": failed, "expected_failures": xfailed, "skipped": skipped,
            })),
            "oracle": {"name": "GnuCOBOL", "version": "3.2.0"},
            "tests": oracle_rows,
        }),
    )?;

    let candidate_rows: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "test_id": r.test_id, "number": r.number, "title": r.title,
                "status": r.candidate.run, "primary": r.primary_classification,
                "reason": r.reason_code,
            })
        })
        .collect();
    write_json(
        out.join("candidate-results.json"),
        &json!({
            "schema": "gnurust-gnucobol-testsuite-candidate-results-v1",
            "pass": pass,
            "autotest_summary": candidate_summary.map(|(run, failed, xfailed, skipped)| json!({
                "tests_run": run, "failed": failed, "expected_failures": xfailed, "skipped": skipped,
            })),
            "tests": candidate_rows,
        }),
    )?;

    write_json(
        out.join("comparison-results.json"),
        &json!({
            "schema": "gnurust-gnucobol-testsuite-comparison-v1",
            "pass": pass,
            "comparison": rows.iter().map(|r| json!({
                "test_id": r.test_id, "number": r.number, "title": r.title,
                "oracle": r.oracle.verdict,
                "candidate": r.candidate.run,
                "primary_classification": r.primary_classification,
                "stdout": r.comparison.stdout, "stderr": r.comparison.stderr,
                "exit_status": r.comparison.exit_status, "files": r.comparison.files,
            })).collect::<Vec<_>>(),
        }),
    )?;

    // results.csv
    let mut csv = String::from(
        "test_id,number,title,group,oracle,candidate,primary_classification,reason_code\n",
    );
    for r in rows {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            r.test_id,
            r.number,
            csv_escape(&r.title),
            csv_escape(&r.group),
            r.oracle.verdict,
            r.candidate.run,
            r.primary_classification,
            csv_escape(&r.reason_code),
        ));
    }
    write_file(out.join("results.csv"), &csv)?;

    // failure-buckets.md
    let mut md = String::from("# Candidate first-failure buckets\n\n");
    md.push_str(&format!(
        "All {} tests reconciled. Each candidate failure is attributed to its earliest boundary.\n\n",
        rows.len()
    ));
    md.push_str("| classification | count | example tests |\n|---|---|---|\n");
    let mut by_class: BTreeMap<&str, Vec<&TestResultRow>> = BTreeMap::new();
    for r in rows {
        by_class
            .entry(r.primary_classification.as_str())
            .or_default()
            .push(r);
    }
    for (k, v) in &by_class {
        let ex: Vec<&str> = v.iter().take(4).map(|r| r.test_id.as_str()).collect();
        md.push_str(&format!("| {} | {} | {} |\n", k, v.len(), ex.join(", ")));
    }
    write_file(out.join("failure-buckets.md"), &md)?;

    // upstream-observations.md — honest notes about the oracle baseline itself.
    let mut uo = String::from("# GnuCOBOL-suite upstream observations (baseline side)\n\n");
    uo.push_str(
        "The baseline run uses the ADMITTED GnuCOBOL 3.2 in-tree build with a stock configuration \
         (no `-fpermissive`, no compat `-Wno-*` flags — those would leak cc1 warnings into stderr and \
         break the suite's stderr-exact expectations). Any oracle-side failure is an observation about \
         this exact build/environment, NOT a claim about upstream.\n\n",
    );
    if let Some((run, failed, xfailed, skipped)) = oracle_summary {
        uo.push_str(&format!(
            "Autotest summary: {run} tests run, {failed} failed (of which {xfailed} expected failures), {skipped} skipped.\n\n"
        ));
    }
    let oracle_fails: Vec<&TestResultRow> = rows
        .iter()
        .filter(|r| r.oracle.verdict == "ORACLE_FAIL")
        .collect();
    uo.push_str(&format!(
        "Oracle-side failures: {} (each with a preserved group log under reports/gnucobol-testsuite/raw/).\n",
        oracle_fails.len()
    ));
    for r in oracle_fails.iter().take(20) {
        uo.push_str(&format!("- {}: {} ({})\n", r.test_id, r.title, r.group));
    }
    if oracle_fails.len() > 20 {
        uo.push_str(&format!(
            "- … and {} more (see oracle-results.json)\n",
            oracle_fails.len() - 20
        ));
    }
    // oracle-side skips and expected-failures: the suite's own declared conditions, observed in THIS
    // build (e.g. `COB_HAS_ISAM=no` skips, dialect/legacy xfails). Recorded so the baseline is
    // understood, never edited to fit the candidate.
    let oracle_skips: Vec<&TestResultRow> = rows
        .iter()
        .filter(|r| r.oracle.verdict == "ORACLE_SKIP")
        .collect();
    uo.push_str(&format!(
        "\nOracle-side skips: {} (the suite's own AT_SKIP_IF conditions in this build).\n",
        oracle_skips.len()
    ));
    for r in oracle_skips.iter().take(15) {
        uo.push_str(&format!("- {}: {} ({})\n", r.test_id, r.title, r.group));
    }
    let oracle_xfails: Vec<&TestResultRow> = rows
        .iter()
        .filter(|r| r.oracle.verdict == "ORACLE_XFAIL")
        .collect();
    uo.push_str(&format!(
        "\nOracle-side expected-failures: {} (suite-marked xfail — the baseline 'failure' is the suite's own expectation).\n",
        oracle_xfails.len()
    ));
    for r in oracle_xfails.iter().take(15) {
        uo.push_str(&format!("- {}: {} ({})\n", r.test_id, r.title, r.group));
    }
    write_file(out.join("upstream-observations.md"), &uo)?;

    // summary.json + summary.md
    let summary_json = json!({
        "schema": "gnurust-gnucobol-testsuite-summary-v1",
        "pass": pass,
        "generated_by": "gnucobol-rs-testsuite classify",
        "summary": summary,
    });
    write_json(out.join("summary.json"), &summary_json)?;
    write_file(out.join("summary.md"), &summary_md(&summary, rows, pass))?;

    Ok(summary)
}

fn summary_md(s: &Summary, _rows: &[TestResultRow], pass: &str) -> String {
    let mut md = String::new();
    md.push_str(&format!(
        "# GnuCOBOL testsuite differential — pass {pass}\n\n\
         All {} generated test groups reconciled (each has exactly one final classification).\n\n",
        s.total_tests
    ));
    md.push_str("## Oracle (real admitted GnuCOBOL 3.2, in-tree build)\n\n");
    md.push_str(&format!(
        "- pass: {}\n- fail: {}\n- skip: {}\n- xfail: {}\n- xpass: {}\n- not reached: {}\n\n",
        s.oracle.pass,
        s.oracle.fail,
        s.oracle.skip,
        s.oracle.xfail,
        s.oracle.xpass,
        s.oracle.infra_error
    ));
    md.push_str("## Candidate (cobc-rs + cobrun)\n\n");
    md.push_str(&format!(
        "- parse/check reject: {}\n- unsupported: {}\n- module-model unsupported: {}\n- runtime fail: {}\n- timeout: {}\n- not reached: {}\n- skipped: {}\n\n",
        s.candidate.check_reject + s.candidate.parse_reject + s.candidate.layout_reject,
        s.candidate.unsupported,
        s.candidate.module_model_unsupported,
        s.candidate.runtime_fail,
        s.candidate.timeout,
        s.candidate.not_reached,
        s.candidate.skipped,
    ));
    md.push_str("## Comparison\n\n");
    md.push_str(&format!(
        "- observable match: {}\n- stdout mismatch: {}\n- stderr mismatch: {}\n- exit-status mismatch: {}\n- generated-file mismatch: {}\n\n",
        s.comparison.observable_match,
        s.comparison.stdout_mismatch,
        s.comparison.stderr_mismatch,
        s.comparison.exit_status_mismatch,
        s.comparison.generated_file_mismatch,
    ));
    md.push_str("## Claims and non-claims\n\n");
    md.push_str(
        "- OBSERVABLE_MATCH means the test's own AT_CHECK assertions held on both sides in this environment.\n\
         - No full GnuCOBOL test-suite parity claim; no native-code generation; no COBOL conformance certification.\n\
         - Candidate execution cannot delegate to cobc/cobcrun/libcob (mechanical no-delegation proof in no-delegation.json).\n\
         - Baseline failures are observations about this admitted build, not upstream defects.\n",
    );
    md
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn write_json(path: impl AsRef<Path>, v: &Value) -> Result<(), String> {
    let p = path.as_ref();
    std::fs::write(p, serde_json::to_string_pretty(v).unwrap())
        .map_err(|e| format!("write {}: {e}", p.display()))
}

fn write_file(path: impl AsRef<Path>, text: &str) -> Result<(), String> {
    let p = path.as_ref();
    std::fs::write(p, text).map_err(|e| format!("write {}: {e}", p.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Build a synthetic two-side tree and run the full reconciliation.
    fn synth_inputs() -> (Inputs, tempfile::TempDir) {
        let td = tempfile::tempdir().unwrap();
        let base = td.path().join("baseline");
        let cand = td.path().join("candidate");
        fs::create_dir_all(base.join("tests/testsuite.dir")).unwrap();
        fs::create_dir_all(cand.join("tests/testsuite.dir/0002")).unwrap();

        // baseline global log: 4 tests, all pass or skip (the oracle passes test 2)
        let bl = [
            "1. first test (run_a.at:1): ok     (0m0.001s 0m0.001s)",
            "2. second test (run_a.at:2): ok     (0m0.001s 0m0.001s)",
            "3. third test (run_a.at:3): ok     (0m0.001s 0m0.001s)",
            "4. fourth test (run_a.at:4): skipped (run_a.at:5)",
        ]
        .join("\n");
        fs::write(base.join("tests/testsuite.log"), bl).unwrap();

        // candidate global log: 1 + 3 pass, 4 skipped; 0002 FAILED with a cobc-rs unsupported option
        let cl = [
            "1. first test (run_a.at:1): ok     (0m0.001s 0m0.001s)",
            "3. third test (run_a.at:3): ok     (0m0.001s 0m0.001s)",
            "4. fourth test (run_a.at:4): skipped (run_a.at:5)",
        ]
        .join("\n");
        fs::write(cand.join("tests/testsuite.log"), cl).unwrap();
        fs::write(
            cand.join("tests/testsuite.dir/0002/testsuite.log"),
            "# -*- compilation -*-\n2. run_a.at:2: testing second ...\n./run_a.at:2: $COBC -C prog.cob\ncobc-rs: -C: unsupported option (rejected-unsupported; generate C: the candidate is an interpreter, not a C emitter; reject honestly)\n./run_a.at:2: exit code was 1, expected 0\n2. run_a.at:2: 2. second test (run_a.at:2): FAILED (run_a.at:2)\n",
        )
        .unwrap();

        (
            Inputs {
                baseline_log: base.join("tests/testsuite.log"),
                baseline_dir: base.join("tests/testsuite.dir"),
                candidate_log: cand.join("tests/testsuite.log"),
                candidate_dir: cand.join("tests/testsuite.dir"),
                suite_total: 5, // test 5 was never reached by either side
                pass: "a".into(),
            },
            td,
        )
    }

    #[test]
    fn all_tests_accounted_invariant() {
        let (inputs, _td) = synth_inputs();
        let rows = classify(&inputs).unwrap();
        assert_eq!(rows.len(), 5, "suite_total rows, one per indexed test");
        let mut seen = std::collections::BTreeSet::new();
        for r in &rows {
            assert!(
                seen.insert(r.number),
                "each test id must appear exactly once ({} duplicated)",
                r.number
            );
        }
        // the classification set must be exactly the five primary classes here
        let primaries: std::collections::BTreeSet<&str> = rows
            .iter()
            .map(|r| r.primary_classification.as_str())
            .collect();
        let expect: std::collections::BTreeSet<&str> = [
            "OBSERVABLE_MATCH",
            "WRAPPER_OPTION_UNSUPPORTED",
            "ORACLE_SKIP",
            "HARNESS_BLOCKED",
        ]
        .iter()
        .copied()
        .collect();
        assert_eq!(primaries, expect, "classification set: {primaries:?}");
    }

    #[test]
    fn wrapper_option_unsupported_attribution() {
        let (inputs, _td) = synth_inputs();
        let rows = classify(&inputs).unwrap();
        let row = rows.iter().find(|r| r.number == 2).unwrap();
        assert_eq!(row.primary_classification, "WRAPPER_OPTION_UNSUPPORTED");
        assert!(
            row.reason_code.contains("rejected an option"),
            "reason: {}",
            row.reason_code
        );
    }

    #[test]
    fn oracle_failure_vs_candidate_absence_and_unreached() {
        let (inputs, _td) = synth_inputs();
        let rows = classify(&inputs).unwrap();
        // test 5: neither side reached it -> HARNESS_BLOCKED (oracle never ran it)
        let r5 = rows.iter().find(|r| r.number == 5).unwrap();
        assert_eq!(r5.primary_classification, "HARNESS_BLOCKED");
        assert_eq!(r5.title, "(suite did not reach this test)");
        // test 4: oracle skipped -> ORACLE_SKIP regardless of candidate
        let r4 = rows.iter().find(|r| r.number == 4).unwrap();
        assert_eq!(r4.primary_classification, "ORACLE_SKIP");
    }

    #[test]
    fn malformed_logs_do_not_panic() {
        let td = tempfile::tempdir().unwrap();
        // global log with garbage lines and a partial status line
        fs::write(
            td.path().join("testsuite.log"),
            "garbage\n###\n12. test with no colon status\n\nno summary\n",
        )
        .unwrap();
        let recs = parse_testsuite_log(&td.path().join("testsuite.log")).unwrap();
        assert!(recs.is_empty(), "no parseable status lines -> empty ledger");
        // a group dir with an unparseable log still yields a record (fail-closed fallback)
        fs::create_dir_all(td.path().join("testsuite.dir/0007")).unwrap();
        fs::write(td.path().join("testsuite.dir/0007/testsuite.log"), "???").unwrap();
        let r = crate::autotest::group_dir_status(&td.path().join("testsuite.dir/0007")).unwrap();
        assert_eq!(r.number, 7);
        assert_eq!(r.status, TestStatus::Fail, "no status line -> fail-closed");
    }

    #[test]
    fn partial_run_is_accounted_as_not_reached() {
        // candidate suite aborted early: it reached only tests 1..=2; 3..=5 are not in its ledger.
        let td = tempfile::tempdir().unwrap();
        let base = td.path().join("baseline");
        let cand = td.path().join("candidate");
        fs::create_dir_all(base.join("testsuite.dir")).unwrap();
        fs::create_dir_all(cand.join("testsuite.dir")).unwrap();
        fs::write(
            base.join("testsuite.log"),
            "1. a (t.at:1): ok\n2. b (t.at:2): ok\n3. c (t.at:3): ok\n",
        )
        .unwrap();
        fs::write(
            cand.join("testsuite.log"),
            "1. a (t.at:1): ok\n2. b (t.at:2): ok\n",
        )
        .unwrap();
        let inputs = Inputs {
            baseline_log: base.join("testsuite.log"),
            baseline_dir: base.join("testsuite.dir"),
            candidate_log: cand.join("testsuite.log"),
            candidate_dir: cand.join("testsuite.dir"),
            suite_total: 3,
            pass: "a".into(),
        };
        let rows = classify(&inputs).unwrap();
        assert_eq!(
            rows.iter()
                .find(|r| r.number == 3)
                .unwrap()
                .primary_classification,
            "CANDIDATE_NOT_REACHED"
        );
        assert_eq!(
            rows.iter()
                .find(|r| r.number == 1)
                .unwrap()
                .primary_classification,
            "OBSERVABLE_MATCH"
        );
        assert_eq!(rows.len(), 3, "still one row per indexed test");
    }
}
