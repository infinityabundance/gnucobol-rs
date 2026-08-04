//! TRUST.2 — generated receipts. Port of lab/receipt/run.py. For each campaign, runs its sweep LIVE and
//! emits reports/receipts/<CAMPAIGN>/{receipt.json,receipt.md}. check regenerates in memory and FAILs on
//! evidence drift / hand-edited .md / missing receipt.
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn prefix(root: &str) -> PathBuf {
    Path::new(root).join("lab/oracle/prefix")
}
fn recdir(root: &str) -> PathBuf {
    Path::new(root).join("reports/receipts")
}
fn manifest(root: &str) -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(Path::new(root).join("lab/receipt/manifest.json"))
            .unwrap_or_default(),
    )
    .unwrap_or(Value::Null)
}

fn crate_version(root: &str) -> String {
    for ln in std::fs::read_to_string(Path::new(root).join("crates/gnucobol-rs/Cargo.toml"))
        .unwrap_or_default()
        .lines()
    {
        if ln.starts_with("version") {
            return ln.split('"').nth(1).unwrap_or("?").to_string();
        }
    }
    "?".into()
}

fn oracle_version(root: &str) -> String {
    let cobc = prefix(root).join("bin/cobc");
    if !cobc.exists() {
        return "not-built".into();
    }
    match std::process::Command::new(&cobc)
        .arg("--version")
        .env("LD_LIBRARY_PATH", prefix(root).join("lib"))
        .output()
    {
        Ok(o) => String::from_utf8_lossy(&o.stdout)
            .lines()
            .next()
            .unwrap_or("error")
            .to_string(),
        Err(_) => "error".into(),
    }
}

fn dialect_ref(root: &str) -> Value {
    let dp = Path::new(root).join("reports/dialect-profile/default.json");
    match std::fs::read_to_string(&dp)
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
    {
        Some(p) => {
            json!({"dialect_profile_id": p["profile_id"], "dialect_profile_sha256": p["profile_sha256"]})
        }
        None => json!({}),
    }
}

fn run_sweep(root: &str, script: &str, arg: Option<&str>) -> String {
    let path = Path::new(root).join("lab/oracle").join(script);
    if !prefix(root).join("bin/cobc").exists() || !path.exists() {
        return "oracle-not-built".into();
    }
    let mut cmd = std::process::Command::new("bash");
    cmd.arg(&path);
    if let Some(a) = arg {
        cmd.arg(a);
    }
    cmd.current_dir(root);
    if let Ok(o) = cmd.output() {
        for line in String::from_utf8_lossy(&o.stdout).lines() {
            if line.starts_with("PASS=") && line.contains("FAIL=") {
                return line.trim().to_string();
            }
        }
    }
    "no-result".into()
}

