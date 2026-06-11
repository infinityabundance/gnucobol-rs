//! Small helpers replacing inline `python3 -c` in the gate scripts.
use serde_json::Value;
use std::path::Path;

const GCODE_EXCLUDE: [&str; 19] = ["GNURUST.COVERAGE.1","GNURUST.FILE.STATUS.1","GNURUST.INTRINSIC.ATLAS.1","GNURUST.PROCEDURE.FLOW.ATLAS.1","GNURUST.PUBLIC.CORPUS.1","GNURUST.BUILD.PROFILE.1","GNURUST.PUBLIC.GAP.1","GNURUST.CALL.EXTENSION.ATLAS.1","GNURUST.INDEXED.FILE.ATLAS.1","GNURUST.SORT.MERGE.ATLAS.1","GNURUST.RELATIVE.FILE.ATLAS.1","GNURUST.DIALECT.RUNTIME.ATLAS.1","GNURUST.DIRECTIVE.VARIANCE.ATLAS.1","GNURUST.DECLARATIVES.ATLAS.1","GNURUST.CALL.LAYOUT.ATLAS.1","GNURUST.LINEAGE.CORPUS.20M.0","GNURUST.LINEAGE.CORPUS.20M.SMOKE","GNURUST.LINEAGE.CORPUS.20M.1","GNURUST.VALUE.NEGZERO.EDGE.1"];

/// Print the GNURUST courts that must carry a casefile/doc reference (the GCODES the doc-gate iterates).
pub fn gcodes(root: &str) -> i32 {
    let cl: Value = std::fs::read_to_string(Path::new(root).join("reports/claim-ladder.json")).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(Value::Null);
    let ids: Vec<String> = cl["courts"].as_array().map(|a| a.iter().filter_map(|c| c["id"].as_str())
        .filter(|id| id.starts_with("GNURUST.") && !GCODE_EXCLUDE.contains(id)).map(String::from).collect()).unwrap_or_default();
    println!("{}", ids.join(" "));
    0
}

/// Validate that every archaeology/*.json parses; print the count, exit 1 on any invalid.
pub fn atlas_check(root: &str) -> i32 {
    let mut ok = 0;
    let mut bad = 0;
    fn walk(dir: &Path, ok: &mut i32, bad: &mut i32) {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() { walk(&p, ok, bad); }
                else if p.extension().map(|x| x == "json").unwrap_or(false) {
                    match std::fs::read_to_string(&p).ok().and_then(|s| serde_json::from_str::<Value>(&s).ok()) {
                        Some(_) => *ok += 1,
                        None => { println!("atlas JSON invalid: {}", p.display()); *bad += 1; }
                    }
                }
            }
        }
    }
    walk(&Path::new(root).join("archaeology"), &mut ok, &mut bad);
    println!("atlas: {ok} JSON files valid");
    if bad > 0 { 1 } else { 0 }
}

/// Generic sweep glue: join cobc `label=value` output (arg2) into the cases TSV (arg1), appending the
/// oracle value as the last column per row (replaces the shared inline-python join in the *_sweep.sh).
pub fn sweep_join(cases: &str, out: &str) -> i32 {
    let mut map = std::collections::HashMap::new();
    for line in std::fs::read_to_string(out).unwrap_or_default().lines() {
        if let Some(eq) = line.find('=') { map.insert(line[..eq].to_string(), line[eq + 1..].to_string()); }
    }
    for line in std::fs::read_to_string(cases).unwrap_or_default().lines() {
        let label = line.split('\t').next().unwrap_or("");
        println!("{}\t{}", line, map.get(label).map(String::as_str).unwrap_or(""));
    }
    0
}
