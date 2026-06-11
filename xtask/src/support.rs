//! SUPPORT-PACKET.1 — reviewer/operator evidence bundle gathered from existing generated artifacts (no new
//! truth). Port of lab/support/run.py. check = regenerate-and-diff (json + md).
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::Path;

fn out(root: &str) -> std::path::PathBuf { Path::new(root).join("reports/support-packet") }

fn sha(root: &str, rel: &str) -> Option<String> {
    let p = Path::new(root).join(rel);
    std::fs::read(&p).ok().map(|b| { let mut s = String::new(); for x in Sha256::digest(&b) { s.push_str(&format!("{x:02x}")); } s })
}

fn crate_version(root: &str) -> String {
    for ln in std::fs::read_to_string(Path::new(root).join("crates/gnucobol-rs/Cargo.toml")).unwrap_or_default().lines() {
        if ln.starts_with("version") { return ln.split('"').nth(1).unwrap_or("?").to_string(); }
    }
    "?".into()
}

/// relpath from reports/support-packet to a path relative to the repo root.
fn relpath(target: &str) -> String {
    let base = ["reports", "support-packet"];
    let tcomp: Vec<&str> = target.split('/').collect();
    let mut common = 0;
    while common < base.len() && common < tcomp.len() && base[common] == tcomp[common] { common += 1; }
    let ups = base.len() - common;
    let mut parts: Vec<String> = std::iter::repeat("..".to_string()).take(ups).collect();
    parts.extend(tcomp[common..].iter().map(|s| s.to_string()));
    parts.join("/")
}

fn gather(root: &str) -> Value {
    let committed = [
        ("status", "STATUS.md", "generated_doc"),
        ("changelog", "CHANGELOG.md", "generated_doc"),
        ("claim_ladder", "reports/claim-ladder.json", "machine_spine"),
        ("negative_capabilities", "reports/negative-capabilities.json", "refusal_registry"),
        ("dsse_verification", "reports/signing/verification-report.json", "attestation_report"),
        ("size_error_atlas", "reports/size-error-atlas.json", "observed_atlas"),
        ("truth_boundaries", "docs/truth-boundaries.md", "doctrine_doc"),
        ("future_risk_register", "docs/future-risk-register.md", "doctrine_doc"),
    ];
    let mut artifacts: Vec<Value> = Vec::new();
    for (aid, path, kind) in committed {
        let s = sha(root, path);
        artifacts.push(json!({"id": aid, "path": path, "kind": kind, "present": s.is_some(), "sha256": s}));
    }
    // casefile index
    let mut cases = Vec::new();
    let cdir = Path::new(root).join("reports/casefiles");
    let mut dirs: Vec<_> = std::fs::read_dir(&cdir).map(|rd| rd.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect::<Vec<_>>()).unwrap_or_default();
    dirs.sort();
    for d in dirs {
        let court = d.file_name().unwrap().to_string_lossy().to_string();
        cases.push(json!({
            "court": court, "casefile_sha256": sha(root, &format!("reports/casefiles/{court}/casefile.json")),
            "sarif": d.join("sarif.json").exists(), "intoto": d.join("intoto-statement.json").exists(), "dsse": d.join("dsse-envelope.json").exists()
        }));
    }
    artifacts.push(json!({"id": "casefile_index", "kind": "casefile_index", "present": !cases.is_empty(), "count": cases.len(), "casefiles": cases}));
    // release index
    let mut rels: Vec<String> = std::fs::read_dir(Path::new(root).join("reports/releases")).map(|rd| rd.flatten().map(|e| e.path()).filter(|p| p.is_dir()).filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(String::from)).collect()).unwrap_or_default();
    rels.sort();
    artifacts.push(json!({"id": "release_packets", "kind": "release_index", "present": !rels.is_empty(), "packets": rels}));
    // runtime pointers
    let runtime = [
        ("bench_receipt", "kobold-bench reports/BENCH-2-receipt.json", "kobold-bench2 --features rayon"),
        ("scale_receipt", "kobold-bench reports/SCALE-1-receipt-*.json", "kobold-scale 1g"),
        ("bank_reconcile_report", "kobold-bank-reconcile-report-v1", "bank_reconcile_report() at runtime"),
        ("diff_report", "kobold-diff-report-v1", "diff_artifacts() at runtime"),
        ("redaction_policy", "kobold-redaction-policy-v1", "operator-declared"),
        ("currency_manifest", "kobold-currency-manifest-v1", "currency_validate() at runtime"),
        ("date_manifest", "kobold-date-manifest-v1", "date_validate() at runtime"),
        ("sentinel_manifest", "kobold-sentinel-manifest-v1", "sentinel_scan() at runtime"),
    ];
    for (aid, pointer, how) in runtime {
        artifacts.push(json!({"id": aid, "kind": "runtime_or_operator", "present": false, "pointer": pointer, "produced_by": how}));
    }
    json!({
        "schema": "kobold-support-packet-v1", "court": "SUPPORT-PACKET.1",
        "generated_from": "existing generated artifacts only; creates no new truth",
        "crate_versions": {"gnucobol-rs": format!("{} (this repo)", crate_version(root)), "kobold-data-shim": "sibling repo (see its Cargo.toml/crates.io)", "kobold-bench": "sibling repo (bench/scale receipts)", "kobold-attest": "external repo: github.com/infinityabundance/kobold-attest (Apache-2.0; cargo install kobold-attest)"},
        "truth_boundary_summary": "bytes < record < transform < custody < reconciliation evidence < privacy-preserved < generated attestation; REFUSED above: posting, ledger, settlement, account-balance, business truth, production readiness, customer-workload representativeness, regulatory compliance.",
        "artifacts": artifacts,
        "non_claims": ["NEG.SUPPORT.NO_NEW_TRUTH","NEG.SUPPORT.NOT_CERTIFICATION","NEG.SUPPORT.NOT_COMPLIANCE","NEG.SUPPORT.NOT_PRODUCTION_APPROVAL","NEG.SUPPORT.NOT_CUSTOMER_ACCEPTANCE","NEG.SUPPORT.SNAPSHOT_NOT_LIVE"]
    })
}

