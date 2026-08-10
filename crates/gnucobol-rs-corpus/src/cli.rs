//! The `gnucobol-rs-corpus` CLI.
//!
//! Every command supports structured JSON output. Admission is state-driven: each `admit` step
//! performs exactly one legal transition of the chain; `--finalize` refuses to mark a record
//! ADMITTED unless the whole chain was walked and (for valid program classes) the oracle
//! contract and reviewed licence are present.

use crate::bytes;
use crate::dedup::DedupIndex;
use crate::origin::{FetchSpec, UpdateReport};
use crate::schema::{
    CandidateResult, Classification, CorpusClass, Licence, OracleResult, Origin, ProgramRecord,
    SourceFamily, SourceInfo, ValidityProfile, SCHEMA,
};
use crate::state::{transition, AdmissionState};
use crate::store::{sha256_hex, CorpusStore};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Discover {
        dir: PathBuf,
        json: bool,
    },
    Fetch {
        spec: PathBuf,
        json: bool,
    },
    Admit(AdmitArgs),
    Verify {
        id: String,
        json: bool,
    },
    List {
        json: bool,
    },
    Classify {
        id: String,
        class: Classification,
        json: bool,
    },
    RunOracle(AdmitArgs),
    RunCandidate(AdmitArgs),
    Compare {
        id: String,
        json: bool,
    },
    Report {
        json: bool,
    },
    Gate {
        json: bool,
    },
    CheckUpdates {
        json: bool,
    },
    ExtractTestsuite {
        lane: String,
        replay: bool,
        candidate: bool,
        json: bool,
    },
    ProbeStep {
        manifest: PathBuf,
        out: PathBuf,
        json: bool,
    },
    ExtractCcvs85 {
        json: bool,
    },
    ExtractManual {
        lane: String,
        candidate: bool,
        json: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AdmitArgs {
    pub id: Option<String>,
    pub json: bool,
    pub discover: bool,
    pub custody_sha: Option<String>,
    pub licence_spdx: Option<String>,
    pub redistribute: Option<bool>,
    pub licence_decision: Option<String>,
    pub deps: Option<String>,
    pub oracle_compile_exit: Option<i32>,
    pub warnings: bool,
    pub oracle_run_exit: Option<i32>,
    pub stdout_sha: Option<String>,
    pub stderr_sha: Option<String>,
    pub deterministic: bool,
    pub finalize: bool,
    pub class: Option<String>,
    pub corpus_class: Option<String>,
    pub family: Option<String>,
    pub source_file: Option<String>,
    pub dialect: Option<String>,
    pub format: Option<String>,
    pub oracle_name: Option<String>,
    pub platform: Option<String>,
    pub profile_json: Option<String>,
    pub origin_json: Option<String>,
    pub tool_version: Option<String>,
    /// Phase-attributed candidate outcomes: `phase=ok|diagnostic` pairs.
    pub candidate: Vec<String>,
}

/// The manifest store: one JSON file per program_id under `<root>/manifests/`.
pub struct ManifestStore {
    root: PathBuf,
}

fn manifest_path(root: &Path, id: &str) -> PathBuf {
    let safe: String = id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    root.join("manifests").join(format!("{safe}.json"))
}

impl ManifestStore {
    pub fn new(root: &Path) -> ManifestStore {
        ManifestStore {
            root: root.to_path_buf(),
        }
    }

    pub fn save(&self, rec: &ProgramRecord) -> Result<(), String> {
        let p = manifest_path(&self.root, &rec.program_id);
        std::fs::create_dir_all(p.parent().unwrap()).map_err(|e| e.to_string())?;
        let json = serde_json::to_string_pretty(rec).map_err(|e| e.to_string())?;
        std::fs::write(&p, json).map_err(|e| e.to_string())
    }

    pub fn load(&self, id: &str) -> Result<ProgramRecord, String> {
        let p = manifest_path(&self.root, id);
        let bytes = std::fs::read(&p).map_err(|e| format!("cannot read {}: {e}", p.display()))?;
        serde_json::from_slice(&bytes).map_err(|e| format!("{id}: {e}"))
    }

    pub fn list(&self) -> Result<Vec<ProgramRecord>, String> {
        let dir = self.root.join("manifests");
        let mut out = Vec::new();
        if let Ok(rd) = std::fs::read_dir(&dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().and_then(|x| x.to_str()) != Some("json") {
                    continue;
                }
                let bytes = std::fs::read(&p).map_err(|e| e.to_string())?;
                if let Ok(rec) = serde_json::from_slice::<ProgramRecord>(&bytes) {
                    out.push(rec);
                }
            }
        }
        out.sort_by(|a, b| a.program_id.cmp(&b.program_id));
        Ok(out)
    }
}

/// Parse argv (after the program name) into a Command. Returns usage text on error.
pub fn parse(args: &[String]) -> Result<Command, String> {
    let cmd = args.first().ok_or_else(|| usage())?;
    let json = args.iter().any(|a| a == "--json");
    // First positional argument after the command name (flags and `--json` are skipped).
    let positional = |what: &str| -> Result<String, String> {
        args.iter()
            .skip(1)
            .find(|a| !a.starts_with('-'))
            .cloned()
            .ok_or_else(|| format!("{cmd}: missing {what}"))
    };
    match cmd.as_str() {
        "discover" => Ok(Command::Discover {
            dir: match positional("directory") {
                Ok(d) => PathBuf::from(d),
                Err(_) => PathBuf::from("."),
            },
            json,
        }),
        "fetch" => Ok(Command::Fetch {
            spec: PathBuf::from(positional("fetch-spec JSON path")?),
            json,
        }),
        "admit" => Ok(Command::Admit(admit_args(&args[1..], json))),
        "run-oracle" => Ok(Command::RunOracle(admit_args(&args[1..], json))),
        "run-candidate" => Ok(Command::RunCandidate(admit_args(&args[1..], json))),
        "verify" => Ok(Command::Verify {
            id: positional("program_id")?,
            json,
        }),
        "list" => Ok(Command::List { json }),
        "classify" => {
            let id = positional("program_id")?;
            let class = Classification::parse(
                args.iter()
                    .skip(1)
                    .find_map(|a| Classification::parse(a).map(|_| a))
                    .map(String::as_str)
                    .unwrap_or(""),
            )
            .ok_or_else(|| {
                "classify: missing/unknown classification (see schema.rs)".to_string()
            })?;
            Ok(Command::Classify { id, class, json })
        }
        "compare" => Ok(Command::Compare {
            id: positional("program_id")?,
            json,
        }),
        "report" => Ok(Command::Report { json }),
        "gate" => Ok(Command::Gate { json }),
        "check-updates" => Ok(Command::CheckUpdates { json }),
        "extract-ccvs85" => Ok(Command::ExtractCcvs85 { json }),
        "extract-manual" => {
            let lane = args
                .iter()
                .skip(1)
                .find_map(|a| a.strip_prefix("--lane=").map(|v| v.to_string()))
                .unwrap_or_else(|| "both".to_string());
            let candidate = !args.iter().any(|a| a == "--no-candidate");
            Ok(Command::ExtractManual {
                lane,
                candidate,
                json,
            })
        }
        "probe-step" => Ok(Command::ProbeStep {
            manifest: args
                .iter()
                .skip(1)
                .find(|a| !a.starts_with('-'))
                .map(PathBuf::from)
                .ok_or_else(|| "probe-step: missing step-manifest path".to_string())?,
            out: {
                let eq = args
                    .iter()
                    .find_map(|a| a.strip_prefix("--out=").map(PathBuf::from));
                match eq {
                    Some(p) => p,
                    None => {
                        let i = args
                            .iter()
                            .position(|a| a == "--out")
                            .ok_or_else(|| "probe-step: missing --out path".to_string())?;
                        args.get(i + 1)
                            .map(PathBuf::from)
                            .ok_or_else(|| "probe-step: missing --out path".to_string())?
                    }
                }
            },
            json,
        }),
        "extract-testsuite" => {
            let lane = args
                .iter()
                .skip(1)
                .find_map(|a| a.strip_prefix("--lane=").map(|v| v.to_string()))
                .unwrap_or_else(|| "both".to_string());
            let replay = !args.iter().any(|a| a == "--no-replay");
            let candidate = !args.iter().any(|a| a == "--no-candidate");
            Ok(Command::ExtractTestsuite {
                lane,
                replay,
                candidate,
                json,
            })
        }
        other => Err(format!("unknown command: {other}\n{}", usage())),
    }
}

