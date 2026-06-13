//! `GNURUST.COBOL-CORPUS-ATLAS.1` — a custody/index manifest of the external COBOL validation corpora the
//! port is regression-checked against, across THREE evidence classes:
//!   1. historical conformance suites      (NIST CCVS85)
//!   2. upstream compiler regression suites (GnuCOBOL 3.2 tests/, GCC gcobol testsuite)
//!   3. independent real-world / defect corpora (OpenCBS defects, X-COBOL repositories)
//!
//! CUSTODY ONLY — each corpus is admitted by source identity (immutable git commit / Zenodo DOI / admitted
//! tarball) + content hash + file/program counts. NO conformance, suite-pass, or behaviour-parity claim is
//! made here; compile/run baselines are deferred to per-corpus `.2`/`.3`/`.4` tiers. The large raw corpora
//! are re-downloadable from their permanent sources (gitignored); this manifest is the committed spine.

use crate::ccvs85::sha256_hex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

pub const RECEIPT_MD: &str = "reports/provenance/cobol-corpus-atlas-receipt.md";
pub const RECEIPT_JSON: &str = "reports/provenance/cobol-corpus-atlas-receipt.json";

/// A registered corpus: static metadata (the registry) + derived custody facts.
#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct CorpusEntry {
    pub gate: String,
    pub id: String,
    pub name: String,
    pub evidence_class: String,
    pub priority: String,
    pub source: String,
    pub license: String,
    pub claim: String,
    /// `LOCAL` (custody derived from present files) or `SOURCE-ONLY` (recorded source, files not present).
    pub status: String,
    /// Immutable custody identity: a git commit SHA, a Zenodo file sha256, or the admitted-tarball sha256.
    pub custody_id: String,
    pub custody_kind: String,
    pub counts: BTreeMap<String, u64>,
}

/// The whole atlas receipt.
#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct Atlas {
    pub gate: String,
    pub conformance_claim: String,
    pub corpora: Vec<CorpusEntry>,
}

/// Static registry: (id, name, evidence_class, gate, priority, source, license, local_path, kind).
/// kind ∈ { git, zip:<file>, tree, ccvs85 }.
fn registry() -> Vec<(&'static str, &'static str, &'static str, &'static str, &'static str, &'static str, &'static str, &'static str, &'static str)> {
    vec![
        (
            "CCVS85", "NIST CCVS85 COBOL-85 Validation Suite (VERSION 4.0, 1992)",
            "historical conformance suite", "GNURUST.CCVS85.1", "required",
            "NIST CCVS85 newcob.val.Z (committed spine; mirrors e.g. github.com/.../nistcobol85)",
            "US Government work / public domain (NIST)", "lab/corpus/ccvs85/newcob.val.Z", "ccvs85",
        ),
        (
            "GNUCOBOL-TESTS", "GnuCOBOL 3.2 upstream test suite (tests/, incl. cobol85 NIST-derived)",
            "upstream compiler regression suite", "GNURUST.GNUCOBOL-TESTS.1", "high",
            "GnuCOBOL 3.2 admitted source tarball (research/gnucobol-3.2.tar.lz)",
            "GPL-3.0 / LGPL-3.0 (GnuCOBOL project)", "lab/admit/gnucobol-3.2/tests", "tree",
        ),
        (
            "GCOBOL", "GCC gcobol testsuite (cobol.dg, COBOL-2023 front end)",
            "upstream compiler regression suite", "GNURUST.GCOBOL.1", "medium-high",
            "git https://gcc.gnu.org/git/gcc.git (gcc/testsuite/cobol.dg, gcc/cobol, libgcobol)",
            "GPL-3.0-or-later with GCC Runtime Library Exception", "lab/corpus/gcobol/gcc", "git",
        ),
        (
            "OPEN-CBS", "OpenCBS — Open-Source COBOL Defects Benchmark Suite (43 programs)",
            "independent defect corpus", "GNURUST.OPEN-CBS.1", "medium-high",
            "git https://github.com/PhaseChangeSoftware/cobol-defects-suite",
            "see repo LICENSE (Phase Change Software et al.)", "lab/corpus/opencbs/repo", "git",
        ),
        (
            "X-COBOL", "X-COBOL — Dataset of Open-Source COBOL Repositories (84 repos, 1255 files)",
            "independent real-world corpus", "GNURUST.XCOBOL.1", "medium",
            "Zenodo doi:10.5281/zenodo.14269462 (full record archive)",
            "per-repository upstream licenses (mined open-source)", "research", "zip:14269462.zip",
        ),
    ]
}