fn render_md(pk: &Value, root: &str) -> String {
    let arts = pk["artifacts"].as_array().unwrap();
    let present = arts.iter().filter(|a| a["present"].as_bool().unwrap_or(false)).count();
    let count = arts.get(8).map(|a| a["count"].to_string()).unwrap_or("?".into());
    let mut l = vec![
        "<!-- generated by xtask support — do not edit by hand -->".to_string(), String::new(),
        "# Support packet (SUPPORT-PACKET.1)".to_string(), String::new(),
        "> [!IMPORTANT]".to_string(),
        "> A reviewer/operator evidence bundle gathered from **existing generated artifacts**. It creates **no** new truth, certification, compliance, production approval, or customer acceptance.".to_string(),
        String::new(),
        format!("- crate (this repo): `gnucobol-rs {}`", crate_version(root)),
        format!("- artifacts gathered: **{present}** committed + pointers to runtime/operator artifacts"),
        format!("- casefiles: **{count}**"), String::new(),
        "## Truth boundary".to_string(), String::new(), format!("> {}", pk["truth_boundary_summary"].as_str().unwrap_or("")), String::new(),
        "## Committed governance artifacts".to_string(), String::new(), "| id | path | sha256 |".to_string(), "|---|---|---|".to_string(),
    ];
    for a in arts {
        if a["present"].as_bool().unwrap_or(false) {
            if let Some(s) = a["sha256"].as_str() {
                let path = a["path"].as_str().unwrap();
                l.push(format!("| `{}` | [`{}`]({}) | `{}…` |", a["id"].as_str().unwrap(), path, relpath(path), &s[..16.min(s.len())]));
            }
        }
    }
    l.extend(["".to_string(), "## Runtime / operator artifacts (pointers — not embedded)".to_string(), String::new(), "| id | produced by |".to_string(), "|---|---|".to_string()]);
    for a in arts {
        if a["kind"] == "runtime_or_operator" {
            l.push(format!("| `{}` | {} |", a["id"].as_str().unwrap(), a["produced_by"].as_str().unwrap()));
        }
    }
    l.extend(["".to_string(), "## Non-claims".to_string(), String::new()]);
    for n in pk["non_claims"].as_array().unwrap() { l.push(format!("- `{}`", n.as_str().unwrap())); }
    l.push(String::new());
    l.join("\n") + "\n"
}

fn sorted(v: &Value) -> Value {
    match v {
        Value::Object(m) => { let mut bt = std::collections::BTreeMap::new(); for (k, val) in m { bt.insert(k.clone(), sorted(val)); } json!(bt) }
        Value::Array(a) => Value::Array(a.iter().map(sorted).collect()),
        _ => v.clone(),
    }
}

pub fn run(cmd: &str, root: &str) -> i32 {
    let fresh = gather(root);
    match cmd {
        "generate" => {
            let _ = std::fs::create_dir_all(out(root));
            let _ = std::fs::write(out(root).join("support-packet.json"), serde_json::to_vec_pretty(&fresh).unwrap_or_default());
            let _ = std::fs::write(out(root).join("support-packet.md"), render_md(&fresh, root));
            println!("support packet: {} committed artifacts + runtime pointers", fresh["artifacts"].as_array().unwrap().iter().filter(|a| a["present"].as_bool().unwrap_or(false)).count());
            0
        }
        "check" => {
            let jp = out(root).join("support-packet.json");
            if !jp.exists() { println!("GATE: support-packet.json missing"); return 1; }
            let on_disk: Value = serde_json::from_str(&std::fs::read_to_string(&jp).unwrap_or_default()).unwrap_or(Value::Null);
            if serde_json::to_string(&sorted(&on_disk)).unwrap_or_default() != serde_json::to_string(&sorted(&fresh)).unwrap_or_default() {
                println!("GATE: support-packet.json != a fresh re-gather (stale or hand-edited)"); return 1;
            }
            if std::fs::read_to_string(out(root).join("support-packet.md")).unwrap_or_default() != render_md(&fresh, root) {
                println!("GATE: support-packet.md != regenerated"); return 1;
            }
            println!("SUPPORT-PACKET.1: bundle fresh ({} committed artifacts)", fresh["artifacts"].as_array().unwrap().iter().filter(|a| a["present"].as_bool().unwrap_or(false)).count());
            0
        }
        _ => { eprintln!("usage: support generate|check"); 2 }
    }
}
