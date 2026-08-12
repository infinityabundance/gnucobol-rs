//! Parser/checker rejection census + census reconciliation (GNURUST.GNUCOBOL-TESTSUITE evidence).
//!
//! The census answers: of the groups whose FIRST failure is a candidate check/parse reject,
//! what phase rejects them and with which diagnostic?
//!
//! Counting-unit doctrine (machine-enforced; see `parser-census-reconciliation.md`):
//!   unique_test_groups   — distinct Autotest group IDs in the suite inventory, one final
//!                          classification each (1282 in the stable-3.2 ledger).
//!   first_failure_groups — groups whose primary classification is a candidate check/parse
//!                          reject (682 in the stable-3.2 ledger; the v2 ledger was 683, the
//!                          v3 ledger reports 682 after the 2026-08 determinism fixes).
//!   phase_observations   — one census row per first-failure group, each with exactly one
//!                          phase (== first_failure_groups; never more).
//!   unique_test_steps    — step-level (AT_CHECK) identities. The group-level ledger does not
//!                          decompose steps, so this count is NOT produced here and step rows
//!                          are NEVER labelled "tests".
//!
//! Every census row carries `counting_unit: "first_failure_group"`.
//!
//! Historical reconciliation: the original census (commit 8d8c499e8) reported 700 rows from an
//! intermediate pipeline pass; the final two-pass ledger (commit 2748a02d0) reports 683. The
//! Markdown/CSV/dependency-graph artifacts were never regenerated after that pass and remained
//! stale at the 700-era content. This module regenerates the whole family from the machine
//! ledger (`test-inventory.json`) + the raw candidate group logs, reproduces the ledger rows
//! exactly (validated), and documents the deltas. The v3 ledger (2026-08, after the
//! deterministic program-iteration and diagnostic-path-normalization fixes) reports 682: a net
//! of one group left the check/parse first-failure set (the sorted check-program iteration
//! consolidated the previously order-dependent diagnostics; the `cannot read <file>`
//! normalization removed the machine path from a group key).

use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub const CENSUS_SCHEMA: &str = "gnurust-gnucobol-testsuite-parser-reject-census-v3";

/// Historical v1 census provenance (the intermediate pass, superseded by the final ledger).
/// Preserved as a documented historical exhibit; the v1 artifact itself is committed at
/// `reports/gnucobol-testsuite/parser-reject-census.v1.json` (extracted from commit 8d8c499e8).
pub const V1_HISTORICAL: [(&str, &str); 4] = [
    ("commit", "8d8c499e8"),
    ("parser_or_check_rejects", "700"),
    ("phases", "checker 399 / data-layout 137 / grammar 102 / name-resolution 29 / semantic-check 33"),
    ("delta", "700 - 683 = 17 groups left the check/parse first-failure set (12 -> OBSERVABLE_MATCH, 5 -> CANDIDATE_RUNTIME_FAIL); none entered. The v2 -> v3 delta is 683 - 682 = 1 group (the 2026-08 deterministic program-iteration + diagnostic-path-normalization fixes consolidated the order-dependent diagnostics)."),
];

/// Ordered phase rules. This table is validated against the committed v2 census rows in the
/// regression tests: every v2 (diagnostic -> phase) pair must classify identically.
pub fn phase_of(diagnostic: &str) -> &'static str {
    if diagnostic.contains("undefined data name") {
        return "name-resolution";
    }
    if diagnostic.contains("not a numeric literal") {
        return "grammar";
    }
    if diagnostic.contains(": verb ") {
        return "grammar";
    }
    if diagnostic.contains("expected program name after PROGRAM-ID") {
        return "grammar";
    }
    if diagnostic.contains("PIC ") {
        return "data-layout";
    }
    if diagnostic.contains("unsupported level number") {
        return "data-layout";
    }
    if diagnostic.contains("OCCURS ") {
        return "data-layout";
    }
    if diagnostic.contains("unrecognized USAGE") {
        return "data-layout";
    }
    if diagnostic.contains("USAGE NATIONAL") {
        return "data-layout";
    }
    if diagnostic.contains("is not a declared file") {
        return "semantic-check";
    }
    if diagnostic.contains("condition: ") {
        return "semantic-check";
    }
    "checker"
}

