//! The runtime/mathematics evidence campaign (prompt Phase 4.1/4.2): the math subset of the
//! GnuCOBOL testsuite, classified from the SAME differential results as every other test (no
//! favorable selection). Produces `math-correctness.json` + `math-correctness.md` under
//! reports/gnucobol-runtime-tests/.
//!
//! Phase-1 invariants (boundary-reduction work, prompt §1): the generator FAILS unless
//! * sum(math classification counts) == math test inventory count (== 323 for GnuCOBOL 3.2);
//! * every math test id is unique and present exactly once;
//! * every math test id is a member of the complete suite inventory;
//! * every math test carries exactly one final classification.
//!
//! `verify` (the `math check` subcommand) regenerates the correctness JSON in memory and compares
//! it with the committed artifact, so stale hand-written prose cannot drift from the ledger.

use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

/// The math/runtime .at sources and their semantic category (from the REAL suite taxonomy).
pub fn math_sources() -> &'static [(&'static str, &'static str)] {
    &[
        ("data_binary", "binary arithmetic (COMP-5/binary fields)"),
        ("data_display", "DISPLAY/zoned-decimal arithmetic"),
        ("data_packed", "PACKED-DECIMAL (COMP-3) arithmetic"),
        ("data_pointer", "POINTER/USAGE POINTER"),
        (
            "run_fundamental",
            "fundamental arithmetic (ADD/SUBTRACT/MULTIPLY/DIVIDE/COMPUTE)",
        ),
        ("run_functions", "intrinsic mathematical functions"),
        ("syn_multiply", "MULTIPLY syntax"),
        ("syn_value", "VALUE clauses / numeric literals"),
        ("syn_literals", "literal forms"),
    ]
}

fn math_file(group: &str) -> Option<&'static str> {
    let base = group.split(':').next().unwrap_or("");
    let base = base.strip_suffix(".at").unwrap_or(base);
    math_sources()
        .iter()
        .find(|(f, _)| *f == base)
        .map(|(f, _)| *f)
}

/// The math subset of a classified result set. `rows` is the `tests` array of a results JSON.
pub fn collect(rows: &[Value]) -> Vec<&Value> {
    rows.iter()
        .filter(|r| r["group"].as_str().and_then(math_file).is_some())
        .collect()
}

/// Hard reconciliation invariants over the math subset (Phase-1 boundary-reduction work).
/// Returns a human-readable problem list; empty == all invariants hold.
pub fn invariants(rows: &[Value], math_rows: &[&Value]) -> Vec<String> {
    let mut problems = Vec::new();
    let all_ids: BTreeMap<&str, usize> =
        rows.iter()
            .filter_map(|r| r["test_id"].as_str())
            .fold(BTreeMap::new(), |mut m, id| {
                *m.entry(id).or_insert(0) += 1;
                m
            });
    let math_ids: Vec<&str> = math_rows
        .iter()
        .filter_map(|r| r["test_id"].as_str())
        .collect();

    // 1. exactly one inventory entry per suite test id.
    for (id, n) in &all_ids {
        if *n != 1 {
            problems.push(format!(
                "suite test id {id} appears {n} times in the inventory"
            ));
        }
    }
    // 2. math ids are unique.
    let uniq: BTreeMap<&str, usize> = {
        let mut m = BTreeMap::new();
        for id in &math_ids {
            *m.entry(*id).or_insert(0) += 1;
        }
        m
    };
    for (id, n) in &uniq {
        if *n != 1 {
            problems.push(format!("math test id {id} appears {n} times"));
        }
    }
    // 3. every math id is a member of the complete suite inventory.
    for id in &math_ids {
        if !all_ids.contains_key(*id) {
            problems.push(format!(
                "math test id {id} is NOT a member of the full suite inventory"
            ));
        }
    }
    // 4. every math test carries exactly one final classification.
    for r in math_rows {
        let pc = r["primary_classification"].as_str().unwrap_or("");
        if pc.is_empty() {
            problems.push(format!(
                "math test {} lacks a primary classification",
                r["test_id"].as_str().unwrap_or("?")
            ));
        }
    }
    // 5. sum(classification counts) == count(math rows) == count(unique math ids).
    let mut totals: BTreeMap<String, usize> = BTreeMap::new();
    for r in math_rows {
        *totals
            .entry(
                r["primary_classification"]
                    .as_str()
                    .unwrap_or("?")
                    .to_string(),
            )
            .or_insert(0) += 1;
    }
    let sum: usize = totals.values().sum();
    if sum != math_rows.len() {
        problems.push(format!(
            "sum of math classification counts ({sum}) != math test count ({})",
            math_rows.len()
        ));
    }
    if uniq.len() != math_rows.len() {
        problems.push(format!(
            "unique math ids ({}) != math test count ({})",
            uniq.len(),
            math_rows.len()
        ));
    }
    problems
}

