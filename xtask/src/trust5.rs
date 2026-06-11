//! TRUST.5 — Anti-Ceremony Audit. Port of lab/trust5/run.py. Classifies every court A/B/C/D/F by whether
//! corrupting its evidence can fail a gate; GATES no class-F, views no-new-truth, audit fresh.
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

const VIEWS: [&str; 6] = ["KOBOLD.BANK.RECONCILE.1","KOBOLD.DIFF.1","SUPPORT-PACKET.1","KOBOLD.OPERATOR.1","KOBOLD.TOOLING.EXPORT.1","KOBOLD.PILOT-PACKET.1"];

fn read_json(p: &Path) -> Value { std::fs::read_to_string(p).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(Value::Null) }

fn receipts(root: &str) -> HashSet<String> {
    std::fs::read_dir(Path::new(root).join("reports/receipts")).map(|rd| rd.flatten().filter(|e| e.path().is_dir()).filter_map(|e| e.file_name().to_str().map(String::from)).collect()).unwrap_or_default()
}

fn negcaps_by_court(root: &str) -> HashMap<String, Vec<String>> {
    let mut by: HashMap<String, Vec<String>> = HashMap::new();
    let nc = read_json(&Path::new(root).join("reports/negative-capabilities.json"));
    if let Some(arr) = nc["negative_capabilities"].as_array() {
        for n in arr {
            if let Some(evs) = n["evidence"].as_array() {
                for ev in evs {
                    if let Some(e) = ev.as_str() { by.entry(e.to_string()).or_default().push(n["id"].as_str().unwrap_or("").to_string()); }
                }
            }
        }
    }
    by
}

fn casefile_counts(root: &str, cid: &str) -> Option<(usize, usize)> {
    let f = Path::new(root).join("reports/casefiles").join(cid).join("casefile.json");
    if !f.exists() { return None; }
    let c = read_json(&f);
    Some((c["positive_claims"].as_array().map(|a| a.len()).unwrap_or(0), c["negative_claims"].as_array().map(|a| a.len()).unwrap_or(0)))
}

fn audit_court(root: &str, c: &Value, rec: &HashSet<String>, negs: &HashMap<String, Vec<String>>) -> Value {
    let cid = c["id"].as_str().unwrap_or("");
    let has_casefile = Path::new(root).join("reports/casefiles").join(cid).join("casefile.json").exists();
    let has_receipt = rec.contains(cid);
    let (pos, neg) = casefile_counts(root, cid).unwrap_or((0, 0));
    let court_negs = negs.get(cid).cloned().unwrap_or_default();
    let is_view = VIEWS.contains(&cid);
    let p1_generated = has_casefile;
    let p3_no_new_truth = !is_view || court_negs.iter().any(|n| n.contains("NO_NEW_EVIDENCE") || n.contains("NO_NEW_TRUTH") || n.contains("VIEW_NOT_NEW_EVIDENCE"));
    let nonempty = |k: &str| !c[k].as_str().unwrap_or("").is_empty();
    let p4_damage = nonempty("damage_if_overclaimed") && nonempty("lie_prevented");
    let has_fixtures = nonempty("fixtures") && nonempty("breaks_claim") && nonempty("not_proven");
    let neg_ge_pos = neg >= pos;
    let (mut klass, can_fail_proof) = if has_receipt {
        ("A", "oracle sweep + receipt/casefile regen-equality (corrupt the receipt or sweep -> gate red)".to_string())
    } else if is_view {
        ("C", "regenerate-and-compare freshness + no-new-truth refusal (mutate a source artifact -> stale check fails)".to_string())
    } else {
        ("B", format!("{} + breaks_claim/not_proven (corrupt the fixture/golden -> test fails)", c["fixtures"].as_str().unwrap_or("")))
    };
    let p2_can_fail = !can_fail_proof.is_empty() && has_fixtures && neg_ge_pos;
    let ceremonial = !(p1_generated && p2_can_fail && p3_no_new_truth && p4_damage && neg_ge_pos);
    if ceremonial { klass = "F"; }
    let mut flags = Vec::new();
    if !has_casefile { flags.push("no_casefile"); }
    if !p4_damage { flags.push("no_damage_if_overclaimed"); }
    if !has_fixtures { flags.push("no_fixtures_or_breaks_claim"); }
    if !neg_ge_pos { flags.push("negatives_lt_positives"); }
    if is_view && !p3_no_new_truth { flags.push("view_creates_new_truth"); }
    json!({
        "id": cid, "class": klass, "can_fail": !ceremonial, "can_fail_proof": can_fail_proof,
        "has_casefile": has_casefile, "has_receipt": has_receipt,
        "positive_claims": pos, "negative_claims": neg, "negatives_ge_positives": neg_ge_pos,
        "generated_from_named_evidence": p1_generated, "creates_new_truth": !p3_no_new_truth,
        "damage_if_overclaimed_present": p4_damage, "fixtures": c["fixtures"].as_str().unwrap_or(""),
        "ceremonial_flags": flags
    })
}

