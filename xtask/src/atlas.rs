//! Atlas-gen oracle sweeps (observed-behavior courts). Port of the inline-python in *_atlas_sweep.sh:
//! parse cobc output -> compare to the witnessed `expect` facts -> (re)write the atlas JSON -> PASS=/FAIL=.
//! The atlas content is static observed data (embedded); the court is the assert.
use std::path::Path;

fn kv(text: &str) -> std::collections::HashMap<String, String> {
    let mut d = std::collections::HashMap::new();
    for line in text.lines() {
        if let Some(eq) = line.find('=') { d.insert(line[..eq].trim().to_string(), line[eq + 1..].trim().to_string()); }
    }
    d
}
fn writeatlas(root: &str, rel: &str, embedded: &str) {
    let _ = std::fs::write(Path::new(root).join(rel), embedded);
}

const FILE_STATUS_ATLAS: &str = include_str!("data/file-status-atlas.json");
const INTRINSIC_ATLAS: &str = include_str!("data/intrinsic-atlas.json");

/// file_status: VALID/MISSING cobc output via env; compare observed statuses to expected; write atlas.
pub fn file_status(root: &str) -> i32 {
    let valid = kv(&std::env::var("VALID").unwrap_or_default());
    let missing = kv(&std::env::var("MISSING").unwrap_or_default());
    let expect: [((&str, &str), &str); 7] = [
        (("valid","open_input"),"00"), (("valid","read_first"),"00"), (("valid","read_at_eof"),"10"),
        (("valid","read_past_eof"),"46"), (("valid","close"),"00"),
        (("missing","open_input"),"35"), (("missing","close"),"42"),
    ];
    let obs = |cat: &str, k: &str| -> Option<String> { if cat == "valid" { valid.get(k).cloned() } else { missing.get(k).cloned() } };
    let mut fails = Vec::new();
    for ((cat, k), e) in expect {
        if obs(cat, k).as_deref() != Some(e) { fails.push(format!("(('{cat}','{k}'), {e}, {:?})", obs(cat, k))); }
    }
    writeatlas(root, "reports/file-status-atlas.json", FILE_STATUS_ATLAS);
    println!("PASS={} FAIL={}", expect.len() - fails.len(), fails.len());
    for f in &fails { println!("  MISMATCH {f}"); }
    if fails.is_empty() { 0 } else { 1 }
}

/// intrinsic_atlas: cobc output via env OUT; compare to the witnessed intrinsic facts; write atlas.
pub fn intrinsic(root: &str) -> i32 {
    let kvm = kv(&std::env::var("OUT").unwrap_or_default());
    let expect: [(&str, &str); 20] = [
        ("LENGTH","00000005.00"),("BYTE_LENGTH","00000005.00"),("NUMVAL","00000123.45"),("NUMVAL_C","00001234.56"),
        ("INTEGER_P","+00000003.00"),("INTEGER_N","-00000004.00"),("INTPART_P","+00000003.00"),("INTPART_N","-00000003.00"),
        ("MOD_P","+00000002.00"),("MOD_N","+00000003.00"),("REM_P","+00000002.00"),("REM_N","-00000002.00"),
        ("UPPER","[ABC     ]"),("LOWER","[abc     ]"),("REVERSE","[dcba    ]"),("ORD","00000066.00"),("CHAR","[A       ]"),
        ("CURRENT_DATE_LEN","000000021"),("WHEN_COMPILED_LEN","21"),("BYTE_LENGTH","00000005.00"),
    ];
    // de-dup keys (BYTE_LENGTH appears once logically); use a map of unique expects
    let mut uniq: std::collections::BTreeMap<&str, &str> = std::collections::BTreeMap::new();
    for (k, e) in expect { uniq.insert(k, e); }
    let mut fails = Vec::new();
    for (k, e) in &uniq { if kvm.get(*k).map(String::as_str) != Some(e) { fails.push(format!("({k}, {e}, {:?})", kvm.get(*k))); } }
    writeatlas(root, "reports/intrinsic-atlas.json", INTRINSIC_ATLAS);
    println!("PASS={} FAIL={}", uniq.len() - fails.len(), fails.len());
    for f in &fails { println!("  MISMATCH {f}"); }
    if fails.is_empty() { 0 } else { 1 }
}