/// Count files under `dir` (recursively), keyed by lowercase extension buckets relevant to COBOL corpora.
fn count_files(dir: &Path) -> BTreeMap<String, u64> {
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(p) = stack.pop() {
        let entries = match std::fs::read_dir(&p) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for e in entries.flatten() {
            let path = e.path();
            if path.is_dir() {
                if path.file_name().map(|n| n == ".git").unwrap_or(false) {
                    continue;
                }
                stack.push(path);
                continue;
            }
            *counts.entry("files".into()).or_insert(0) += 1;
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase();
            let bucket = match ext.as_str() {
                "cob" | "cbl" | "cobol" | "pco" => "cobol",
                "cpy" => "copybook",
                "jcl" => "jcl",
                "at" => "autotest",
                "txt" => "text",
                _ => "",
            };
            if !bucket.is_empty() {
                *counts.entry(bucket.into()).or_insert(0) += 1;
            }
        }
    }
    counts
}

/// `git -C <path> rev-parse HEAD`, if the path is a git work tree.
fn git_commit(path: &Path) -> Option<String> {
    let out = Command::new("git").arg("-C").arg(path).arg("rev-parse").arg("HEAD").output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Read the CCVS85 receipt's committed `compressed_sha256` as that corpus's custody identity.
fn ccvs85_custody(root: &Path) -> Option<String> {
    let s = std::fs::read_to_string(root.join(crate::ccvs85::RECEIPT_JSON)).ok()?;
    let v: serde_json::Value = serde_json::from_str(&s).ok()?;
    v.get("compressed_sha256").and_then(|x| x.as_str()).map(|x| x.to_string())
}

/// Read the admission receipt's tarball sha256 (the custody root for the GnuCOBOL tests/).
fn admission_custody(root: &Path) -> Option<String> {
    let s = std::fs::read_to_string(root.join("reports/admission/RECEIPT-ADMISSION.md")).ok()?;
    // the admitted-tarball sha256 is the `| sha256 | <hex> |` table row (not the legacy-receipt mention).
    for line in s.lines() {
        let l = line.to_ascii_lowercase();
        if l.contains("| sha256 |") {
            for tok in line.split(|c: char| !c.is_ascii_hexdigit()) {
                if tok.len() == 64 {
                    return Some(tok.to_ascii_lowercase());
                }
            }
        }
    }
    None
}

fn derive_one(root: &Path, e: &(&str, &str, &str, &str, &str, &str, &str, &str, &str)) -> CorpusEntry {
    let (id, name, class, gate, prio, source, license, local_path, kind) = *e;
    let path = root.join(local_path);
    let mut status = "SOURCE-ONLY".to_string();
    let mut custody_id = String::new();
    let mut custody_kind = String::new();
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();

    if kind == "ccvs85" {
        if let Some(h) = ccvs85_custody(root) {
            status = "LOCAL".into();
            custody_id = h;
            custody_kind = "compressed-sha256 (see GNURUST.CCVS85.1)".into();
        }
    } else if kind == "tree" {
        if path.exists() {
            status = "LOCAL".into();
            custody_id = admission_custody(root).unwrap_or_default();
            custody_kind = "admitted-tarball-sha256 (custody root)".into();
            counts = count_files(&path);
        }
    } else if kind == "git" {
        if let Some(commit) = git_commit(&path) {
            status = "LOCAL".into();
            custody_id = commit;
            custody_kind = "git-commit-sha".into();
            counts = count_files(&path);
        }
    } else if let Some(file) = kind.strip_prefix("zip:") {
        let zip = path.join(file);
        if let Ok(bytes) = std::fs::read(&zip) {
            status = "LOCAL".into();
            custody_id = sha256_hex(&bytes);
            custody_kind = "archive-sha256".into();
            counts.insert("archive_bytes".into(), bytes.len() as u64);
            // also fold in any sibling metadata files
            let dir_counts = count_files(&path);
            for (k, v) in dir_counts {
                counts.entry(k).or_insert(v);
            }
        }
    }

    CorpusEntry {
        gate: gate.into(),
        id: id.into(),
        name: name.into(),
        evidence_class: class.into(),
        priority: prio.into(),
        source: source.into(),
        license: license.into(),
        claim: "custody/index only; no conformance, suite-pass, or behaviour-parity claim".into(),
        status,
        custody_id,
        custody_kind,
        counts,
    }
}

fn derive(root: &Path) -> Atlas {
    Atlas {
        gate: "GNURUST.COBOL-CORPUS-ATLAS.1".into(),
        conformance_claim: "NONE — corpus custody/index across three evidence classes; compile/run/behaviour \
            baselines are deferred to per-corpus tiered gates."
            .into(),
        corpora: registry().iter().map(|e| derive_one(root, e)).collect(),
    }
}

fn atlas_md(a: &Atlas) -> String {
    let mut rows = String::new();
    for c in &a.corpora {
        let cobol = c.counts.get("cobol").copied().unwrap_or(0);
        rows.push_str(&format!(
            "### {} — {}\n\n\
            - gate: `{}`  ·  class: {}  ·  priority: {}  ·  status: **{}**\n\
            - source: {}\n\
            - license: {}\n\
            - custody: `{}` ({})\n\
            - counts: {}{}\n\
            - claim: {}\n\n",
            c.id, c.name, c.gate, c.evidence_class, c.priority, c.status, c.source, c.license,
            if c.custody_id.is_empty() { "(files not local)" } else { &c.custody_id }, c.custody_kind,
            c.counts.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(", "),
            if cobol > 0 { "" } else { "" },
            c.claim,
        ));
    }
    format!(
        "# GNURUST.COBOL-CORPUS-ATLAS.1 — COBOL validation corpus atlas\n\n\
        **GENERATED** by `cargo run -p gnucobol-rs-port-index -- corpus-atlas generate` — do not edit by hand.\n\n\
        gnucobol-rs tracks COBOL validation across THREE evidence classes — historical conformance suites,\n\
        upstream compiler regression suites, and independent real-world / defect corpora. Each corpus is\n\
        admitted FIRST by custody + index (immutable source identity + content hash + counts) before any\n\
        compile/run or behaviour claim.\n\n\
        **Conformance claim:** {claim}\n\n\
        Raw corpora are re-downloadable from their permanent sources (gitignored); this manifest +\n\
        `reports/cobol-corpus-atlas/atlas.json` are the committed custody spine. `corpus-atlas check`\n\
        re-derives custody for any locally-present corpus and diffs it against the committed receipt.\n\n\
        ## Corpora\n\n{rows}",
        claim = a.conformance_claim,
        rows = rows,
    )
}

pub fn generate(root: &Path) -> i32 {
    let atlas = derive(root);
    let _ = std::fs::create_dir_all(root.join("reports/cobol-corpus-atlas"));
    let _ = std::fs::create_dir_all(root.join("reports/provenance"));
    let _ = std::fs::write(root.join("reports/cobol-corpus-atlas/atlas.json"), serde_json::to_string_pretty(&atlas).unwrap() + "\n");
    let _ = std::fs::write(root.join(RECEIPT_JSON), serde_json::to_string_pretty(&atlas).unwrap() + "\n");
    let _ = std::fs::write(root.join(RECEIPT_MD), atlas_md(&atlas));
    let local = atlas.corpora.iter().filter(|c| c.status == "LOCAL").count();
    println!("GNURUST.COBOL-CORPUS-ATLAS.1: {} corpora ({} with local custody)", atlas.corpora.len(), local);
    0
}

/// `corpus-atlas check`: the receipt must be present + match a re-derivation, EXCEPT that a corpus which is
/// locally absent (`SOURCE-ONLY` now) keeps its committed `LOCAL` custody (the source is permanent). This
/// keeps the gate green without the gitignored raw corpora, while catching drift when they ARE present.
pub fn check(root: &Path) -> i32 {
    let committed: Atlas = match std::fs::read_to_string(root.join(RECEIPT_JSON)).ok().and_then(|s| serde_json::from_str(&s).ok()) {
        Some(a) => a,
        None => {
            eprintln!("GNURUST.COBOL-CORPUS-ATLAS.1 STALE: receipt missing (run `corpus-atlas generate`)");
            return 1;
        }
    };
    let fresh = derive(root);
    if fresh.corpora.len() != committed.corpora.len() {
        eprintln!("GNURUST.COBOL-CORPUS-ATLAS.1 STALE: corpus count changed");
        return 1;
    }
    for (c, f) in committed.corpora.iter().zip(&fresh.corpora) {
        // metadata must always match
        if c.id != f.id || c.gate != f.gate || c.source != f.source || c.evidence_class != f.evidence_class {
            eprintln!("GNURUST.COBOL-CORPUS-ATLAS.1 STALE: metadata drift for {}", c.id);
            return 1;
        }
        // custody must match only when the corpus is present locally now
        if f.status == "LOCAL" && (c.custody_id != f.custody_id || c.counts != f.counts) {
            eprintln!("GNURUST.COBOL-CORPUS-ATLAS.1 STALE: custody drift for {} (re-run generate)", c.id);
            return 1;
        }
    }
    let local = committed.corpora.iter().filter(|c| c.status == "LOCAL").count();
    println!("GNURUST.COBOL-CORPUS-ATLAS.1: atlas STABLE ({} corpora, {} with committed local custody)", committed.corpora.len(), local);
    0
}