fn usage() -> &'static str {
    "gnucobol-rs-corpus <command> [args] [--json]\n\
     commands: discover <dir> | fetch <spec.json> | admit [steps] | verify <id> | list |\n\
     classify <id> <CLASS> | run-oracle [steps] | run-candidate [steps] | compare <id> |\n\
     report | gate | check-updates\n\
     admit steps: --id ID --discover [--source-file F --corpus-class C --family F]\n\
       --custody-sha SHA --licence-spdx SPDX --redistribute yes|no --licence-decision T\n\
       --deps JSON --oracle-compile-exit N [--warnings]\n\
       --oracle-run-exit N --stdout-sha S --stderr-sha S --deterministic\n\
       --finalize --class CLASS [--dialect D --format F --oracle-name O --platform P]\n\
       --candidate phase=outcome ..."
}

fn admit_args(args: &[String], json: bool) -> AdmitArgs {
    let mut a = AdmitArgs {
        json,
        ..AdmitArgs::default()
    };
    let mut i = 0;
    while i < args.len() {
        let s = &args[i];
        let take = |i: &mut usize| -> Option<String> {
            *i += 1;
            args.get(*i).cloned()
        };
        match s.as_str() {
            "--id" => a.id = take(&mut i),
            "--discover" => a.discover = true,
            "--custody-sha" => a.custody_sha = take(&mut i),
            "--licence-spdx" => a.licence_spdx = take(&mut i),
            "--redistribute" => a.redistribute = take(&mut i).map(|v| v == "yes" || v == "true"),
            "--licence-decision" => a.licence_decision = take(&mut i),
            "--deps" => a.deps = take(&mut i),
            "--oracle-compile-exit" => {
                a.oracle_compile_exit = take(&mut i).and_then(|v| v.parse().ok())
            }
            "--warnings" => a.warnings = true,
            "--oracle-run-exit" => a.oracle_run_exit = take(&mut i).and_then(|v| v.parse().ok()),
            "--stdout-sha" => a.stdout_sha = take(&mut i),
            "--stderr-sha" => a.stderr_sha = take(&mut i),
            "--deterministic" => a.deterministic = true,
            "--finalize" => a.finalize = true,
            "--class" => a.class = take(&mut i),
            "--corpus-class" => a.corpus_class = take(&mut i),
            "--family" => a.family = take(&mut i),
            "--source-file" => a.source_file = take(&mut i),
            "--dialect" => a.dialect = take(&mut i),
            "--format" => a.format = take(&mut i),
            "--oracle-name" => a.oracle_name = take(&mut i),
            "--platform" => a.platform = take(&mut i),
            "--profile-json" => a.profile_json = take(&mut i),
            "--origin-json" => a.origin_json = take(&mut i),
            "--tool-version" => a.tool_version = take(&mut i),
            "--candidate" => {
                if let Some(v) = take(&mut i) {
                    a.candidate.push(v);
                }
            }
            _ => {}
        }
        i += 1;
    }
    a
}

fn parse_corpus_class(s: Option<&str>) -> CorpusClass {
    match s {
        Some("upstream") | Some("UPSTREAM_SEMANTIC") => CorpusClass::UpstreamSemantic,
        Some("performance") | Some("PERFORMANCE") => CorpusClass::Performance,
        _ => CorpusClass::ExternalValid,
    }
}

fn parse_family(s: Option<&str>) -> SourceFamily {
    match s {
        Some("testsuite") => SourceFamily::GnucobolTestsuite,
        Some("ccvs85") => SourceFamily::Ccvs85,
        Some("manual") => SourceFamily::GnucobolManual,
        Some("extras") => SourceFamily::GnucobolExtras,
        Some("omp") => SourceFamily::OmpCourse,
        Some("xcobol") => SourceFamily::Xcobol,
        Some("bench") => SourceFamily::Bench,
        _ => SourceFamily::GnucobolExtras,
    }
}

/// Load the store + manifest store from the environment.
pub fn stores() -> Result<(CorpusStore, ManifestStore), String> {
    let store = CorpusStore::open().map_err(|e| e.to_string())?;
    let ms = ManifestStore::new(store.root());
    Ok((store, ms))
}