/// Extract the candidate's primary diagnostic from a group's `testsuite.log`.
///
/// Autotest renders a failed AT_CHECK as a diff: the `+++ <...>stderr` marker introduces the
/// candidate's actual stderr, each line prefixed with `+`. The first such line (trimmed) is the
/// candidate's primary diagnostic; empty actual stderr yields `""`. When no stderr diff exists
/// (e.g. an AT_CHECK whose expected value is an expression, which Autotest renders as a bare
/// `stderr:` section), fall back to the first non-empty content line after a bare `stderr:`
/// marker, skipping command echoes (`./...:` lines).
pub fn extract_diagnostic(raw: &str) -> String {
    let mut in_stderr = false;
    for line in raw.lines() {
        if line.starts_with("+++ ") {
            in_stderr = line.contains("stderr");
            continue;
        }
        if line.starts_with("--- ") {
            in_stderr = false;
            continue;
        }
        if in_stderr && line.starts_with('+') && !line.starts_with("+++") {
            let d = line[1..].trim();
            if !d.is_empty() {
                return d.to_string();
            }
        }
    }
    // Fallback: a bare `stderr:` section (expression-valued AT_CHECK expectations). The first
    // non-empty content line decides: a command echo (`./...`), a diff marker, or a group-summary
    // line (`NNNN. ...`) means the section is empty (no diagnostic) -> "".
    let mut in_plain = false;
    for line in raw.lines() {
        if in_plain {
            let t = line.trim();
            if t.is_empty() {
                continue;
            }
            if t.starts_with("./") || t.starts_with("---") || t.starts_with("+++") {
                return String::new();
            }
            if t.chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                return String::new(); // group-summary line, not a diagnostic
            }
            return t.to_string();
        }
        if line.trim() == "stderr:" {
            in_plain = true;
        }
    }
    String::new()
}

fn group_log(raw_candidate: &Path, number: u64) -> Option<String> {
    let p = raw_candidate
        .join("testsuite.dir")
        .join(format!("{number:04}"))
        .join("testsuite.log");
    std::fs::read_to_string(p).ok()
}

#[derive(Debug)]
pub struct CensusRow {
    pub test_id: String,
    pub number: u64,
    pub title: String,
    pub group: String,
    pub classification: String,
    pub reason: String,
    pub diagnostic: String,
    pub phase: &'static str,
}

impl CensusRow {
    fn to_json(&self) -> Value {
        json!({
            "test_id": self.test_id,
            "number": self.number,
            "title": self.title,
            "group": self.group,
            "classification": self.classification,
            "reason": self.reason,
            "diagnostic": self.diagnostic,
            "phase": self.phase,
            "counting_unit": "first_failure_group",
        })
    }
}

#[derive(Debug)]
pub struct Invariants {
    pub unique_test_groups: usize,
    pub first_failure_groups: usize,
    pub phase_observations: usize,
    pub unique_test_steps: String,
    pub summary_check_plus_parse: Option<usize>,
    pub inventory_check_plus_parse: usize,
}

impl Invariants {
    fn to_json(&self) -> Value {
        json!({
            "unique_test_groups": self.unique_test_groups,
            "first_failure_groups": self.first_failure_groups,
            "phase_observations": self.phase_observations,
            "unique_test_steps": self.unique_test_steps,
            "summary_check_plus_parse": self.summary_check_plus_parse,
            "inventory_check_plus_parse": self.inventory_check_plus_parse,
            "check": {
                "first_failure_groups_eq_summary": self.summary_check_plus_parse
                    .map(|s| s == self.first_failure_groups)
                    .unwrap_or(true),
                "first_failure_groups_eq_inventory": self.inventory_check_plus_parse == self.first_failure_groups,
                "first_failure_groups_eq_phase_observations": self.phase_observations == self.first_failure_groups,
            },
        })
    }
}

