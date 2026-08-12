//! Phase 9 measurement views (spec 9.5) + benchmark controls (spec 9.7).
//!
//! Five views, kept strictly separate in the output (never merged):
//! - View A — end-to-end one-shot: oracle compile+run vs candidate prepare+run. Labelled
//!   "unlike workflows" (native compiled binary vs interpreted), never equivalent work.
//! - View B — front-end only: oracle compile command (same as `validate`) vs the candidate's
//!   per-phase timings (spec 9.1/9.2: preprocess/lex/parse/resolution/layout/check/prepare),
//!   plus bytes/sec and lines/sec from the source size.
//! - View C — repeated execution: compiled binary run `iters` times vs already-prepared program
//!   run `iters` times (no reparsing).
//! - View D — runtime-operation microbenchmarks (correctness-gated, see [`crate::micro`]).
//! - View E — corpus throughput: full corpus (10 workloads x 4 scales) one pass each lane.

use crate::candidate_prepared;
use crate::gen;
use crate::micro;
use crate::native_runs;
use crate::validate;
use crate::validate_all;
use crate::Oracle;
use crate::SampleSet;
use crate::Workload;
use crate::WORKLOADS;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// The exact compiler flags recorded in every report (same command `validate` uses).
pub const COMPILER_FLAGS: &str = "cobc -x -O2 -o <artifact> <sources...>";
/// One warmup iteration precedes the measured samples (already the pattern in
/// `candidate_prepared`/`native_runs`; kept for every lane of every view).
pub const WARMUP_RUNS: usize = 1;
/// Documented outlier policy: no samples are discarded -- min = best case, median = typical.
pub const OUTLIER_POLICY: &str = "min = best-case, median = typical, no samples discarded";
/// View A's mandatory label: the two lanes are NOT equivalent runtime work.
pub const VIEW_A_LABEL: &str = "unlike workflows: native lane is a compiled binary (compile+run); \
     candidate lane is interpreted (parse/check/prepare+run) -- NOT equivalent runtime work";

// ---------------------------------------------------------------------------------------------
// benchmark controls (spec 9.7)
// ---------------------------------------------------------------------------------------------

/// Host CPU identity: the first `model name` line of `/proc/cpuinfo` (Linux); `None` elsewhere.
pub fn host_cpu() -> Option<String> {
    let text = std::fs::read_to_string("/proc/cpuinfo").ok()?;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("model name") {
            let value = rest.split(':').nth(1).unwrap_or("").trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

/// First line of `cobc --version` (spec 9.7 compiler identity).
pub fn cobc_version(oracle: &Oracle) -> Option<String> {
    let cobc = oracle.cobc.to_string_lossy().into_owned();
    let (code, out, _err) = crate::run_cmd(oracle, &oracle.prefix, &[cobc.as_str(), "--version"]);
    if code != Some(0) {
        return None;
    }
    String::from_utf8(out)
        .ok()?
        .lines()
        .next()
        .map(str::trim)
        .map(str::to_string)
}

/// Peak resident set (VmHWM) of THIS process from `/proc/self/status` (Linux only). The
/// candidate runs in-process, so this captures its high-water mark; the oracle's cobc/run
/// children are separate processes and are not included.
pub fn peak_memory_kb() -> Option<u64> {
    let text = std::fs::read_to_string("/proc/self/status").ok()?;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("VmHWM:") {
            return rest
                .trim()
                .strip_suffix(" kB")
                .and_then(|v| v.trim().parse().ok());
        }
    }
    None
}

/// UTC timestamp in ISO 8601 (no external time crate; Howard Hinnant's civil-from-days).
pub fn generated_at_utc() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    let (h, mi, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    format!("{y:04}-{m:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if month <= 2 { y + 1 } else { y }, month, day)
}

/// The control block recorded in every report (spec 9.7).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ControlMeta {
    pub host_cpu: Option<String>,
    pub cobc_version: Option<String>,
    pub compiler_flags: &'static str,
    pub iters: usize,
    pub warmup: usize,
    pub outlier_policy: &'static str,
    pub generated_at_utc: String,
    pub candidate_compat: String,
    pub peak_memory_kb: Option<u64>,
}

// ---------------------------------------------------------------------------------------------
// view selection + CLI argument parsing
// ---------------------------------------------------------------------------------------------

/// The five measurement views.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    A,
    B,
    C,
    D,
    E,
}

impl View {
    /// Stable JSON key (views stay separate in the output).
    pub fn key(self) -> &'static str {
        match self {
            View::A => "view_a",
            View::B => "view_b",
            View::C => "view_c",
            View::D => "view_d",
            View::E => "view_e",
        }
    }
}

/// Parse a view name; rejects unknown names with a clear message.
pub fn parse_view(s: &str) -> Result<View, String> {
    match s {
        "view-a" | "view_a" | "a" => Ok(View::A),
        "view-b" | "view_b" | "b" => Ok(View::B),
        "view-c" | "view_c" | "c" => Ok(View::C),
        "view-d" | "view_d" | "d" => Ok(View::D),
        "view-e" | "view_e" | "e" => Ok(View::E),
        other => Err(format!(
            "unknown view {other:?} (expected view-a | view-b | view-c | view-d | view-e | all)"
        )),
    }
}

/// Parsed `measure` arguments: `[view] [workload] [scale] [--iters N]`.
#[derive(Debug, Clone)]
pub struct MeasureArgs {
    /// `None` = all five views.
    pub view: Option<View>,
    /// `None`/`"all"` = all workloads.
    pub workload: Option<String>,
    /// `None` = the per-view default scale (small for A/B/C; all for E).
    pub scale: Option<String>,
    pub iters: usize,
}