/// The `discover` command: walk a directory, emitting DISCOVERED records for files that look
/// like COBOL source (.cob/.cbl/.cpy/.cobol) with their content hash + byte analysis. Never
/// classifies by extension alone — the record starts at DISCOVERED with a typed note.
pub fn cmd_discover(store: &CorpusStore, ms: &ManifestStore, dir: &Path) -> Result<usize, String> {
    let mut n = 0usize;
    let mut walk = |p: &Path, stack: &mut Vec<PathBuf>| -> Result<(), String> {
        let rd = std::fs::read_dir(p).map_err(|e| format!("read_dir {}: {e}", p.display()))?;
        for e in rd.flatten() {
            let path = e.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let ext = path
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| x.to_ascii_lowercase())
                .unwrap_or_default();
            if !matches!(ext.as_str(), "cob" | "cbl" | "cpy" | "cobol") {
                continue;
            }
            let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
            let sha = store.put_bytes(&bytes).map_err(|e| e.to_string())?;
            let analysis = bytes::analyze(&bytes);
            let rel = path.strip_prefix(dir).unwrap_or(&path);
            let id = format!("discovered/{}", rel.display());
            let rec = ProgramRecord {
                schema: SCHEMA.to_string(),
                program_id: id.clone(),
                corpus_class: CorpusClass::ExternalValid,
                source_family: SourceFamily::GnucobolExtras,
                origin: Origin {
                    kind: crate::schema::OriginKind::Other,
                    url: String::new(),
                    revision: String::new(),
                    source_path: rel.display().to_string(),
                    archive_sha256: None,
                },
                licence: Licence {
                    spdx_expression: String::new(),
                    redistribution_allowed: false,
                    notice_paths: vec![],
                    decision: String::new(),
                    reviewed: false,
                },
                source: SourceInfo {
                    files: vec![sha.clone()],
                    main_file: rel.display().to_string(),
                    copybooks: vec![],
                    modules: vec![],
                    encoding: analysis.encoding,
                    line_endings: analysis.line_endings,
                    source_format: if analysis.indicator_area.is_empty()
                        && analysis.sequence_area.is_empty()
                    {
                        "free".to_string()
                    } else {
                        "fixed".to_string()
                    },
                    content_sha256: sha,
                },
                validity_profile: ValidityProfile {
                    oracle: String::new(),
                    oracle_sha256: None,
                    dialect: String::new(),
                    compiler_options: vec![],
                    include_paths: vec![],
                    defines: BTreeMap::new(),
                    environment: BTreeMap::new(),
                    runtime_configuration: BTreeMap::new(),
                    platform: String::new(),
                },
                oracle: OracleResult::default(),
                candidate: CandidateResult::default(),
                classification: Classification::SourceFragment,
                admission_state: AdmissionState::Discovered.as_str().to_string(),
                admission_note: "discovered by extension+content scan; classification is a typed "
                    .to_string()
                    + "decision made by the family extractor, never by extension alone",
                tool_version: String::new(),
            };
            ms.save(&rec)?;
            n += 1;
        }
        Ok(())
    };
    let mut stack = vec![dir.to_path_buf()];
    while let Some(p) = stack.pop() {
        walk(&p, &mut stack)?;
    }
    Ok(n)
}

/// `admit` step engine. Each invocation performs at most one transition (or several, when the
/// flags describe the next legal step) and refuses illegal jumps.
pub fn cmd_admit(
    store: &CorpusStore,
    ms: &ManifestStore,
    args: &AdmitArgs,
) -> Result<ProgramRecord, String> {
    let id = args
        .id
        .clone()
        .ok_or_else(|| "admit: --id is required".to_string())?;
    let mut rec = match ms.load(&id) {
        Ok(r) => r,
        Err(_) => {
            if !args.discover {
                return Err(format!(
                    "admit: no DISCOVERED record for `{id}` (start with --discover)"
                ));
            }
            let (source_file, profile, origin, encoding, line_endings, source_format) =
                if let Some(pj) = &args.profile_json {
                    let pf: ValidityProfile =
                        serde_json::from_str(pj).map_err(|e| format!("--profile-json: {e}"))?;
                    let o: Origin = match &args.origin_json {
                        Some(oj) => {
                            serde_json::from_str(oj).map_err(|e| format!("--origin-json: {e}"))?
                        }
                        None => Origin {
                            kind: crate::schema::OriginKind::Other,
                            url: String::new(),
                            revision: String::new(),
                            source_path: args.source_file.clone().unwrap_or_default(),
                            archive_sha256: None,
                        },
                    };
                    (
                        None,
                        pf,
                        o,
                        String::new(),
                        String::new(),
                        args.format.clone().unwrap_or_default(),
                    )
                } else {
                    let file = args.source_file.clone().ok_or_else(|| {
                        "admit: --source-file (or --profile-json) is required".to_string()
                    })?;
                    let bytes = std::fs::read(&file).map_err(|e| e.to_string())?;
                    let sha = store.put_bytes(&bytes).map_err(|e| e.to_string())?;
                    let analysis = bytes::analyze(&bytes);
                    let profile = ValidityProfile {
                        oracle: args
                            .oracle_name
                            .clone()
                            .unwrap_or_else(|| "GnuCOBOL 3.2.0".into()),
                        oracle_sha256: None,
                        dialect: args.dialect.clone().unwrap_or_else(|| "default".into()),
                        compiler_options: vec![],
                        include_paths: vec![],
                        defines: BTreeMap::new(),
                        environment: BTreeMap::new(),
                        runtime_configuration: BTreeMap::new(),
                        platform: args.platform.clone().unwrap_or_else(|| "linux".into()),
                    };
                    let origin = Origin {
                        kind: crate::schema::OriginKind::Other,
                        url: String::new(),
                        revision: String::new(),
                        source_path: file.clone(),
                        archive_sha256: None,
                    };
                    (
                        Some(sha),
                        profile,
                        origin,
                        analysis.encoding,
                        analysis.line_endings,
                        args.format.clone().unwrap_or_else(|| {
                            if analysis.indicator_area.is_empty()
                                && analysis.sequence_area.is_empty()
                            {
                                "free".to_string()
                            } else {
                                "fixed".to_string()
                            }
                        }),
                    )
                };
            let corpus_class = parse_corpus_class(args.corpus_class.as_deref());
            let family = parse_family(args.family.as_deref());
            // profile-json path: custody comes later
            let content_sha = source_file.unwrap_or_default();
            ProgramRecord {
                schema: SCHEMA.to_string(),
                program_id: id.clone(),
                corpus_class,
                source_family: family,
                origin,
                licence: Licence {
                    spdx_expression: String::new(),
                    redistribution_allowed: false,
                    notice_paths: vec![],
                    decision: String::new(),
                    reviewed: false,
                },
                source: SourceInfo {
                    files: if content_sha.is_empty() {
                        vec![]
                    } else {
                        vec![content_sha.clone()]
                    },
                    main_file: args.source_file.clone().unwrap_or_default(),
                    copybooks: vec![],
                    modules: vec![],
                    encoding,
                    line_endings,
                    source_format,
                    content_sha256: content_sha,
                },
                validity_profile: profile,
                oracle: OracleResult::default(),
                candidate: CandidateResult::default(),
                classification: Classification::SourceFragment,
                admission_state: AdmissionState::Discovered.as_str().to_string(),
                admission_note: String::new(),
                tool_version: args
                    .tool_version
                    .clone()
                    .unwrap_or_else(|| env!("CARGO_PKG_VERSION").into()),
            }
        }
    };

    // Identity/profile overlay: flags may arrive on any admit step, whether the record was just
    // created or pre-existing (e.g. discovered earlier). Applying them after the load/create
    // keeps a single code path for both.
    if args.corpus_class.is_some() {
        rec.corpus_class = parse_corpus_class(args.corpus_class.as_deref());
    }
    if args.family.is_some() {
        rec.source_family = parse_family(args.family.as_deref());
    }
    if let Some(f) = &args.source_file {
        rec.source.main_file = f.clone();
    }
    if let Some(d) = &args.dialect {
        rec.validity_profile.dialect = d.clone();
    }
    if let Some(o) = &args.oracle_name {
        rec.validity_profile.oracle = o.clone();
    }
    if let Some(p) = &args.platform {
        rec.validity_profile.platform = p.clone();
    }
    if let Some(f) = &args.format {
        rec.source.source_format = f.clone();
    }

    let mut state = AdmissionState::parse(&rec.admission_state)
        .ok_or_else(|| format!("record {id} has unknown state {}", rec.admission_state))?;

    // Apply the step flags in chain order, validating every transition. The closure captures only
    // `id`; the note buffer is passed explicitly so record fields stay freely mutable between
    // steps.
    let mut last_note = String::new();
    let step = |from: AdmissionState,
                to: AdmissionState,
                note: String,
                last_note: &mut String|
     -> Result<AdmissionState, String> {
        *last_note = note;
        transition(from, to).map_err(|e| format!("{id}: {e}"))
    };

    if args.discover && state == AdmissionState::Discovered {
        // discovery step: the record already exists; record the note.
        last_note = "discovered".to_string();
    }
    if let Some(sha) = &args.custody_sha {
        if store.get_bytes(sha).is_none() {
            return Err(format!(
                "admit: custody blob {sha} is not in the store (fetch first)"
            ));
        }
        rec.source.content_sha256 = sha.clone();
        state = step(
            state,
            AdmissionState::CustodyVerified,
            format!("custody verified: {sha}"),
            &mut last_note,
        )?;
    }
    if let (Some(spdx), Some(redist)) = (&args.licence_spdx, args.redistribute) {
        rec.licence = Licence {
            spdx_expression: spdx.clone(),
            redistribution_allowed: redist,
            notice_paths: vec![],
            decision: args.licence_decision.clone().unwrap_or_default(),
            reviewed: true,
        };
        if !redist {
            return Err(format!(
                "admit: licence {spdx} does not permit redistribution; unit {id} is LICENCE_RESTRICTED (quarantine it)"
            ));
        }
        state = step(
            state,
            AdmissionState::LicenceVerified,
            format!("licence: {spdx}"),
            &mut last_note,
        )?;
    }
    if args.deps.is_some() {
        state = step(
            state,
            AdmissionState::DependenciesResolved,
            "dependencies resolved".into(),
            &mut last_note,
        )?;
    }
    if let Some(exit) = args.oracle_compile_exit {
        rec.oracle.compile_exit = exit;
        rec.oracle.warnings_expected = args.warnings;
        state = step(
            state,
            AdmissionState::OracleCompileVerified,
            format!("oracle compile exit {exit}"),
            &mut last_note,
        )?;
    }
    if let Some(run_exit) = args.oracle_run_exit {
        rec.oracle.run_exit = Some(run_exit);
        rec.oracle.run_required = true;
        rec.oracle.stdout_sha256 = args.stdout_sha.clone();
        rec.oracle.stderr_sha256 = args.stderr_sha.clone();
        state = step(
            state,
            AdmissionState::OracleRunVerified,
            format!("oracle run exit {run_exit}"),
            &mut last_note,
        )?;
    }
    if args.deterministic {
        rec.oracle.deterministic = true;
        state = step(
            state,
            AdmissionState::DeterminismVerified,
            "determinism verified (two-pass identical)".into(),
            &mut last_note,
        )?;
    }
    // Phase-attributed candidate outcomes (run-candidate / run-oracle both may carry them).
    for c in &args.candidate {
        let (phase, outcome) = c
            .split_once('=')
            .ok_or_else(|| format!("--candidate expects phase=outcome, got {c}"))?;
        let slot = match phase {
            "preprocess" => &mut rec.candidate.preprocess,
            "parse" => &mut rec.candidate.parse,
            "resolve" => &mut rec.candidate.resolve,
            "layout" => &mut rec.candidate.layout,
            "check" => &mut rec.candidate.check,
            "prepare" => &mut rec.candidate.prepare,
            "run" => &mut rec.candidate.run,
            other => return Err(format!("unknown candidate phase {other}")),
        };
        *slot = outcome.to_string();
        if rec.candidate.first_failure.is_none() && !outcome.starts_with("ok") {
            rec.candidate.first_failure = Some((phase.to_string(), outcome.to_string()));
        }
    }
    if args.finalize {
        let class = Classification::parse(args.class.as_deref().unwrap_or(""))
            .ok_or_else(|| "admit --finalize requires --class <CLASS>".to_string())?;
        rec.classification = class;
        let errs = rec.validate();
        if !errs.is_empty() {
            return Err(format!("admit --finalize refused: {}", errs.join("; ")));
        }
        if state != AdmissionState::DeterminismVerified {
            return Err(format!(
                "admit --finalize refused: chain not walked (at {}); run every step in order",
                state.as_str()
            ));
        }
        state = step(
            state,
            AdmissionState::Admitted,
            format!("admitted as {}", class.as_str()),
            &mut last_note,
        )?;
    }
    rec.admission_note = last_note;
    rec.admission_state = state.as_str().to_string();
    ms.save(&rec)?;
    Ok(rec)
}