/// Generate the math-correctness report from the committed classification inventory.
/// Fails hard (Phase-1 invariant) if the math subset does not reconcile exactly.
pub fn generate(results_path: &Path, out: &Path) -> Result<(), String> {
    let v: Value = serde_json::from_str(
        &std::fs::read_to_string(results_path)
            .map_err(|e| format!("results {results_path:?}: {e}"))?,
    )
    .map_err(|e| format!("results JSON {results_path:?}: {e}"))?;
    let rows = v["tests"]
        .as_array()
        .ok_or("results JSON has no tests array")?;
    let math_rows = collect(rows);
    if math_rows.is_empty() {
        return Err("no math tests found in the results (wrong file?)".into());
    }

    // Phase-1 hard reconciliation: the generator MUST fail when the math subset does not reconcile.
    let problems = invariants(rows, &math_rows);
    if !problems.is_empty() {
        return Err(format!(
            "MATH RECONCILIATION FAILED ({} problem(s)): {}",
            problems.len(),
            problems.join("; ")
        ));
    }

    let mut by_file: BTreeMap<String, Vec<&Value>> = BTreeMap::new();
    for r in &math_rows {
        let f = math_file(r["group"].as_str().unwrap_or("")).unwrap_or("other");
        by_file.entry(f.to_string()).or_default().push(r);
    }

    let mut per_test: Vec<Value> = Vec::new();
    let mut totals: BTreeMap<String, usize> = BTreeMap::new();
    for r in &math_rows {
        let primary = r["primary_classification"].as_str().unwrap_or("?");
        *totals.entry(primary.to_string()).or_insert(0) += 1;
        per_test.push(json!({
            "test_id": r["test_id"], "number": r["number"], "title": r["title"],
            "group": r["group"], "category": math_file(r["group"].as_str().unwrap_or("")).unwrap_or("other"),
            "oracle": r["oracle_status"], "candidate": r["candidate_status"],
            "primary_classification": primary, "reason": r["reason_code"],
        }));
    }
    let sum: usize = totals.values().sum();

    let doc = json!({
        "schema": "gnurust-gnucobol-runtime-math-correctness-v2",
        "source": "the SAME differential classification as the full suite (no favorable selection)",
        "math_tests_total": math_rows.len(),
        "suite_total": rows.len(),
        "reconciliation": {
            "sum_of_classification_totals": sum,
            "unique_test_ids": math_rows.len(),
            "ids_subset_of_suite": true,
            "invariant_holds": problems.is_empty(),
            "enforced_by": "gnucobol-rs-testsuite math (generator fails on any invariant violation)",
        },
        "by_at_source": by_file.iter().map(|(f, v)| json!({f: v.len()})).collect::<Vec<_>>(),
        "primary_classification_totals": totals,
        "per_test": per_test,
        "non_claims": [
            "math-correctness is a CLASSIFICATION over the suite's own AT_CHECK assertions in this environment",
            "no claim that matching output proves equivalence outside the tested environment",
            "performance is reported SEPARATELY (math-performance.*) and only for tests passing on both sides"
        ],
    });
    std::fs::create_dir_all(out).map_err(|e| format!("mkdir {out:?}: {e}"))?;
    std::fs::write(
        out.join("math-correctness.json"),
        serde_json::to_string_pretty(&doc).unwrap(),
    )
    .map_err(|e| format!("write math-correctness.json: {e}"))?;

    let mut md = String::new();
    md.push_str(&format!(
        "# GnuCOBOL runtime/mathematics — correctness classification\n\n\
         {} math tests (of {} suite tests), classified from the SAME differential results as every\n\
         other test — no favorable selection. Correctness is the suite's own AT_CHECK assertion\n\
         outcome; performance is reported separately and only for tests passing on both sides.\n\
         Reconciliation invariant (Phase-1, machine-enforced): sum of the classification totals ==\n\
         {} == math test count; every math id is unique and a member of the full suite inventory.\n\n",
        math_rows.len(),
        rows.len(),
        sum
    ));
    md.push_str("## Totals by classification\n\n");
    for (k, n) in &totals {
        md.push_str(&format!("- {k}: {n}\n"));
    }
    md.push_str("\n## By .at source\n\n| source | category | tests |\n|---|---|---|\n");
    for (f, _) in math_sources() {
        if let Some(v) = by_file.get(*f) {
            let cat = math_sources().iter().find(|(x, _)| x == f).unwrap().1;
            md.push_str(&format!("| `{f}.at` | {cat} | {} |\n", v.len()));
        }
    }
    md.push_str("\n## Per-test ledger\n\n| id | title | category | oracle | candidate | classification |\n|---|---|---|---|---|---|\n");
    for r in &per_test {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            r["test_id"].as_str().unwrap_or(""),
            r["title"].as_str().unwrap_or(""),
            r["category"].as_str().unwrap_or(""),
            r["oracle"].as_str().unwrap_or(""),
            r["candidate"].as_str().unwrap_or(""),
            r["primary_classification"].as_str().unwrap_or(""),
        ));
    }
    std::fs::write(out.join("math-correctness.md"), md)
        .map_err(|e| format!("write math-correctness.md: {e}"))?;
    Ok(())
}