/// Parse the positional arguments after `measure`. Unknown view names are rejected.
pub fn parse_measure_args(args: &[String]) -> Result<MeasureArgs, String> {
    let mut iters: usize = 7;
    let mut positionals: Vec<&str> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--iters" => {
                i += 1;
                let value = args.get(i).ok_or("--iters requires a value")?;
                iters = value
                    .parse()
                    .map_err(|_| format!("invalid --iters value {value:?}"))?;
                if iters == 0 {
                    return Err("--iters must be >= 1".to_string());
                }
            }
            flag if flag.starts_with("--") => return Err(format!("unknown option {flag:?}")),
            positional => positionals.push(positional),
        }
        i += 1;
    }
    if positionals.len() > 3 {
        return Err(format!(
            "too many positional arguments: {positionals:?} (expected [view] [workload] [scale])"
        ));
    }
    let mut view = None;
    if let Some(name) = positionals.first() {
        view = if *name == "all" {
            None
        } else {
            Some(parse_view(name)?)
        };
    }
    let workload = positionals.get(1).map(|s| s.to_string());
    if let Some(w) = &workload {
        if w != "all" && crate::workload(w).is_none() {
            return Err(format!("unknown workload {w:?}"));
        }
    }
    let scale = positionals.get(2).map(|s| s.to_string());
    if let Some(s) = &scale {
        if !matches!(s.as_str(), "small" | "medium" | "large" | "stress" | "all") {
            return Err(format!(
                "unknown scale {s:?} (small | medium | large | stress | all)"
            ));
        }
    }
    Ok(MeasureArgs {
        view,
        workload,
        scale,
        iters,
    })
}

fn select_workloads(w: Option<&str>) -> Result<Vec<&'static Workload>, String> {
    match w {
        None | Some("all") => Ok(WORKLOADS.iter().collect()),
        Some(name) => crate::workload(name)
            .map(|w| vec![w])
            .ok_or_else(|| format!("unknown workload {name:?}")),
    }
}

/// Default scale per view: small for A/B/C; View E always runs all scales.
fn scale_for(view: View, scale: Option<&str>) -> String {
    match scale {
        Some(s) if s != "all" => s.to_string(),
        _ => match view {
            View::A | View::B | View::C => "small".to_string(),
            View::D | View::E => "all".to_string(),
        },
    }
}

fn first_lines(bytes: &[u8], n: usize) -> String {
    String::from_utf8_lossy(bytes)
        .lines()
        .take(n)
        .collect::<Vec<_>>()
        .join(" | ")
}

// ---------------------------------------------------------------------------------------------
// shared lane helpers
// ---------------------------------------------------------------------------------------------

/// Compile a workload with the host oracle using the same command as `validate` (spec: the
/// compile phase is timed with the exact command used in validation).
fn oracle_compile(oracle: &Oracle, w: &Workload, dir: &Path) -> Result<(), String> {
    let artifact = w.run_artifact.trim_start_matches("./");
    let cobc = oracle.cobc.to_string_lossy().into_owned();
    let mut argv: Vec<String> = vec![
        cobc,
        "-x".to_string(),
        "-O2".to_string(),
        "-o".to_string(),
        artifact.to_string(),
    ];
    for s in w.sources {
        argv.push(s.to_string());
    }
    let refs: Vec<&str> = argv.iter().map(|s| s.as_str()).collect();
    let (code, _out, err) = crate::run_cmd(oracle, dir, &refs);
    if code != Some(0) {
        return Err(format!(
            "oracle compile failed (exit {code:?}): {}",
            first_lines(&err, 3)
        ));
    }
    Ok(())
}

/// Candidate-lane byte-exactness gate: the candidate's output file must equal the independent
/// expectation (same comparison as `validate`). Never time a lane with wrong output.
fn candidate_output_matches(
    w: &Workload,
    dir: &Path,
    records: usize,
    scale: &str,
) -> Result<bool, String> {
    let actual = std::fs::read(dir.join(w.output_file)).map_err(|e| e.to_string())?;
    let expected = (w.expected)(records, scale).1;
    let actual = String::from_utf8_lossy(&actual);
    Ok(actual.trim_end_matches('\n') == expected.trim_end_matches('\n'))
}

/// The candidate-lane adaptation oracle proof (spec 5.3 "oracle result after adaptation"):
/// write the TRANSFORMED source into a scratch dir, compile it with the real `cobc` (same flags
/// as `validate`), run it against the generated input, and compare the output with the original
/// workload's output. Returns `(proved, compile_exit)` -- `proved` is true only when the
/// transformed source compiles AND its run output equals the original's. A workload whose
/// adaptation is NOT oracle-proved still records `false` honestly (never fabricated).
fn oracle_prove_transformed(
    oracle: &Oracle,
    work_root: &Path,
    w: &Workload,
    ad: &crate::CandidateAdaptation,
) -> Result<(bool, Option<i32>), String> {
    let n = gen::record_count("small");
    let dir = work_root.join(format!("adapt-prove-{}", w.name));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    // the original input (same generated input the oracle lane runs on)
    let (input, _expected, _note) = (w.expected)(n, "small");
    std::fs::write(dir.join(w.input_file), &input).map_err(|e| e.to_string())?;
    // write the transformed copy as the single source the oracle compiles
    let transformed_file = format!("transformed.cob");
    std::fs::write(dir.join(&transformed_file), &ad.source).map_err(|e| e.to_string())?;
    let artifact = "transformed";
    let cobc = oracle.cobc.to_string_lossy().into_owned();
    let argv: Vec<&str> = vec![
        cobc.as_str(),
        "-x",
        "-O2",
        "-o",
        artifact,
        &transformed_file,
    ];
    let (code, _out, _err) = crate::run_cmd(oracle, &dir, &argv);
    let compile_exit = code;
    if code != Some(0) {
        // the transformed copy is not oracle-valid: record that honestly (never a fabricated
        // proof); the candidate lane's own byte-exact gate still protects the timings.
        return Ok((false, compile_exit));
    }
    let (run_code, _out, _err) = crate::run_cmd(oracle, &dir, &["./transformed"]);
    if run_code != Some(0) {
        return Ok((false, compile_exit));
    }
    let actual = std::fs::read(dir.join(w.output_file)).map_err(|e| e.to_string())?;
    let expected_str = _expected.trim_end_matches('\n');
    let actual_str = String::from_utf8_lossy(&actual);
    Ok((
        actual_str.trim_end_matches('\n') == expected_str,
        compile_exit,
    ))
}

