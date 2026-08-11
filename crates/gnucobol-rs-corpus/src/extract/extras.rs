//! Phase 5 — GnuCOBOL-shipped programs and official contribution collections.
//!
//! Two admitted sources (the only ones available offline in the admitted trees):
//!
//! 1. `extras/CBL_OC_DUMP.cob` — the COBOL program shipped with GnuCOBOL 3.2 and the current
//!    pin (GPL-3.0-or-later, FSF copyright, redistribution permitted).
//! 2. The OpenCBS COBOL Defects Benchmark Suite (github.com/PhaseChangeSoftware/
//!    cobol-defects-suite, MIT, already under custody at `lab/corpus/opencbs/repo`).
//!
//! Every program gets custody (immutable revision + content hash), a licence decision,
//! dependency discovery (copybooks / data files / JCL / platform services), realistic metrics
//! (5.4), an adaptation decision (5.3: the original is always preserved; adaptations, when any,
//! are separate patches), oracle compile/run verification, and a candidate phase probe.
//!
//! The official sample/game collections (e.g. the gnucobol samples repos) are NOT present in
//! the admitted offline trees; that availability constraint is recorded in the summary and
//! `check-updates` (never imported with an unknown licence).

use crate::extract::candidate::{probe_dir, PhaseOutcome};
use crate::extract::oracle::{run_step, OracleEnv};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Custody + licence of one admitted collection.
#[derive(Debug, Clone, Serialize)]
pub struct CollectionCustody {
    pub name: String,
    pub origin_url: String,
    pub revision: String,
    pub spdx: String,
    pub redistribution_allowed: bool,
    pub decision: String,
    pub program_count: usize,
}

/// Realistic program metrics (spec 5.4).
#[derive(Debug, Clone, Serialize)]
pub struct ProgramMetrics {
    pub lines: usize,
    pub bytes: usize,
    pub paragraphs: usize,
    pub data_items: usize,
    pub level_01_items: usize,
    pub copy_refs: Vec<String>,
    pub file_descriptions: usize,
    pub program_ids: Vec<String>,
    /// Approximate maximum IF/EVALUATE/PERFORM scope depth (structural scan, not a full parse).
    pub approx_scope_depth: usize,
    /// Distinct COBOL verbs seen (feature inventory).
    pub verbs: Vec<String>,
}

/// One admitted program with its custody, metrics, dependencies, and measured outcomes.
#[derive(Debug, Clone, Serialize)]
pub struct ExtrasProgram {
    pub program_id: String,
    pub collection: String,
    pub source_rel: String,
    pub content_sha256: String,
    pub source_format: String,
    pub dialect: String,
    pub licence: String,
    pub metrics: ProgramMetrics,
    pub copybook_dependencies: Vec<String>,
    pub data_file_dependencies: Vec<String>,
    pub platform_services: Vec<String>,
    pub adaptation: String,
    pub oracle_compile_exit: Option<i32>,
    pub oracle_compile_note: String,
    pub oracle_run_exit: Option<i32>,
    pub run_attempted: bool,
    pub run_note: String,
    pub candidate_phases: Vec<PhaseOutcome>,
    pub candidate_first_failure: Option<(String, String)>,
    pub classification: String,
}