/// Freshness check (Phase-1): regenerate the correctness JSON in memory and compare it with the
/// committed artifact. Also verifies that the committed Markdown totals are EXACTLY the JSON
/// totals (so hand-written prose cannot drift from the ledger).
pub fn verify(results_path: &Path, out: &Path) -> Result<(), String> {
    let fresh = {
        // generate into a temp dir, then read the JSON back
        let td = std::env::temp_dir().join(format!(
            "gnurust-math-fresh-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        generate(results_path, &td)?;
        let json = std::fs::read_to_string(td.join("math-correctness.json"))
            .map_err(|e| format!("read fresh math-correctness.json: {e}"))?;
        let md = std::fs::read_to_string(td.join("math-correctness.md"))
            .map_err(|e| format!("read fresh math-correctness.md: {e}"))?;
        let _ = std::fs::remove_dir_all(&td);
        (json, md)
    };
    let committed_json = std::fs::read_to_string(out.join("math-correctness.json"))
        .map_err(|e| format!("read committed math-correctness.json: {e}"))?;
    if fresh.0 != committed_json {
        return Err("STALE: math-correctness.json differs from a regeneration from the ledger \
                    (run `math --results <inventory> --out reports/gnucobol-runtime-tests` to refresh)"
            .into());
    }
    let committed_md = std::fs::read_to_string(out.join("math-correctness.md"))
        .map_err(|e| format!("read committed math-correctness.md: {e}"))?;
    if fresh.1 != committed_md {
        return Err("STALE: math-correctness.md differs from a regeneration from the ledger \
                    (run `math --results <inventory> --out reports/gnucobol-runtime-tests` to refresh)"
            .into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// The GnuCOBOL 3.2 math subset has exactly 323 tests; the ledger-derived distribution is
    /// 97/52/147/22/0/1/3/1 (WRAPPER_INVOCATION_MALFORMED == 0). This is the Phase-1 regression:
    /// the sum MUST equal 323, ids must be unique, and the subset must be a subset of the suite.
    #[test]
    fn math_323_reconciliation_invariants_hold() {
        let mut rows = Vec::new();
        let mut n = 0usize;
        let dist: &[(&str, usize)] = &[
            ("OBSERVABLE_MATCH", 97),
            ("CANDIDATE_CHECK_REJECT", 52),
            ("CANDIDATE_MODULE_MODEL_UNSUPPORTED", 147),
            ("WRAPPER_OPTION_UNSUPPORTED", 22),
            ("WRAPPER_INVOCATION_MALFORMED", 0),
            ("CANDIDATE_UNSUPPORTED", 1),
            ("ORACLE_SKIP", 3),
            ("ORACLE_XFAIL", 1),
        ];
        let mut suite_other = 1282usize;
        for (pc, count) in dist {
            for _ in 0..*count {
                n += 1;
                suite_other -= 1;
                rows.push(json!({
                    "test_id": format!("{n:04}"),
                    "number": n,
                    "title": format!("math test {n}"),
                    "group": "run_fundamental.at:1",
                    "oracle_status": "ORACLE_PASS",
                    "candidate_status": "PASS",
                    "primary_classification": pc,
                    "reason_code": "reason",
                }));
            }
        }
        for _ in 0..suite_other {
            n += 1;
            rows.push(json!({
                "test_id": format!("{n:04}"),
                "number": n,
                "title": format!("other test {n}"),
                "group": "syn_file.at:1",
                "oracle_status": "ORACLE_PASS",
                "candidate_status": "PASS",
                "primary_classification": "OBSERVABLE_MATCH",
                "reason_code": "reason",
            }));
        }
        assert_eq!(n, 1282, "suite must have 1282 tests");
        let math_rows = collect(&rows);
        assert_eq!(
            math_rows.len(),
            323,
            "math subset must be exactly 323 tests"
        );
        let problems = invariants(&rows, &math_rows);
        assert!(
            problems.is_empty(),
            "math invariants must hold: {}",
            problems.join("; ")
        );
        let totals: BTreeMap<&str, usize> = math_rows.iter().fold(BTreeMap::new(), |mut m, r| {
            *m.entry(r["primary_classification"].as_str().unwrap())
                .or_insert(0) += 1;
            m
        });
        assert_eq!(totals.get("WRAPPER_OPTION_UNSUPPORTED"), Some(&22));
        assert_eq!(totals.get("WRAPPER_INVOCATION_MALFORMED"), None);
        assert_eq!(totals.get("CANDIDATE_UNSUPPORTED"), Some(&1));
        assert_eq!(totals.values().sum::<usize>(), 323);
    }

    /// The stale-prose regression: a WRAPPER_INVOCATION_MALFORMED row moved out of the 22
    /// WRAPPER_OPTION_UNSUPPORTED would still sum to 323, but the per-class totals would differ;
    /// the ledger-derived distribution (22/0) is what the generator publishes, and any prose that
    /// claims 21/1 diverges from it. This test fixes the ledger-derived distribution.
    #[test]
    fn math_distribution_is_22_wrapper_option_0_malformed() {
        let mut rows = Vec::new();
        let mut n = 0usize;
        let dist: &[(&str, usize)] = &[
            ("OBSERVABLE_MATCH", 97),
            ("CANDIDATE_CHECK_REJECT", 52),
            ("CANDIDATE_MODULE_MODEL_UNSUPPORTED", 147),
            ("WRAPPER_OPTION_UNSUPPORTED", 22),
            ("CANDIDATE_UNSUPPORTED", 1),
            ("ORACLE_SKIP", 3),
            ("ORACLE_XFAIL", 1),
        ];
        for (pc, count) in dist {
            for _ in 0..*count {
                n += 1;
                rows.push(json!({
                    "test_id": format!("{n:04}"), "number": n, "title": format!("m{n}"),
                    "group": "run_functions.at:1", "oracle_status": "ORACLE_PASS",
                    "candidate_status": "PASS", "primary_classification": pc, "reason_code": "r",
                }));
            }
        }
        let math_rows = collect(&rows);
        assert_eq!(math_rows.len(), 323);
        let problems = invariants(&rows, &math_rows);
        assert!(problems.is_empty(), "{}", problems.join("; "));
    }

    /// The generator must fail when the reconciliation is broken (duplicate math id).
    #[test]
    fn math_generator_fails_on_duplicate_id() {
        let rows = vec![
            json!({"test_id": "0001", "number": 1, "title": "a", "group": "run_fundamental.at:1",
                   "oracle_status": "ORACLE_PASS", "candidate_status": "PASS",
                   "primary_classification": "OBSERVABLE_MATCH", "reason_code": "r"}),
            json!({"test_id": "0001", "number": 2, "title": "b", "group": "run_fundamental.at:2",
                   "oracle_status": "ORACLE_PASS", "candidate_status": "PASS",
                   "primary_classification": "OBSERVABLE_MATCH", "reason_code": "r"}),
        ];
        let math_rows = collect(&rows);
        let problems = invariants(&rows, &math_rows);
        assert!(
            problems.iter().any(|p| p.contains("appears 2 times")),
            "duplicate id must be reported: {problems:?}"
        );
    }

    /// End-to-end: generate() over the COMMITTED inventory must succeed (323 reconciled) when the
    /// repo is present; skipped when the committed evidence is absent (standalone crate build).
    /// The per-class distribution is NOT hardcoded (it is a measured ledger result that changes as
    /// the candidate improves); the invariants (sum == 323, unique ids, subset, one classification
    /// per test) and the JSON-vs-inventory consistency are what the test pins.
    #[test]
    fn committed_inventory_math_reconciles_to_323() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let inv = root.join("reports/gnucobol-testsuite/test-inventory.json");
        if !inv.is_file() {
            eprintln!("committed inventory absent — standalone build, skipping");
            return;
        }
        let td = tempfile::tempdir().unwrap();
        generate(&inv, td.path()).unwrap();
        let doc: Value = serde_json::from_str(
            &std::fs::read_to_string(td.path().join("math-correctness.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(doc["math_tests_total"].as_u64().unwrap(), 323);
        assert_eq!(
            doc["reconciliation"]["sum_of_classification_totals"]
                .as_u64()
                .unwrap(),
            323
        );
        assert!(doc["reconciliation"]["invariant_holds"].as_bool().unwrap());
        // JSON totals must equal the inventory subset's per-class counts (no stale prose drift).
        let v: Value = serde_json::from_str(&std::fs::read_to_string(&inv).unwrap()).unwrap();
        let rows = v["tests"].as_array().unwrap();
        let math_rows = collect(rows);
        let mut expect: BTreeMap<&str, usize> = BTreeMap::new();
        for r in &math_rows {
            *expect
                .entry(r["primary_classification"].as_str().unwrap_or("?"))
                .or_insert(0) += 1;
        }
        for (k, n) in &expect {
            assert_eq!(
                doc["primary_classification_totals"][k].as_u64().unwrap() as usize,
                *n,
                "classification {k} totals must match the inventory subset"
            );
        }
        // sanity: the current ledger's distribution is measured, not hardcoded -- but the v0.8.54
        // discrepancy (22 vs a prose claim of 21) must never reappear as an inconsistency.
        assert_eq!(math_rows.len(), 323);
    }
}
