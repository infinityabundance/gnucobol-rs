//! TRUST.4.DOCS — generated documentation authority. Port of lab/docs/generate.py. Renders docs from
//! docs-src/<id>.model.json + machine evidence; check = regenerate-and-diff + version/legacy/court freshness.
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;

fn src(root: &str) -> std::path::PathBuf { Path::new(root).join("docs-src") }
fn sha_b(b: &[u8]) -> String { let mut s = String::new(); for x in Sha256::digest(b) { s.push_str(&format!("{x:02x}")); } s }
fn read_json(p: &Path) -> Value { std::fs::read_to_string(p).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(Value::Null) }

fn crate_version(root: &str) -> String {
    for ln in std::fs::read_to_string(Path::new(root).join("crates/gnucobol-rs/Cargo.toml")).unwrap_or_default().lines() {
        if ln.starts_with("version") { return ln.split('"').nth(1).unwrap_or("?").to_string(); }
    }
    "?".into()
}

fn oracle_str(root: &str) -> String {
    let mut recs: Vec<std::path::PathBuf> = std::fs::read_dir(Path::new(root).join("reports/receipts")).map(|rd| rd.flatten().map(|e| e.path().join("receipt.json")).filter(|p| p.exists()).collect()).unwrap_or_default();
    recs.sort();
    for r in recs {
        let v = read_json(&r)["oracle"]["version"].as_str().map(String::from);
        if let Some(v) = v { if !v.is_empty() { return v; } }
    }
    "GnuCOBOL 3.2.0".into()
}

fn keyf(id: &str) -> (String, i64) {
    let first = id.split('.').next().unwrap_or("").to_string();
    let last = id.rsplit('.').next().unwrap_or("");
    let num: String = last.chars().filter(|c| c.is_ascii_digit()).collect();
    (first, num.parse().unwrap_or(0))
}

fn verdict(root: &str, cid: &str) -> String {
    let jf = Path::new(root).join("reports/casefiles").join(cid).join("casefile.json");
    if !jf.exists() { return "❔ —".into(); }
    let v = read_json(&jf)["results"]["verdict"].as_str().unwrap_or("").to_string();
    format!("{} {}", if v == "pass" {"✅"} else {"❌"}, v)
}

fn court_table(root: &str, courts: &[Value], title: &str, with_links: bool) -> String {
    let mut out = vec![format!("### {title}"), String::new(), "| 🔒 court | name | verdict | casefile |".to_string(), "|---|---|:---:|---|".to_string()];
    let mut sorted: Vec<&Value> = courts.iter().collect();
    sorted.sort_by(|a, b| keyf(a["id"].as_str().unwrap_or("")).cmp(&keyf(b["id"].as_str().unwrap_or(""))));
    for c in sorted {
        let id = c["id"].as_str().unwrap_or("");
        let cf = if with_links { format!("[`reports/casefiles/{id}/`](reports/casefiles/{id}/)") } else { format!("`reports/casefiles/{id}/`") };
        out.push(format!("| `{}` | {} | {} | {} |", id, c["name"].as_str().unwrap_or(""), verdict(root, id), cf));
    }
    out.join("\n")
}

fn badge(label: &str, msg: &str, color: &str) -> String {
    let esc = |s: &str| s.replace('-', "--").replace(' ', "_");
    format!("![{label}](https://img.shields.io/badge/{}-{}-{color})", esc(label), esc(msg))
}