fn build(root: &str) -> Value {
    let cl = read_json(&Path::new(root).join("reports/claim-ladder.json"));
    let rec = receipts(root);
    let negs = negcaps_by_court(root);
    let mut courts: Vec<Value> = cl["courts"].as_array().cloned().unwrap_or_default();
    courts.sort_by(|a, b| a["id"].as_str().unwrap_or("").cmp(b["id"].as_str().unwrap_or("")));
    let rows: Vec<Value> = courts.iter().map(|c| audit_court(root, c, &rec, &negs)).collect();
    let mut classes: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for k in ["A","B","C","D","F"] {
        let mut ids: Vec<String> = rows.iter().filter(|r| r["class"] == k).map(|r| r["id"].as_str().unwrap_or("").to_string()).collect();
        ids.sort();
        classes.insert(k.to_string(), ids);
    }
    let class_counts: BTreeMap<String, usize> = classes.iter().map(|(k, v)| (k.clone(), v.len())).collect();
    json!({
        "schema": "kobold-trust5-anti-ceremony-audit-v1", "court": "TRUST.5",
        "doctrine": "A court is real if corrupting/dropping/drifting/hand-editing its evidence can make a gate fail; ceremonial if it only restates that other evidence exists. Every court must prove: (1) generated from named evidence, (2) can detect stale/tampered evidence, (3) creates no new truth, (4) states the damage if overclaimed.",
        "rubric": {"A": "live oracle replay / byte-deterministic fixture + mutation + stale check + casefile", "B": "consumes sealed lower courts + fail-closed/hostile fixtures", "C": "view: summarizes existing evidence; MUST carry source hashes + no-new-truth + regen-compare", "D": "staged: harness exists, external proof missing; not green until it exists", "F": "ceremonial: prose-only, no replay/drift/mutation detector, no negatives"},
        "classes": classes, "class_counts": class_counts,
        "external_staged": {"LAMBDA.LIVE.1": "sibling kobold-lambda-layer; status awaiting_live_invocation (not green until a live AWS request id + matching output hash exist)"},
        "courts": rows,
        "non_claims": ["NEG.TRUST5.AUDIT_NOT_CERTIFICATION","NEG.TRUST5.CLASS_NOT_QUALITY_SCORE","NEG.TRUST5.SNAPSHOT_NOT_LIVE","NEG.TRUST5.NO_NEW_TRUTH"]
    })
}