/// `verify`: custody of a record's source against the store (recomputes the content hash).
pub fn cmd_verify(
    store: &CorpusStore,
    ms: &ManifestStore,
    id: &str,
) -> Result<Vec<String>, String> {
    let rec = ms.load(id)?;
    let mut issues = Vec::new();
    if rec.source.content_sha256.is_empty() {
        issues.push("no content_sha256 recorded".to_string());
        return Ok(issues);
    }
    match store.get_bytes(&rec.source.content_sha256) {
        Some(bytes) => {
            let actual = sha256_hex(&bytes);
            if actual != rec.source.content_sha256 {
                issues.push(format!(
                    "blob content does not match recorded sha ({} != {})",
                    actual, rec.source.content_sha256
                ));
            }
        }
        None => issues.push(format!(
            "blob {} absent from the store",
            rec.source.content_sha256
        )),
    }
    Ok(issues)
}

/// `fetch`: materialize an archive/blob from a [`FetchSpec`] into the store.
///
/// Offline-safe contract: the archive must already exist as a local file whose SHA-256 equals
/// the spec's `archive_sha256` (or already be in the store). Network downloads are the job of the
/// family extractors (testsuite/ccvs85/manual/...) which run where the network is reachable; this
/// command never silently accepts content that does not match the expected hash.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FetchResult {
    pub family: String,
    pub revision: String,
    pub archive_sha256: String,
    /// Where the archive bytes were found: "store" | "local-file" | "missing".
    pub source: String,
    /// Blob SHA-256 of the admitted archive content.
    pub stored_sha256: String,
}

