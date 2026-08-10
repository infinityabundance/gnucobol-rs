//! Phase 7 — X-COBOL (Zenodo 10.5281/zenodo.7968845, CC-BY-4.0) and large public collections.
//!
//! Immutable custody: DOI + archive md5/sha256, per-repository GitHub metadata (a.json), licence
//! quarantine at repository level (unknown licences are never published). Structural
//! classification per file; a bounded oracle admission matrix (default/cobol85/cobol2002/
//! cobol2014/ibm/mf/acu, first success); repository-level dependency resolution and
//! deduplication; frozen DEVELOPMENT/VALIDATION/HELD_OUT_EVALUATION partitions (recorded seed);
//! and large-scale robustness measurement (compiles, candidate parse/check, crashes, timeouts,
//! encodings, dialect/size distributions). No input may hang the candidate (bounded probes).

use crate::dedup::{exact_hash, normalized_hash, structural_hash};
use crate::extract::candidate::probe_dir;
use crate::extract::oracle::{run_step, OracleEnv};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub const DOI: &str = "10.5281/zenodo.7968845";
pub const ZIP_MD5: &str = "1a05a95e5320bde93fadcecea4c1926a";
pub const PARTITION_SEED: u64 = 20260810;

/// The bounded dialect matrix (GnuCOBOL 3.2 supports all of these).
pub const DIALECTS: [&str; 7] = [
    "default",
    "cobol85",
    "cobol2002",
    "cobol2014",
    "ibm",
    "mf",
    "acu",
];

#[derive(Debug, Clone, Serialize)]
pub struct RepoCustody {
    pub repo: String,
    pub full_name: String,
    pub license_spdx: String,
    pub license_decision: String,
    pub file_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct XFile {
    pub file_id: String,
    pub repo: String,
    pub path: String,
    pub bytes: usize,
    pub extension: String,
    pub structural_class: String,
    pub encoding: String,
    pub licence: String,
    pub copy_dependencies: Vec<String>,
    pub missing_copybooks: Vec<String>,
    pub dialect_accepted: Option<String>,
    pub oracle_compile_note: String,
    pub candidate_first_failure: Option<(String, String)>,
    pub candidate_phases_ok: bool,
    pub partition: String,
    pub exact_sha256: String,
    pub normalized_sha256: String,
    pub structural_sha256: String,
    pub near_duplicate_of: Option<String>,
}

/// The recorded partition assignment (deterministic from the seed; repo-level so no fork leaks
/// across sets).
pub fn partition_of(repo: &str) -> String {
    // deterministic hash of the repo name -> DEVELOPMENT / VALIDATION / HELD_OUT_EVALUATION
    let h = structural_hash(repo.as_bytes());
    let v: u64 = h
        .bytes()
        .take(8)
        .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
    let r = (v ^ PARTITION_SEED) % 100;
    if r < 70 {
        "DEVELOPMENT"
    } else if r < 85 {
        "VALIDATION"
    } else {
        "HELD_OUT_EVALUATION"
    }
    .to_string()
}

/// Structural classification of one COBOL file (spec 7.3).
pub fn structural_class(text: &str) -> String {
    let up = text.to_ascii_uppercase();
    if up.contains("PROGRAM-ID") && (up.contains("PROCEDURE DIVISION") || up.contains("STOP RUN")) {
        if up.contains("IDENTIFICATION") {
            "COMPLETE_PROGRAM".to_string()
        } else {
            "PROGRAM_FRAGMENT".to_string()
        }
    } else if up.contains("IDENTIFICATION") && up.contains("PROCEDURE DIVISION") {
        "COMPLETE_PROGRAM".to_string()
    } else if up.contains("WORKING-STORAGE")
        || up.contains("LINKAGE")
        || up.contains("COPY ")
        || (up.contains("PIC ") && !up.contains("PROCEDURE"))
    {
        "COPYBOOK_OR_DATA".to_string()
    } else if up.contains("EXEC SQL") || up.contains("DB2") {
        "VENDOR_SPECIFIC_DB2".to_string()
    } else if up.contains("CICS") || up.contains("DFH") {
        "VENDOR_SPECIFIC_CICS".to_string()
    } else if text.trim().is_empty() {
        "EMPTY".to_string()
    } else if text.lines().count() < 5 {
        "FRAGMENT".to_string()
    } else {
        "FRAGMENT".to_string()
    }
}

/// Extract + measure the X-COBOL dataset.
pub fn extract_xcobol(
    repo_root: &Path,
    packages_root: &Path,
    out_dir: &Path,
    with_candidate: bool,
    with_oracle: bool,
) -> Result<BTreeMap<String, usize>, String> {
    let oracle = OracleEnv::host_default()?;
    let data = repo_root.join("lab/corpus/x-cobol/extracted/X-COBOL");
    let files_dir = data.join("COBOL_Files");
    std::fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    if !files_dir.exists() {
        return Err(format!(
            "X-COBOL dataset not admitted (download DOI {DOI} zip md5 {ZIP_MD5} into lab/corpus/x-cobol/X-COBOL.zip and unzip to extracted/)"
        ));
    }

    // per-repo metadata from the dataset's GitHub API dump
    let mut meta: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    for jf in ["a.json", "b.json"] {
        let p = data.join(jf);
        if let Ok(bytes) = std::fs::read(&p) {
            let s = String::from_utf8_lossy(&bytes);
            if let Some(start) = s.find('{') {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s[start..]) {
                    if let Some(name) = v.get("full_name").and_then(|x| x.as_str()) {
                        meta.insert(name.to_string(), v);
                    }
                }
            }
        }
    }