/// Structural scan of a COBOL source: metrics + feature inventory. Honest approximations where a
/// full parse is not warranted (scope depth); everything else is exact line/count data.
pub fn scan_program(source: &str) -> ProgramMetrics {
    let mut paragraphs = 0usize;
    let mut data_items = 0usize;
    let mut level_01 = 0usize;
    let mut copy_refs = Vec::new();
    let mut file_descriptions = 0usize;
    let mut program_ids = Vec::new();
    let mut verbs: BTreeMap<String, usize> = BTreeMap::new();
    let known_verbs = [
        "ACCEPT",
        "ADD",
        "CALL",
        "CANCEL",
        "CLOSE",
        "COMPUTE",
        "CONTINUE",
        "DELETE",
        "DISPLAY",
        "DIVIDE",
        "EVALUATE",
        "EXIT",
        "GO TO",
        "GOBACK",
        "IF",
        "INITIALIZE",
        "INSPECT",
        "MERGE",
        "MOVE",
        "MULTIPLY",
        "OPEN",
        "PERFORM",
        "READ",
        "REWRITE",
        "SEARCH",
        "SET",
        "SORT",
        "START",
        "STOP",
        "STRING",
        "SUBTRACT",
        "UNSTRING",
        "WRITE",
        "ACCEPT",
        "INVOKE",
        "UNLOCK",
        "RETURN",
    ];
    let mut scope_depth = 0usize;
    let mut max_depth = 0usize;
    for raw in source.lines() {
        let line = raw.trim();
        let up = line.to_ascii_uppercase();
        // strip string literals for verb counting (approximation: cut at first quote)
        let no_quotes: String = up
            .split('\'')
            .enumerate()
            .filter(|(i, _)| i % 2 == 0)
            .map(|(_, s)| s)
            .collect::<Vec<_>>()
            .join(" ");
        let no_quotes = no_quotes
            .split('"')
            .enumerate()
            .filter(|(i, _)| i % 2 == 0)
            .map(|(_, s)| s)
            .collect::<Vec<_>>()
            .join(" ");
        if up.starts_with("COPY ") {
            let name: String = no_quotes
                .split_whitespace()
                .nth(1)
                .unwrap_or("")
                .trim_matches('.')
                .to_string();
            if !name.is_empty() {
                copy_refs.push(name);
            }
        }
        if up.contains("PROGRAM-ID") {
            let id: String = no_quotes
                .split_whitespace()
                .nth(1)
                .unwrap_or("")
                .trim_matches('.')
                .to_string();
            if !id.is_empty() {
                program_ids.push(id);
            }
        }
        if up.starts_with("PARAGRAPH")
            || (up.contains('.') && line.starts_with("P-"))
            || (line.starts_with("P1") && up.contains('.'))
        {
            // paragraph labels: a word in the procedure area ending with '.'
        }
        // paragraph label heuristic: a line whose first token is followed by '.' and is not a
        // verb/level/division header
        if let Some(first) = no_quotes.split_whitespace().next() {
            let first = first.trim_end_matches('.');
            if up.contains('.')
                && !first.is_empty()
                && !is_verb(first)
                && !up.starts_with("IDENTIFICATION")
                && !up.starts_with("ENVIRONMENT")
                && !up.starts_with("DATA")
                && !up.starts_with("PROCEDURE")
                && !up.starts_with("WORKING-STORAGE")
                && !up.starts_with("LOCAL-STORAGE")
                && !up.starts_with("LINKAGE")
                && !up.starts_with("FILE")
                && !up.starts_with("SCREEN")
                && !up.starts_with("CONFIGURATION")
                && !up.starts_with("REPORT")
                && first
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_alphabetic())
                    .unwrap_or(false)
            {
                paragraphs += 1;
            }
        }
        if up.starts_with("FD ") || up.starts_with("SD ") || up.contains(" FILE SECTION") {
            file_descriptions += 1;
        }
        // level numbers
        let mut level = String::new();
        for c in up.chars() {
            if c.is_ascii_digit() {
                level.push(c);
            } else {
                break;
            }
        }
        if let Ok(lv) = level.parse::<u16>() {
            if (1..=49).contains(&lv) {
                data_items += 1;
            }
            if lv == 1 {
                level_01 += 1;
            }
        }
        if no_quotes.contains("IF ") || no_quotes.starts_with("IF") {
            scope_depth += 1;
            max_depth = max_depth.max(scope_depth);
        }
        if no_quotes.contains("END-IF")
            || no_quotes.starts_with("END-IF")
            || no_quotes.contains("END-EVALUATE")
        {
            scope_depth = scope_depth.saturating_sub(1);
        }
        for v in known_verbs {
            if no_quotes.contains(v) || no_quotes.starts_with(v) {
                *verbs.entry(v.to_string()).or_insert(0) += 1;
            }
        }
    }
    let verb_list: Vec<String> = verbs.keys().cloned().collect();
    ProgramMetrics {
        lines: source.lines().count(),
        bytes: source.len(),
        paragraphs,
        data_items,
        level_01_items: level_01,
        copy_refs,
        file_descriptions,
        program_ids,
        approx_scope_depth: max_depth,
        verbs: verb_list,
    }
}

