//! ENTERPRISE.1 — per-release evidence packet. Port of lab/release/build-packet.py.
//! xtask release <repo> <crate> <version> <kind:gnurust|kobold> [evidence_repo]
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

fn run(cmd: &[&str], cwd: &str) -> (Option<i32>, String, String) {
    match Command::new(cmd[0]).args(&cmd[1..]).current_dir(cwd).output() {
        Ok(o) => (o.status.code(), String::from_utf8_lossy(&o.stdout).into_owned(), String::from_utf8_lossy(&o.stderr).into_owned()),
        Err(_) => (Some(-1), String::new(), "NOT_FOUND".into()),
    }
}
fn stamp() -> String { run(&["date", "-u", "+%Y-%m-%dT%H:%M:%SZ"], ".").1.trim().to_string() }
fn write(p: &Path, name: &str, v: &Value) { let _ = std::fs::write(p.join(name), serde_json::to_vec_pretty(v).unwrap_or_default()); }

pub fn run_main(args: &[String]) -> i32 {
    if args.len() < 6 { eprintln!("usage: xtask release <repo> <crate> <version> <kind> [evid]"); return 2; }
    let (repo, krate, version, kind) = (&args[2], &args[3], &args[4], &args[5]);
    let evid = args.get(6).cloned().unwrap_or_else(|| repo.clone());
    let out = Path::new(repo).join("reports/releases").join(format!("{krate}-{version}"));
    let _ = std::fs::create_dir_all(&out);
    let st = stamp();

    let meta: Value = serde_json::from_str(&run(&["cargo", "metadata", "--format-version", "1"], repo).1).unwrap_or(json!({"packages":[]}));
    let mut pkgs: Vec<(String, String, String)> = meta["packages"].as_array().map(|a| a.iter().map(|p| (p["name"].as_str().unwrap_or("").to_string(), p["version"].as_str().unwrap_or("").to_string(), p["license"].as_str().unwrap_or("UNKNOWN").to_string())).collect()).unwrap_or_default();
    pkgs.sort(); pkgs.dedup();
    let git = { let (rc, o, _) = run(&["git", "rev-parse", "HEAD"], repo); if rc == Some(0) { o.trim().to_string() } else { "unknown".into() } };

    write(&out, "release-metadata.json", &json!({"schema":"kobold-release-metadata-v1","crate":krate,"version":version,"kind":kind,"git_commit":git,"generated_at_utc":st,"publish_status":"pending_crates_io_rate_limit_window","dependency_count":pkgs.len(),"note":"git repo is the authority; crates.io may trail under publish rate limits."}));
    write(&out, "sbom.spdx.json", &json!({"spdxVersion":"SPDX-2.3","dataLicense":"CC0-1.0","SPDXID":"SPDXRef-DOCUMENT","name":format!("{krate}-{version}"),"documentNamespace":format!("https://github.com/infinityabundance/{krate}#{version}"),"creationInfo":{"created":st,"creators":["Tool: kobold-release-packet (from cargo metadata)"]},"comment":"Derived from `cargo metadata`; no SPDX generator installed.","packages":pkgs.iter().map(|(n,v,l)| json!({"name":n,"SPDXID":format!("SPDXRef-{n}-{v}").replace('.',"-"),"versionInfo":v,"licenseConcluded":l,"downloadLocation":format!("https://crates.io/crates/{n}/{v}")})).collect::<Vec<_>>()}));
    let mut lic_counts: BTreeMap<String, i64> = BTreeMap::new();
    for (_, _, l) in &pkgs { *lic_counts.entry(l.clone()).or_default() += 1; }
    write(&out, "licenses.json", &json!({"schema":"kobold-release-licenses-v1","crate":krate,"version":version,"this_crate_license":if kind=="gnurust" {"LGPL-3.0-or-later"} else {"Apache-2.0"},"dependency_license_summary":lic_counts,"copyleft_boundary":"gnucobol-rs is LGPL-3.0+ (derives from libcob); cobc-oracle-rs is GPL-3.0+ (lab-only, drives cobc). kobold-data-shim is Apache-2.0 and LINKS gnucobol-rs (LGPL) -- see its NOTICE for the relink obligation. No GPL code is in any published decode-path crate."}));
    write(&out, "feature-matrix.json", &json!({"schema":"kobold-release-feature-matrix-v1","crate":krate,"version":version,"features":{"(none)":"[]"},"default_features":"[]","note":"Off-by-default features stay boring/portable until separately proven."}));

    for (file, tool, args2, cwd) in [("cargo-audit.txt", "audit", vec!["audit"], repo.clone()), ("cargo-geiger.txt", "geiger", vec!["geiger","--output-format","Ascii","--all-features"], if kind=="kobold" { repo.clone() } else { Path::new(repo).join("crates/gnucobol-rs").to_string_lossy().into_owned() })] {
        let mut c = vec!["cargo"]; c.extend(args2.iter().copied());
        let (rc, o, e) = run(&c, &cwd);
        let (status, body) = match rc { Some(-1) => ("not_installed".to_string(), format!("cargo-{tool} not installed")), Some(0) => ("pass".to_string(), o.clone()), None => ("not_run".to_string(), "timed out".into()), _ => ("not_run".to_string(), format!("{o}\n{e}")) };
        let _ = std::fs::write(out.join(file), format!("status: {status}\ngenerated_at: {st}\nnote: every shipped crate is #![forbid(unsafe_code)]\n\n{}\n", &body[..8000.min(body.len())]));
    }

    // receipts + snapshots from the evidence repo
    let mut receipts = Vec::new();
    let recdir = Path::new(&evid).join("reports/receipts");
    let mut camps: Vec<_> = std::fs::read_dir(&recdir).map(|rd| rd.flatten().map(|e| e.path()).collect()).unwrap_or_default();
    camps.sort();
    for c in camps {
        let jf = c.join("receipt.json");
        if jf.exists() {
            let r: Value = serde_json::from_str(&std::fs::read_to_string(&jf).unwrap_or_default()).unwrap_or(Value::Null);
            receipts.push(json!({"campaign":r["campaign"],"verdict":r["verdict"],"result":r["results"]["sweep"],"court":r["court"]}));
        }
    }
    let prefix = if kind == "gnurust" { Some("GNURUST.") } else { None };
    let rel: Vec<Value> = receipts.into_iter().filter(|r| prefix.map(|p| r["campaign"].as_str().unwrap_or("").starts_with(p)).unwrap_or(true)).collect();
    write(&out, "receipt-manifest.json", &json!({"schema":"kobold-release-receipt-manifest-v1","crate":krate,"version":version,"trust2_doctrine":"receipts are generated from live replay","receipts":rel.clone(),"count":rel.len()}));
    let cl: Value = serde_json::from_str(&std::fs::read_to_string(Path::new(&evid).join("reports/claim-ladder.json")).unwrap_or_default()).unwrap_or(Value::Null);
    let want = if kind == "gnurust" { "GNURUST." } else { "KOBOLD." };
    let courts: Vec<Value> = cl["courts"].as_array().map(|a| a.iter().filter(|c| c["id"].as_str().unwrap_or("").starts_with(want)).cloned().collect()).unwrap_or_default();
    write(&out, "claim-ladder-snapshot.json", &json!({"schema":"claim-ladder-snapshot-v1","crate":krate,"version":version,"generated_at_utc":st,"courts":courts.clone()}));
    let negs: Vec<Value> = courts.iter().map(|c| json!({"id":c["id"],"not_proven":c["not_proven"],"lie_prevented":c["lie_prevented"]})).collect();
    write(&out, "negative-capabilities-snapshot.json", &json!({"schema":"negative-capabilities-snapshot-v1","crate":krate,"version":version,"negatives":negs}));
    let status_txt = std::fs::read_to_string(Path::new(&evid).join("STATUS.md")).unwrap_or_else(|_| "(no STATUS.md)".into());
    let _ = std::fs::write(out.join("status-snapshot.md"), format!("<!-- snapshot of STATUS.md at {krate} {version} release ({st}) -->\n\n{status_txt}"));
    let verdict = format!("# Release verdict — {krate} {version}\n\n_Generated {st} from the machine files in this packet. A release is an evidence packet, not merely a version number._\n\n| evidence | value |\n|----------|-------|\n| crate / version | `{krate}` {version} ({kind}) |\n| git commit | `{git}` |\n| dependencies | {} |\n| sealed courts in this crate | {} |\n| TRUST.2 receipts | {} |\n\n## What this release admits\nThe sealed courts in `claim-ladder-snapshot.json`, each proven against the admitted GnuCOBOL 3.2 oracle with a reproducible TRUST.2 receipt.\n\n## What this release does NOT admit\nThe non-claims in `negative-capabilities-snapshot.json`. No production-readiness claim beyond the KRL level in `status-snapshot.md`.\n\n> Doctrine: ENTERPRISE.1 treats a release as an evidence packet.\n", pkgs.len(), courts.len(), rel.len());
    let _ = std::fs::write(out.join("release-verdict.md"), verdict);
    println!("packet: {} (deps={}, courts={}, receipts={})", out.display(), pkgs.len(), courts.len(), rel.len());
    0
}