/// One cold candidate shot: prepare (full front-end) + run once, in `dir`.
fn candidate_one_shot(w: &Workload, dir: &Path) -> Result<f64, String> {
    let t = Instant::now();
    let prepared = crate::prepare_candidate(w)?;
    let _ = crate::run_prepared_in_dir(&prepared, dir)?;
    Ok(t.elapsed().as_secs_f64() * 1000.0)
}

fn ratio(a: f64, b: f64) -> f64 {
    if b > 0.0 {
        a / b
    } else {
        0.0
    }
}

// ---------------------------------------------------------------------------------------------
// View A — end-to-end one-shot
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize)]
pub struct ViewAEntry {
    pub workload: String,
    pub scale: String,
    pub records: usize,
    /// One cold `compile + run` shot per sample (total wall time).
    pub oracle_total_ms: SampleSet,
    /// One cold `parse/check/prepare + interpret` shot per sample (total wall time).
    pub candidate_total_ms: SampleSet,
    /// Mandatory label: the lanes are unlike workflows.
    pub label: String,
    pub note: String,
}

pub fn run_view_a(
    oracle: &Oracle,
    work_root: &Path,
    workloads: &[&'static Workload],
    scale: &str,
    iters: usize,
) -> Result<Vec<ViewAEntry>, String> {
    let mut out = Vec::new();
    for w in workloads {
        let dir = work_root.join(format!("{}-{}", w.name, scale));
        let r = validate(w, scale, work_root)?;
        if !r.byte_exact {
            return Err(format!(
                "{} @ {}: correctness gate failed before View A: {}",
                w.name, scale, r.note
            ));
        }
        let records = gen::record_count(scale);
        let mut oracle_samples = Vec::with_capacity(iters);
        let mut candidate_samples = Vec::with_capacity(iters);
        for _ in 0..iters {
            let t0 = Instant::now();
            oracle_compile(oracle, w, &dir)?;
            let (code, _out, err) = crate::run_cmd(oracle, &dir, &[w.run_artifact]);
            if code != Some(0) {
                return Err(format!(
                    "{} @ {}: oracle run failed (exit {code:?}): {}",
                    w.name,
                    scale,
                    first_lines(&err, 2)
                ));
            }
            oracle_samples.push(t0.elapsed().as_secs_f64() * 1000.0);
            candidate_samples.push(candidate_one_shot(w, &dir)?);
        }
        if !candidate_output_matches(w, &dir, records, scale)? {
            return Err(format!(
                "{} @ {}: candidate output mismatch (View A never times a wrong lane)",
                w.name, scale
            ));
        }
        out.push(ViewAEntry {
            workload: w.name.to_string(),
            scale: scale.to_string(),
            records,
            oracle_total_ms: crate::stats(&oracle_samples),
            candidate_total_ms: crate::stats(&candidate_samples),
            label: VIEW_A_LABEL.to_string(),
            note: r.note.clone(),
        });
    }
    Ok(out)
}

// ---------------------------------------------------------------------------------------------
// View B — front-end only (per-phase, spec 9.1/9.2)
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize)]
pub struct PhaseReport {
    pub preprocess_ms: f64,
    pub lex_ms: f64,
    pub parse_ms: f64,
    pub resolution_ms: f64,
    pub layout_ms: f64,
    pub check_ms: f64,
    pub prepare_ms: f64,
    pub source_bytes: usize,
    pub source_lines: usize,
    pub bytes_per_sec: f64,
    pub lines_per_sec: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ViewBEntry {
    pub workload: String,
    pub scale: String,
    pub records: usize,
    /// Oracle compile phase only (the same `cobc -x -O2` command `validate` uses), `iters`
    /// cold shots after one warmup.
    pub oracle_compile_ms: SampleSet,
    /// Candidate per-phase timings over the prepared source.
    pub candidate: PhaseReport,
    pub note: String,
}

pub fn run_view_b(
    oracle: &Oracle,
    work_root: &Path,
    workloads: &[&'static Workload],
    scale: &str,
    iters: usize,
) -> Result<Vec<ViewBEntry>, String> {
    let mut out = Vec::new();
    for w in workloads {
        let dir = work_root.join(format!("{}-{}", w.name, scale));
        let r = validate(w, scale, work_root)?;
        if !r.byte_exact {
            return Err(format!(
                "{} @ {}: correctness gate failed before View B: {}",
                w.name, scale, r.note
            ));
        }
        let records = gen::record_count(scale);
        let mut comp = Vec::with_capacity(iters);
        for i in 0..iters + 1 {
            let t = Instant::now();
            oracle_compile(oracle, w, &dir)?;
            let ms = t.elapsed().as_secs_f64() * 1000.0;
            if i > 0 {
                comp.push(ms);
            }
        }
        let adaptation = crate::candidate_source(w)?;
        let source = &adaptation.source;
        let (_prepared, timings) = gnucobol_rs::frontend::prepare_program_timed(
            source,
            gnucobol_rs::dialect::Dialect::DEFAULT,
        )
        .map_err(|e| format!("{} @ {}: candidate prepare failed: {e}", w.name, scale))?;
        let bytes = source.len();
        let lines = source.lines().count();
        let secs = (timings.prepare_ms / 1000.0).max(1e-9);
        out.push(ViewBEntry {
            workload: w.name.to_string(),
            scale: scale.to_string(),
            records,
            oracle_compile_ms: crate::stats(&comp),
            candidate: PhaseReport {
                preprocess_ms: timings.preprocess_ms,
                lex_ms: timings.lex_ms,
                parse_ms: timings.parse_ms,
                resolution_ms: timings.resolution_ms,
                layout_ms: timings.layout_ms,
                check_ms: timings.check_ms,
                prepare_ms: timings.prepare_ms,
                source_bytes: bytes,
                source_lines: lines,
                bytes_per_sec: bytes as f64 / secs,
                lines_per_sec: lines as f64 / secs,
            },
            note: r.note.clone(),
        });
    }
    Ok(out)
}

// ---------------------------------------------------------------------------------------------
// View C — repeated execution (no candidate reparsing)
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize)]
pub struct ViewCEntry {
    pub workload: String,
    pub scale: String,
    pub records: usize,
    /// Already-compiled native binary run `iters` times.
    pub native: SampleSet,
    /// Already-prepared program run `iters` times without reparsing.
    pub candidate: SampleSet,
    pub candidate_prepare_ms: u64,
    pub oracle_output_sha256: String,
    pub candidate_output_sha256: String,
    pub outputs_agree: bool,
    pub note: String,
}