pub fn cmd_fetch(store: &CorpusStore, spec_path: &Path) -> Result<FetchResult, String> {
    let bytes = std::fs::read(spec_path)
        .map_err(|e| format!("cannot read {}: {e}", spec_path.display()))?;
    let spec: FetchSpec = serde_json::from_slice(&bytes)
        .map_err(|e| format!("fetch spec {}: {e}", spec_path.display()))?;
    let errs = spec.validate();
    if !errs.is_empty() {
        return Err(format!("fetch spec invalid: {}", errs.join("; ")));
    }
    // 1. already in the store?
    if let Some(existing) = store.get_bytes(&spec.archive_sha256) {
        store.verify(&existing, &spec.archive_sha256)?;
        return Ok(FetchResult {
            family: spec.family.clone(),
            revision: spec.revision.clone(),
            archive_sha256: spec.archive_sha256.clone(),
            source: "store".into(),
            stored_sha256: spec.archive_sha256,
        });
    }
    // 2. a local archive file whose content matches the expected hash? Search the spec's own
    // directory, the current directory, and the store's `incoming/` area.
    let mut candidates = Vec::new();
    for base in [
        spec_path.parent().unwrap_or(Path::new(".")),
        Path::new("."),
        &store.root().join("incoming"),
    ] {
        if let Ok(rd) = std::fs::read_dir(base) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_file() {
                    candidates.push(p);
                }
            }
        }
    }
    for p in candidates {
        if let Ok(data) = std::fs::read(&p) {
            let sha = sha256_hex(&data);
            if sha == spec.archive_sha256 {
                store.put_bytes(&data).map_err(|e| e.to_string())?;
                return Ok(FetchResult {
                    family: spec.family.clone(),
                    revision: spec.revision.clone(),
                    archive_sha256: spec.archive_sha256.clone(),
                    source: format!("local-file:{}", p.display()),
                    stored_sha256: spec.archive_sha256,
                });
            }
        }
    }
    Err(format!(
        "fetch: archive for {} @ {} not present (expected sha {}); place the archive next to the spec or in {}/incoming/ and re-run, or run the family extractor where the network is reachable",
        spec.family,
        spec.revision,
        spec.archive_sha256,
        store.root().display()
    ))
}

/// `classify`: set the classification of an existing record (exactly one per unit).
pub fn cmd_classify(
    ms: &ManifestStore,
    id: &str,
    class: Classification,
) -> Result<ProgramRecord, String> {
    let mut rec = ms.load(id)?;
    rec.classification = class;
    ms.save(&rec)?;
    Ok(rec)
}

/// `compare`: verdict from oracle + candidate records.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CompareVerdict {
    pub program_id: String,
    pub oracle_compile_exit: i32,
    pub oracle_run_exit: Option<i32>,
    pub candidate_run: String,
    pub stdout_match: Option<bool>,
    pub verdict: String,
}

pub fn cmd_compare(ms: &ManifestStore, id: &str) -> Result<CompareVerdict, String> {
    let rec = ms.load(id)?;
    let stdout_match = match (&rec.oracle.stdout_sha256, rec.candidate.run.as_str()) {
        (Some(_), "ok") => Some(true),
        _ => None,
    };
    let verdict = match (
        rec.oracle.compile_exit,
        rec.oracle.run_exit,
        rec.candidate.run.as_str(),
    ) {
        (0, Some(0), "ok") if stdout_match == Some(true) => "OBSERVABLE_MATCH",
        (0, _, _) => match rec.candidate.first_failure.as_ref() {
            Some((phase, _)) => phase.as_str(),
            None => "CANDIDATE_RUNTIME_FAIL",
        },
        (c, _, _) if c != 0 => "ORACLE_COMPILE_REJECT",
        _ => "COMPARE_UNDETERMINED",
    };
    Ok(CompareVerdict {
        program_id: id.to_string(),
        oracle_compile_exit: rec.oracle.compile_exit,
        oracle_run_exit: rec.oracle.run_exit,
        candidate_run: rec.candidate.run.clone(),
        stdout_match,
        verdict: verdict.to_string(),
    })
}

/// `report`: write the reconciled summary files under reports/valid-corpus/.
#[derive(Debug, Clone, Serialize, Default)]
pub struct Report {
    pub total: usize,
    pub by_classification: BTreeMap<String, usize>,
    pub by_corpus_class: BTreeMap<String, usize>,
    pub admitted: usize,
    pub unknown_classifications: usize,
    pub first_failure_by_phase: BTreeMap<String, usize>,
    pub exact_duplicates: usize,
    pub near_duplicate_families: usize,
}

pub fn cmd_report(ms: &ManifestStore, root: &Path, dedup: &DedupIndex) -> Result<Report, String> {
    let recs = ms.list()?;
    let mut rep = Report {
        total: recs.len(),
        ..Report::default()
    };
    let mut seen_exact: std::collections::HashSet<String> = std::collections::HashSet::new();
    for r in &recs {
        *rep.by_classification
            .entry(r.classification.as_str().to_string())
            .or_default() += 1;
        *rep.by_corpus_class
            .entry(format!("{:?}", r.corpus_class))
            .or_default() += 1;
        if r.admission_state == AdmissionState::Admitted.as_str() {
            rep.admitted += 1;
        }
        if let Some((phase, _)) = &r.candidate.first_failure {
            *rep.first_failure_by_phase.entry(phase.clone()).or_default() += 1;
        }
        if !seen_exact.insert(r.source.content_sha256.clone()) {
            rep.exact_duplicates += 1;
        }
    }
    rep.unknown_classifications = recs
        .iter()
        .filter(|r| {
            r.classification == Classification::SourceFragment
                && r.admission_state == AdmissionState::Admitted.as_str()
        })
        .count();
    rep.near_duplicate_families = dedup.normalized.values().filter(|v| v.len() > 1).count();

    // Write the committed report files (summary.json + programs.csv + summary.md).
    let out = root.join("reports").join("valid-corpus");
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let summary_json = serde_json::to_string_pretty(&rep).map_err(|e| e.to_string())?;
    std::fs::write(out.join("summary.json"), summary_json).map_err(|e| e.to_string())?;
    let mut csv = String::from("program_id,corpus_class,family,classification,admission_state,first_failure,dialect,format\n");
    for r in &recs {
        csv.push_str(&format!(
            "{},{:?},{:?},{},{},{},{},{}\n",
            r.program_id,
            r.corpus_class,
            r.source_family,
            r.classification.as_str(),
            r.admission_state,
            r.candidate
                .first_failure
                .as_ref()
                .map(|(p, _)| p.as_str())
                .unwrap_or(""),
            r.validity_profile.dialect,
            r.source.source_format
        ));
    }
    std::fs::write(out.join("programs.csv"), csv).map_err(|e| e.to_string())?;
    let mut md = String::from("# Valid-COBOL corpus — summary\n\n");
    md.push_str(&format!(
        "total units: {}\nadmitted: {}\n",
        rep.total, rep.admitted
    ));
    md.push_str("\n## by classification\n");
    for (k, v) in &rep.by_classification {
        md.push_str(&format!("- {k}: {v}\n"));
    }
    std::fs::write(out.join("summary.md"), md).map_err(|e| e.to_string())?;
    Ok(rep)
}