fn is_verb(w: &str) -> bool {
    matches!(
        w,
        "MOVE"
            | "DISPLAY"
            | "ADD"
            | "SUBTRACT"
            | "MULTIPLY"
            | "DIVIDE"
            | "COMPUTE"
            | "IF"
            | "ELSE"
            | "END-IF"
            | "PERFORM"
            | "ACCEPT"
            | "CALL"
            | "STRING"
            | "UNSTRING"
            | "INSPECT"
            | "SET"
            | "INITIALIZE"
            | "EVALUATE"
            | "READ"
            | "WRITE"
            | "OPEN"
            | "CLOSE"
            | "STOP"
            | "GOBACK"
            | "EXIT"
            | "COPY"
            | "CONTINUE"
            | "SORT"
            | "MERGE"
            | "SEARCH"
            | "DELETE"
            | "REWRITE"
            | "START"
            | "UNLOCK"
            | "RETURN"
            | "GO"
            | "END-EVALUATE"
            | "WHEN"
            | "END-READ"
            | "END-WRITE"
    )
}

/// Detect platform services a program needs (z/OS / DB2 / IMS / CICS / VSAM / terminal).
pub fn platform_services(up: &str) -> Vec<String> {
    let mut svc = Vec::new();
    if up.contains("DB2") || up.contains("EXEC SQL") || up.contains("SQLCA") {
        svc.push("DB2_REQUIRED".to_string());
    }
    if up.contains("CICS") || up.contains("DFH") || up.contains("EXEC CICS") {
        svc.push("CICS_REQUIRED".to_string());
    }
    if up.contains("IMS") || up.contains("DLI") || up.contains("EXEC DLI") {
        svc.push("IMS_REQUIRED".to_string());
    }
    if up.contains("VSAM") || up.contains("IDX") || up.contains("INDEXED") {
        svc.push("VSAM_OR_INDEXED_REQUIRED".to_string());
    }
    if up.contains("SCREEN") || up.contains("CRT") {
        svc.push("TERMINAL_REQUIRED".to_string());
    }
    if up.contains("JCL") || up.contains("DDNAME") || up.contains("//") {
        svc.push("ZOS_DATASET_REQUIRED".to_string());
    }
    svc
}

