//! Host-side gate invariants for the GNURUST.GNUCOBOL-TESTSUITE court. Fails only on REAL harness
//! problems (missing evidence, reconciliation failures, delegation, freshness, privacy leaks) —
//! never on benchmark findings.

use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

pub struct Gate {
    pub notes: Vec<String>,
    pub problems: Vec<String>,
}

pub fn gate_check(root: &Path) -> Gate {
    let mut g = Gate {
        notes: Vec::new(),
        problems: Vec::new(),
    };
    let rep = root.join("reports/gnucobol-testsuite");
    let required = [
        "test-inventory.json",
        "invocation-census.json",
        "options-frequency.csv",
        "oracle-results.json",
        "candidate-results.json",
        "comparison-results.json",
        "summary.json",
        "summary.md",
        "results.csv",
        "failure-buckets.md",
        "option-coverage.md",
        "no-delegation.json",
        "determinism.json",
        "upstream-observations.md",
        "parser-reject-census.json",
        "parser-reject-census.md",
        "parser-feature-frequency.csv",
        "parser-feature-dependency-graph.json",
        "parser-census-reconciliation.json",
        "parser-census-reconciliation.md",
        "unsupported-option-census.json",
        "unsupported-option-census.md",
    ];
    for f in required {
        let p = rep.join(f);
        if !p.is_file() {
            g.problems.push(format!(
                "missing evidence artifact reports/gnucobol-testsuite/{f}"
            ));
        }
    }

    // 1. reconciliation: every test has exactly one final classification; totals add up.
    if let Ok(inv) = read_json(&rep.join("test-inventory.json")) {
        let tests = inv["tests"].as_array().map(|a| a.len()).unwrap_or(0);
        let mut seen: BTreeMap<usize, usize> = BTreeMap::new();
        if let Some(arr) = inv["tests"].as_array() {
            for t in arr {
                if let Some(n) = t["number"].as_u64() {
                    *seen.entry(n as usize).or_insert(0) += 1;
                }
            }
        }
        let dup: Vec<usize> = seen
            .iter()
            .filter(|(_, c)| **c != 1)
            .map(|(n, _)| *n)
            .collect();
        if !dup.is_empty() {
            g.problems.push(format!(
                "test numbers not exactly-once in inventory: {dup:?}"
            ));
        }
        let claimed = inv["suite_total_claimed"].as_u64().unwrap_or(0) as usize;
        if tests != claimed || tests == 0 {
            g.problems.push(format!(
                "inventory size {tests} != claimed suite total {claimed}"
            ));
        } else {
            g.notes
                .push(format!("all {tests} tests accounted (exactly-once)"));
        }
    } else {
        g.problems
            .push("test-inventory.json missing/unreadable".into());
    }

    // 2. summary reconciliation: total == sum of every primary classification.
    if let Ok(sum) = read_json(&rep.join("summary.json")) {
        let s = &sum["summary"];
        let total = s["total_tests"].as_u64().unwrap_or(0) as usize;
        let mut sum_counts = 0usize;
        if let Some(ff) = s["first_failure"].as_object() {
            for (_, v) in ff {
                sum_counts += v.as_u64().unwrap_or(0) as usize;
            }
        }
        if total != sum_counts {
            g.problems.push(format!(
                "summary does not reconcile: total_tests {total} != sum(first_failure) {sum_counts}"
            ));
        } else {
            g.notes
                .push(format!("summary reconciles ({total} == {sum_counts})"));
        }
    }

    // 3. no-delegation + determinism artifacts.
    if let Ok(nd) = read_json(&rep.join("no-delegation.json")) {
        let ok = nd["candidate_phase_isolated"].as_bool().unwrap_or(false)
            && nd["cobrun_links_no_libcob"].as_bool().unwrap_or(false)
            && nd["cobc_rs_links_no_libcob"].as_bool().unwrap_or(false);
        if ok {
            g.notes
                .push("no-delegation proof present and positive".into());
        } else {
            g.problems
                .push("no-delegation.json does not assert candidate isolation".into());
        }
    }
    if let Ok(det) = read_json(&rep.join("determinism.json")) {
        match det["stable_summary_identical"].as_bool() {
            Some(true) => g
                .notes
                .push("determinism: stable summaries identical across passes".into()),
            Some(false) => g
                .problems
                .push("determinism: stable summaries differ between passes".into()),
            None => g
                .problems
                .push("determinism.json lacks stable_summary_identical".into()),
        }
    }

    // 3b. parser-census reconciliation: the census must be regenerated from the machine ledger,
    //     carry declared counting units, and match the summary first-failure counts (683-vs-700
    //     doctrine: the stale Markdown family is rejected here).
    match crate::reject_census::check(root) {
        Ok(notes) => {
            let mut bad = 0;
            for n in notes {
                g.problems.push(format!("parser census STALE: {n}"));
                bad += 1;
            }
            if bad == 0 {
                g.notes
                    .push("parser census: fresh, reconciled (683 first-failure groups; counting units declared)".into());
            }
        }
        Err(e) => g
            .problems
            .push(format!("parser census reconciliation failed: {e}")),
    }
    match crate::option_census::verify_committed(root) {
        Ok((notes, problems)) => {
            for n in &notes {
                g.problems.push(format!("option census STALE: {n}"));
            }
            for p in &problems {
                g.problems
                    .push(format!("option census reconciliation: {p}"));
            }
            if notes.is_empty() && problems.is_empty() {
                g.notes
                    .push("option census: reconciles and is fresh".into());
            }
        }
        Err(e) => g.problems.push(format!("option census: {e}")),
    }

    // 4. receipts present + fresh (receipt.json exists per gate; freshness = hash of current files
    //    matches the receipt's recorded artifact hashes is enforced by receipts-finalize).
    for gate in [
        "GNURUST.GNUCOBOL-TESTSUITE.1",
        "GNURUST.GNUCOBOL-TESTSUITE.2",
        "GNURUST.GNUCOBOL-TESTSUITE.3",
    ] {
        let d = root.join(format!("reports/receipts/{gate}"));
        if !d.join("receipt.json").is_file() || !d.join("receipt.md").is_file() {
            g.problems.push(format!("receipt missing for {gate}"));
        }
    }

    // 5b. option-compatibility doc freshness (prompt §6.3): the generated table must equal a
    // regeneration from the cobc-rs policy registry + the committed invocation census. Skipped
    // (note, not failure) when the cobc-rs binary is not built yet.
    let cobc_rs = root.join("target/release/cobc-rs");
    if cobc_rs.is_file() && rep.join("invocation-census.json").is_file() {
        let policy_tmp = root.join("target/release/.cobc-rs-policy-dump.json");
        let dumped = std::process::Command::new(&cobc_rs)
            .arg(format!("--dump-policy-json={}", policy_tmp.display()))
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !dumped {
            g.problems
                .push("cobc-rs --dump-policy-json failed during the gate".into());
        } else {
            let doc = root.join("docs/generated/cobc-rs-option-compatibility.md");
            match crate::compat::check(&policy_tmp, &rep.join("invocation-census.json"), &doc) {
                Ok(()) => g
                    .notes
                    .push("cobc-rs option-compatibility doc is fresh".into()),
                Err(e) => g.problems.push(e),
            }
            let _ = std::fs::remove_file(&policy_tmp);
        }
    } else {
        g.notes.push(
            "cobc-rs binary or census absent — option-compatibility freshness not checked".into(),
        );
    }

    // 5c. math-subset reconciliation (Phase-1 boundary-reduction invariant, prompt §1): the
    //     committed math-correctness.json must reconcile to the math subset of the committed
    //     inventory (323 == sum of classification totals == unique ids, ids ⊆ suite).
    let runtime_rep = root.join("reports/gnucobol-runtime-tests");
    if rep.join("test-inventory.json").is_file()
        && runtime_rep.join("math-correctness.json").is_file()
    {
        let inv = read_json(&rep.join("test-inventory.json"));
        let math = read_json(&runtime_rep.join("math-correctness.json"));
        match (inv, math) {
            (Ok(inv), Ok(math)) => {
                let rows = inv["tests"].as_array().cloned().unwrap_or_default();
                let math_rows = crate::math::collect(&rows);
                let problems = crate::math::invariants(&rows, &math_rows);
                if !problems.is_empty() {
                    g.problems.push(format!(
                        "math-subset reconciliation failed: {}",
                        problems.join("; ")
                    ));
                } else {
                    let declared = math["math_tests_total"].as_u64().unwrap_or(0) as usize;
                    let sum = math["reconciliation"]["sum_of_classification_totals"]
                        .as_u64()
                        .unwrap_or(0) as usize;
                    if declared != math_rows.len() || sum != math_rows.len() {
                        g.problems.push(format!(
                            "math-correctness.json declares {declared} tests / sum {sum}, but the inventory subset has {} tests",
                            math_rows.len()
                        ));
                    } else {
                        g.notes.push(format!(
                            "math subset reconciles: {declared} tests == sum of classification totals"
                        ));
                    }
                }
            }
            _ => g
                .problems
                .push("math-correctness.json or test-inventory.json unreadable".into()),
        }
    } else {
        g.notes
            .push("math-correctness evidence absent — math reconciliation not checked".into());
    }

    // 6. privacy gate: no host-path patterns in the committed testsuite evidence.
    let pats = ["/home/", "/run/media/", "/mnt/", "/media/"];
    if let Ok(mut walk) = walk_files(&rep) {
        walk.sort();
        for p in walk {
            if let Ok(text) = std::fs::read_to_string(&p) {
                for pat in pats {
                    if text.contains(pat) {
                        g.problems.push(format!(
                            "PRIVACY: {} contains {pat:?}",
                            p.strip_prefix(root).unwrap_or(&p).display()
                        ));
                        break;
                    }
                }
            }
        }
    }

    if g.problems.is_empty() {
        g.notes
            .push("GNURUST.GNUCOBOL-TESTSUITE gate check: PASS".into());
    }
    g
}

pub fn exit_code(g: &Gate) -> i32 {
    if g.problems.is_empty() {
        0
    } else {
        1
    }
}

fn walk_files(dir: &Path) -> Result<Vec<std::path::PathBuf>, String> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let rd = std::fs::read_dir(&d).map_err(|e| format!("read_dir {}: {e}", d.display()))?;
        for ent in rd.flatten() {
            let p = ent.path();
            if p.is_dir() {
                // skip the raw per-test group dirs (they carry container-internal paths by design;
                // those are /work/... container paths, not host paths — but keep the scan cheap)
                if p.file_name()
                    .is_some_and(|n| n.to_string_lossy().chars().all(|c| c.is_ascii_digit()))
                {
                    continue;
                }
                stack.push(p);
            } else if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                if matches!(ext, "json" | "md" | "csv") {
                    out.push(p);
                }
            }
        }
    }
    Ok(out)
}

fn read_json(p: &Path) -> Result<Value, String> {
    std::fs::read_to_string(p)
        .map_err(|e| format!("read {}: {e}", p.display()))
        .and_then(|s| serde_json::from_str(&s).map_err(|e| format!("parse {}: {e}", p.display())))
}