/// Build the census from the machine ledger. Fails closed on any invariant violation.
pub fn build_census(
    inventory_path: &Path,
    raw_candidate: &Path,
    summary_path: Option<&Path>,
    v1_census_path: Option<&Path>,
) -> Result<(Vec<CensusRow>, Invariants, Value), String> {
    let inv: Value = serde_json::from_str(
        &std::fs::read_to_string(inventory_path)
            .map_err(|e| format!("inventory unreadable: {e}"))?,
    )
    .map_err(|e| format!("inventory malformed: {e}"))?;
    let tests = inv["tests"]
        .as_array()
        .ok_or("inventory has no tests array")?;
    let claimed = inv["suite_total_claimed"].as_u64().unwrap_or(0) as usize;
    // I1: group totals reconcile to the suite inventory.
    if tests.len() != claimed {
        return Err(format!(
            "I1 FAIL: inventory suite_total_claimed {claimed} != tests len {}",
            tests.len()
        ));
    }

    let mut rows = Vec::new();
    let mut ids = BTreeSet::new();
    let mut inventory_ff: BTreeSet<String> = BTreeSet::new();
    for t in tests {
        let cls = t["primary_classification"].as_str().unwrap_or("");
        let id = t["test_id"].as_str().unwrap_or("").to_string();
        let number = t["number"].as_u64().unwrap_or(0);
        if cls == "CANDIDATE_CHECK_REJECT" || cls == "CANDIDATE_PARSE_REJECT" {
            if !ids.insert(id.clone()) {
                return Err(format!(
                    "duplicate test-step identity in census input: {id}"
                ));
            }
            inventory_ff.insert(id.clone());
            let diagnostic = group_log(raw_candidate, number)
                .map(|g| extract_diagnostic(&g))
                .unwrap_or_default();
            rows.push(CensusRow {
                test_id: id,
                number,
                title: t["title"].as_str().unwrap_or("").to_string(),
                group: t["group"].as_str().unwrap_or("").to_string(),
                classification: cls.to_string(),
                reason: t["reason_code"].as_str().unwrap_or("").to_string(),
                diagnostic,
                phase: "".into(),
            });
        }
    }
    // I6: census ids == inventory check/parse ids (bidirectional).
    let census_ids: BTreeSet<String> = rows.iter().map(|r| r.test_id.clone()).collect();
    if census_ids != inventory_ff {
        return Err(format!(
            "I6 FAIL: census ids != inventory check/parse ids ({} vs {})",
            census_ids.len(),
            inventory_ff.len()
        ));
    }
    for r in &mut rows {
        r.phase = phase_of(&r.diagnostic);
    }

    let summary_check_plus_parse = summary_path
        .map(|p| {
            let s: Value = serde_json::from_str(
                &std::fs::read_to_string(p).map_err(|e| format!("summary unreadable: {e}"))?,
            )
            .map_err(|e| format!("summary malformed: {e}"))?;
            let ff = &s["summary"]["first_failure"];
            Ok::<usize, String>(
                ff["CANDIDATE_CHECK_REJECT"].as_u64().unwrap_or(0) as usize
                    + ff["CANDIDATE_PARSE_REJECT"].as_u64().unwrap_or(0) as usize,
            )
        })
        .transpose()?;

    let phases: BTreeMap<&str, usize> = {
        let mut m = BTreeMap::new();
        for r in &rows {
            *m.entry(r.phase).or_insert(0) += 1;
        }
        m
    };
    // I4: phase observations sum to the group count.
    let phase_observations: usize = phases.values().sum();
    if phase_observations != rows.len() {
        return Err(format!(
            "I4 FAIL: phase observations {phase_observations} != census rows {}",
            rows.len()
        ));
    }
    // I2/I3 already enforced during row collection (classification filter + unique ids).

    let by_diagnostic: BTreeMap<&str, usize> = {
        let mut m = BTreeMap::new();
        for r in &rows {
            *m.entry(r.diagnostic.as_str()).or_insert(0) += 1;
        }
        m
    };

    let inv_block = Invariants {
        unique_test_groups: tests.len(),
        first_failure_groups: rows.len(),
        phase_observations,
        unique_test_steps: "not-decomposed-at-group-level (step decomposition is a separate phase)"
            .into(),
        summary_check_plus_parse: summary_check_plus_parse,
        inventory_check_plus_parse: inventory_ff.len(),
    };
    // I5: first-failure groups == summary first-failure count.
    if let Some(s) = summary_check_plus_parse {
        if s != rows.len() {
            return Err(format!(
                "I5 FAIL: summary check+parse first-failure {s} != census rows {}",
                rows.len()
            ));
        }
    }

    let v1_delta = v1_census_path.and_then(|p| {
        let raw = std::fs::read_to_string(p).ok()?;
        let v1: Value = serde_json::from_str(&raw).ok()?;
        let v1_ids: BTreeSet<String> = v1["tests"]
            .as_array()?
            .iter()
            .filter_map(|t| t["test_id"].as_str().map(String::from))
            .collect();
        let only_v1: Vec<String> = v1_ids.difference(&census_ids).cloned().collect();
        let only_v2: Vec<String> = census_ids.difference(&v1_ids).cloned().collect();
        Some(json!({
            "v1_rows": v1_ids.len(),
            "v1_only_ids": only_v1,
            "v2_only_ids": only_v2,
            "delta_v1_minus_v2": v1_ids.len() as i64 - census_ids.len() as i64,
        }))
    });

    let census_json = json!({
        "schema": CENSUS_SCHEMA,
        "parser_or_check_rejects": rows.len(),
        "counting_unit_doctrine": {
            "unique_test_groups": "distinct Autotest group IDs in the suite inventory",
            "first_failure_groups": "groups whose primary classification is a candidate check/parse reject",
            "phase_observations": "one census row per first-failure group, each with exactly one phase",
            "unique_test_steps": "step-level (AT_CHECK) identities; not counted in the group-level ledger and never labelled tests",
        },
        "invariants": inv_block.to_json(),
        "historical": {
            "v1": {
                "commit": "8d8c499e8",
                "parser_or_check_rejects": 700,
                "phases": {"checker": 399, "data-layout": 137, "grammar": 102, "name-resolution": 29, "semantic-check": 33},
            },
            "delta": v1_delta,
        },
        "phases": phases.iter().map(|(k, v)| (k.to_string(), json!(v))).collect::<BTreeMap<_, _>>(),
        "by_diagnostic": by_diagnostic.iter().map(|(k, v)| (k.to_string(), json!(v))).collect::<BTreeMap<_, _>>(),
        "tests": rows.iter().map(|r| r.to_json()).collect::<Vec<_>>(),
    });

    Ok((rows, inv_block, census_json))
}