pub fn run_view_c(
    oracle: &Oracle,
    work_root: &Path,
    workloads: &[&'static Workload],
    scale: &str,
    iters: usize,
) -> Result<Vec<ViewCEntry>, String> {
    let mut out = Vec::new();
    for w in workloads {
        let dir = work_root.join(format!("{}-{}", w.name, scale));
        let r = validate(w, scale, work_root)?;
        if !r.byte_exact {
            return Err(format!(
                "{} @ {}: correctness gate failed before View C: {}",
                w.name, scale, r.note
            ));
        }
        let records = gen::record_count(scale);
        let (native, _native_out) = native_runs(oracle, w, &dir, iters)?;
        let native_file = std::fs::read(dir.join(w.output_file)).map_err(|e| e.to_string())?;
        let (candidate, _candidate_out, prepare_ms, _compat) =
            candidate_prepared(w, scale, &dir, iters)?;
        let candidate_file = std::fs::read(dir.join(w.output_file)).map_err(|e| e.to_string())?;
        let oracle_sha = crate::sha256_hex(&native_file);
        let candidate_sha = crate::sha256_hex(&candidate_file);
        out.push(ViewCEntry {
            workload: w.name.to_string(),
            scale: scale.to_string(),
            records,
            native,
            candidate,
            candidate_prepare_ms: prepare_ms,
            oracle_output_sha256: oracle_sha.clone(),
            candidate_output_sha256: candidate_sha.clone(),
            outputs_agree: oracle_sha == candidate_sha,
            note: r.note.clone(),
        });
    }
    Ok(out)
}

// ---------------------------------------------------------------------------------------------
// View D — runtime-operation microbenchmarks
// ---------------------------------------------------------------------------------------------

pub fn run_view_d(
    oracle: &Oracle,
    work_root: &Path,
    iters: usize,
) -> Result<Vec<micro::MicroEntry>, String> {
    let mut out = Vec::new();
    for m in micro::MICRO_WORKLOADS {
        out.push(micro::measure_micro(oracle, work_root, m, iters)?);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------------------------
// View E — corpus throughput
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize)]
pub struct ViewEOne {
    pub workload: String,
    pub scale: String,
    pub records: usize,
    pub oracle_ms: f64,
    pub candidate_ms: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ViewEResult {
    pub passes: usize,
    pub oracle_total_ms: f64,
    pub candidate_total_ms: f64,
    pub oracle_per_workload_ms: BTreeMap<String, f64>,
    pub candidate_per_workload_ms: BTreeMap<String, f64>,
    pub entries: Vec<ViewEOne>,
    pub peak_memory_kb: Option<u64>,
    pub memory_note: String,
    pub note: String,
}

pub fn run_view_e(oracle: &Oracle, work_root: &Path) -> Result<ViewEResult, String> {
    let _all = validate_all(work_root)?;
    let mut entries = Vec::new();
    let mut oracle_total = 0.0;
    let mut candidate_total = 0.0;
    let mut oracle_per_w: BTreeMap<String, f64> = BTreeMap::new();
    let mut candidate_per_w: BTreeMap<String, f64> = BTreeMap::new();
    for w in WORKLOADS {
        for scale in ["small", "medium", "large", "stress"] {
            let dir = work_root.join(format!("{}-{}", w.name, scale));
            let records = gen::record_count(scale);
            let t0 = Instant::now();
            oracle_compile(oracle, w, &dir)?;
            let (code, _out, err) = crate::run_cmd(oracle, &dir, &[w.run_artifact]);
            if code != Some(0) {
                return Err(format!(
                    "{} @ {}: oracle run failed (exit {code:?}): {}",
                    w.name,
                    scale,
                    first_lines(&err, 2)
                ));
            }
            let oracle_ms = t0.elapsed().as_secs_f64() * 1000.0;
            let t1 = Instant::now();
            let prepared = crate::prepare_candidate(w)?;
            let _ = crate::run_prepared_in_dir(&prepared, &dir)?;
            let candidate_ms = t1.elapsed().as_secs_f64() * 1000.0;
            if !candidate_output_matches(w, &dir, records, scale)? {
                return Err(format!(
                    "{} @ {}: candidate output mismatch (View E never reports a wrong lane)",
                    w.name, scale
                ));
            }
            oracle_total += oracle_ms;
            candidate_total += candidate_ms;
            *oracle_per_w.entry(w.name.to_string()).or_insert(0.0) += oracle_ms;
            *candidate_per_w.entry(w.name.to_string()).or_insert(0.0) += candidate_ms;
            entries.push(ViewEOne {
                workload: w.name.to_string(),
                scale: scale.to_string(),
                records,
                oracle_ms,
                candidate_ms,
            });
        }
    }
    let peak = peak_memory_kb();
    let memory_note = match peak {
        Some(_) => "VmHWM of the bench process (candidate runs in-process; oracle cobc/run are \
                    child processes, not included)"
            .to_string(),
        None => "peak memory omitted: /proc/self/status not readable (non-Linux)".to_string(),
    };
    Ok(ViewEResult {
        passes: 1,
        oracle_total_ms: oracle_total,
        candidate_total_ms: candidate_total,
        oracle_per_workload_ms: oracle_per_w,
        candidate_per_workload_ms: candidate_per_w,
        entries,
        peak_memory_kb: peak,
        memory_note,
        note: "one full pass over the corpus: 10 workloads x 4 scales, compile+run (oracle) vs \
               prepare+run (candidate), correctness-gated"
            .to_string(),
    })
}

// ---------------------------------------------------------------------------------------------
// measure orchestrator
// ---------------------------------------------------------------------------------------------

/// The outcome of one `measure` run: the control block plus one optional section per view
/// (views stay separate -- never merged).
pub struct MeasureOutcome {
    pub control: ControlMeta,
    pub view_a: Option<Vec<ViewAEntry>>,
    pub view_b: Option<Vec<ViewBEntry>>,
    pub view_c: Option<Vec<ViewCEntry>>,
    pub view_d: Option<Vec<micro::MicroEntry>>,
    pub view_e: Option<ViewEResult>,
    /// The candidate-lane adaptation ledger (spec 5.3): for every workload the candidate lane
    /// measured, the original/transformed source hashes + the rewrites applied. The oracle
    /// always compiles the original sources; the candidate lane is the only adapted copy.
    pub adaptations: Vec<AdaptationLedgerEntry>,
}

/// One spec-5.3 adaptation-ledger entry: binds the candidate lane's transformed copy to the
/// original source, names the applied rewrites, AND proves with the real oracle that the
/// transformed source is itself a valid program producing byte-identical output (the
/// adaptation is semantics-preserving under the oracle, not merely accepted by the candidate).
/// Never a pristine-parity claim.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AdaptationLedgerEntry {
    pub workload: String,
    pub source_files: Vec<String>,
    pub original_sha256: String,
    pub transformed_sha256: String,
    pub rewrites: Vec<String>,
    /// Oracle acceptance + byte-exactness of the TRANSFORMED source (spec 5.3 "oracle result
    /// after adaptation"): `true` when cobc compiles the transformed copy and its run output
    /// equals the original's output.
    pub oracle_proved_preserving: bool,
    pub oracle_compile_exit: Option<i32>,
    pub note: String,
}

/// Run the requested views. Every view gates correctness first (validate/validate_all or the
/// micro byte-exact gate); if the host oracle is unavailable this fails with a clear message --
/// numbers are never fabricated.
pub fn measure(args: &MeasureArgs, work_root: &Path) -> Result<MeasureOutcome, String> {
    let oracle = Oracle::host_default()
        .map_err(|e| format!("measure requires the host oracle (correctness gate): {e}"))?;
    let views: Vec<View> = match args.view {
        None => vec![View::A, View::B, View::C, View::D, View::E],
        Some(v) => vec![v],
    };
    let workloads = select_workloads(args.workload.as_deref())?;
    // candidate sanity + compat stamp (one cheap prepare of the first selected workload)
    let compat = crate::prepare_candidate(workloads[0])
        .map(|p| p.compat.to_string())
        .unwrap_or_else(|_| "unavailable".to_string());
    // the candidate-lane adaptation ledger (spec 5.3) for every measured workload
    let mut adaptations: Vec<AdaptationLedgerEntry> = Vec::new();
    for w in &workloads {
        match crate::candidate_source(w) {
            Ok(ad) => {
                let (oracle_proved, compile_exit) =
                    oracle_prove_transformed(&oracle, work_root, w, &ad)?;
                adaptations.push(AdaptationLedgerEntry {
                    workload: w.name.to_string(),
                    source_files: w.sources.iter().map(|s| s.to_string()).collect(),
                    original_sha256: ad.original_hash,
                    transformed_sha256: ad.transformed_hash,
                    rewrites: ad.rewrites,
                    oracle_proved_preserving: oracle_proved,
                    oracle_compile_exit: compile_exit,
                    note: "candidate lane only; oracle compiles the original sources unchanged"
                        .to_string(),
                });
            }
            Err(e) => {
                return Err(format!("{}: adaptation ledger failed: {e}", w.name));
            }
        }
    }

    let mut view_a = None;
    let mut view_b = None;
    let mut view_c = None;
    let mut view_d = None;
    let mut view_e = None;
    for view in &views {
        match view {
            View::A => {
                let scale = scale_for(View::A, args.scale.as_deref());
                view_a = Some(run_view_a(
                    &oracle, work_root, &workloads, &scale, args.iters,
                )?);
            }
            View::B => {
                let scale = scale_for(View::B, args.scale.as_deref());
                view_b = Some(run_view_b(
                    &oracle, work_root, &workloads, &scale, args.iters,
                )?);
            }
            View::C => {
                let scale = scale_for(View::C, args.scale.as_deref());
                view_c = Some(run_view_c(
                    &oracle, work_root, &workloads, &scale, args.iters,
                )?);
            }
            View::D => {
                view_d = Some(run_view_d(&oracle, work_root, args.iters)?);
            }
            View::E => {
                view_e = Some(run_view_e(&oracle, work_root)?);
            }
        }
    }
    let control = ControlMeta {
        host_cpu: host_cpu(),
        cobc_version: cobc_version(&oracle),
        compiler_flags: COMPILER_FLAGS,
        iters: args.iters,
        warmup: WARMUP_RUNS,
        outlier_policy: OUTLIER_POLICY,
        generated_at_utc: generated_at_utc(),
        candidate_compat: compat,
        peak_memory_kb: peak_memory_kb(),
    };
    Ok(MeasureOutcome {
        control,
        view_a,
        view_b,
        view_c,
        view_d,
        view_e,
        adaptations,
    })
}

// ---------------------------------------------------------------------------------------------
// reports
// ---------------------------------------------------------------------------------------------

/// `reports/valid-corpus/performance/` under the workspace root.
pub fn performance_report_dir() -> Result<PathBuf, String> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest
        .ancestors()
        .nth(2)
        .ok_or_else(|| "workspace root".to_string())?;
    Ok(root.join("reports/valid-corpus/performance"))
}

