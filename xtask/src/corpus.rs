//! GNURUST.PUBLIC.CORPUS.1 — admitted public-COBOL corpus index (gap discovery, index-only). Port of
//! lab/corpus/run.py. The curated index is the source of truth (embedded as data); the logic re-derives
//! counts/tiers/best-first ordering and validates.
use serde_json::{json, Value};
use std::path::Path;

const DATA: &str = include_str!("corpus_data.json");

fn build() -> Value {
    let d: Value = serde_json::from_str(DATA).unwrap_or(Value::Null);
    let corpora_raw = d["corpora"].as_array().cloned().unwrap_or_default();
    let tier = |t: i64| corpora_raw.iter().filter(|c| c["tier"].as_i64() == Some(t)).count();
    // best-first: corpora with a priority, sorted by priority
    let mut bf: Vec<(i64, &str)> = corpora_raw.iter()
        .filter_map(|c| c["priority"].as_i64().map(|p| (p, c["id"].as_str().unwrap_or(""))))
        .collect();
    bf.sort_by_key(|x| x.0);
    let best_first: Vec<&str> = bf.iter().map(|x| x.1).collect();
    let corpora: Vec<Value> = corpora_raw.iter().map(|c| json!({
        "id": c["id"], "name": c["name"], "url": c["url"], "tier": c["tier"], "priority": c["priority"],
        "features": c["features"], "note": c["note"],
        "license": "unverified-at-index-time", "commit": "unverified-at-index-time", "status": "indexed"
    })).collect();
    json!({
        "schema": "gnurust-public-corpus-index-v1", "court": "GNURUST.PUBLIC.CORPUS.1",
        "doctrine": d["doctrine"],
        "corpus_count": corpora.len(),
        "tiers": {"1_gnucobol_runnable": tier(1), "2_mainframe_realistic": tier(2), "3_research_mining": tier(3)},
        "best_first_10": best_first,
        "corpora": corpora,
        "future_campaigns": d["future_campaigns"],
        "behavioral_parity_ladder": d["behavioral_parity_ladder"],
        "negative_capabilities": ["NEG.PUBLIC_CORPUS.NOT_PARITY","NEG.PUBLIC_CORPUS.NOT_FETCHED","NEG.PUBLIC_CORPUS.NOT_RUN","NEG.PUBLIC_CORPUS.LICENSE_NOT_LEGAL_ADVICE","NEG.PUBLIC_CORPUS.FEATURES_DECLARED_NOT_VERIFIED","NEG.PUBLIC_CORPUS.NOT_EXHAUSTIVE"]
    })
}

pub fn run(cmd: &str, root: &str) -> i32 {
    let b = build();
    match cmd {
        "generate" => {
            let _ = std::fs::write(Path::new(root).join("reports/public-corpus-index.json"), serde_json::to_vec_pretty(&b).unwrap_or_default());
            println!("public corpus index: {} corpora; best-first {}", b["corpus_count"], b["best_first_10"].as_array().unwrap().len());
            0
        }
        "check" => {
            let mut bad = 0;
            let mut seen = std::collections::HashSet::new();
            for c in b["corpora"].as_array().unwrap() {
                let id = c["id"].as_str().unwrap_or("");
                if !seen.insert(id.to_string()) {
                    println!("GATE: duplicate corpus id {id}");
                    bad += 1;
                }
                if c["url"].as_str().unwrap_or("").is_empty() || c["features"].as_array().map(|a| a.is_empty()).unwrap_or(true) {
                    println!("GATE: corpus row incomplete {id}");
                    bad += 1;
                }
            }
            if b["best_first_10"].as_array().unwrap().len() != 10 {
                println!("GATE: best_first_10 is not 10");
                bad += 1;
            }
            if bad > 0 {
                println!("!! {bad} corpus-index finding(s)");
                return 1;
            }
            println!("GNURUST.PUBLIC.CORPUS.1: {} corpora indexed, {} prioritized; index-only (not fetched/run)", b["corpus_count"], b["best_first_10"].as_array().unwrap().len());
            0
        }
        _ => { eprintln!("usage: corpus generate|check"); 2 }
    }
}