fn render_md(census: &Value) -> String {
    let total = census["parser_or_check_rejects"].as_u64().unwrap_or(0);
    let mut md = String::new();
    md.push_str(&format!(
        "# GnuCOBOL testsuite — parser/checker rejection census\n\n\
         **{total} first-failure groups** whose primary classification is a candidate check/parse reject, \
         decomposed by PHASE (checker/data-layout/grammar/name-resolution/semantic-check) and by \
         diagnostic. A construct rejected at run (the launcher ran) is attributed the same way as at \
         syntax-only (first-failure consistency). Counting unit: **first_failure_group** (one row per \
         unique group; step-level AT_CHECK identities are not counted here and are never labelled \
         \"tests\").\n\n"
    ));
    md.push_str("## Phases\n\n");
    for (k, v) in census["phases"]
        .as_object()
        .unwrap_or(&serde_json::Map::new())
    {
        md.push_str(&format!("- {k}: {}\n", v.as_u64().unwrap_or(0)));
    }
    md.push_str("\n## Top diagnostics\n\n");
    let mut diags: Vec<(&str, u64)> = census["by_diagnostic"]
        .as_object()
        .map(|m| {
            m.iter()
                .map(|(k, v)| (k.as_str(), v.as_u64().unwrap_or(0)))
                .collect()
        })
        .unwrap_or_default();
    diags.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    for (d, n) in diags.iter().take(40) {
        md.push_str(&format!("- {n}: `{}`\n", csv_escape(d)));
    }
    md.push_str("\n## Per-test ledger (counting_unit = first_failure_group)\n\n");
    md.push_str("| id | title | phase | diagnostic | classification |\n|---|---|---|---|---|\n");
    if let Some(arr) = census["tests"].as_array() {
        for r in arr {
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                r["test_id"].as_str().unwrap_or(""),
                csv_escape(r["title"].as_str().unwrap_or("")),
                r["phase"].as_str().unwrap_or(""),
                csv_escape(r["diagnostic"].as_str().unwrap_or("")),
                r["classification"].as_str().unwrap_or(""),
            ));
        }
    }
    md.push_str(&format!("\n{total} rows; generated by `gnucobol-rs-testsuite reject-census generate` — do not edit by hand.\n"));
    md
}

fn render_csv(census: &Value) -> String {
    let mut out = String::from("diagnostic,phase,count,example_test_id\n");
    // by (diagnostic, phase) pairs from the rows
    let mut pairs: BTreeMap<(&str, &str), (u64, String)> = BTreeMap::new();
    if let Some(arr) = census["tests"].as_array() {
        for r in arr {
            let d = r["diagnostic"].as_str().unwrap_or("");
            let p = r["phase"].as_str().unwrap_or("");
            let id = r["test_id"].as_str().unwrap_or("").to_string();
            let e = pairs.entry((d, p)).or_insert((0, id));
            e.0 += 1;
        }
    }
    for ((d, p), (n, ex)) in pairs {
        out.push_str(&format!("{},{},{},{}\n", csv_escape(d), p, n, ex));
    }
    out
}

