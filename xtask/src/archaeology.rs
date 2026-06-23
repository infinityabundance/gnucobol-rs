//! GNURUST.ARCHIVE-ARCHAEOLOGY.1 -- generate `docs/ARCHIVE-ARCHAEOLOGY.md` from the committed, COPYRIGHT-
//! STRIPPED finding metadata in `reports/archaeology/archive-findings.json` (curated by the local-only
//! `cobol_archive_miner` from Archive.org public-domain-derivative text; verbatim source quotes are
//! deliberately NOT committed -- only finding metadata, the public Archive.org citation, and derived
//! relevance analysis). `generate` writes the doc; `check` regenerates in memory and fails on drift.
use serde_json::Value;
use std::path::Path;

fn data_path(root: &str) -> std::path::PathBuf {
    Path::new(root).join("reports/archaeology/archive-findings.json")
}
fn doc_path(root: &str) -> std::path::PathBuf {
    Path::new(root).join("docs/ARCHIVE-ARCHAEOLOGY.md")
}

fn esc(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ").trim().to_string()
}

/// Render the Markdown atlas from the committed JSON data (pure function -- generate and check share it).
fn render(data: &Value) -> String {
    let findings = data["findings"].as_array().cloned().unwrap_or_default();
    let mut out = String::new();
    out.push_str("<!-- GENERATED from reports/archaeology/archive-findings.json by `xtask archaeology generate` — DO NOT EDIT BY HAND. -->\n");
    out.push_str("# Archive Archaeology Atlas\n\n");
    out.push_str(&format!(
        "Curated operational knowledge mined from **Archive.org public-domain-derivative** COBOL manuals/books by the\n\
         local-only `cobol_archive_miner`, integrated here as a *generated, freshness-gated* ledger. **No copyrighted\n\
         source text is reproduced** — only finding metadata, the public Archive.org identifier/URL citation, and\n\
         derived relevance analysis are committed (the verbatim OCR `quote` field is excluded at ingest). OCR-noise and\n\
         beginner-tutorial filler are filtered; only high-value / actionable findings are kept.\n\n"));
    out.push_str(&format!("- **{}** curated findings · selection: {}\n",
        findings.len(), esc(data["selection"].as_str().unwrap_or(""))));
    out.push_str(&format!("- source tool: {}\n\n", esc(data["source_tool"].as_str().unwrap_or(""))));

    // group by topic, then by claim_kind ordering within
    let mut topics: Vec<String> = findings.iter().filter_map(|f| f["topic"].as_str().map(str::to_string)).collect();
    topics.sort();
    topics.dedup();
    // a per-topic count summary table.
    out.push_str("## Findings by topic\n\n| topic | findings | high-confidence |\n|---|---:|---:|\n");
    for t in &topics {
        let n = findings.iter().filter(|f| f["topic"].as_str() == Some(t)).count();
        let hc = findings.iter().filter(|f| f["topic"].as_str() == Some(t) && f["confidence"].as_str() == Some("high")).count();
        out.push_str(&format!("| {} | {} | {} |\n", esc(t), n, hc));
    }
    out.push('\n');

    for t in &topics {
        out.push_str(&format!("## {}\n\n", esc(t)));
        out.push_str("| score | conf | kind | finding | gnucobol-rs impact | suggested court | Archive.org citation |\n");
        out.push_str("|---:|---|---|---|---|---|---|\n");
        let mut rows: Vec<&Value> = findings.iter().filter(|f| f["topic"].as_str() == Some(t)).collect();
        rows.sort_by_key(|f| -(f["score"].as_i64().unwrap_or(0)));
        for f in rows {
            let cite = match (f["identifier"].as_str(), f["date"].as_str(), f["page_hint"].as_str()) {
                (Some(id), d, p) => format!("`{}`{}{}", esc(id),
                    d.map(|d| format!(" ({})", esc(d))).unwrap_or_default(),
                    p.map(|p| format!(" {}", esc(p))).unwrap_or_default()),
                _ => String::new(),
            };
            out.push_str(&format!("| {} | {} | {} | {} | {} | {} | {} |\n",
                f["score"].as_i64().unwrap_or(0),
                esc(f["confidence"].as_str().unwrap_or("")),
                esc(f["claim_kind"].as_str().unwrap_or("")),
                esc(f["title"].as_str().unwrap_or("")),
                esc(f["gnucobol_rs_impact"].as_str().unwrap_or("")),
                esc(f["suggested_court"].as_str().unwrap_or("")),
                cite));
        }
        out.push('\n');
    }
    out.push_str("> Every row cites a public Archive.org identifier (and approximate line) so the original\n\
                  > public-domain-derivative text can be consulted at the source; this atlas reproduces none of it.\n");
    out
}

pub fn run(cmd: &str, root: &str) -> i32 {
    let data: Value = match std::fs::read_to_string(data_path(root)).ok().and_then(|s| serde_json::from_str(&s).ok()) {
        Some(d) => d,
        None => { eprintln!("ARCHAEOLOGY: missing/invalid reports/archaeology/archive-findings.json"); return 2; }
    };
    // Integrity: a committed finding must never carry a verbatim source quote (copyright guard).
    if data["findings"].as_array().map(|a| a.iter().any(|f| f.get("quote").is_some())).unwrap_or(false) {
        eprintln!("ARCHAEOLOGY: a finding carries a verbatim `quote` -- copyrighted source text must not be committed");
        return 2;
    }
    match cmd {
        "generate" => {
            let _ = std::fs::create_dir_all(doc_path(root).parent().unwrap());
            let _ = std::fs::write(doc_path(root), render(&data));
            println!("ARCHIVE-ARCHAEOLOGY.md generated from {} findings", data["findings"].as_array().map(|a| a.len()).unwrap_or(0));
            0
        }
        "check" => {
            let committed = std::fs::read_to_string(doc_path(root)).unwrap_or_default();
            if committed != render(&data) {
                println!("ARCHAEOLOGY STALE: docs/ARCHIVE-ARCHAEOLOGY.md != `xtask archaeology generate` (re-run it)");
                return 1;
            }
            println!("ARCHIVE-ARCHAEOLOGY: fresh ({} findings, no committed quotes)", data["findings"].as_array().map(|a| a.len()).unwrap_or(0));
            0
        }
        _ => { eprintln!("usage: xtask archaeology generate|check"); 2 }
    }
}
