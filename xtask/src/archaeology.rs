//! GNURUST.COBOL-ARCHAEOLOGY.1 -- generate `docs/COBOL-ARCHAEOLOGY.md` from the committed FINDINGS in
//! `reports/archaeology/findings.json`. Each finding is a COBOL operational fact distilled IN OUR OWN WORDS
//! (facts are not copyrightable) from the recurring themes the archive research surfaced, then verified
//! against standard COBOL. NO sources, books, links, identifiers, dates, page references, or verbatim quotes
//! are stored or shown -- findings only. `generate` writes the doc; `check` regenerates and fails on drift.
use serde_json::Value;
use std::path::Path;

fn data_path(root: &str) -> std::path::PathBuf {
    Path::new(root).join("reports/archaeology/findings.json")
}
fn doc_path(root: &str) -> std::path::PathBuf {
    Path::new(root).join("docs/COBOL-ARCHAEOLOGY.md")
}
fn esc(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ").trim().to_string()
}
// A committed finding must carry NONE of these -- they would reveal provenance.
const BANNED: &[&str] = &["identifier", "archive_text_url", "title", "date", "creator", "page_hint", "quote", "source_text_file", "keywords"];

fn render(data: &Value) -> String {
    let findings = data["findings"].as_array().cloned().unwrap_or_default();
    let mut out = String::new();
    out.push_str("<!-- GENERATED from reports/archaeology/findings.json by `xtask archaeology generate` — DO NOT EDIT BY HAND. -->\n");
    out.push_str("# COBOL Archaeology Atlas\n\n");
    out.push_str(
        "COBOL operational facts distilled **in our own words** from the recurring themes in the archive\n\
         research and verified against standard COBOL — each a specific rule, where it bites in `gnucobol-rs` /\n\
         the KOBOLD data layer, and a candidate court. No verbatim source text, no sources/links, no generic\n\
         tags; only verifiable substance.\n\n");
    out.push_str(&format!("- **{}** distilled findings\n\n", findings.len()));

    let mut topics: Vec<String> = findings.iter().filter_map(|f| f["topic"].as_str().map(str::to_string)).collect();
    topics.sort();
    topics.dedup();
    out.push_str("## Findings by topic\n\n| topic | findings | high-confidence |\n|---|---:|---:|\n");
    for t in &topics {
        let n = findings.iter().filter(|f| f["topic"].as_str() == Some(t)).count();
        let hc = findings.iter().filter(|f| f["topic"].as_str() == Some(t) && f["confidence"].as_str() == Some("high")).count();
        out.push_str(&format!("| {} | {} | {} |\n", esc(t), n, hc));
    }
    out.push('\n');

    for t in &topics {
        out.push_str(&format!("## {}\n\n", esc(t)));
        out.push_str("| id | conf | finding (rule) | gnucobol-rs / KOBOLD touchpoint | candidate court |\n");
        out.push_str("|---|---|---|---|---|\n");
        let rows: Vec<&Value> = findings.iter().filter(|f| f["topic"].as_str() == Some(t)).collect();
        for f in rows {
            out.push_str(&format!("| {} | {} | {} | {} | {} |\n",
                esc(f["id"].as_str().unwrap_or("")),
                esc(f["confidence"].as_str().unwrap_or("")),
                esc(f["rule"].as_str().unwrap_or("")),
                esc(f["area"].as_str().unwrap_or("")),
                esc(f["suggested_court"].as_str().unwrap_or(""))));
        }
        out.push('\n');
    }
    out
}

pub fn run(cmd: &str, root: &str) -> i32 {
    let data: Value = match std::fs::read_to_string(data_path(root)).ok().and_then(|s| serde_json::from_str(&s).ok()) {
        Some(d) => d,
        None => { eprintln!("ARCHAEOLOGY: missing/invalid reports/archaeology/findings.json"); return 2; }
    };
    if let Some(arr) = data["findings"].as_array() {
        for f in arr {
            for b in BANNED {
                if f.get(*b).is_some() {
                    eprintln!("ARCHAEOLOGY: finding carries provenance field `{b}` -- findings must be source-free");
                    return 2;
                }
            }
        }
    }
    let blob = serde_json::to_string(&data["findings"]).unwrap_or_default().to_lowercase();
    for marker in ["archive.org", "http://", "https://", "bitsavers", "_djvu", ".txt"] {
        if blob.contains(marker) {
            eprintln!("ARCHAEOLOGY: findings contain a source marker `{marker}` -- must be source-free");
            return 2;
        }
    }
    match cmd {
        "generate" => {
            let _ = std::fs::create_dir_all(doc_path(root).parent().unwrap());
            let _ = std::fs::write(doc_path(root), render(&data));
            println!("COBOL-ARCHAEOLOGY.md generated from {} findings (source-free)", data["findings"].as_array().map(|a| a.len()).unwrap_or(0));
            0
        }
        "check" => {
            if std::fs::read_to_string(doc_path(root)).unwrap_or_default() != render(&data) {
                println!("ARCHAEOLOGY STALE: docs/COBOL-ARCHAEOLOGY.md != `xtask archaeology generate` (re-run it)");
                return 1;
            }
            println!("COBOL-ARCHAEOLOGY: fresh ({} findings, source-free)", data["findings"].as_array().map(|a| a.len()).unwrap_or(0));
            0
        }
        _ => { eprintln!("usage: xtask archaeology generate|check"); 2 }
    }
}