/// Write every report for a measure run: `views.json` (merged -- measured views replace their
/// section, unmeasured views keep the previously written section), `phase-metrics.json` (when
/// View B ran), `raw/<view>.json` for every measured view, the merged `summary.md`, and
/// `adaptations.json` (the spec-5.3 candidate-lane adaptation ledger).
/// Merging lets views be run independently (`measure view-a` now, `measure view-e` later)
/// without clobbering earlier reports; the five view sections always stay separate.
pub fn write_reports(out: &MeasureOutcome) -> Result<(), String> {
    let report_dir = performance_report_dir()?;
    std::fs::create_dir_all(&report_dir).map_err(|e| e.to_string())?;
    merge_views_json(out, &report_dir)?;
    if let Some(view_b) = &out.view_b {
        write_phase_metrics_json(view_b, &out.control, &report_dir)?;
    }
    write_adaptations(out, &report_dir)?;
    write_raw(out, &report_dir)?;
    merge_summary(out, &report_dir)?;
    Ok(())
}

/// `adaptations.json`: the candidate-lane adaptation ledger (spec 5.3) -- original/transformed
/// hashes + applied rewrites per measured workload, so no report implies pristine parity for a
/// transformed lane.
fn write_adaptations(out: &MeasureOutcome, report_dir: &Path) -> Result<(), String> {
    if out.adaptations.is_empty() {
        return Ok(());
    }
    let doc = serde_json::json!({
        "note": "candidate-lane adaptations (spec 5.3): the oracle ALWAYS compiles the original \
                 corpus sources unchanged; the candidate lane prepares an adapted copy whose \
                 output is gated byte-exact against the independent expectation before any \
                 timing is reported. No pristine-parity claim is made for transformed lanes.",
        "adaptations": out.adaptations,
    });
    write_json(&report_dir.join("adaptations.json"), &doc)
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    std::fs::write(path, text).map_err(|e| e.to_string())
}

