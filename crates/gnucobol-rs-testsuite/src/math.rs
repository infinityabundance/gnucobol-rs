//! The runtime/mathematics evidence campaign (prompt Phase 4.1/4.2): the math subset of the
//! GnuCOBOL testsuite, classified from the SAME differential results as every other test (no
//! favorable selection). Produces `math-correctness.json` + `math-correctness.md` under
//! reports/gnucobol-runtime-tests/.

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

/// Generate the math-correctness report from the committed classification inventory.
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

    let doc = json!({
        "schema": "gnurust-gnucobol-runtime-math-correctness-v1",
        "source": "the SAME differential classification as the full suite (no favorable selection)",
        "math_tests_total": math_rows.len(),
        "suite_total": rows.len(),
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
         outcome; performance is reported separately and only for tests passing on both sides.\n\n",
        math_rows.len(),
        rows.len()
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