fn machine_values(root: &str) -> BTreeMap<String, String> {
    let cl = read_json(&Path::new(root).join("reports/claim-ladder.json"));
    let courts = cl["courts"].as_array().cloned().unwrap_or_default();
    let gnurust: Vec<Value> = courts.iter().filter(|c| c["id"].as_str().unwrap_or("").starts_with("GNURUST.")).cloned().collect();
    let kobold: Vec<Value> = courts.iter().filter(|c| c["id"].as_str().unwrap_or("").starts_with("KOBOLD.")).cloned().collect();
    let casefiles = std::fs::read_dir(Path::new(root).join("reports/casefiles")).map(|rd| rd.flatten().filter(|e| e.path().join("casefile.json").exists()).count()).unwrap_or(0);
    let neg_count = read_json(&Path::new(root).join("reports/negative-capabilities.json"))["count"].as_i64().unwrap_or(0);
    let sealed_byte = read_json(&Path::new(root).join("reports/kani-fuzz-coverage.json"))["impl_courts"].as_i64().unwrap_or(0);
    let badges = [
        badge("license", "LGPL-3.0-or-later", "blue"), badge("unsafe", "forbidden", "success"),
        badge("oracle", "GnuCOBOL 3.2", "orange"), badge("sealed courts", &courts.len().to_string(), "brightgreen"),
        badge("casefiles", &casefiles.to_string(), "blueviolet"),
    ].join(" ");
    let mut m = BTreeMap::new();
    m.insert("version".into(), crate_version(root));
    m.insert("gnurust_court_count".into(), gnurust.len().to_string());
    m.insert("kobold_court_count".into(), kobold.len().to_string());
    m.insert("court_count".into(), courts.len().to_string());
    m.insert("oracle".into(), oracle_str(root));
    m.insert("casefile_count".into(), casefiles.to_string());
    m.insert("neg_count".into(), neg_count.to_string());
    m.insert("sealed_byte_court_count".into(), sealed_byte.to_string());
    // live magnitude counts (so the README / crate READMEs never name a stale figure).
    let count_in = |sub: &str, pred: &dyn Fn(&str) -> bool| -> usize {
        std::fs::read_dir(Path::new(root).join(sub))
            .map(|rd| rd.flatten().filter(|e| e.file_name().to_str().map(pred).unwrap_or(false)).count())
            .unwrap_or(0)
    };
    let frontend_progs = count_in("lab/corpus/frontend", &|n| n.ends_with(".cob"));
    let sweep_count = count_in("lab/oracle", &|n| n.contains("sweep") && n.ends_with(".sh"));
    let dox = read_json(&Path::new(root).join("reports/doxygen-parity.json"));
    let dox_files = dox["files"].as_array().map(|a| a.len()).or_else(|| dox.as_array().map(|a| a.len())).unwrap_or(0);
    m.insert("frontend_program_count".into(), frontend_progs.to_string());
    m.insert("sweep_count".into(), sweep_count.to_string());
    m.insert("runtime_file_count".into(), dox_files.to_string());
    // whole-tree census (axis b of the three-axis claim ledger) -- from the SAME data as FILE-PARITY.md.
    let (tree_built, tree_total) = crate::cobol_parity::tree_census();
    m.insert("tree_built".into(), tree_built.to_string());
    m.insert("tree_total".into(), tree_total.to_string());
    m.insert("tree_built_pct".into(), format!("{:.0}", if tree_total == 0 { 0.0 } else { 100.0 * tree_built as f64 / tree_total as f64 }));
    m.insert("publish_note".into(), "(The git repo is the authority; crates.io may trail by a version under publish rate limits.)".into());
    m.insert("sealed_courts_table".into(), court_table(root, &courts, "Sealed courts (generated from `reports/claim-ladder.json`)", true));
    m.insert("gnurust_courts_table".into(), court_table(root, &gnurust, "Sealed GnuCOBOL data/arithmetic courts", false));
    m.insert("badges".into(), badges);
    m
}

fn header(model: &Value) -> String {
    let lp = &model["legacy_preservation"];
    let has_legacy = lp["legacy_paths"].as_array().map(|a| !a.is_empty()).unwrap_or(false);
    let (legacy_line, legacy_md) = if has_legacy {
        let p = lp["legacy_paths"][0].as_str().unwrap_or("");
        let s = lp["legacy_sha256"][0].as_str().unwrap_or("");
        (format!("Legacy source preserved (lossless) under {p} (sha256 {s})."), format!(" Legacy source preserved losslessly under `{p}`."))
    } else {
        ("No legacy source (born generated).".to_string(), " Born generated (no legacy source).".to_string())
    };
    format!("<!--\nDO NOT EDIT BY HAND. Generated by xtask docs from docs-src/{}.model.json\n+ machine evidence (crates/gnucobol-rs/Cargo.toml, reports/claim-ladder.json, reports/casefiles/*,\nreports/receipts/*). Regenerate: cargo run -p xtask -- docs generate. Check: cargo run -p xtask -- docs check.\nEvidence authority: the claim-ladder + generated casefiles. {legacy_line}\n-->\n\n# {}\n\n> _Generated document (TRUST.4.DOCS). Machine authority: `reports/claim-ladder.json` + `reports/casefiles/`.{legacy_md}_\n\n",
        model["document_id"].as_str().unwrap_or(""), model["title"].as_str().unwrap_or(""))
}