fn section<T: serde::Serialize>(v: &Option<T>) -> Result<Option<serde_json::Value>, String> {
    v.as_ref()
        .map(|v| serde_json::to_value(v).map_err(|e| e.to_string()))
        .transpose()
}

/// `views.json`: one document with separate `view_a`..`view_e` sections (never merged). A
/// section measured by this run replaces the previous one; a section not measured keeps the
/// previously written value (or `skipped` when there is none).
fn merge_views_json(out: &MeasureOutcome, report_dir: &Path) -> Result<(), String> {
    let path = report_dir.join("views.json");
    let existing: serde_json::Value = std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or(serde_json::Value::Null);
    let mut obj = serde_json::Map::new();
    obj.insert(
        "control".to_string(),
        serde_json::to_value(&out.control).map_err(|e| e.to_string())?,
    );
    for (key, new) in [
        ("view_a", section(&out.view_a)?),
        ("view_b", section(&out.view_b)?),
        ("view_c", section(&out.view_c)?),
        ("view_d", section(&out.view_d)?),
        ("view_e", section(&out.view_e)?),
    ] {
        let value = match new {
            Some(v) => v,
            None => existing
                .get(key)
                .cloned()
                .unwrap_or(serde_json::json!({ "skipped": true })),
        };
        obj.insert(key.to_string(), value);
    }
    write_json(&path, &serde_json::Value::Object(obj))
}

/// `phase-metrics.json`: per-workload per-scale PhaseTimings + source size + bytes/sec +
/// lines/sec (spec 9.1/9.2 attribution).
fn write_phase_metrics_json(
    view_b: &[ViewBEntry],
    control: &ControlMeta,
    report_dir: &Path,
) -> Result<(), String> {
    let mut workloads: BTreeMap<String, BTreeMap<String, &PhaseReport>> = BTreeMap::new();
    for e in view_b {
        workloads
            .entry(e.workload.clone())
            .or_default()
            .insert(e.scale.clone(), &e.candidate);
    }
    let doc = serde_json::json!({
        "control": control,
        "workloads": workloads,
    });
    write_json(&report_dir.join("phase-metrics.json"), &doc)
}