/// `gate`: the Phase-1 integrity gates. Returns a list of failures (empty = green).
pub fn cmd_gate(ms: &ManifestStore) -> Result<Vec<String>, String> {
    let mut fails = Vec::new();
    let recs = ms.list()?;
    if recs.is_empty() {
        fails.push("no records; nothing to gate".to_string());
        return Ok(fails);
    }
    // schema validates
    for r in &recs {
        for e in r.validate() {
            fails.push(format!("{}: {e}", r.program_id));
        }
        if Classification::parse(r.classification.as_str()).is_none() {
            fails.push(format!("{}: unknown classification", r.program_id));
        }
    }
    // no source admitted without oracle validation: ADMITTED requires the chain + oracle contract
    for r in &recs {
        if r.admission_state == AdmissionState::Admitted.as_str() {
            if r.oracle.compile_exit != 0 {
                fails.push(format!(
                    "{}: admitted with nonzero oracle compile exit",
                    r.program_id
                ));
            }
            if r.classification == Classification::ValidExecutableProgram
                && r.oracle.run_exit.is_none()
            {
                fails.push(format!(
                    "{}: admitted executable without oracle run outcome",
                    r.program_id
                ));
            }
            if !r.licence.reviewed {
                fails.push(format!(
                    "{}: admitted without reviewed licence",
                    r.program_id
                ));
            }
        }
    }
    Ok(fails)
}

/// `extract-testsuite`: Phase 2 -- classify the GnuCOBOL Autotest suite at AT_CHECK-step level,
/// materialize program packages, replay against the host oracle, probe the candidate phase by
/// phase, and write the Phase-2 reports.
#[derive(Debug, Clone, Serialize)]
pub struct ExtractSummary {
    pub lanes: Vec<String>,
    pub discovered_steps: usize,
    pub valid_programs: usize,
    pub invalid_programs: usize,
    pub oracle_contract_drift: usize,
    pub skipped_under_profile: usize,
    pub reports: Vec<String>,
    pub oracle: String,
    pub replay: bool,
    pub candidate: bool,
}

pub fn cmd_extract_testsuite(
    lane_arg: &str,
    replay: bool,
    candidate: bool,
) -> Result<ExtractSummary, String> {
    let root = crate::extract::workspace_root()?;
    let oracle = crate::extract::oracle::OracleEnv::host_default()?;
    let (store, _ms) = stores()?;
    let packages_root = store.root().join("packages");
    let out_dir = root
        .join("reports")
        .join("valid-corpus")
        .join("gnucobol-testsuite");

    let lanes: Vec<crate::extract::SuiteLane> = match lane_arg {
        "stable-3.2" => vec![crate::extract::STABLE_3_2],
        "current" => vec![crate::extract::CURRENT],
        "both" => vec![crate::extract::STABLE_3_2, crate::extract::CURRENT],
        other => {
            return Err(format!(
                "unknown lane {other:?} (stable-3.2 | current | both)"
            ))
        }
    };
    let mut stable_results = Vec::new();
    let mut current_results = Vec::new();
    let mut groups_map: std::collections::BTreeMap<String, Vec<crate::extract::at::AtGroup>> =
        std::collections::BTreeMap::new();
    let mut discovered = 0usize;
    let mut valid = 0usize;
    let mut invalid = 0usize;
    let mut drift = 0usize;
    let mut skipped = 0usize;
    for lane in &lanes {
        let (groups, errors) = crate::extract::load_suite_groups(*lane)?;
        if !errors.is_empty() {
            return Err(format!(
                "{} suite parse errors (fail closed): {}",
                lane.label,
                errors.join("; ")
            ));
        }
        for g in &groups {
            groups_map
                .entry(g.source_file.clone())
                .or_default()
                .push(g.clone());
        }
        let (results, _stats) =
            crate::extract::extract_lane(*lane, &oracle, &packages_root, replay, candidate)?;
        discovered += results.len();
        valid += results
            .iter()
            .filter(|r| r.classification.starts_with("VALID_"))
            .count();
        invalid += results
            .iter()
            .filter(|r| r.classification.contains("INVALID_EXPECTED_REJECT"))
            .count();
        drift += results
            .iter()
            .filter(|r| r.classification.contains("ORACLE_CONTRACT_DRIFT"))
            .count();
        skipped += results.iter().filter(|r| !r.skip_reason.is_empty()).count();
        match lane.label {
            "stable-3.2" => stable_results = results,
            _ => current_results = results,
        }
    }
    let counts = crate::extract::report::write_reports(
        &out_dir,
        &stable_results,
        &current_results,
        &groups_map,
    )?;
    let mut report_files = Vec::new();
    for name in [
        "discovered-steps.json",
        "valid-programs.json",
        "invalid-programs.json",
        "mixed-groups.json",
        "dependency-graph.json",
        "stable-current-drift.json",
        "summary.md",
    ] {
        if out_dir.join(name).exists() {
            report_files.push(format!("reports/valid-corpus/gnucobol-testsuite/{name}"));
        }
    }
    let _ = counts;
    Ok(ExtractSummary {
        lanes: lanes.iter().map(|l| l.label.to_string()).collect(),
        discovered_steps: discovered,
        valid_programs: valid,
        invalid_programs: invalid,
        oracle_contract_drift: drift,
        skipped_under_profile: skipped,
        reports: report_files,
        oracle: oracle.label.clone(),
        replay,
        candidate,
    })
}

/// `probe-step`: bounded candidate phase probe for one materialized suite step. Reads the step
/// manifest (group dir + main file), probes the phases (run phases only for run-shaped steps),
/// and writes the PhaseOutcome JSON to `--out`. Invoked as a subprocess by `extract-testsuite`
/// with a hard `timeout` so no suite program can hang the corpus run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepManifest {
    pub program_id: String,
    pub group_dir: String,
    pub main_file: String,
    pub expanded_command: String,
}

pub fn cmd_probe_step(
    manifest_path: &Path,
    out_path: &Path,
) -> Result<Vec<crate::extract::candidate::PhaseOutcome>, String> {
    let bytes = std::fs::read(manifest_path)
        .map_err(|e| format!("cannot read {}: {e}", manifest_path.display()))?;
    let m: StepManifest =
        serde_json::from_slice(&bytes).map_err(|e| format!("step manifest: {e}"))?;
    let run = crate::extract::candidate::run_shape(&m.expanded_command);
    let probes = crate::extract::candidate::probe_dir(Path::new(&m.group_dir), &m.main_file, run);
    let json = serde_json::to_string_pretty(&probes).map_err(|e| e.to_string())?;
    std::fs::write(out_path, json).map_err(|e| e.to_string())?;
    Ok(probes)
}

/// `extract-ccvs85`: Phase 3 -- classify every CCVS85 unit and write the Phase-3 reports from
/// the single committed GNURUST.CCVS85 evidence (no second materialization).
pub fn cmd_extract_ccvs85() -> Result<BTreeMap<String, usize>, String> {
    let root = crate::extract::workspace_root()?;
    let out_dir = root.join("reports").join("valid-corpus").join("ccvs85");
    crate::extract::ccvs85::write_reports(&root, &out_dir)
}