fn render_graph(census: &Value) -> Value {
    // Machine-derived feature dependency graph: node per phase + per normalized diagnostic
    // (feature); edge feature -> phase (the phase whose rejection blocks the feature). The v1
    // hand-generated graph (commit 8d8c499e8) is superseded by this deterministic derivation.
    let mut nodes: BTreeMap<String, Value> = BTreeMap::new();
    let mut edges: Vec<Value> = Vec::new();
    if let Some(arr) = census["tests"].as_array() {
        for r in arr {
            let d = r["diagnostic"].as_str().unwrap_or("").to_string();
            let p = r["phase"].as_str().unwrap_or("").to_string();
            let feat = format!("feature::{d}");
            nodes
                .entry(p.clone())
                .or_insert_with(|| json!({"id": p, "kind": "phase"}));
            nodes
                .entry(feat.clone())
                .or_insert_with(|| json!({"id": feat, "kind": "feature", "label": d}));
            edges.push(json!({"source": feat, "target": p}));
        }
    }
    edges.sort_by(|a, b| {
        (
            a["source"].as_str().unwrap_or(""),
            a["target"].as_str().unwrap_or(""),
        )
            .cmp(&(
                b["source"].as_str().unwrap_or(""),
                b["target"].as_str().unwrap_or(""),
            ))
    });
    edges.dedup();
    json!({
        "schema": "gnurust-gnucobol-testsuite-parser-feature-dependency-graph-v2",
        "counting_unit": "first_failure_group",
        "nodes": nodes.into_values().collect::<Vec<_>>(),
        "edges": edges,
    })
}

fn render_reconciliation_json(census: &Value, inv: &Invariants) -> Value {
    json!({
        "schema": "gnurust-gnucobol-testsuite-parser-census-reconciliation-v1",
        "question": "the parser-reject census headline said ~700 while the suite summary reports 652 check + 31 parse = 683 first-failure groups; resolve exactly",
        "answer": "the census Markdown (and the feature-frequency CSV and dependency graph) were generated from the v1 census (commit 8d8c499e8, an intermediate pipeline pass, 700 rows) and never regenerated after the final two-pass ledger (commit 2748a02d0, 683 rows). The machine JSONs (summary.json, test-inventory.json, parser-reject-census.json) always agreed on 683; only the regenerated-document family was stale. 700 - 683 = 17 groups left the check/parse first-failure set (12 -> OBSERVABLE_MATCH, 5 -> CANDIDATE_RUNTIME_FAIL); none entered. The v3 ledger (2026-08, after the deterministic program-iteration and diagnostic-path-normalization fixes) reports 682: one group left the set (the sorted check-program iteration consolidated the previously order-dependent USE/IDENTIFICATION diagnostics).",
        "terminology": {
            "unique_test_groups": "distinct Autotest group IDs in the suite inventory",
            "first_failure_groups": "groups whose primary classification is a candidate check/parse reject",
            "phase_observations": "one census row per first-failure group, each with exactly one phase",
            "unique_test_steps": "step-level (AT_CHECK) identities; not counted in the group-level ledger and never labelled tests",
        },
        "invariants": inv.to_json(),
        "current": {
            "unique_test_groups": inv.unique_test_groups,
            "first_failure_groups": inv.first_failure_groups,
            "phase_observations": inv.phase_observations,
        },
        "historical_v1": {
            "commit": "8d8c499e8",
            "rows": 700,
            "phases": {"checker": 399, "data-layout": 137, "grammar": 102, "name-resolution": 29, "semantic-check": 33},
            "exhibit": "reports/gnucobol-testsuite/parser-reject-census.v1.json",
        },
        "historical_delta": census["historical"]["delta"],
        "verdict": "RECONCILED",
    })
}