fn render(root: &str, model: &Value) -> String {
    let mut body = model["body_template"].as_str().unwrap_or("").to_string();
    for (k, v) in machine_values(root) {
        body = body.replace(&format!("{{{{{k}}}}}"), &v);
    }
    header(model) + &body + "\n"
}

fn models(root: &str) -> Vec<Value> {
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(src(root)).map(|rd| rd.flatten().map(|e| e.path()).filter(|p| p.file_name().and_then(|n| n.to_str()).map(|n| n.ends_with(".model.json")).unwrap_or(false)).collect()).unwrap_or_default();
    files.sort();
    files.iter().map(|f| read_json(f)).collect()
}

pub fn run(cmd: &str, root: &str) -> i32 {
    let ms = models(root);
    match cmd {
        "generate" => {
            for m in &ms {
                let _ = std::fs::write(Path::new(root).join(m["generated_path"].as_str().unwrap_or("")), render(root, m));
            }
            println!("generated {} authoritative doc(s) from docs-src/ models", ms.len());
            0
        }
        "check" => {
            let mut bad = 0;
            let cv = crate_version(root);
            let cl = read_json(&Path::new(root).join("reports/claim-ladder.json"));
            let cl_ids: Vec<String> = cl["courts"].as_array().map(|a| a.iter().filter_map(|c| c["id"].as_str().map(String::from)).collect()).unwrap_or_default();
            for m in &ms {
                let gp = Path::new(root).join(m["generated_path"].as_str().unwrap_or(""));
                if !gp.exists() { println!("NEG.DOC.LEGACY_NOT_REPLACED: {} not generated", m["generated_path"].as_str().unwrap_or("")); bad += 1; continue; }
                let actual = std::fs::read_to_string(&gp).unwrap_or_default();
                if actual != render(root, m) { println!("NEG.DOC.MANUAL_EDIT: {} != regenerated (hand-edited or stale)", m["generated_path"].as_str().unwrap_or("")); bad += 1; }
                let declares_version = m["machine_fields"].as_array().map(|a| a.iter().any(|x| x == "version")).unwrap_or(false);
                if declares_version && !actual.contains(&format!("gnucobol-rs {cv}")) { println!("NEG.DOC.STALE_VERSION: {} does not name current version {cv}", m["generated_path"].as_str().unwrap_or("")); bad += 1; }
                let lp = &m["legacy_preservation"];
                if lp.is_null() { println!("NEG.DOC.INFORMATION_LOSS: {} model lacks legacy_preservation", m["generated_path"].as_str().unwrap_or("")); bad += 1; }
                else if lp["legacy_paths"].as_array().map(|a| !a.is_empty()).unwrap_or(false) {
                    let lpath = Path::new(root).join(lp["legacy_paths"][0].as_str().unwrap_or(""));
                    if !lpath.exists() { println!("NEG.DOC.LEGACY_NOT_REPLACED: legacy source missing {}", lp["legacy_paths"][0].as_str().unwrap_or("")); bad += 1; }
                    else if !lp["legacy_sha256"].as_array().map(|a| a.iter().any(|s| s.as_str() == Some(&sha_b(&std::fs::read(&lpath).unwrap_or_default())))).unwrap_or(false) { println!("NEG.DOC.INFORMATION_LOSS: {} legacy_sha256 != actual", m["generated_path"].as_str().unwrap_or("")); bad += 1; }
                    if !lp["information_superset"].as_bool().unwrap_or(false) || lp["missing_items"].as_array().map(|a| !a.is_empty()).unwrap_or(false) { println!("NEG.DOC.INFORMATION_LOSS: {} not an information superset", m["generated_path"].as_str().unwrap_or("")); bad += 1; }
                }
                if m["document_id"] == "STATUS" {
                    for cid in &cl_ids {
                        if !actual.contains(cid) { println!("NEG.DOC.STALE_CLAIM: STATUS missing claim-ladder court {cid}"); bad += 1; }
                    }
                }
            }
            if bad > 0 { println!("!! {bad} TRUST.4.DOCS finding(s)"); return 1; }
            println!("TRUST.4.DOCS: all {} generated doc(s) current, version fresh, legacy preserved as superset", ms.len());
            0
        }
        _ => { eprintln!("usage: docs generate|check"); 2 }
    }
}