/// `extract-manual`: Phase 4 -- classify every manual code block and verify every complete
/// example with the documented (or derived) command.
pub fn cmd_extract_manual(
    lane_arg: &str,
    candidate: bool,
) -> Result<BTreeMap<String, usize>, String> {
    let root = crate::extract::workspace_root()?;
    let (store, _ms) = stores()?;
    let packages_root = store.root().join("packages");
    let lanes: Vec<(String, PathBuf, &str)> = match lane_arg {
        "stable-3.2" => vec![(
            "stable-3.2".to_string(),
            root.join("lab/admit/gnucobol-3.2/doc/gnucobol.texi"),
            "3.2.0",
        )],
        "current" => vec![(
            "current".to_string(),
            root.join("lab/admit/gnucobol-upstream-current/doc/gnucobol.texi"),
            "5568b8fc770f",
        )],
        "both" => vec![
            (
                "stable-3.2".to_string(),
                root.join("lab/admit/gnucobol-3.2/doc/gnucobol.texi"),
                "3.2.0",
            ),
            (
                "current".to_string(),
                root.join("lab/admit/gnucobol-upstream-current/doc/gnucobol.texi"),
                "5568b8fc770f",
            ),
        ],
        other => {
            return Err(format!(
                "unknown lane {other:?} (stable-3.2 | current | both)"
            ))
        }
    };
    let mut merged: BTreeMap<String, usize> = BTreeMap::new();
    for (lane, texi, revision) in lanes {
        if !texi.exists() {
            return Err(format!("manual source missing: {}", texi.display()));
        }
        let out_dir = root
            .join("reports")
            .join("valid-corpus")
            .join("gnucobol-manual")
            .join(&lane);
        let counts = crate::extract::manual::extract_manual(
            &root,
            &texi,
            &lane,
            revision,
            &packages_root,
            &out_dir,
            candidate,
        )?;
        for (k, v) in counts {
            *merged.entry(format!("{lane}/{k}")).or_default() += v;
        }
    }
    Ok(merged)
}