    // work area: one dir per repo (dependencies resolve within the repo)
    let work = packages_root.join("xcobol");
    std::fs::create_dir_all(&work).map_err(|e| e.to_string())?;

    let mut custodies: Vec<RepoCustody> = Vec::new();
    let mut files: Vec<XFile> = Vec::new();
    let mut exact_seen: BTreeMap<String, String> = BTreeMap::new(); // sha -> file_id
    let mut near_groups: BTreeMap<String, Vec<String>> = BTreeMap::new(); // structural sha -> ids
    let mut repo_names: Vec<String> = Vec::new();

    for entry in std::fs::read_dir(&files_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let repo_dir = entry.path();
        if !repo_dir.is_dir() {
            continue;
        }
        let repo = repo_dir.file_name().unwrap().to_string_lossy().to_string();
        repo_names.push(repo.clone());
        let meta_val = meta
            .iter()
            .find(|(k, _)| k.replace('/', "_") == repo)
            .map(|(_, v)| v.clone());
        let license = meta_val
            .as_ref()
            .and_then(|v| v.get("license"))
            .and_then(|l| l.get("spdx_id"))
            .and_then(|s| s.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();
        let (licence, decision) = if license.is_empty() {
            (
                "UNKNOWN".to_string(),
                "REFERENCE_ONLY (no per-repo licence in the dataset metadata; quarantined from \
                 redistribution)"
                    .to_string(),
            )
        } else {
            (
                license.clone(),
                format!(
                    "per-repo SPDX {license} from the dataset metadata (a.json); redistribution \
                     only where the licence permits"
                ),
            )
        };
        // repo work dir + copy files
        let repo_work = work.join(&repo);
        std::fs::create_dir_all(&repo_work).map_err(|e| e.to_string())?;
        let mut cobol_files: Vec<PathBuf> = Vec::new();
        let mut stack = vec![repo_dir.clone()];
        while let Some(d) = stack.pop() {
            if let Ok(rd) = std::fs::read_dir(&d) {
                for e in rd.flatten() {
                    let p = e.path();
                    if p.is_dir() {
                        stack.push(p);
                        continue;
                    }
                    let ext = p
                        .extension()
                        .and_then(|x| x.to_str())
                        .map(|x| x.to_ascii_lowercase())
                        .unwrap_or_default();
                    if matches!(ext.as_str(), "cob" | "cbl" | "cpy" | "cobol") {
                        cobol_files.push(p);
                    }
                }
            }
        }
        cobol_files.sort();
        let n_files = cobol_files.len();
        for p in cobol_files {
            let bytes = std::fs::read(&p).unwrap_or_default();
            let text = String::from_utf8_lossy(&bytes).into_owned();
            let rel = p
                .strip_prefix(&files_dir)
                .unwrap_or(&p)
                .to_string_lossy()
                .to_string();
            let fname = p.file_name().unwrap().to_string_lossy().to_string();
            // copy to the repo work dir for probing
            let dst = repo_work.join(&fname);
            if !dst.exists() {
                let _ = std::fs::write(&dst, &bytes);
            }
            let exact = exact_hash(&bytes);
            let norm = normalized_hash(&bytes);
            let struct_h = structural_hash(&bytes);
            let file_id = format!("xcobol/{repo}/{fname}");
            let dup_of = exact_seen.get(&exact).cloned();
            if dup_of.is_none() {
                exact_seen.insert(exact.clone(), file_id.clone());
            }
            near_groups
                .entry(struct_h.clone())
                .or_default()
                .push(file_id.clone());
            let sclass = structural_class(&text);
            *counts.entry(format!("structural:{sclass}")).or_default() += 1;
            let encoding =
                if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) || std::str::from_utf8(&bytes).is_ok() {
                    "UTF-8/ASCII".to_string()
                } else {
                    "NON-UTF8".to_string()
                };
            if encoding == "NON-UTF8" {
                *counts.entry("encoding:non-utf8".into()).or_default() += 1;
            }
            // COPY dependencies resolved within the repo work dir
            let up = text.to_ascii_uppercase();
            let mut copies = Vec::new();
            for line in up.lines() {
                let t = line.trim();
                if t.starts_with("COPY ") {
                    let name: String = t
                        .split_whitespace()
                        .nth(1)
                        .unwrap_or("")
                        .trim_matches('.')
                        .to_string();
                    if !name.is_empty() {
                        copies.push(name);
                    }
                }
            }
            let missing: Vec<String> = copies
                .iter()
                .filter(|c| {
                    !repo_work.join(c).exists() && !repo_work.join(format!("{c}.cpy")).exists()
                })
                .cloned()
                .collect();
            // oracle admission matrix (bounded: first successful dialect wins)
            let (dialect, note) = if with_oracle {
                let mut found = None;
                let mut last_err = String::new();
                for d in DIALECTS {
                    let cmd = format!(
                        "cobc -fsyntax-only -std={d} -I {} {}",
                        repo_work.display(),
                        fname
                    );
                    let out = run_step(&oracle, &repo_work, &cmd.trim(), &[]);
                    if out.exit == Some(0) {
                        found = Some(d.to_string());
                        break;
                    } else {
                        last_err = String::from_utf8_lossy(&out.stderr)
                            .lines()
                            .take(1)
                            .collect::<Vec<_>>()
                            .join(" | ");
                    }
                }
                match found {
                    Some(d) => (Some(d), String::new()),
                    None => (None, format!("rejected by all dialects: {last_err}")),
                }
            } else {
                (None, "oracle pass disabled".to_string())
            };
            if let Some(d) = &dialect {
                *counts.entry(format!("oracle-accepted:{d}")).or_default() += 1;
            } else if with_oracle {
                *counts.entry("oracle-rejected:all".into()).or_default() += 1;
            }
            // candidate probe: bounded subprocess (no input may crash or hang the corpus run)
            let (cand_ff, cand_ok) = if with_candidate {
                let out_path = repo_work.join(format!("{fname}.candidate.json"));
                let exe = std::env::current_exe().unwrap_or_default();
                let status = std::process::Command::new("timeout")
                    .arg("90")
                    .arg(&exe)
                    .arg("probe-file")
                    .arg(&repo_work)
                    .arg(format!("--file={fname}"))
                    .arg("--out")
                    .arg(&out_path)
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status();
                let probes: Vec<crate::extract::candidate::PhaseOutcome> = match status {
                    Ok(s) if s.success() => std::fs::read_to_string(&out_path)
                        .ok()
                        .and_then(|j| serde_json::from_str(&j).ok())
                        .unwrap_or_default(),
                    _ => vec![crate::extract::candidate::PhaseOutcome {
                        phase: "check".to_string(),
                        ok: false,
                        diagnostic: "candidate probe crashed, timed out, or failed to start"
                            .to_string(),
                    }],
                };
                let ff = probes.iter().find(|x| !x.ok);
                if let Some(x) = ff {
                    *counts
                        .entry(format!("candidate-reject:{}", x.phase))
                        .or_default() += 1;
                } else if !probes.is_empty() {
                    *counts.entry("candidate-accepted".into()).or_default() += 1;
                }
                (
                    ff.map(|x| (x.phase.clone(), x.diagnostic.clone())),
                    ff.is_none(),
                )
            } else {
                (None, false)
            };
            files.push(XFile {
                file_id,
                repo: repo.clone(),
                path: rel,
                bytes: bytes.len(),
                extension: p
                    .extension()
                    .and_then(|x| x.to_str())
                    .unwrap_or("")
                    .to_string(),
                structural_class: sclass,
                encoding,
                licence: licence.clone(),
                copy_dependencies: copies,
                missing_copybooks: missing,
                dialect_accepted: dialect,
                oracle_compile_note: note,
                candidate_first_failure: cand_ff,
                candidate_phases_ok: cand_ok,
                partition: partition_of(&repo),
                exact_sha256: exact,
                normalized_sha256: norm,
                structural_sha256: struct_h,
                near_duplicate_of: dup_of,
            });
        }
        custodies.push(RepoCustody {
            repo: repo.clone(),
            full_name: meta_val
                .as_ref()
                .and_then(|v| v.get("full_name"))
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            license_spdx: license,
            license_decision: decision,
            file_count: n_files,
        });
    }
    repo_names.sort();