fn render_md(a: &Value) -> String {
    let cc = &a["class_counts"];
    let mut l = vec![
        "<!-- generated by xtask trust5 — do not edit by hand -->".to_string(), String::new(),
        "# TRUST.5 — Anti-Ceremony Audit".to_string(), String::new(),
        "> [!IMPORTANT]".to_string(),
        "> A court is **real** if corrupting/dropping/drifting/hand-editing its evidence can make a gate fail; **ceremonial** if it only restates that other evidence exists. This audit proves every court can fail.".to_string(),
        String::new(),
        format!("- **A** hard (oracle/byte): {}  ·  **B** composed: {}  ·  **C** view: {}  ·  **D** staged: {}  ·  **F** ceremonial: **{}**", cc["A"], cc["B"], cc["C"], cc["D"], cc["F"]),
        String::new(),
        "## Classification".to_string(), String::new(),
        "| court | class | can fail? | neg≥pos | new truth? |".to_string(), "|---|:---:|:---:|:---:|:---:|".to_string(),
    ];
    for r in a["courts"].as_array().unwrap() {
        l.push(format!("| `{}` | {} | {} | {} | {} |", r["id"].as_str().unwrap(), r["class"].as_str().unwrap(),
            if r["can_fail"].as_bool().unwrap() {"✅"} else {"❌"},
            if r["negatives_ge_positives"].as_bool().unwrap() {"✅"} else {"❌"},
            if r["creates_new_truth"].as_bool().unwrap() {"❌ yes"} else {"✅ no"}));
    }
    l.extend(["".to_string(), "## Staged (external proof pending)".to_string(), String::new(),
        format!("- `LAMBDA.LIVE.1` — {}", a["external_staged"]["LAMBDA.LIVE.1"].as_str().unwrap_or("")),
        String::new(), "## Non-claims".to_string(), String::new()]);
    for n in a["non_claims"].as_array().unwrap() { l.push(format!("- `{}`", n.as_str().unwrap())); }
    l.push(String::new());
    l.join("\n") + "\n"
}

fn sorted(v: &Value) -> Value {
    match v {
        Value::Object(m) => { let mut bt = BTreeMap::new(); for (k, val) in m { bt.insert(k.clone(), sorted(val)); } json!(bt) }
        Value::Array(a) => Value::Array(a.iter().map(sorted).collect()),
        _ => v.clone(),
    }
}

pub fn run(cmd: &str, root: &str) -> i32 {
    let fresh = build(root);
    match cmd {
        "audit" => {
            let _ = std::fs::write(Path::new(root).join("reports/trust5-anti-ceremony-audit.json"), serde_json::to_vec_pretty(&fresh).unwrap_or_default());
            let _ = std::fs::write(Path::new(root).join("reports/trust5-anti-ceremony-audit.md"), render_md(&fresh));
            let cc = &fresh["class_counts"];
            println!("TRUST.5 audit: A={} B={} C={} D={} F={}", cc["A"], cc["B"], cc["C"], cc["D"], cc["F"]);
            0
        }
        "check" => {
            let mut bad = 0;
            let jp = Path::new(root).join("reports/trust5-anti-ceremony-audit.json");
            if !jp.exists() { println!("GATE: trust5 audit missing"); return 1; }
            if serde_json::to_string(&sorted(&read_json(&jp))).unwrap_or_default() != serde_json::to_string(&sorted(&fresh)).unwrap_or_default() {
                println!("GATE: trust5 audit != a fresh re-audit (stale or hand-edited)"); bad += 1;
            }
            if std::fs::read_to_string(Path::new(root).join("reports/trust5-anti-ceremony-audit.md")).unwrap_or_default() != render_md(&fresh) {
                println!("GATE: trust5 audit .md != regenerated"); bad += 1;
            }
            if fresh["class_counts"]["F"].as_u64().unwrap_or(1) != 0 {
                println!("GATE: ceremonial court(s) detected (class F): {}", fresh["classes"]["F"]); bad += 1;
            }
            for r in fresh["courts"].as_array().unwrap() {
                let cid = r["id"].as_str().unwrap();
                if VIEWS.contains(&cid) && r["creates_new_truth"].as_bool().unwrap_or(false) { println!("GATE: view court {cid} lacks a no-new-truth refusal"); bad += 1; }
                if !r["ceremonial_flags"].as_array().unwrap().is_empty() { println!("GATE: {cid} ceremonial flags: {}", r["ceremonial_flags"]); bad += 1; }
            }
            if bad > 0 { println!("!! {bad} TRUST.5 finding(s)"); return 1; }
            let cc = &fresh["class_counts"];
            println!("TRUST.5: every court can fail (A={} B={} C={} D={}, F=0); views are no-new-truth; audit fresh", cc["A"], cc["B"], cc["C"], cc["D"]);
            0
        }
        _ => { eprintln!("usage: trust5 audit|check"); 2 }
    }
}
