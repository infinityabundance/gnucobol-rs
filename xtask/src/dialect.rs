//! DIALECT.PROFILE.1 — the declared GnuCOBOL witness profile. Port of lab/dialect/run.py. The
//! profile_sha256 is sha256(python-canonical-json(content)) -- ported exactly so the committed hash (which
//! flows into receipts) is byte-stable.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

fn prefix(root: &str) -> PathBuf { Path::new(root).join("lab/oracle/prefix") }
fn out_path(root: &str) -> PathBuf { Path::new(root).join("reports/dialect-profile/default.json") }

fn sha_hex(b: &[u8]) -> String { let mut s = String::new(); for x in Sha256::digest(b) { s.push_str(&format!("{x:02x}")); } s }
fn sha_file(p: &Path) -> Option<String> { std::fs::read(p).ok().map(|b| sha_hex(&b)) }

fn oracle_present(root: &str) -> bool { prefix(root).join("bin/cobc").exists() }

fn cobc_version(root: &str) -> Option<String> {
    if !oracle_present(root) { return None; }
    let out = std::process::Command::new(prefix(root).join("bin/cobc")).arg("--version")
        .env("LD_LIBRARY_PATH", prefix(root).join("lib")).output().ok()?;
    let s = String::from_utf8_lossy(&out.stdout);
    // parse "(GnuCOBOL) <ver>"
    let idx = s.find("(GnuCOBOL)")? + "(GnuCOBOL)".len();
    let rest = s[idx..].trim_start();
    let ver: String = rest.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
    if ver.is_empty() { None } else { Some(ver) }
}

fn libcob_path(root: &str) -> Option<PathBuf> {
    let d = prefix(root).join("lib");
    let mut cands: Vec<PathBuf> = std::fs::read_dir(&d).ok()?.flatten().map(|e| e.path())
        .filter(|p| { let n = p.file_name().and_then(|n| n.to_str()).unwrap_or(""); n.starts_with("libcob.so.") && n.matches('.').count() >= 3 }).collect();
    cands.sort();
    cands.pop()
}

/// Python json.dumps(obj, sort_keys=True) canonical bytes (separators ", "/": ", ensure_ascii).
fn py_json(v: &Value) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => py_str(s),
        Value::Array(a) => format!("[{}]", a.iter().map(py_json).collect::<Vec<_>>().join(", ")),
        Value::Object(m) => {
            let mut keys: Vec<&String> = m.keys().collect();
            keys.sort();
            format!("{{{}}}", keys.iter().map(|k| format!("{}: {}", py_str(k), py_json(&m[*k]))).collect::<Vec<_>>().join(", "))
        }
    }
}
fn py_str(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c if (c as u32) > 0x7f => { let mut b = [0u16; 2]; for u in c.encode_utf16(&mut b) { out.push_str(&format!("\\u{u:04x}")); } }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn build_profile(std: &str, version: Option<&str>, cobc_sha: Option<&str>, libcob_sha: Option<&str>) -> Value {
    let content = json!({
        "schema": "kobold-dialect-profile-v1",
        "profile_id": format!("gnucobol-{}-{}", version.unwrap_or("unknown"), std),
        "witness": {"compiler": "GnuCOBOL", "version": version.unwrap_or("unknown"), "cobc_sha256": cobc_sha, "libcob_sha256": libcob_sha},
        "dialect": {"std": std, "source_format": "free", "options": []},
        "oracle_status": "admitted_witness",
        "non_claims": ["NEG.DIALECT.IMPLICIT","NEG.DIALECT.GENERAL_COBOL","NEG.DIALECT.VENDOR_PARITY","NEG.DIALECT.RUNTIME_PORTABILITY","NEG.COBOL.NIST_CONFORMANCE","NEG.PLATFORM.RUNTIME_NOT_CLAIMED"]
    });
    let psha = sha_hex(py_json(&content).as_bytes());
    let mut full = content;
    full["profile_sha256"] = json!(psha);
    full
}

fn derive_default(root: &str) -> Option<Value> {
    if oracle_present(root) {
        return Some(build_profile("default", cobc_version(root).as_deref(),
            sha_file(&prefix(root).join("bin/cobc")).as_deref(),
            libcob_path(root).and_then(|p| sha_file(&p)).as_deref()));
    }
    let c: Value = serde_json::from_str(&std::fs::read_to_string(out_path(root)).ok()?).ok()?;
    Some(build_profile("default", c["witness"]["version"].as_str(), c["witness"]["cobc_sha256"].as_str(), c["witness"]["libcob_sha256"].as_str()))
}

pub fn run(cmd: &str, root: &str) -> i32 {
    match cmd {
        "generate" => match derive_default(root) {
            Some(p) => {
                let _ = std::fs::create_dir_all(out_path(root).parent().unwrap());
                let _ = std::fs::write(out_path(root), serde_json::to_vec_pretty(&p).unwrap_or_default());
                println!("dialect profile: {} profile_sha256 {}...", p["profile_id"].as_str().unwrap_or(""), &p["profile_sha256"].as_str().unwrap_or("")[..12.min(p["profile_sha256"].as_str().unwrap_or("").len())]);
                0
            }
            None => { println!("DIALECT.PROFILE.1: oracle absent and no committed profile -> cannot generate"); 2 }
        },
        "check" => {
            let op = out_path(root);
            if !op.exists() { println!("GATE: reports/dialect-profile/default.json missing"); return 1; }
            let c: Value = match serde_json::from_str(&std::fs::read_to_string(&op).unwrap_or_default()) { Ok(v) => v, Err(_) => { println!("GATE: profile unreadable"); return 1; } };
            let mut bad = 0;
            // 1. profile_sha256 self-consistency
            let mut body = c.clone();
            body.as_object_mut().unwrap().remove("profile_sha256");
            if sha_hex(py_json(&body).as_bytes()) != c["profile_sha256"].as_str().unwrap_or("") {
                println!("GATE: dialect profile_sha256 != recomputed (hand-edited or stale)"); bad += 1;
            }
            // 2. std explicit "default"
            if c["dialect"]["std"].as_str() != Some("default") { println!("GATE: default profile std is not 'default'"); bad += 1; }
            // 3. changing -std changes the hash
            let (v, cs, ls) = (c["witness"]["version"].as_str(), c["witness"]["cobc_sha256"].as_str(), c["witness"]["libcob_sha256"].as_str());
            if build_profile("default", v, cs, ls)["profile_sha256"] == build_profile("ibm-strict", v, cs, ls)["profile_sha256"] {
                println!("GATE: changing -std did NOT change profile_sha256"); bad += 1;
            }
            // 4. oracle present -> live match
            if oracle_present(root) {
                if c["witness"]["version"].as_str() != cobc_version(root).as_deref() { println!("GATE: committed dialect version != live cobc"); bad += 1; }
                if c["witness"]["cobc_sha256"].as_str() != sha_file(&prefix(root).join("bin/cobc")).as_deref() { println!("GATE: committed cobc_sha256 != live cobc binary"); bad += 1; }
            }
            if bad > 0 { println!("!! {bad} DIALECT.PROFILE.1 finding(s)"); return 1; }
            println!("DIALECT.PROFILE.1: profile {} self-consistent; -std binds the hash; witness {}", c["profile_id"].as_str().unwrap_or(""), if oracle_present(root) {"matches live oracle"} else {"pinned (oracle absent)"});
            0
        }
        _ => { eprintln!("usage: dialect generate|check"); 2 }
    }
}
