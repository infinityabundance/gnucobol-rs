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

const INDEXED_ATLAS: &str = include_str!("data/indexed-file-atlas.json");
const RELATIVE_ATLAS: &str = include_str!("data/relative-file-atlas.json");
const SORT_MERGE_ATLAS: &str = include_str!("data/sort-merge-atlas.json");

fn kv_compare(root: &str, atlas_rel: &str, embedded: &str, expect: &[(&str, &str)]) -> i32 {
    let kvm = kv(&std::env::var("OUT").unwrap_or_default());
    let mut fails = Vec::new();
    for (k, e) in expect { if kvm.get(*k).map(String::as_str) != Some(e) { fails.push(format!("({k}, {e}, {:?})", kvm.get(*k))); } }
    writeatlas(root, atlas_rel, embedded);
    println!("PASS={} FAIL={}", expect.len() - fails.len(), fails.len());
    for f in &fails { println!("  MISMATCH {f}"); }
    if fails.is_empty() { 0 } else { 1 }
}

pub fn indexed_file(root: &str) -> i32 {
    kv_compare(root, "reports/indexed-file-atlas.json", INDEXED_ATLAS,
        &[("dup","22"),("read_hit","00/beta"),("read_miss","23"),("start","00"),("n1","AAA"),("n2","BBB"),("n3","CCC"),("del","00"),("read_del","23")])
}
pub fn relative_file(root: &str) -> i32 {
    kv_compare(root, "reports/relative-file-atlas.json", RELATIVE_ATLAS, &[("r3","00/three"),("r2","23"),("r1","00/one")])
}
pub fn sort_merge(root: &str) -> i32 {
    let out = std::env::var("OUT").unwrap_or_default();
    let take3 = |pfx: &str| -> Vec<String> { out.lines().filter(|l| l.starts_with(pfx)).filter_map(|l| l.split_once('=').map(|(_, v)| v.chars().take(3).collect())).collect() };
    let asc = take3("asc=");
    let desc = take3("desc=");
    let mut fails = Vec::new();
    if asc != ["010","020","050","099"] { fails.push(format!("(ascending, {asc:?})")); }
    if desc != ["099","050","020","010"] { fails.push(format!("(descending, {desc:?})")); }
    writeatlas(root, "reports/sort-merge-atlas.json", SORT_MERGE_ATLAS);
    println!("PASS={} FAIL={}", 2 - fails.len(), fails.len());
    for f in &fails { println!("  MISMATCH {f}"); }
    if fails.is_empty() { 0 } else { 1 }
}

const PROCEDURE_FLOW_ATLAS: &str = include_str!("data/procedure-flow-atlas.json");
const CALL_ATLAS: &str = include_str!("data/call-atlas.json");
const DECLARATIVES_ATLAS: &str = include_str!("data/declaratives-atlas.json");

pub fn procedure_flow(root: &str) -> i32 {
    kv_compare(root, "reports/procedure-flow-atlas.json", PROCEDURE_FLOW_ATLAS,
        &[("if","THEN"),("eval","2"),("perform_times","003"),("varying_body","004"),("varying_ends","005"),("until","005"),("perform_para","007"),("goto_skipped","007")])
}
pub fn call_atlas(root: &str) -> i32 {
    kv_compare(root, "reports/call-atlas.json", CALL_ATLAS,
        &[("ref_A","101"),("content_B","100"),("toupper","[ABCDE]"),("exception","caught"),("cancel","ok")])
}
pub fn declaratives(root: &str) -> i32 {
    let out = std::env::var("OUT").unwrap_or_default();
    let rc = std::env::var("RC").unwrap_or_default();
    let checks: [(&str, bool); 5] = [
        ("open-failure-fires-decl", out.contains("DECL-F fs=35")),
        ("close-failure-fires-decl", out.contains("DECL-F fs=42")),
        ("success-fires-nothing", !out.contains("DECL-G")),
        ("status-visible-inside", out.contains("fs=35") && out.contains("fs=42")),
        ("execution-resumes", out.contains("REACHED-END") && rc == "0"),
    ];
    let fails: Vec<&str> = checks.iter().filter(|(_, ok)| !ok).map(|(n, _)| *n).collect();
    writeatlas(root, "reports/declaratives-atlas.json", DECLARATIVES_ATLAS);
    println!("PASS={} FAIL={}", checks.len() - fails.len(), fails.len());
    for n in &fails { println!("  MISMATCH {n}"); }
    if fails.is_empty() { 0 } else { 1 }
}

const CALL_LAYOUT_ATLAS: &str = include_str!("data/call-layout-atlas.json");
const DIRECTIVE_VARIANCE_ATLAS: &str = include_str!("data/directive-variance-atlas.json");

fn report(root: &str, atlas_rel: &str, embedded: &str, checks: &[(&str, bool)]) -> i32 {
    let fails: Vec<&str> = checks.iter().filter(|(_, ok)| !ok).map(|(n, _)| *n).collect();
    writeatlas(root, atlas_rel, embedded);
    println!("PASS={} FAIL={}", checks.len() - fails.len(), fails.len());
    for n in &fails { println!("  MISMATCH {n}"); }
    if fails.is_empty() { 0 } else { 1 }
}

pub fn call_layout(root: &str) -> i32 {
    let out = std::env::var("OUT").unwrap_or_default();
    let kvre = regex::Regex::new(r"^([A-Z0-9-]+)=\[?(.*?)\]?$").unwrap();
    let mut kvm = std::collections::HashMap::new();
    for ln in out.lines() { if let Some(c) = kvre.captures(ln) { kvm.insert(c[1].to_string(), c[2].to_string()); } }
    let see5: Vec<String> = regex::Regex::new(r"SEE5=\[(.*?)\]").unwrap().captures_iter(&out).map(|c| c[1].to_string()).collect();
    let g = |k: &str| kvm.get(k).map(String::as_str);
    report(root, "reports/call-layout-atlas.json", CALL_LAYOUT_ATLAS, &[
        ("byref-overlay-adjacent", !see5.is_empty() && see5[0] == "ABCXY"),
        ("byref-callee-write-visible", g("REF-CALLER") == Some("ZBC")),
        ("bycontent-clean-copy", g("SEE3") == Some("DEF")),
        ("bycontent-caller-untouched", g("CONTENT-CALLER") == Some("DEF")),
        ("numeric-narrower-leading-bytes", g("SEEN2") == Some("12")),
    ])
}

pub fn directive_variance(root: &str) -> i32 {
    let g = |k: &str| std::env::var(k).unwrap_or_default();
    report(root, "reports/directive-variance-atlas.json", DIRECTIVE_VARIANCE_ATLAS, &[
        ("binary-size-default-len7", g("SZ_DEF") == "7"),
        ("binary-size-248-grows-to-8", g("SZ_248") == "8"),
        ("byteorder-default-big", g("OD_DEF") == "1234"),
        ("byteorder-native-host-little", g("OD_NAT") == "3412"),
        ("truncate-default-ansi", g("TR_DEF") == "00"),
        ("no-truncate-keeps-binary", g("TR_NO") == "44"),
    ])
}
