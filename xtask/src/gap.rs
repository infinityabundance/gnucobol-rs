//! GNURUST.PUBLIC.GAP.1 — surface gap board over the admitted GnuCOBOL testsuite. Port of lab/gap/run.py.
use regex::RegexBuilder;
use serde_json::{json, Value};
use std::path::Path;

// (regex, surface, status, court-or-reason)
const SURFACES: [(&str, &str, &str, &str); 27] = [
    (r"\bINITIALIZE\b", "INITIALIZE", "sealed", "GNURUST.INITIALIZE.1"),
    (r"\bINSPECT\b", "INSPECT", "sealed", "GNURUST.INSPECT.1"),
    (r"\b(STRING|UNSTRING)\b", "STRING/UNSTRING", "sealed", "GNURUST.STRING.UNSTRING.1"),
    (r"\b(ACCEPT|DISPLAY)\b", "ACCEPT/DISPLAY", "sealed", "GNURUST.ACCEPT.DISPLAY.1/.2"),
    (r"\b(COMPUTE|ADD|SUBTRACT|MULTIPLY)\b", "arithmetic", "sealed", "GNURUST.7/13"),
    (r"\bDIVIDE\b", "DIVIDE", "sealed", "GNURUST.19/REMAINDER.1"),
    (r"\bFUNCTION\b", "intrinsics", "sealed", "GNURUST.INTRINSIC.* (LENGTH/NUMVAL/INTEGER/MOD/REM/DATE/CASE/ORD-CHAR)"),
    (r"\bEVALUATE\b", "EVALUATE", "sealed", "GNURUST.IF.EVALUATE.SLICE.1 (bounded slice)"),
    (r"\bPERFORM\b", "PERFORM", "sealed", "GNURUST.PERFORM.SLICE.1 + TABLE.PERFORM.SLICE.1 (bounded slices)"),
    (r"\bMOVE\b", "MOVE", "sealed", "GNURUST.2 (decimal/display) + slices"),
    (r"\b(WRITE|READ|OPEN|CLOSE)\b", "sequential file I/O", "sealed", "GNURUST.FILE.SEQUENTIAL.1/WRITE.1"),
    (r"ORGANIZATION\s+IS\s+(RECORD|LINE)\s+SEQUENTIAL", "sequential org", "sealed", "GNURUST.FILE.SEQUENTIAL.1"),
    (r"\bGO\s+TO\b", "GO TO / procedure flow", "observed", "GNURUST.PROCEDURE.FLOW.ATLAS.1"),
    (r"FILE\s+STATUS", "file status", "observed", "GNURUST.FILE.STATUS.1"),
    (r"ON\s+SIZE\s+ERROR", "SIZE ERROR", "sealed", "GNURUST.SIZE.ERROR.1"),
    (r"\bCALL\b", "CALL / linkage", "observed", "GNURUST.CALL.EXTENSION.ATLAS.1"),
    (r"\b(SORT|MERGE)\b", "SORT / MERGE", "observed", "GNURUST.SORT.MERGE.ATLAS.1"),
    (r"\bSEARCH\b", "SEARCH (table lookup)", "sealed", "GNURUST.SEARCH.TABLE.1"),
    (r"\bSTART\b", "START (indexed/relative key positioning)", "observed", "GNURUST.INDEXED.FILE.ATLAS.1"),
    (r"\bDELETE\b", "DELETE (indexed/relative)", "observed", "GNURUST.INDEXED.FILE.ATLAS.1"),
    (r"ORGANIZATION\s+IS\s+INDEXED", "indexed file org", "observed", "GNURUST.INDEXED.FILE.ATLAS.1"),
    (r"ORGANIZATION\s+IS\s+RELATIVE", "relative file org", "observed", "GNURUST.RELATIVE.FILE.ATLAS.1"),
    (r"\bDECLARATIVES\b|USE\s+AFTER\s+STANDARD\s+ERROR", "DECLARATIVES / USE error handler", "observed", "GNURUST.DECLARATIVES.ATLAS.1"),
    (r"\bSCREEN\s+SECTION\b", "SCREEN SECTION", "refused", "NEG (screen I/O out of the data-evidence lane)"),
    (r"\bRD\b|REPORT\s+SECTION", "REPORT WRITER", "refused", "NEG (report writer out of scope)"),
    (r"EXEC\s+SQL", "embedded SQL / DB2", "refused", "NEG.DB2.* / NEG.SQL.PRECOMPILER"),
    (r"EXEC\s+CICS", "CICS", "refused", "NEG.CICS.*"),
];

