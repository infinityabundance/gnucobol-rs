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

const DIALECT_RUNTIME_ATLAS: &str = include_str!("data/dialect-runtime-atlas.json");

pub fn dialect_runtime(root: &str) -> i32 {
    let load = |envk: &str| -> std::collections::HashMap<String, String> {
        let mut d = std::collections::HashMap::new();
        for ln in std::fs::read_to_string(std::env::var(envk).unwrap_or_default()).unwrap_or_default().lines() {
            let parts: Vec<&str> = ln.split_whitespace().collect();
            if !parts.is_empty() { d.insert(parts[0].to_string(), parts.get(1).copied().unwrap_or("").to_string()); }
        }
        d
    };
    let store = load("STORE_TXT");
    let disp = load("DISP_TXT");
    let e = |k: &str| std::env::var(k).unwrap_or_default();
    let store_vals: std::collections::HashSet<String> = store.values().filter(|v| *v != "REJECT").cloned().collect();
    let leading: std::collections::HashSet<&String> = disp.iter().filter(|(_, v)| *v == "-0123").map(|(k, _)| k).collect();
    let trailing: std::collections::HashSet<&String> = disp.iter().filter(|(_, v)| *v == "0123-").map(|(k, _)| k).collect();
    let dl = |d: &str| disp.get(d).map(String::as_str);
    let lead_ok = ["default", "mf-strict"].iter().all(|d| leading.contains(&d.to_string()));
    let trail_ok = ["ibm-strict", "mvs-strict", "bs2000-strict", "rm-strict"].iter().all(|d| trailing.contains(&d.to_string()));
    report(root, "reports/dialect-runtime-atlas.json", DIALECT_RUNTIME_ATLAS, &[
        ("stored-sign-bytes-invariant", store_vals.len() == 1 && store_vals.contains("30313273")),
        ("present-default-leading", dl("default") == Some("-0123")),
        ("present-ibm-trailing", dl("ibm-strict") == Some("0123-")),
        ("present-two-camps", lead_ok && trail_ok),
        ("comp5-rejected-by-strict-std", e("C5_85") == "REJ" && e("C5_DEF") == "OK"),
        ("trim-rejected-by-cobol85", e("TRIM_85") == "REJ"),
        ("binary-long-rejected-by-ibm", e("BL_IBM") == "REJ" && e("BL_DEF") == "OK"),
    ])
}

const SIZE_ERROR_ATLAS: &str = include_str!("data/size-error-atlas.json");

pub fn size_error(root: &str, tmp: &str) -> i32 {
    let scen: [(&str, usize); 6] = [("ADDDISP",3),("ADDC3",2),("MULDISP",3),("SUBSIGN",3),("DIV0DISP",3),("ROUNDDISP",3)];
    let grab = |buf: &[u8], marker: &[u8], n: usize| -> Option<Vec<u8>> {
        buf.windows(marker.len()).position(|w| w == marker).map(|i| { let s = i + marker.len(); buf[s..(s + n).min(buf.len())].to_vec() })
    };
    let (mut pf, mut fl) = (0, 0);
    for (base, n) in scen {
        for var in ["P", "S"] {
            let f = Path::new(tmp).join(format!("{base}{var}.out"));
            if !f.exists() { println!("NO-OUTPUT {base}{var}"); fl += 1; continue; }
            let buf = std::fs::read(&f).unwrap_or_default();
            let (before, after, se) = (grab(&buf, b"BEFORE[", n), grab(&buf, b"AFTER[", n), grab(&buf, b"SE[", 1));
            if before.is_none() || after.is_none() || se.is_none() { println!("PARSE-FAIL {base}{var}"); fl += 1; continue; }
            let written = before != after;
            let signaled = se.as_deref() == Some(b"Y");
            let ok = if var == "S" { signaled && !written } else { !signaled };
            if ok { pf += 1; } else { println!("MISMATCH {base}{var}: written={written} signaled={signaled}"); fl += 1; }
        }
    }
    writeatlas(root, "reports/size-error-atlas.json", SIZE_ERROR_ATLAS);
    println!("PASS={pf} FAIL={fl}");
    if fl > 0 { 1 } else { 0 }
}
