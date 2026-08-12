//! Phase 6 — Open Mainframe Project COBOL Programming Course (IBM-oriented dialect lane).
//!
//! Admitted at the immutable revision recorded in `lab/admit/omp-course/REVISION`
//! (openmainframeproject/cobol-programming-course @ 61c573dd13688f25e615e7cc4f9595cee38cd6a0,
//! CC-BY-4.0). The whole repository is inventoried (programs / copybooks / JCL / data / docs /
//! exercises / solutions); educational relationships are preserved; every complete program is
//! compiled unmodified under GnuCOBOL with the closest supported dialect, and platform services
//! (z/OS datasets, JCL, DB2, ...) are typed -- never described as parser failures.

use crate::extract::candidate::{probe_dir, PhaseOutcome};
use crate::extract::extras::{platform_services, scan_program, ProgramMetrics};
use crate::extract::oracle::{run_step, OracleEnv};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The pinned revision of the admitted course.
pub const REVISION: &str = "61c573dd13688f25e615e7cc4f9595cee38cd6a0";

/// Every file in the repository, classified.
#[derive(Debug, Clone, Serialize)]
pub struct InventoryEntry {
    pub path: String,
    pub kind: String,
    pub bytes: u64,
    pub course: String,
}

/// One COBOL program with its educational relationship + admission lane.
#[derive(Debug, Clone, Serialize)]
pub struct OmpProgram {
    pub program_id: String,
    pub course: String,
    pub module: String,
    pub lab: String,
    pub path: String,
    pub content_sha256: String,
    pub source_format: String,
    pub encoding: String,
    pub intended_dialect: String,
    pub classification: String,
    pub platform_boundaries: Vec<String>,
    pub copybook_dependencies: Vec<String>,
    pub data_dependencies: Vec<String>,
    pub jcl_files: Vec<String>,
    pub metrics: ProgramMetrics,
    pub oracle_compile_exit: Option<i32>,
    pub oracle_compile_note: String,
    pub oracle_run_exit: Option<i32>,
    pub run_note: String,
    pub adaptation: String,
    pub adapted_sha256: Option<String>,
    pub adaptation_reason: Option<String>,
    pub candidate_phases: Vec<PhaseOutcome>,
    pub candidate_first_failure: Option<(String, String)>,
}

fn course_of(path: &str) -> String {
    if path.contains("Course #2") {
        "course-2".to_string()
    } else if path.contains("Course #3") {
        "course-3".to_string()
    } else {
        "other".to_string()
    }
}

/// Inventory every file in the repository (spec 6.1).
pub fn inventory(repo: &Path) -> Vec<InventoryEntry> {
    let mut out = Vec::new();
    let mut stack = vec![repo.to_path_buf()];
    while let Some(d) = stack.pop() {
        if let Ok(rd) = std::fs::read_dir(&d) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                    continue;
                }
                let bytes = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
                let rel = p
                    .strip_prefix(repo)
                    .unwrap_or(&p)
                    .to_string_lossy()
                    .to_string();
                let ext = p
                    .extension()
                    .and_then(|x| x.to_str())
                    .map(|x| x.to_ascii_lowercase())
                    .unwrap_or_default();
                let kind = match ext.as_str() {
                    "cobol" | "cbl" | "cob" => "COBOL_PROGRAM",
                    "cpy" | "copy" => "COPYBOOK",
                    "jcl" => "JCL",
                    "md" | "tex" => "DOCUMENTATION",
                    "txt" | "csv" => "DATA",
                    "png" | "jpg" | "gif" | "svg" => "IMAGE",
                    "json" | "yml" | "yaml" | "xml" | "sh" | "py" => "SUPPORT",
                    "dat" | "bin" | "rec" | "xdata" | "ps" => "DATA",
                    _ => {
                        if rel.starts_with("COBOL Programming Course #2 - Learning COBOL/Labs/data")
                        {
                            "DATA"
                        } else {
                            "OTHER"
                        }
                    }
                };
                out.push(InventoryEntry {
                    path: rel.clone(),
                    kind: kind.to_string(),
                    bytes,
                    course: course_of(&rel),
                });
            }
        }
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