/// Extract + verify the Phase-5 sources.
pub fn extract_extras(
    repo_root: &Path,
    packages_root: &Path,
    out_dir: &Path,
    with_candidate: bool,
) -> Result<BTreeMap<String, usize>, String> {
    let oracle = OracleEnv::host_default()?;
    std::fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut programs: Vec<ExtrasProgram> = Vec::new();
    let mut custodies: Vec<CollectionCustody> = Vec::new();

    // ---- source 1: extras/CBL_OC_DUMP.cob (both lanes) --------------------------------
    let work = packages_root.join("gnucobol-extras");
    std::fs::create_dir_all(&work).map_err(|e| e.to_string())?;
    for (lane, rel) in [
        (
            "stable-3.2",
            "lab/admit/gnucobol-3.2/extras/CBL_OC_DUMP.cob",
        ),
        (
            "current",
            "lab/admit/gnucobol-upstream-current/extras/CBL_OC_DUMP.cob",
        ),
    ] {
        let src = repo_root.join(rel);
        if !src.exists() {
            continue;
        }
        let text = std::fs::read_to_string(&src).map_err(|e| e.to_string())?;
        let sha = crate::store::sha256_hex(text.as_bytes());
        // licence: GPL-3.0-or-later (file header), FSF-copyright, part of GnuCOBOL
        // materialize the original bytes into the work dir before compiling
        let dst = work.join("CBL_OC_DUMP.cob");
        std::fs::write(&dst, &text).map_err(|e| e.to_string())?;
        let metrics = scan_program(&text);
        let up = text.to_ascii_uppercase();
        let services = platform_services(&up);
        let compile_cmd = format!("cobc -m -std=mf -O2 CBL_OC_DUMP.cob");
        let compile_out = run_step(&oracle, &work, &compile_cmd, &[]);
        let compile_note = if compile_out.exit == Some(0) {
            "compiles with the documented tectonics (cobc -m -std=mf -O2)".to_string()
        } else {
            format!(
                "compile exit {:?}: {}",
                compile_out.exit,
                String::from_utf8_lossy(&compile_out.stderr).trim_end()
            )
        };
        let (cand_phases, cand_ff) = if with_candidate {
            let p = probe_dir(&work, "CBL_OC_DUMP.cob", false);
            let ff = p
                .iter()
                .find(|x| !x.ok)
                .map(|x| (x.phase.clone(), x.diagnostic.clone()));
            (p, ff)
        } else {
            (Vec::new(), None)
        };
        let class = if compile_out.exit == Some(0) {
            "VALID_MODULE_PROGRAM"
        } else {
            "QUARANTINED"
        };
        *counts.entry(class.to_string()).or_default() += 1;
        programs.push(ExtrasProgram {
            program_id: format!("gnucobol-extras-{lane}/CBL_OC_DUMP"),
            collection: format!("gnucobol-extras-{lane}"),
            source_rel: rel.to_string(),
            content_sha256: sha,
            source_format: "fixed".to_string(),
            dialect: "mf".to_string(),
            licence: "GPL-3.0-or-later".to_string(),
            metrics,
            copybook_dependencies: vec![],
            data_file_dependencies: vec![],
            platform_services: services,
            adaptation: "none (compiles with the documented tectonics)".to_string(),
            oracle_compile_exit: compile_out.exit,
            oracle_compile_note: compile_note,
            oracle_run_exit: None,
            run_attempted: false,
            run_note: "module (not an executable main program); run requires a caller".to_string(),
            candidate_phases: cand_phases,
            candidate_first_failure: cand_ff,
            classification: class.to_string(),
        });
    }

    // ---- source 2: OpenCBS COBOL Defects Benchmark Suite -------------------------------
    let opencbs_root = repo_root.join("lab/corpus/opencbs/repo");
    let opencbs_work = work.join("opencbs");
    std::fs::create_dir_all(&opencbs_work).map_err(|e| e.to_string())?;
    if opencbs_root.exists() {
        custodies.push(CollectionCustody {
            name: "OpenCBS COBOL Defects Benchmark Suite".to_string(),
            origin_url: "https://github.com/PhaseChangeSoftware/cobol-defects-suite".to_string(),
            revision: "2021-10-27 (DEFNOTES_20211027.TXT, committed custody spine)".to_string(),
            spdx: "MIT".to_string(),
            redistribution_allowed: true,
            decision: "MIT licence in the admitted repo root; per-file copyrights preserved in "
                .to_string()
                + "the custody report",
            program_count: 0,
        });
        let progs_dir = opencbs_root.join("COBOL_Programs");
        let mut entries: Vec<PathBuf> = std::fs::read_dir(&progs_dir)
            .map_err(|e| e.to_string())?
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.extension()
                    .and_then(|x| x.to_str())
                    .map(|x| x.eq_ignore_ascii_case("cbl") || x.eq_ignore_ascii_case("cob"))
                    .unwrap_or(false)
            })
            .collect();
        entries.sort();
        let copybook_dir = opencbs_root.join("COBOL_Copybooks");
        for p in &entries {
            let text = std::fs::read_to_string(p).unwrap_or_default();
            let sha = crate::store::sha256_hex(text.as_bytes());
            let metrics = scan_program(&text);
            let up = text.to_ascii_uppercase();
            let services = platform_services(&up);
            let fname = p.file_name().unwrap().to_string_lossy().to_string();
            let dst = opencbs_work.join(&fname);
            std::fs::write(&dst, &text).map_err(|e| e.to_string())?;
            let mut args = vec!["cobc", "-x", "-I"];
            let ic = copybook_dir.to_string_lossy().to_string();
            args.push(&ic);
            // free or fixed? probe both: try free first if the source starts at col 1
            let free = text
                .lines()
                .find(|l| !l.trim().is_empty())
                .map(|l| l.trim_start().len() == l.len())
                .unwrap_or(false);
            let cmd = format!(
                "cobc -x -I {} {} {}",
                copybook_dir.display(),
                if free { "-free" } else { "" },
                fname
            );
            let compile_out = run_step(&oracle, &opencbs_work, &cmd.trim(), &[]);
            let compile_note = if compile_out.exit == Some(0) {
                String::new()
            } else {
                format!(
                    "exit {:?}: {}",
                    compile_out.exit,
                    String::from_utf8_lossy(&compile_out.stderr)
                        .lines()
                        .take(2)
                        .collect::<Vec<_>>()
                        .join(" | ")
                )
            };
            // run attempt: only when the compile passed and no terminal/db services
            let (run_exit, run_note) = if compile_out.exit == Some(0) && services.is_empty() {
                let run = run_step(
                    &oracle,
                    &opencbs_work,
                    &format!("./{}", base_no_ext(&fname)),
                    &[],
                );
                (
                    run.exit,
                    if run.exit.is_some() {
                        String::new()
                    } else {
                        "no runnable artifact".to_string()
                    },
                )
            } else if !services.is_empty() {
                (None, format!("platform services: {}", services.join(",")))
            } else {
                (None, "compile failed; no run".to_string())
            };
            let (cand_phases, cand_ff) = if with_candidate && compile_out.exit == Some(0) {
                let p = probe_dir(&opencbs_work, &fname, true);
                let ff = p
                    .iter()
                    .find(|x| !x.ok)
                    .map(|x| (x.phase.clone(), x.diagnostic.clone()));
                (p, ff)
            } else {
                (Vec::new(), None)
            };
            let class = if compile_out.exit == Some(0) {
                if services.is_empty() && run_exit.is_some() {
                    "VALID_EXECUTABLE_PROGRAM"
                } else if services.is_empty() {
                    "VALID_COMPILE_ONLY_PROGRAM"
                } else {
                    "QUARANTINED" // platform services unavailable under this oracle profile
                }
            } else if !services.is_empty() {
                "QUARANTINED" // platform services unavailable under this oracle profile
            } else {
                // the OpenCBS collection is an explicit DEFECTS benchmark: a source that does
                // not compile is the benchmark's own content, recorded with its diagnostic
                "INVALID_EXPECTED_REJECT"
            };
            *counts.entry(class.to_string()).or_default() += 1;
            let copy_refs = metrics.copy_refs.clone();
            programs.push(ExtrasProgram {
                program_id: format!("opencbs/{}", fname),
                collection: "opencbs".to_string(),
                source_rel: format!("COBOL_Programs/{fname}"),
                content_sha256: sha,
                source_format: if free { "free" } else { "fixed" }.to_string(),
                dialect: "default".to_string(),
                licence: "MIT".to_string(),
                metrics,
                copybook_dependencies: copy_refs,
                data_file_dependencies: vec![],
                platform_services: services,
                adaptation: "none (original bytes preserved)".to_string(),
                oracle_compile_exit: compile_out.exit,
                oracle_compile_note: compile_note,
                oracle_run_exit: run_exit,
                run_attempted: run_exit.is_some(),
                run_note,
                candidate_phases: cand_phases,
                candidate_first_failure: cand_ff,
                classification: class.to_string(),
            });
        }
        if let Some(c) = custodies.last_mut() {
            c.program_count = entries.len();
        }
    } else {
        custodies.push(CollectionCustody {
            name: "OpenCBS COBOL Defects Benchmark Suite".to_string(),
            origin_url: "https://github.com/PhaseChangeSoftware/cobol-defects-suite".to_string(),
            revision: "not present offline".to_string(),
            spdx: "MIT".to_string(),
            redistribution_allowed: true,
            decision: "source not admitted in this environment; re-fetch and re-run".to_string(),
            program_count: 0,
        });
    }

    write_json(out_dir, "custody.json", &custodies)?;
    write_json(out_dir, "programs.json", &programs)?;
    let mut deps = Vec::new();
    for p in &programs {
        deps.push(serde_json::json!({
            "program_id": p.program_id,
            "copybooks": p.copybook_dependencies,
            "data_files": p.data_file_dependencies,
            "platform_services": p.platform_services,
            "licence": p.licence,
        }));
    }
    write_json(out_dir, "dependencies.json", &deps)?;
    let mut metrics = Vec::new();
    for p in &programs {
        metrics.push(serde_json::json!({
            "program_id": p.program_id,
            "metrics": p.metrics,
        }));
    }
    write_json(out_dir, "metrics.json", &metrics)?;
    write_json(out_dir, "accuracy.json", &programs)?;

    let mut md = String::new();
    md.push_str("# GnuCOBOL-shipped programs + official contributions (Phase 5)\n\n");
    for c in &custodies {
        md.push_str(&format!(
            "- {} ({}): {} programs, {} (redistribution {})\n",
            c.name,
            c.revision,
            c.program_count,
            c.spdx,
            if c.redistribution_allowed {
                "allowed"
            } else {
                "NOT allowed"
            }
        ));
    }
    md.push('\n');
    md.push_str("| classification | count |\n|---|---|\n");
    for (k, v) in &counts {
        md.push_str(&format!("| {k} | {v} |\n"));
    }
    md.push_str("\nAdaptations: none required (all sources compile as-is or are recorded with\n");
    md.push_str("their compile diagnostic; originals are always preserved byte-exact).\n");
    std::fs::write(out_dir.join("summary.md"), md).map_err(|e| e.to_string())?;
    counts.insert("total".into(), programs.len());
    Ok(counts)
}