fn build(root: &str, code: &str, stamp: &str, git_commit: &str) -> Value {
    let man = manifest(root);
    let m = &man["campaigns"][code];
    let result = run_sweep(root, m["sweep"].as_str().unwrap_or(""), m["arg"].as_str());
    // FAIL-token-aware (not ends_with) so a sweep whose terminal line carries extra counters after FAIL=0
    // (e.g. `PASS=.. FAIL=0 SKIP=.. MATCH=..`) still verdicts correctly. Backward-compatible: every existing
    // clean line contains the token `FAIL=0`.
    let fail_clean = result.split_whitespace().any(|t| t == "FAIL=0");
    let mut verdict = if fail_clean {
        "pass"
    } else if result.contains("not-built") {
        "oracle-not-built"
    } else {
        "fail"
    };
    // RATCHET: if the manifest sets `min_match`, the sweep's `MATCH=` count must not drop below it. MATCH can
    // only ever rise; raising the floor is a deliberate `min_match` bump + regenerate (never a silent drift).
    if verdict == "pass" {
        if let Some(min) = m.get("min_match").and_then(|v| v.as_i64()) {
            let got = result
                .split_whitespace()
                .find_map(|t| t.strip_prefix("MATCH="))
                .and_then(|n| n.parse::<i64>().ok());
            if matches!(got, Some(g) if g < min) {
                verdict = "fail";
            }
        }
    }
    let mut oracle = json!({"name": "GnuCOBOL", "version": oracle_version(root)});
    if let Some(dr) = dialect_ref(root).as_object() {
        for (k, v) in dr {
            oracle[k] = v.clone();
        }
    }
    let replay = format!(
        "bash lab/oracle/{}{}",
        m["sweep"].as_str().unwrap_or(""),
        m["arg"]
            .as_str()
            .map(|a| format!(" {a}"))
            .unwrap_or_default()
    );
    json!({
        "schema": "gnurust-replay-receipt-v1", "campaign": code, "court": m["court"],
        "generated_at": stamp, "git_commit": git_commit, "crate_version": crate_version(root),
        "oracle": oracle, "command": {"replay": replay}, "byte_domain": m["byte_domain"],
        "results": {"sweep": result}, "non_claims": m["non_claims"], "verdict": verdict,
        "receipt_status": m.get("receipt_status").cloned().unwrap_or(json!("current")),
        "superseded_by": m.get("superseded_by").cloned().unwrap_or(Value::Null),
        "current_authority": "STATUS.md"
    })
}

fn evidence(r: &Value) -> Value {
    json!({
        "campaign": r["campaign"], "court": r["court"], "crate_version": r["crate_version"],
        "command": r["command"], "byte_domain": r["byte_domain"], "results": r["results"],
        "non_claims": r["non_claims"], "verdict": r["verdict"], "receipt_status": r["receipt_status"],
        "superseded_by": r["superseded_by"], "oracle_version": r["oracle"]["version"]
    })
}

fn render_md(r: &Value) -> String {
    let nc = r["non_claims"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|n| format!("- {}", n.as_str().unwrap_or("")))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();
    let sup = r["superseded_by"]
        .as_str()
        .map(|s| format!(" (superseded_by {s})"))
        .unwrap_or_default();
    format!("<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.\n     Regenerate: cargo run -p xtask -- receipt generate -->\n# {} — {}\n\n**Verdict: {}** · replay `{}`\n\n| field | value |\n|-------|-------|\n| campaign | `{}` |\n| court | {} |\n| crate_version | `{}` |\n| oracle | {} |\n| byte_domain | {} |\n| replay command | `{}` |\n| generated_at | {} |\n| git_commit | `{}` |\n| receipt_status | {}{} |\n\n## Non-claims\n{}\n\n> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is\n> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with\n> `cargo run -p xtask -- receipt generate`.\n",
        r["campaign"].as_str().unwrap_or(""), r["court"].as_str().unwrap_or(""),
        r["verdict"].as_str().unwrap_or("").to_uppercase(), r["results"]["sweep"].as_str().unwrap_or(""),
        r["campaign"].as_str().unwrap_or(""), r["court"].as_str().unwrap_or(""), r["crate_version"].as_str().unwrap_or(""),
        r["oracle"]["version"].as_str().unwrap_or(""), r["byte_domain"].as_str().unwrap_or(""), r["command"]["replay"].as_str().unwrap_or(""),
        r["generated_at"].as_str().unwrap_or(""), r["git_commit"].as_str().unwrap_or(""), r["receipt_status"].as_str().unwrap_or(""), sup, nc)
}

/// Courts whose receipts are generated by their OWN dedicated harness, not by the xtask sweep
/// runner. The CCVS85 courts run a Docker-isolated full corpus pipeline (`gnucobol-rs-ccvs85
/// receipts-finalize`) whose evidence cannot be reproduced by `bash lab/oracle/<sweep>`; they stay
/// in the manifest for documentation but the xtask flow must not try to re-run their sweep.
const NON_XTASK_COURTS: [&str; 6] = [
    "GNURUST.CCVS85.2",
    "GNURUST.CCVS85.3",
    "GNURUST.CCVS85.4",
    "GNURUST.GNUCOBOL-TESTSUITE.1",
    "GNURUST.GNUCOBOL-TESTSUITE.2",
    "GNURUST.GNUCOBOL-TESTSUITE.3",
];