fn build(root: &str) -> Value {
    let tests = Path::new(root).join("lab/admit/gnucobol-3.2/tests/testsuite.src");
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&tests) {
        for e in rd.flatten() {
            let n = e.file_name().to_string_lossy().to_string();
            if (n.starts_with("run_") || n.starts_with("syn_")) && n.ends_with(".at") {
                files.push(e.path());
            }
        }
    }
    files.sort();
    let mut blob = String::new();
    for f in &files {
        blob.push_str(&std::fs::read_to_string(f).unwrap_or_default().to_uppercase());
    }
    let mut rows = Vec::new();
    for (rx, surface, cls, court) in SURFACES {
        let re = RegexBuilder::new(rx).case_insensitive(true).build().unwrap();
        let n = re.find_iter(&blob).count();
        rows.push(json!({"surface": surface, "occurrences": n, "status": cls, "court": court}));
    }
    let count = |s: &str| rows.iter().filter(|r| r["status"] == s).count();
    let mut missing: Vec<Value> = rows.iter().filter(|r| r["status"] == "missing" && r["occurrences"].as_i64().unwrap_or(0) > 0)
        .map(|r| json!({"surface": r["surface"], "occurrences": r["occurrences"], "proposed_court": r["court"]})).collect();
    missing.sort_by_key(|m| -(m["occurrences"].as_i64().unwrap_or(0)));
    let mut surfaces_sorted = rows.clone();
    surfaces_sorted.sort_by_key(|r| -(r["occurrences"].as_i64().unwrap_or(0)));
    json!({
        "schema": "gnurust-public-gap-board-v1", "court": "GNURUST.PUBLIC.GAP.1",
        "corpus": "GnuCOBOL 3.2.0 upstream testsuite (admitted, lab/admit)",
        "scope": "surface-frequency scan of .at run/syn tests -- NOT compilation/execution/parity",
        "files_scanned": files.len(),
        "counts": {"sealed": count("sealed"), "observed": count("observed"), "refused": count("refused"), "missing": count("missing")},
        "missing_court_board": missing,
        "surfaces": surfaces_sorted,
        "negative_capabilities": ["NEG.PUBLIC_GAP.NOT_EXECUTION","NEG.PUBLIC_GAP.NOT_PARITY","NEG.PUBLIC_GAP.SINGLE_CORPUS","NEG.PUBLIC_GAP.VERB_PRESENCE_NOT_COURT_NEED","NEG.PUBLIC_GAP.MISSING_IS_CANDIDATE_NOT_COMMITMENT"]
    })
}

pub fn run(cmd: &str, root: &str) -> i32 {
    let b = build(root);
    match cmd {
        "generate" => {
            let _ = std::fs::write(Path::new(root).join("reports/public-gap-board.json"), serde_json::to_vec_pretty(&b).unwrap_or_default());
            let c = &b["counts"];
            println!("gap board: {} files; surfaces sealed {} observed {} refused {} missing {}", b["files_scanned"], c["sealed"], c["observed"], c["refused"], c["missing"]);
            0
        }
        "check" => {
            if b["files_scanned"].as_u64().unwrap_or(0) == 0 {
                println!("GATE: admitted GnuCOBOL testsuite not found");
                return 1;
            }
            println!("GNURUST.PUBLIC.GAP.1: {} testsuite files scanned; {} missing surfaces on the board (0 = every exercised surface is sealed/observed/refused); surface-scan only", b["files_scanned"], b["missing_court_board"].as_array().unwrap().len());
            0
        }
        _ => { eprintln!("usage: gap generate|check"); 2 }
    }
}