fn render_reconciliation_md(census: &Value, inv: &Invariants) -> String {
    let v1 = &census["historical"]["v1"];
    let delta = &census["historical"]["delta"];
    let mut md = String::new();
    md.push_str(&format!(
        "# Parser-census reconciliation — 683 vs 700\n\n\
         ## Question\n\n\
         The suite summary reports **652 CANDIDATE_CHECK_REJECT + 31 CANDIDATE_PARSE_REJECT = \
         683 first-failure groups**, while the parser-reject census Markdown headline said **700 \
         candidate check/parse rejects**. Resolve exactly, mechanically.\n\n\
         ## Answer\n\n\
         The machine ledgers always agreed on **683**. The census **Markdown** (and the \
         feature-frequency CSV and the feature-dependency graph) were generated from the **v1** \
         census — an intermediate pipeline pass committed at `8d8c499e8` with **700** rows — and \
         were never regenerated when the final two-pass rerun committed the **v2** ledger at \
         `2748a02d0` with **683** rows. The v1 -> v2 delta is **700 - 683 = 17**: \
         **12** groups moved to `OBSERVABLE_MATCH` (front-end DISPLAY-literal / ROUNDED fixes) and \
         **5** to `CANDIDATE_RUNTIME_FAIL`; **none** entered the set. The stale Markdown was the \
         only source of the 700 figure.\n\
         \
         **v3 ledger (2026-08): 683 -> 682.** The deterministic program-iteration fix (the check \
         phase examines contained programs in sorted order, never HashMap-random order) and the \
         diagnostic path normalization (the `cannot read <file>` diagnostic no longer embeds the \
         machine corpus path) consolidated the previously order-dependent diagnostics: a net of \
         **one** group left the check/parse first-failure set. The counting-unit table below \
         carries the v3 values; the v1/v2 narrative above is preserved as history.\n\
         \
         ## Counting units (machine-enforced)\n\n\
         | term | meaning | current value |\n\
         |---|---|---|\n\
         | unique_test_groups | distinct Autotest group IDs in the suite inventory | {} |\n\
         | first_failure_groups | groups whose primary classification is a candidate check/parse reject | {} |\n\
         | phase_observations | one census row per first-failure group, each with exactly one phase | {} |\n\
         | unique_test_steps | step-level (AT_CHECK) identities; not counted in the group-level ledger, never labelled \"tests\" | not-decomposed-at-group-level |\n\n\
         ## Invariants\n\n\
         ```text\n\
         {} \n\
         ```\n\n\
         ## Historical v1\n\n\
         - commit: `{}`\n\
         - rows: {}\n\
         - phases: checker {} / data-layout {} / grammar {} / name-resolution {} / semantic-check {}\n\
         - exhibit: `reports/gnucobol-testsuite/parser-reject-census.v1.json` (preserved)\n\n\
         ## Historical v2 (the two-pass ledger, commit 2748a02d0)\n\n\
         - rows: 683\n\
         - the 700 -> 683 delta above; superseded by the v3 counting units.\n\
         ## Delta (machine-computed from the preserved v1 exhibit)\n\n\
         ```json\n\
         {}\n\
         ```\n\n\
         Generated by `gnucobol-rs-testsuite reject-census generate` — do not edit by hand.\n",
        inv.unique_test_groups,
        inv.first_failure_groups,
        inv.phase_observations,
        serde_json::to_string_pretty(&inv.to_json()["check"]).unwrap_or_default(),
        v1["commit"].as_str().unwrap_or(""),
        v1["parser_or_check_rejects"].as_u64().unwrap_or(0),
        v1["phases"]["checker"].as_u64().unwrap_or(0),
        v1["phases"]["data-layout"].as_u64().unwrap_or(0),
        v1["phases"]["grammar"].as_u64().unwrap_or(0),
        v1["phases"]["name-resolution"].as_u64().unwrap_or(0),
        v1["phases"]["semantic-check"].as_u64().unwrap_or(0),
        serde_json::to_string_pretty(delta).unwrap_or_default(),
    ));
    md
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn write(path: &Path, text: &str) -> Result<(), String> {
    std::fs::write(path, text).map_err(|e| format!("write {} failed: {e}", path.display()))
}

pub struct Outputs {
    pub json: Value,
    pub md: String,
    pub csv: String,
    pub graph: Value,
    pub reconciliation_json: Value,
    pub reconciliation_md: String,
}

impl Outputs {
    fn write_all(&self, out: &Path) -> Result<(), String> {
        write(
            &out.join("parser-reject-census.json"),
            &serde_json::to_string_pretty(&self.json).unwrap_or_default(),
        )?;
        write(&out.join("parser-reject-census.md"), &self.md)?;
        write(&out.join("parser-feature-frequency.csv"), &self.csv)?;
        write(
            &out.join("parser-feature-dependency-graph.json"),
            &serde_json::to_string_pretty(&self.graph).unwrap_or_default(),
        )?;
        write(
            &out.join("parser-census-reconciliation.json"),
            &serde_json::to_string_pretty(&self.reconciliation_json).unwrap_or_default(),
        )?;
        write(
            &out.join("parser-census-reconciliation.md"),
            &self.reconciliation_md,
        )?;
        Ok(())
    }
}

/// Regenerate the full parser-reject census family from the machine ledger.
pub fn generate(
    inventory_path: &Path,
    raw_candidate: &Path,
    summary_path: Option<&Path>,
    v1_census_path: Option<&Path>,
    out: &Path,
) -> Result<Outputs, String> {
    let (_, inv, census_json) =
        build_census(inventory_path, raw_candidate, summary_path, v1_census_path)?;
    let outputs = Outputs {
        json: census_json.clone(),
        md: render_md(&census_json),
        csv: render_csv(&census_json),
        graph: render_graph(&census_json),
        reconciliation_json: render_reconciliation_json(&census_json, &inv),
        reconciliation_md: render_reconciliation_md(&census_json, &inv),
    };
    outputs.write_all(out)?;
    Ok(outputs)
}

/// Freshness + reconciliation gate: regenerate in a temp dir and diff against the committed
/// artifacts; also enforce the invariants against the committed inventory + summary.
pub fn check(root: &Path) -> Result<Vec<String>, String> {
    let rep = root.join("reports/gnucobol-testsuite");
    let tmp = std::env::temp_dir().join(format!(
        "gnucobol-reject-census-check-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let inventory = rep.join("test-inventory.json");
    let summary = rep.join("summary.json");
    let raw = root.join("reports/gnucobol-testsuite/raw/candidate");
    let v1 = rep.join("parser-reject-census.v1.json");
    generate(
        &inventory,
        &raw,
        Some(&summary),
        v1.exists().then_some(v1.as_path()),
        &tmp,
    )?;
    let mut notes = Vec::new();
    for f in [
        "parser-reject-census.json",
        "parser-reject-census.md",
        "parser-feature-frequency.csv",
        "parser-feature-dependency-graph.json",
        "parser-census-reconciliation.json",
        "parser-census-reconciliation.md",
    ] {
        let fresh = std::fs::read(&tmp.join(f)).unwrap_or_default();
        let committed = std::fs::read(rep.join(f)).unwrap_or_default();
        if fresh != committed {
            notes.push(format!(
                "STALE: reports/gnucobol-testsuite/{f} != regeneration"
            ));
        }
    }
    let _ = std::fs::remove_dir_all(&tmp);
    Ok(notes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_rules_cover_every_committed_v2_row() {
        // The committed v2 census is the validation target: every (diagnostic -> phase) pair
        // must classify identically under the documented rule table.
        let p = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("reports/gnucobol-testsuite/parser-reject-census.json");
        if !p.exists() {
            return; // ledger not checked out; logic tests below still run
        }
        let v: Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        let mut mismatches = 0;
        let mut checked = 0;
        for t in v["tests"].as_array().unwrap() {
            let d = t["diagnostic"].as_str().unwrap_or("");
            let want = t["phase"].as_str().unwrap_or("");
            let got = phase_of(d);
            checked += 1;
            if got != want {
                mismatches += 1;
                if mismatches <= 5 {
                    eprintln!("phase mismatch: {d:?} -> got {got}, want {want}");
                }
            }
        }
        assert_eq!(
            mismatches, 0,
            "phase classifier diverges from the committed ledger ({checked} rows)"
        );
    }

    #[test]
    fn phase_rules_are_exact_for_representative_diagnostics() {
        assert_eq!(
            phase_of("cobrun: undefined data name: FILLER"),
            "name-resolution"
        );
        assert_eq!(
            phase_of("cobrun: unsupported: not a numeric literal: X"),
            "grammar"
        );
        assert_eq!(phase_of("cobc-rs: unsupported: verb USE"), "grammar");
        assert_eq!(
            phase_of("cobrun: unsupported: expected program name after PROGRAM-ID"),
            "grammar"
        );
        assert_eq!(
            phase_of("cobc-rs: unsupported: PIC BX: UnsupportedSymbol('X')"),
            "data-layout"
        );
        assert_eq!(
            phase_of("cobrun: unsupported: unsupported level number --"),
            "data-layout"
        );
        assert_eq!(
            phase_of("cobrun: unsupported: OCCURS count MAX-SUB is not an integer"),
            "data-layout"
        );
        assert_eq!(
            phase_of("cobc-rs: unsupported: unrecognized USAGE BIT"),
            "data-layout"
        );
        assert_eq!(
            phase_of("cobrun: unsupported: OPEN: `FILE-OPT` is not a declared file"),
            "semantic-check"
        );
        assert_eq!(
            phase_of("cobrun: unsupported: condition: missing left operand"),
            "semantic-check"
        );
        assert_eq!(
            phase_of("cobc-rs: unsupported: no PROCEDURE DIVISION"),
            "checker"
        );
        assert_eq!(phase_of(""), "checker");
    }

    #[test]
    fn extraction_takes_first_stderr_diff_line() {
        let log = "# -*- compilation -*-\n\
                  73. syn.at:25: testing X ...\n\
                  ./syn.at:30: $COMPILE_ONLY short.cob\n\
                  --- -\t2026-08-05 21:57:01 +0000\n\
                  +++ /work/.../stderr\t2026-08-05 21:57:01 +0000\n\
                  @@ -1,2 +1 @@\n\
                  -short.cob: error: invalid file base name\n\
                  +cobc-rs: unsupported: no PROCEDURE DIVISION\n\
                  \n\
                  73. syn.at:25: FAILED\n";
        assert_eq!(
            extract_diagnostic(log),
            "cobc-rs: unsupported: no PROCEDURE DIVISION"
        );
        // empty actual stderr -> ""
        let empty = "# -*- compilation -*-\n--- - x\n+++ ...stderr y\n@@ -1,2 +1 @@\n-prog.cob:9: warning\n";
        assert_eq!(extract_diagnostic(empty), "");
        // stdout diff lines are NOT diagnostics
        let out_only = "--- - x\n+++ ...stdout y\n@@ -1,2 +1,4 @@\n->hello\n+hello\n";
        assert_eq!(extract_diagnostic(out_only), "");
        // fallback: bare stderr: section (expression-valued AT_CHECK)
        let plain = "# -*- compilation -*-\n155. syn.at:598: testing X ...\n./syn.at:616: $COMPILE_ONLY prog.cob\nstderr:\ncobc-rs: unsupported: no PROCEDURE DIVISION\n./syn.at:617: $COMPILE_ONLY prog.cob 2>&1 | \\n$GREP \"...\"\n";
        assert_eq!(
            extract_diagnostic(plain),
            "cobc-rs: unsupported: no PROCEDURE DIVISION"
        );
    }

    #[test]
    fn duplicate_identities_fail_closed() {
        let dir = tempfile::tempdir().unwrap();
        let inv = dir.path().join("inventory.json");
        let raw = dir.path().join("raw");
        std::fs::create_dir_all(&raw).unwrap();
        std::fs::write(
            &inv,
            serde_json::to_string(&json!({
                "schema": "gnurust-gnucobol-testsuite-inventory-v1",
                "pass": "a",
                "suite_total_claimed": 2,
                "tests": [
                    {"test_id": "0001", "number": 1, "title": "t", "group": "g", "primary_classification": "CANDIDATE_CHECK_REJECT", "reason_code": "r"},
                    {"test_id": "0001", "number": 1, "title": "t", "group": "g", "primary_classification": "CANDIDATE_PARSE_REJECT", "reason_code": "r"},
                ],
            }))
            .unwrap(),
        )
        .unwrap();
        let err = build_census(&inv, &raw, None, None).unwrap_err();
        assert!(err.contains("duplicate test-step identity"), "{err}");
    }

    #[test]
    fn summary_reconciliation_invariant_holds_on_committed_ledger() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let inv = root.join("reports/gnucobol-testsuite/test-inventory.json");
        let sum = root.join("reports/gnucobol-testsuite/summary.json");
        if !inv.exists() || !sum.exists() {
            return;
        }
        let (_, inv_block, _) = build_census(
            &inv,
            &root.join("reports/gnucobol-testsuite/raw/candidate"),
            Some(&sum),
            None,
        )
        .unwrap();
        assert_eq!(inv_block.first_failure_groups, 682);
        assert_eq!(inv_block.summary_check_plus_parse, Some(682));
        assert_eq!(inv_block.unique_test_groups, 1282);
        assert_eq!(inv_block.phase_observations, 682);
    }
}