/// `raw/<view>.json`: every raw sample plus the control metadata (spec 9.7 raw retention).
fn write_raw(out: &MeasureOutcome, report_dir: &Path) -> Result<(), String> {
    let raw_dir = report_dir.join("raw");
    std::fs::create_dir_all(&raw_dir).map_err(|e| e.to_string())?;
    write_raw_view(&raw_dir, "view_a", &out.control, &out.view_a)?;
    write_raw_view(&raw_dir, "view_b", &out.control, &out.view_b)?;
    write_raw_view(&raw_dir, "view_c", &out.control, &out.view_c)?;
    write_raw_view(&raw_dir, "view_d", &out.control, &out.view_d)?;
    write_raw_view(&raw_dir, "view_e", &out.control, &out.view_e)?;
    Ok(())
}

fn write_raw_view<T: serde::Serialize>(
    raw_dir: &Path,
    name: &str,
    control: &ControlMeta,
    entries: &Option<T>,
) -> Result<(), String> {
    if let Some(entries) = entries {
        let doc = serde_json::json!({
            "control": control,
            "view": name,
            "entries": entries,
        });
        write_json(&raw_dir.join(format!("{name}.json")), &doc)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// summary.md (appended; the Phase-8 table stays intact at the top)
// ---------------------------------------------------------------------------------------------

fn lane_cell(s: &SampleSet) -> String {
    format!(
        "{:.2} (min {:.2}, IQR {:.2}, p95 {:.2})",
        s.median_ms, s.min_ms, s.iqr_ms, s.p95_ms
    )
}

fn view_a_md(entries: &[ViewAEntry]) -> String {
    let mut md = String::new();
    md.push_str("### View A — end-to-end one-shot (compile+run vs prepare+run)\n\n");
    md.push_str("**unlike workflows**: the native lane is a compiled binary; the candidate lane\n");
    md.push_str("is interpreted — these are NOT equivalent runtime work.\n\n");
    md.push_str("| workload | scale | oracle total ms (median) | candidate total ms (median) | ratio |\n|---|---|---|---|---|\n");
    for e in entries {
        md.push_str(&format!(
            "| {} | {} | {:.2} | {:.2} | {:.1}x |\n",
            e.workload,
            e.scale,
            e.oracle_total_ms.median_ms,
            e.candidate_total_ms.median_ms,
            ratio(e.oracle_total_ms.median_ms, e.candidate_total_ms.median_ms)
        ));
    }
    md.push('\n');
    md
}

fn view_b_md(entries: &[ViewBEntry]) -> String {
    let mut md = String::new();
    md.push_str("### View B — front-end only (oracle compile vs candidate per-phase prepare)\n\n");
    md.push_str("| workload | scale | oracle compile ms (median) | preprocess | lex | parse | resolution | layout | check | prepare | bytes/sec | lines/sec |\n|---|---|---|---|---|---|---|---|---|---|---|---|\n");
    for e in entries {
        let c = &e.candidate;
        md.push_str(&format!(
            "| {} | {} | {:.2} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {:.0} | {:.0} |\n",
            e.workload,
            e.scale,
            e.oracle_compile_ms.median_ms,
            c.preprocess_ms,
            c.lex_ms,
            c.parse_ms,
            c.resolution_ms,
            c.layout_ms,
            c.check_ms,
            c.prepare_ms,
            c.bytes_per_sec,
            c.lines_per_sec
        ));
    }
    md.push('\n');
    md
}

fn view_c_md(entries: &[ViewCEntry]) -> String {
    let mut md = String::new();
    md.push_str(
        "### View C — repeated execution (compiled binary vs prepared program, no reparse)\n\n",
    );
    md.push_str("| workload | scale | native median (min/p95) | candidate median (min/p95) | candidate prepare ms | outputs agree |\n|---|---|---|---|---|---|\n");
    for e in entries {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            e.workload,
            e.scale,
            lane_cell(&e.native),
            lane_cell(&e.candidate),
            e.candidate_prepare_ms,
            e.outputs_agree
        ));
    }
    md.push('\n');
    md
}

fn view_d_md(entries: &[micro::MicroEntry]) -> String {
    let mut md = String::new();
    md.push_str(
        "### View D — runtime-operation microbenchmarks (50_000 iterations, correctness-gated)\n\n",
    );
    md.push_str("| op | native median (min/p95) ms | candidate median (min/p95) ms | candidate prepare ms | byte-exact |\n|---|---|---|---|---|\n");
    for e in entries {
        md.push_str(&format!(
            "| {} ({}) | {} | {} | {} | {} |\n",
            e.name,
            e.description,
            lane_cell(&e.native),
            lane_cell(&e.candidate),
            e.candidate_prepare_ms,
            e.outputs_agree
        ));
    }
    md.push('\n');
    md
}

fn view_e_md(e: &ViewEResult) -> String {
    let mut md = String::new();
    md.push_str("### View E — corpus throughput (10 workloads x 4 scales, one pass)\n\n");
    md.push_str(&format!(
        "- oracle (compile+run) total: {:.1} ms\n",
        e.oracle_total_ms
    ));
    md.push_str(&format!(
        "- candidate (prepare+run) total: {:.1} ms\n",
        e.candidate_total_ms
    ));
    md.push_str(&format!(
        "- peak memory: {} ({})\n",
        e.peak_memory_kb
            .map(|k| format!("{k} kB"))
            .unwrap_or_else(|| "omitted".to_string()),
        e.memory_note
    ));
    md.push('\n');
    md.push_str("| workload | oracle total ms | candidate total ms |\n|---|---|---|\n");
    for w in WORKLOADS {
        let o = e.oracle_per_workload_ms.get(w.name).copied().unwrap_or(0.0);
        let c = e
            .candidate_per_workload_ms
            .get(w.name)
            .copied()
            .unwrap_or(0.0);
        md.push_str(&format!("| {} | {o:.1} | {c:.1} |\n", w.name));
    }
    md.push('\n');
    md
}