fn campaigns(root: &str) -> Vec<String> {
    manifest(root)["campaigns"]
        .as_object()
        .map(|m| {
            m.keys()
                .cloned()
                .filter(|k| !NON_XTASK_COURTS.contains(&k.as_str()))
                .collect()
        })
        .unwrap_or_default()
}

pub fn run(cmd: &str, root: &str) -> i32 {
    match cmd {
        "render-md" => {
            // re-render every receipt.md from its COMMITTED receipt.json (no sweeps) -- for the one-time
            // tool-reference text update.
            let mut n = 0;
            for code in campaigns(root) {
                let jf = recdir(root).join(&code).join("receipt.json");
                if let Some(r) = std::fs::read_to_string(&jf)
                    .ok()
                    .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                {
                    let _ =
                        std::fs::write(recdir(root).join(&code).join("receipt.md"), render_md(&r));
                    n += 1;
                }
            }
            println!("re-rendered {n} receipt.md from committed receipt.json");
            0
        }
        "generate" => {
            for code in campaigns(root) {
                let r = build(root, &code, "unstamped", "unstamped");
                let d = recdir(root).join(&code);
                let _ = std::fs::create_dir_all(&d);
                let _ = std::fs::write(
                    d.join("receipt.json"),
                    serde_json::to_vec_pretty(&r).unwrap_or_default(),
                );
                let _ = std::fs::write(d.join("receipt.md"), render_md(&r));
            }
            println!(
                "generated {} receipts in reports/receipts/",
                campaigns(root).len()
            );
            0
        }
        "check" => {
            let mut bad = 0;
            for code in campaigns(root) {
                let d = recdir(root).join(&code);
                let jf = d.join("receipt.json");
                let committed = match std::fs::read_to_string(&jf)
                    .ok()
                    .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                {
                    Some(c) => c,
                    None => {
                        println!("DRIFT: {code} has no generated receipt.json");
                        bad += 1;
                        continue;
                    }
                };
                let fresh = build(
                    root,
                    &code,
                    committed["generated_at"].as_str().unwrap_or(""),
                    committed["git_commit"].as_str().unwrap_or(""),
                );
                if evidence(&committed) != evidence(&fresh) {
                    println!("DRIFT: {code} receipt evidence != live replay (regenerate)");
                    bad += 1;
                }
                if std::fs::read_to_string(d.join("receipt.md")).unwrap_or_default()
                    != render_md(&committed)
                {
                    println!("DRIFT: {code} receipt.md hand-edited");
                    bad += 1;
                }
            }
            // claim-ladder must only cite campaigns that have a generated receipt
            let cl: Value = serde_json::from_str(
                &std::fs::read_to_string(Path::new(root).join("reports/claim-ladder.json"))
                    .unwrap_or_default(),
            )
            .unwrap_or(Value::Null);
            let camps = campaigns(root);
            if let Some(courts) = cl["courts"].as_array() {
                for c in courts {
                    let cid = c["id"].as_str().unwrap_or("");
                    if cid.starts_with("GNURUST.")
                        && camps.contains(&cid.to_string())
                        && !recdir(root).join(cid).join("receipt.json").exists()
                    {
                        println!("DRIFT: claim-ladder cites {cid} with no generated receipt");
                        bad += 1;
                    }
                }
            }
            if bad > 0 {
                println!("!! {bad} receipt drift(s)");
                return 1;
            }
            println!("receipts: all current, .md == render(.json), claim-ladder covered");
            0
        }
        _ => {
            eprintln!("usage: receipt generate|check|render-md");
            2
        }
    }
}