    // partitions: repo-level, frozen (recorded seed)
    let mut part_counts: BTreeMap<String, usize> = BTreeMap::new();
    for f in &files {
        *part_counts.entry(f.partition.clone()).or_default() += 1;
    }
    // near-duplicate families (structural hash groups with >1 member)
    let mut families = 0usize;
    for (_h, ids) in &near_groups {
        if ids.len() > 1 {
            families += 1;
            *counts.entry("near-duplicate-family".into()).or_default() += 1;
        }
    }

    write_json(
        out_dir,
        "custody.json",
        &serde_json::json!({
            "doi": DOI,
            "zip_md5": ZIP_MD5,
            "zip_sha256": crate::store::sha256_hex(&std::fs::read(repo_root.join("lab/corpus/x-cobol/X-COBOL.zip")).unwrap_or_default()),
            "licence": "cc-by-4.0",
            "repos": custodies.len(),
            "files": files.len(),
            "repos_detail": custodies,
        }),
    )?;
    write_json(out_dir, "programs.json", &files)?;
    write_json(
        out_dir,
        "licence-quarantine.json",
        &serde_json::json!({
            "policy": "per-repository licence from the dataset metadata; UNKNOWN => REFERENCE_ONLY (quarantined, never published)",
            "repos": custodies,
        }),
    )?;
    write_json(
        out_dir,
        "partitions.json",
        &serde_json::json!({
            "seed": PARTITION_SEED,
            "rule": "repo-name structural hash mod 100: <70 DEVELOPMENT, <85 VALIDATION, else HELD_OUT_EVALUATION",
            "counts": part_counts,
            "held_out_repos": repo_names.iter().filter(|r| partition_of(r) == "HELD_OUT_EVALUATION").collect::<Vec<_>>(),
        }),
    )?;
    write_json(
        out_dir,
        "dedup.json",
        &serde_json::json!({
            "exact_duplicate_files": files.len() - exact_seen.len(),
            "near_duplicate_families": families,
            "note": "grouping is repository-level; the partitions never split a repo",
        }),
    )?;
    write_json(out_dir, "robustness.json", &counts)?;

    let mut md = String::new();
    md.push_str("# X-COBOL corpus (Phase 7)\n\n");
    md.push_str(&format!("DOI {DOI} (cc-by-4.0), zip md5 {ZIP_MD5}\n\n"));
    md.push_str("| measure | count |\n|---|---|\n");
    for (k, v) in &counts {
        md.push_str(&format!("| {k} | {v} |\n"));
    }
    md.push('\n');
    md.push_str("Partitions are frozen (seed recorded); the held-out set is never used for\n");
    md.push_str("implementation tuning. Unknown-licence source is quarantined (REFERENCE_ONLY).\n");
    std::fs::write(out_dir.join("summary.md"), md).map_err(|e| e.to_string())?;
    counts.insert("total_repos".into(), custodies.len());
    counts.insert("total_files".into(), files.len());
    Ok(counts)
}

fn write_json<T: Serialize>(dir: &Path, name: &str, v: &T) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(v).map_err(|e| e.to_string())?;
    std::fs::write(dir.join(name), json).map_err(|e| e.to_string())
}