/// Merge the Phase 9 views summary into `summary.md`, keeping everything before the marker
/// (the Phase-8 table) intact. Per-view subsections for views measured by this run are
/// replaced; subsections for other views are kept. Re-running a view replaces only its own
/// subsection (idempotent).
fn merge_summary(out: &MeasureOutcome, report_dir: &Path) -> Result<(), String> {
    let path = report_dir.join("summary.md");
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    const MARKER: &str = "## Phase 9 — performance views";
    let head = match existing.find(MARKER) {
        Some(i) => existing[..i].to_string(),
        None => existing.clone(),
    };
    // Existing per-view subsections after the marker, keyed by view letter (kept when the
    // view was NOT measured now). Emitted in canonical A..E order afterwards.
    let mut chunks: Vec<(char, String)> = Vec::new();
    if let Some(i) = existing.find(MARKER) {
        let tail = &existing[i + MARKER.len()..];
        for chunk in tail.split("### ") {
            if chunk.trim().is_empty() {
                continue;
            }
            let letter = match chunk.trim_start().chars().nth(5) {
                Some('A') => 'A',
                Some('B') => 'B',
                Some('C') => 'C',
                Some('D') => 'D',
                Some('E') => 'E',
                _ => continue, // unknown chunk: dropped
            };
            chunks.push((letter, format!("### {chunk}")));
        }
    }
    // Fresh chunks for the views measured now replace their letter's kept chunk.
    for (letter, text) in [
        ('A', out.view_a.as_ref().map(|e| view_a_md(e))),
        ('B', out.view_b.as_ref().map(|e| view_b_md(e))),
        ('C', out.view_c.as_ref().map(|e| view_c_md(e))),
        ('D', out.view_d.as_ref().map(|e| view_d_md(e))),
        ('E', out.view_e.as_ref().map(|e| view_e_md(e))),
    ] {
        if let Some(text) = text {
            chunks.retain(|(l, _)| *l != letter);
            chunks.push((letter, text));
        }
    }
    chunks.sort_by_key(|(l, _)| *l);
    let mut md = head;
    md.push_str(MARKER);
    md.push_str("\n\n");
    let c = &out.control;
    md.push_str(&format!(
        "Control: host_cpu={} · cobc_version={} · compiler_flags=`{}` · iters={} · warmup={} · outlier_policy: {} · candidate_compat={} · generated_at_utc={}\n\n",
        c.host_cpu.as_deref().unwrap_or("unavailable"),
        c.cobc_version.as_deref().unwrap_or("unavailable"),
        c.compiler_flags,
        c.iters,
        c.warmup,
        c.outlier_policy,
        c.candidate_compat,
        c.generated_at_utc,
    ));
    for (_, text) in chunks {
        md.push_str(&text);
    }
    std::fs::write(&path, md).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------------------------
// console output
// ---------------------------------------------------------------------------------------------

/// Human-readable summary of a measure run (printed before the reports are written).
pub fn print_console(out: &MeasureOutcome) {
    println!(
        "measure: host_cpu={}, cobc_version={}, iters={}, warmup={}, candidate_compat={}",
        out.control.host_cpu.as_deref().unwrap_or("unavailable"),
        out.control.cobc_version.as_deref().unwrap_or("unavailable"),
        out.control.iters,
        out.control.warmup,
        out.control.candidate_compat
    );
    if let Some(va) = &out.view_a {
        let o: f64 = va.iter().map(|e| e.oracle_total_ms.median_ms).sum();
        let c: f64 = va.iter().map(|e| e.candidate_total_ms.median_ms).sum();
        println!(
            "view_a ({} workloads × small): oracle total median {o:.1} ms vs candidate {c:.1} ms \
             (unlike workflows: compiled vs interpreted)",
            va.len()
        );
    }
    if let Some(vb) = &out.view_b {
        let o: f64 = vb.iter().map(|e| e.oracle_compile_ms.median_ms).sum();
        let c: f64 = vb.iter().map(|e| e.candidate.prepare_ms).sum();
        println!(
            "view_b ({} workloads × small): oracle compile median {o:.1} ms vs candidate prepare \
             {c:.1} ms (per-phase in phase-metrics.json)",
            vb.len()
        );
    }
    if let Some(vc) = &out.view_c {
        let o: f64 = vc.iter().map(|e| e.native.median_ms).sum();
        let c: f64 = vc.iter().map(|e| e.candidate.median_ms).sum();
        println!(
            "view_c ({} workloads × small): native median total {o:.1} ms vs candidate {c:.1} ms \
             (no candidate reparsing)",
            vc.len()
        );
    }
    if let Some(vd) = &out.view_d {
        let o: f64 = vd.iter().map(|e| e.native.median_ms).sum();
        let c: f64 = vd.iter().map(|e| e.candidate.median_ms).sum();
        println!(
            "view_d ({} micro workloads, 50_000 iters each, correctness-gated): native median \
             total {o:.1} ms vs candidate {c:.1} ms",
            vd.len()
        );
    }
    if let Some(ve) = &out.view_e {
        println!(
            "view_e (full corpus, one pass): oracle total {:.1} ms vs candidate {:.1} ms; peak \
             memory {}",
            ve.oracle_total_ms,
            ve.candidate_total_ms,
            ve.peak_memory_kb
                .map(|k| format!("{k} kB"))
                .unwrap_or_else(|| "omitted".to_string())
        );
    }
}