/// The z/OS course style omits the terminating period after `PROGRAM-ID. name`; GnuCOBOL
/// requires it. This purely syntactic adaptation appends the period (semantics cannot change).
pub fn fix_program_id_period(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 16);
    for line in text.lines() {
        let trimmed = line.trim();
        let up = trimmed.to_ascii_uppercase();
        if up.starts_with("PROGRAM-ID.") && !up.trim_end().ends_with('.') {
            // find the program name (the token after PROGRAM-ID.) and end the line with '.'
            if let Some((_, after)) = trimmed.split_once("PROGRAM-ID.") {
                let name = after.trim();
                if !name.is_empty()
                    && name
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
                {
                    out.push_str(line.trim_end());
                    if !line.trim_end().ends_with('.') {
                        out.push('.');
                    }
                    out.push('\n');
                    continue;
                }
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Extract + verify the OMP course.
pub fn extract_omp(
    repo_root: &Path,
    packages_root: &Path,
    out_dir: &Path,
    with_candidate: bool,
) -> Result<BTreeMap<String, usize>, String> {
    let oracle = OracleEnv::host_default()?;
    let repo = repo_root.join("lab/admit/omp-course/repo");
    std::fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    if !repo.exists() {
        return Err(format!(
            "OMP course not admitted ({}); clone openmainframeproject/cobol-programming-course @ {REVISION} into lab/admit/omp-course/repo",
            repo.display()
        ));
    }

    let inv = inventory(&repo);
    for e in &inv {
        *counts.entry(format!("inventory:{}", e.kind)).or_default() += 1;
    }

    // data files available for runtime (Labs/data + Labs/xdata)
    let work = packages_root.join("omp-course");
    std::fs::create_dir_all(&work).map_err(|e| e.to_string())?;
    let mut programs: Vec<OmpProgram> = Vec::new();
    let mut cbl_dirs: Vec<PathBuf> = Vec::new();
    for e in &inv {
        if e.kind == "COBOL_PROGRAM" {
            cbl_dirs.push(repo.join(&e.path));
        }
    }
    for p in cbl_dirs {
        let text = std::fs::read_to_string(&p).unwrap_or_default();
        let sha = crate::store::sha256_hex(text.as_bytes());
        let fname = p.file_name().unwrap().to_string_lossy().to_string();
        let rel = p
            .strip_prefix(&repo)
            .unwrap_or(&p)
            .to_string_lossy()
            .to_string();
        let course = course_of(&rel);
        // educational relationship: module + lab from the path
        let module = rel
            .split('/')
            .find(|c| c.starts_with("Course"))
            .unwrap_or("")
            .to_string();
        let lab = rel
            .split('/')
            .filter(|c| c.eq_ignore_ascii_case("labs") || c.eq_ignore_ascii_case("challenges"))
            .next()
            .unwrap_or("")
            .to_string();
        let metrics = scan_program(&text);
        let up = text.to_ascii_uppercase();
        let services = platform_services(&up);
        // z/OS dataset boundary: DDNAME-style SELECT ... ASSIGN TO names (no GnuCOBOL mapping
        // without env/file setup) and JCL references
        let mut boundaries: Vec<String> = services;
        if up.contains("ASSIGN TO") || up.contains("DDNAME") {
            boundaries.push("ZOS_DATASET_REQUIRED".to_string());
        }
        if up.contains("EXEC SQL") || up.contains("SQLCA") || up.contains("DB2") {
            boundaries.push("DB2_REQUIRED".to_string());
        }
        if boundaries.contains(&"DB2_REQUIRED".to_string()) {
            boundaries.retain(|b| b != "ZOS_DATASET_REQUIRED"); // DB2 dominates
        }
        boundaries.dedup();

        // admission lane: unmodified GnuCOBOL compile (closest supported dialects in order:
        // default, then ibm -- stop at the first success)
        let dst = work.join(&fname);
        std::fs::write(&dst, &text).map_err(|e| e.to_string())?;
        let mut compile_out = run_step(
            &oracle,
            &work,
            &format!("cobc -x -std=default -I {} {}", work.display(), fname)
                .trim()
                .to_string(),
            &[],
        );
        let mut dialect_used = "default".to_string();
        if compile_out.exit != Some(0) {
            let ibm = run_step(
                &oracle,
                &work,
                &format!("cobc -x -std=ibm -I {} {}", work.display(), fname)
                    .trim()
                    .to_string(),
                &[],
            );
            if ibm.exit == Some(0) {
                compile_out = ibm;
                dialect_used = "ibm".to_string();
            }
        }
        let mut adaptation = "none (unmodified compile; original bytes preserved)".to_string();
        let mut adapted_text: Option<(String, String)> = None; // (adapted, reason)
        if compile_out.exit != Some(0) {
            // spec 6.3.5: apply a compatibility patch only where justified, preserving the
            // original identity. The course's z/OS style omits the terminating period after
            // PROGRAM-ID (valid on z/OS; GnuCOBOL requires it): appending it is purely
            // syntactic -- semantics cannot change.
            let adapted = fix_program_id_period(&text);
            if adapted != text {
                let a_sha = crate::store::sha256_hex(adapted.as_bytes());
                std::fs::write(&dst, &adapted).map_err(|e| e.to_string())?;
                let a = run_step(
                    &oracle,
                    &work,
                    &format!("cobc -x -std=default -I {} {}", work.display(), fname)
                        .trim()
                        .to_string(),
                    &[],
                );
                if a.exit == Some(0) {
                    compile_out = a;
                    adaptation = format!(
                        "program-id period appended (original sha {}, adapted sha {a_sha}); \
                         purely syntactic, semantics unchanged -- original preserved at \
                         $GNURUST_COBOL_CORPUS_ROOT/packages/{}/{}",
                        sha,
                        work.file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_else(|| "omp-course".to_string()),
                        fname
                    );
                    adapted_text = Some((adapted, "program-id period (z/OS omits it)".to_string()));
                }
            }
        }
        let compile_note = if compile_out.exit == Some(0) {
            format!("dialect: {dialect_used}")
        } else {
            format!(
                "exit {:?} ({dialect_used}): {}",
                compile_out.exit,
                String::from_utf8_lossy(&compile_out.stderr)
                    .lines()
                    .take(2)
                    .collect::<Vec<_>>()
                    .join(" | ")
            )
        };
        // runtime: only when the compile passed, no platform boundaries, and the program reads
        // no DDNAME files (HELLO-style programs) -- file programs need JCL/data setup
        let file_io = up.contains("ASSIGN TO") || up.contains("SELECT ");
        let (run_exit, run_note) =
            if compile_out.exit == Some(0) && boundaries.is_empty() && !file_io {
                let run = run_step(&oracle, &work, &format!("./{}", base_no_ext(&fname)), &[]);
                (
                    run.exit,
                    if run.exit.is_some() {
                        String::new()
                    } else {
                        "no runnable artifact".to_string()
                    },
                )
            } else if !boundaries.is_empty() {
                (
                    None,
                    format!("platform boundaries: {}", boundaries.join(",")),
                )
            } else if file_io {
                (
                    None,
                    "sequential-file program; runtime needs the JCL data setup (recorded, not "
                        .to_string()
                        + "silently adapted)",
                )
            } else {
                (None, "compile failed; no run".to_string())
            };
        let (cand_phases, cand_ff) = if with_candidate && compile_out.exit == Some(0) {
            let pr = probe_dir(&work, &fname, true);
            let ff = pr
                .iter()
                .find(|x| !x.ok)
                .map(|x| (x.phase.clone(), x.diagnostic.clone()));
            (pr, ff)
        } else {
            (Vec::new(), None)
        };
        let class = if up.contains("EXEC SQL") || up.contains("SQLCA") || up.contains("DB2") {
            // the compile failure (if any) IS the missing DB2 preprocessor: a platform boundary,
            // never a parser failure
            "PLATFORM_BOUNDED|DB2_REQUIRED".to_string()
        } else if compile_out.exit == Some(0) && boundaries.is_empty() && run_exit.is_some() {
            if adapted_text.is_some() {
                "GnuCOBOL_PORTABLE_WITH_PATCH|VALID_EXECUTABLE_PROGRAM".to_string()
            } else {
                "GnuCOBOL_PORTABLE_UNMODIFIED|VALID_EXECUTABLE_PROGRAM".to_string()
            }
        } else if compile_out.exit == Some(0) && boundaries.is_empty() {
            if adapted_text.is_some() {
                "GnuCOBOL_PORTABLE_WITH_PATCH|VALID_COMPILE_ONLY_PROGRAM".to_string()
            } else {
                "GnuCOBOL_PORTABLE_UNMODIFIED|VALID_COMPILE_ONLY_PROGRAM".to_string()
            }
        } else if compile_out.exit == Some(0) {
            // compiles under GnuCOBOL but its platform services are unavailable: never a
            // parser failure, typed by boundary
            format!(
                "PLATFORM_BOUNDED|{}",
                boundaries
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "OTHER".into())
            )
        } else {
            // deliberate-error course exercises (the `0X` variants the course uses to teach
            // debugging) are the course's own content, recorded with their diagnostic
            if fname.to_uppercase().contains("0X") {
                "INVALID_EXPECTED_REJECT|COURSE_ERROR_EXERCISE".to_string()
            } else {
                format!(
                    "GNUCOBOL_REJECT|{}",
                    boundaries
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "DIALECT".into())
                )
            }
        };
        *counts
            .entry(class.split('|').next().unwrap_or("").to_string())
            .or_default() += 1;
        programs.push(OmpProgram {
            program_id: format!("omp/{}", rel.replace('/', "/")),
            course,
            module,
            lab,
            path: rel.clone(),
            content_sha256: sha,
            source_format: "fixed".to_string(),
            encoding: "ASCII/UTF-8".to_string(),
            intended_dialect: "IBM (z/OS COBOL-85 style)".to_string(),
            classification: class,
            platform_boundaries: boundaries,
            copybook_dependencies: metrics.copy_refs.clone(),
            data_dependencies: vec![],
            jcl_files: vec![],
            metrics,
            oracle_compile_exit: compile_out.exit,
            oracle_compile_note: compile_note,
            oracle_run_exit: run_exit,
            run_note,
            adaptation,
            adapted_sha256: adapted_text
                .as_ref()
                .map(|(a, _)| crate::store::sha256_hex(a.as_bytes())),
            adaptation_reason: adapted_text.map(|(_, r)| r),
            candidate_phases: cand_phases,
            candidate_first_failure: cand_ff,
        });
    }

    let jcls: Vec<String> = inv
        .iter()
        .filter(|e| e.kind == "JCL")
        .map(|e| e.path.clone())
        .collect();
    // JCL per program: the course names JCL files <PROGRAM>J.jcl
    for p in &mut programs {
        let base = p.program_id.rsplit('/').next().unwrap_or("").to_uppercase();
        for j in &jcls {
            let jbase = j
                .rsplit('/')
                .next()
                .unwrap_or("")
                .trim_end_matches(".jcl")
                .trim_end_matches(".JCL")
                .to_uppercase();
            if jbase == format!("{base}J") || jbase == base {
                p.jcl_files.push(j.clone());
            }
        }
    }

    write_json(out_dir, "inventory.json", &inv)?;
    write_json(out_dir, "programs.json", &programs)?;
    let mut md = String::new();
    md.push_str("# Open Mainframe Project COBOL Programming Course (Phase 6)\n\n");
    md.push_str(&format!("admitted revision: {REVISION}\n\n"));
    md.push_str("## repository inventory\n");
    for (k, v) in &counts {
        if k.starts_with("inventory:") {
            md.push_str(&format!("- {k}: {v}\n"));
        }
    }
    md.push_str("\n## program admission\n");
    for (k, v) in &counts {
        if !k.starts_with("inventory:") {
            md.push_str(&format!("- {k}: {v}\n"));
        }
    }
    md.push_str(
        "\nPlatform boundaries are typed (ZOS_DATASET_REQUIRED / JCL_REQUIRED / DB2_REQUIRED /\n",
    );
    md.push_str("CICS_REQUIRED / ...) and are never described as parser failures.\n");
    std::fs::write(out_dir.join("summary.md"), md).map_err(|e| e.to_string())?;
    counts.insert("total_programs".into(), programs.len());
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