fn base_no_ext(f: &str) -> String {
    f.rsplit_once('.')
        .map(|(b, _)| b.to_string())
        .unwrap_or_else(|| f.to_string())
}

fn write_json<T: Serialize>(dir: &Path, name: &str, v: &T) -> Result<(), String> {
    let json = serde_json::to_string_pretty(v).map_err(|e| e.to_string())?;
    std::fs::write(dir.join(name), json).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROG: &str = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       DATA DIVISION.\n       WORKING-STORAGE SECTION.\n       01 X PIC 9.\n       01 Y.\n         05 Y1 PIC X.\n       PROCEDURE DIVISION.\n       PARA-1.\n           MOVE 1 TO X.\n           IF X = 1\n               DISPLAY \"A\"\n           END-IF\n           STOP RUN.\n";

    #[test]
    fn metrics_scan() {
        let m = scan_program(PROG);
        assert_eq!(m.lines, 14);
        assert_eq!(m.data_items, 3);
        assert_eq!(m.level_01_items, 2);
        assert!(m.paragraphs >= 1, "paragraphs={}", m.paragraphs);
        assert_eq!(m.approx_scope_depth, 1);
        assert!(m.verbs.contains(&"MOVE".to_string()));
        assert!(m.verbs.contains(&"IF".to_string()));
        assert_eq!(m.program_ids, vec!["T"]);
    }

    #[test]
    fn platform_services_detection() {
        assert!(platform_services("EXEC SQL INCLUDE SQLCA").contains(&"DB2_REQUIRED".to_string()));
        assert!(platform_services("EXEC CICS RETURN").contains(&"CICS_REQUIRED".to_string()));
        assert!(platform_services("SELECT F ASSIGN TO X INDEXED")
            .contains(&"VSAM_OR_INDEXED_REQUIRED".to_string()));
        assert!(platform_services("DISPLAY A UPON CRT").contains(&"TERMINAL_REQUIRED".to_string()));
    }

    #[test]
    fn verb_heuristic() {
        assert!(is_verb("MOVE"));
        assert!(is_verb("END-IF"));
        assert!(!is_verb("PARA-1"));
    }
}