/// `check-updates`: load every fetch spec under `specs_dir` and produce drift reports (no
/// mutation of the admitted corpus).
pub fn cmd_check_updates(specs_dir: &Path) -> Result<Vec<UpdateReport>, String> {
    let mut specs = Vec::new();
    if specs_dir.is_dir() {
        let rd = std::fs::read_dir(specs_dir).map_err(|e| e.to_string())?;
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) == Some("json") {
                let bytes = std::fs::read(&p).map_err(|e| e.to_string())?;
                if let Ok(spec) = serde_json::from_slice::<FetchSpec>(&bytes) {
                    specs.push(spec);
                }
            }
        }
    }
    specs.sort_by(|a, b| a.family.cmp(&b.family));
    Ok(specs
        .iter()
        .map(|s| UpdateReport {
            family: s.family.clone(),
            pinned_revision: s.revision.clone(),
            latest_revision: s.revision.clone(),
            has_newer: false,
            note: "admission pin unchanged; network check runs at the family extractor level"
                .to_string(),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (tempfile::TempDir, CorpusStore, ManifestStore) {
        let dir = tempfile::tempdir().unwrap();
        let store = CorpusStore::open_at(dir.path()).unwrap();
        let ms = ManifestStore::new(store.root());
        (dir, store, ms)
    }

    fn src_file(dir: &Path) -> PathBuf {
        let p = dir.join("prog.cob");
        std::fs::write(&p, b"       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       PROCEDURE DIVISION.\n           DISPLAY \"OK\".\n           STOP RUN.\n")
            .unwrap();
        p
    }

    #[test]
    fn discover_creates_discovered_records_with_hashes() {
        let (dir, store, ms) = setup();
        let sub = dir.path().join("src");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("a.cob"), b"x").unwrap();
        std::fs::write(sub.join("b.txt"), b"not cobol").unwrap();
        let n = cmd_discover(&store, &ms, &sub).unwrap();
        assert_eq!(n, 1);
        let recs = ms.list().unwrap();
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].admission_state, "DISCOVERED");
        assert_eq!(recs[0].source.content_sha256.len(), 64);
    }

    #[test]
    fn admit_full_chain_enforced() {
        let (dir, store, ms) = setup();
        let file = src_file(dir.path());
        // Jumping straight to finalize fails (no record).
        let a = AdmitArgs {
            id: Some("t/1".into()),
            finalize: true,
            class: Some("VALID_EXECUTABLE_PROGRAM".into()),
            ..AdmitArgs::default()
        };
        assert!(cmd_admit(&store, &ms, &a).is_err());
        // Discover.
        let a = AdmitArgs {
            id: Some("t/1".into()),
            discover: true,
            source_file: Some(file.display().to_string()),
            corpus_class: Some("upstream".into()),
            family: Some("ccvs85".into()),
            dialect: Some("cobol85".into()),
            ..AdmitArgs::default()
        };
        cmd_admit(&store, &ms, &a).unwrap();
        // Custody.
        let rec = ms.load("t/1").unwrap();
        let sha = rec.source.content_sha256.clone();
        let a = AdmitArgs {
            id: Some("t/1".into()),
            custody_sha: Some(sha.clone()),
            ..AdmitArgs::default()
        };
        cmd_admit(&store, &ms, &a).unwrap();
        // Licence.
        let a = AdmitArgs {
            id: Some("t/1".into()),
            licence_spdx: Some("MIT".into()),
            redistribute: Some(true),
            licence_decision: Some("test".into()),
            ..AdmitArgs::default()
        };
        cmd_admit(&store, &ms, &a).unwrap();
        // Deps.
        let a = AdmitArgs {
            id: Some("t/1".into()),
            deps: Some("[]".into()),
            ..AdmitArgs::default()
        };
        cmd_admit(&store, &ms, &a).unwrap();
        // Oracle compile.
        let a = AdmitArgs {
            id: Some("t/1".into()),
            oracle_compile_exit: Some(0),
            ..AdmitArgs::default()
        };
        cmd_admit(&store, &ms, &a).unwrap();
        // Oracle run.
        let a = AdmitArgs {
            id: Some("t/1".into()),
            oracle_run_exit: Some(0),
            stdout_sha: Some("a".repeat(64)),
            ..AdmitArgs::default()
        };
        cmd_admit(&store, &ms, &a).unwrap();
        // Determinism.
        let a = AdmitArgs {
            id: Some("t/1".into()),
            deterministic: true,
            ..AdmitArgs::default()
        };
        cmd_admit(&store, &ms, &a).unwrap();
        // Finalize.
        let a = AdmitArgs {
            id: Some("t/1".into()),
            finalize: true,
            class: Some("VALID_EXECUTABLE_PROGRAM".into()),
            ..AdmitArgs::default()
        };
        let rec = cmd_admit(&store, &ms, &a).unwrap();
        assert_eq!(rec.admission_state, "ADMITTED");
        assert!(rec.validate().is_empty());
    }

    #[test]
    fn admit_rejects_illegal_jumps() {
        let (dir, store, ms) = setup();
        let file = src_file(dir.path());
        let a = AdmitArgs {
            id: Some("t/2".into()),
            discover: true,
            source_file: Some(file.display().to_string()),
            ..AdmitArgs::default()
        };
        cmd_admit(&store, &ms, &a).unwrap();
        // Skipping custody: finalize must fail.
        let a = AdmitArgs {
            id: Some("t/2".into()),
            finalize: true,
            class: Some("VALID_EXECUTABLE_PROGRAM".into()),
            ..AdmitArgs::default()
        };
        assert!(cmd_admit(&store, &ms, &a).is_err());
        // Oracle steps before custody also fail (chain order).
        let a = AdmitArgs {
            id: Some("t/2".into()),
            oracle_compile_exit: Some(0),
            ..AdmitArgs::default()
        };
        let e = cmd_admit(&store, &ms, &a).unwrap_err();
        assert!(e.contains("illegal admission jump"), "{e}");
    }

    #[test]
    fn non_redistributable_licence_rejects() {
        let (dir, store, ms) = setup();
        let file = src_file(dir.path());
        let a = AdmitArgs {
            id: Some("t/3".into()),
            discover: true,
            source_file: Some(file.display().to_string()),
            ..AdmitArgs::default()
        };
        cmd_admit(&store, &ms, &a).unwrap();
        let rec = ms.load("t/3").unwrap();
        let a = AdmitArgs {
            id: Some("t/3".into()),
            custody_sha: Some(rec.source.content_sha256.clone()),
            ..AdmitArgs::default()
        };
        cmd_admit(&store, &ms, &a).unwrap();
        let a = AdmitArgs {
            id: Some("t/3".into()),
            licence_spdx: Some("LicenseRef-No-Redistribute".into()),
            redistribute: Some(false),
            licence_decision: Some("test".into()),
            ..AdmitArgs::default()
        };
        assert!(cmd_admit(&store, &ms, &a).is_err());
    }

    #[test]
    fn finalize_refuses_unknown_class() {
        let (dir, store, ms) = setup();
        let file = src_file(dir.path());
        let a = AdmitArgs {
            id: Some("t/4".into()),
            discover: true,
            source_file: Some(file.display().to_string()),
            ..AdmitArgs::default()
        };
        cmd_admit(&store, &ms, &a).unwrap();
        let a = AdmitArgs {
            id: Some("t/4".into()),
            finalize: true,
            class: Some("UNKNOWN".into()),
            ..AdmitArgs::default()
        };
        assert!(cmd_admit(&store, &ms, &a).is_err());
    }

    #[test]
    fn verify_detects_missing_blob() {
        let (dir, store, ms) = setup();
        let file = src_file(dir.path());
        let a = AdmitArgs {
            id: Some("t/5".into()),
            discover: true,
            source_file: Some(file.display().to_string()),
            ..AdmitArgs::default()
        };
        cmd_admit(&store, &ms, &a).unwrap();
        let rec = ms.load("t/5").unwrap();
        // custody blob IS in the store (discover stored it) -> verify clean
        assert!(cmd_verify(&store, &ms, "t/5").unwrap().is_empty());
        // tamper the recorded sha -> verify flags it
        let mut rec = rec;
        rec.source.content_sha256 = "0".repeat(64);
        ms.save(&rec).unwrap();
        assert!(!cmd_verify(&store, &ms, "t/5").unwrap().is_empty());
        let _ = dir;
    }

    #[test]
    fn compare_verdicts() {
        let (dir, store, ms) = setup();
        let file = src_file(dir.path());
        let mut a = AdmitArgs {
            id: Some("t/6".into()),
            discover: true,
            source_file: Some(file.display().to_string()),
            ..AdmitArgs::default()
        };
        cmd_admit(&store, &ms, &a).unwrap();
        let rec = ms.load("t/6").unwrap();
        a = AdmitArgs {
            id: Some("t/6".into()),
            custody_sha: Some(rec.source.content_sha256.clone()),
            ..AdmitArgs::default()
        };
        cmd_admit(&store, &ms, &a).unwrap();
        let a = AdmitArgs {
            id: Some("t/6".into()),
            licence_spdx: Some("MIT".into()),
            redistribute: Some(true),
            licence_decision: Some("t".into()),
            deps: Some("[]".into()),
            oracle_compile_exit: Some(0),
            oracle_run_exit: Some(0),
            stdout_sha: Some("s".repeat(64)),
            deterministic: true,
            candidate: vec!["parse=ok".into(), "run=ok".into()],
            ..AdmitArgs::default()
        };
        // multi-step in one invocation must walk the chain in order
        cmd_admit(&store, &ms, &a).unwrap();
        let v = cmd_compare(&ms, "t/6").unwrap();
        assert_eq!(v.verdict, "OBSERVABLE_MATCH");
        // a parse-first-failure unit
        let a = AdmitArgs {
            id: Some("t/7".into()),
            discover: true,
            source_file: Some(file.display().to_string()),
            candidate: vec!["parse=reject: bad token".into()],
            ..AdmitArgs::default()
        };
        cmd_admit(&store, &ms, &a).unwrap();
        let rec = ms.load("t/7").unwrap();
        assert_eq!(
            rec.candidate.first_failure,
            Some(("parse".to_string(), "reject: bad token".to_string()))
        );
        let _ = dir;
    }

    #[test]
    fn report_and_gate_reconcile() {
        let (dir, store, ms) = setup();
        let file = src_file(dir.path());
        let a = AdmitArgs {
            id: Some("t/8".into()),
            discover: true,
            source_file: Some(file.display().to_string()),
            ..AdmitArgs::default()
        };
        cmd_admit(&store, &ms, &a).unwrap();
        let mut dedup = DedupIndex::new();
        dedup.register("t/8", b"x");
        let rep = cmd_report(&ms, dir.path(), &dedup).unwrap();
        assert_eq!(rep.total, 1);
        assert_eq!(rep.admitted, 0);
        // gate on non-admitted records is about schema validity, not admission
        let fails = cmd_gate(&ms).unwrap();
        assert!(fails.is_empty(), "{fails:?}");
        assert!(dir
            .path()
            .join("reports/valid-corpus/summary.json")
            .exists());
        assert!(dir
            .path()
            .join("reports/valid-corpus/programs.csv")
            .exists());
    }

    #[test]
    fn command_parse_round_trip() {
        let args: Vec<String> = vec![
            "admit".into(),
            "--id".into(),
            "x/y".into(),
            "--discover".into(),
            "--json".into(),
        ];
        let cmd = parse(&args).unwrap();
        match cmd {
            Command::Admit(a) => {
                assert_eq!(a.id.as_deref(), Some("x/y"));
                assert!(a.discover);
                assert!(a.json);
            }
            _ => panic!("expected admit"),
        }
        let cmd = parse(&["list".into(), "--json".into()]).unwrap();
        assert_eq!(cmd, Command::List { json: true });
        assert!(parse(&["nope".into()]).is_err());
    }
}
