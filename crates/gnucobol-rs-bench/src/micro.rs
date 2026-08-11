//! View D micro workloads (spec 9.5): purpose-built single-operation COBOL programs with fixed
//! iteration counts, deterministic output, and an independent Rust expectation. Each micro is
//! correctness-gated: the native oracle run must be byte-exact against the expectation BEFORE
//! any timing is reported (never fabricate numbers).
//!
//! The COBOL sources hardcode the iteration constant (e.g. 50_000); [`MicroWorkload::iters`]
//! must match it. Every micro emits its result via `DISPLAY` (stdout); file I/O appears only
//! in the seqfile micro's INPUT (read), whose stdout is still the output channel.

use crate::gen;
use crate::Oracle;
use crate::SampleSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// One micro workload: a single COBOL source, a fixed iteration count, and an independent
/// expected-output calculator.
pub struct MicroWorkload {
    pub name: &'static str,
    pub description: &'static str,
    /// COBOL source under `cobol/` (relative path).
    pub source: &'static str,
    /// `(filename, content)` written into the run dir before compile; `None` for pure-CPU micros.
    pub input: Option<(&'static str, fn(usize) -> String)>,
    /// Fixed iteration count (the loop bound / record count in the COBOL source).
    pub iters: usize,
    /// Deterministic expected stdout for the fixed iteration count.
    pub expected: fn(usize) -> String,
}

fn tri_sum(iters: usize) -> u64 {
    let n = iters as u64;
    n * (n + 1) / 2
}

fn move_expected(iters: usize) -> String {
    format!("MOVE-DONE {iters:011} {iters:09}\n")
}

fn packed_add_expected(iters: usize) -> String {
    format!("PACKED-ADD-DONE {:012} {iters:09}\n", tri_sum(iters))
}

fn binary_add_expected(iters: usize) -> String {
    format!("BINARY-ADD-DONE {:012} {iters:09}\n", tri_sum(iters))
}

fn float_add_expected(iters: usize) -> String {
    format!(
        "FLOAT-ADD-DONE {:>10.2} {:>13.2} {iters:09}\n",
        iters as f64, iters as f64
    )
}

fn compare_expected(iters: usize) -> String {
    format!("COMPARE-DONE {:09} {iters:09}\n", iters / 2)
}

fn intrinsic_expected(iters: usize) -> String {
    format!("INTRINSIC-DONE {:012} {iters:09}\n", 1234u64 * iters as u64)
}

fn call_expected(iters: usize) -> String {
    format!(
        "CALL-DONE {:012} {iters:09}\n",
        tri_sum(iters) + iters as u64
    )
}

/// The seqfile micro's generated input (same record layout as the corpus seqfile workload).
fn seqfile_input(iters: usize) -> String {
    let (lines, _) = gen::seqfile(iters, gen::seed_for("micro_seqfile", "fixed"));
    lines.join("\n") + "\n"
}

/// The seqfile micro's expected stdout: one `K<key> <amount>` line per record, then the
/// VALID/INVALID summary (the same classification logic as the corpus seqfile workload).
fn seqfile_expected(iters: usize) -> String {
    let (_, recs) = gen::seqfile(iters, gen::seed_for("micro_seqfile", "fixed"));
    let mut out = String::new();
    let mut valid_sum: i128 = 0;
    let mut valid_n: i128 = 0;
    let mut invalid_n: i128 = 0;
    for (i, (amount, ok)) in recs.iter().enumerate() {
        if *ok {
            valid_sum += *amount as i128;
            valid_n += 1;
        } else {
            invalid_n += 1;
        }
        out.push_str(&format!("K{i:07} {amount:012}\n"));
    }
    out.push_str(&format!("VALID {valid_n:09} {valid_sum:>16}\n"));
    out.push_str(&format!("INVALID {invalid_n:09}\n"));
    out
}

pub const MICRO_WORKLOADS: &[MicroWorkload] = &[
    MicroWorkload {
        name: "move",
        description: "decimal MOVE (display -> display)",
        source: "micro_move.cob",
        input: None,
        iters: 50_000,
        expected: move_expected,
    },
    MicroWorkload {
        name: "packed_add",
        description: "packed-decimal ADD (COMP-3 accumulator)",
        source: "micro_packed_add.cob",
        input: None,
        iters: 50_000,
        expected: packed_add_expected,
    },
    MicroWorkload {
        name: "binary_add",
        description: "binary ADD (COMP accumulator)",
        source: "micro_binary_add.cob",
        input: None,
        iters: 50_000,
        expected: binary_add_expected,
    },
    MicroWorkload {
        name: "float_add",
        description: "float ADD (COMP-1 f32 + COMP-2 f64)",
        source: "micro_float_add.cob",
        input: None,
        iters: 50_000,
        expected: float_add_expected,
    },
    MicroWorkload {
        name: "compare",
        description: "alphanumeric comparison (IF A = B)",
        source: "micro_compare.cob",
        input: None,
        iters: 50_000,
        expected: compare_expected,
    },
    MicroWorkload {
        name: "intrinsic",
        description: "FUNCTION intrinsic dispatch (NUMVAL + INTEGER)",
        source: "micro_intrinsic.cob",
        input: None,
        iters: 50_000,
        expected: intrinsic_expected,
    },
    MicroWorkload {
        name: "call",
        description: "module CALL dispatch (contained subprogram)",
        source: "micro_call.cob",
        input: None,
        iters: 50_000,
        expected: call_expected,
    },
    MicroWorkload {
        name: "seqfile",
        description: "sequential-file read/write (50_000 fixed records)",
        source: "micro_seqfile.cob",
        input: Some(("micro_seqfile.dat", seqfile_input)),
        iters: 50_000,
        expected: seqfile_expected,
    },
];

pub fn micro_workload(name: &str) -> Option<&'static MicroWorkload> {
    MICRO_WORKLOADS.iter().find(|m| m.name == name)
}

/// The run directory for a micro workload (created by [`prepare_dir`]).
pub fn micro_dir(work_root: &Path, name: &str) -> PathBuf {
    work_root.join(format!("micro-{name}"))
}

/// Write the micro's generated input (if any) and COBOL source into `dir`.
pub fn prepare_dir(m: &MicroWorkload, dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    if let Some((file, gen_fn)) = m.input {
        let content = gen_fn(m.iters);
        std::fs::write(dir.join(file), content).map_err(|e| e.to_string())?;
    }
    let cobol_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cobol");
    let src = std::fs::read(cobol_dir.join(m.source)).map_err(|e| e.to_string())?;
    std::fs::write(dir.join(m.source), src).map_err(|e| e.to_string())?;
    Ok(())
}

/// The native artifact name for a micro workload (compile output name).
pub fn micro_artifact(m: &MicroWorkload) -> String {
    format!("micro_{}", m.name)
}

/// The run command for a micro workload's compiled binary (leading `./`, like the corpus).
pub fn micro_run_command(m: &MicroWorkload) -> String {
    format!("./micro_{}", m.name)
}

/// Compile a micro workload with the host oracle (`cobc -x -O2`, same flags as the corpus).
pub fn micro_compile(oracle: &Oracle, m: &MicroWorkload, dir: &Path) -> Result<(), String> {
    let artifact = micro_artifact(m);
    let cobc = oracle.cobc.to_string_lossy().into_owned();
    let argv = [
        cobc.as_str(),
        "-x",
        "-O2",
        "-o",
        artifact.as_str(),
        m.source,
    ];
    let (code, _out, err) = crate::run_cmd(oracle, dir, &argv);
    if code != Some(0) {
        return Err(format!(
            "micro {}: oracle compile failed (exit {:?}): {}",
            m.name,
            code,
            String::from_utf8_lossy(&err)
                .lines()
                .take(3)
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    Ok(())
}

/// Correctness gate: run the compiled micro once and require byte-exact stdout against the
/// independent expectation. Returns the expected sha256. Any mismatch aborts before timing.
pub fn micro_gate(oracle: &Oracle, m: &MicroWorkload, dir: &Path) -> Result<String, String> {
    let expected = (m.expected)(m.iters);
    let expected_sha = crate::sha256_hex(expected.as_bytes());
    let artifact = micro_run_command(m);
    let (code, out, err) = crate::run_cmd(oracle, dir, &[artifact.as_str()]);
    if code != Some(0) {
        return Err(format!(
            "micro {}: oracle run failed (exit {:?}): {}",
            m.name,
            code,
            String::from_utf8_lossy(&err)
                .lines()
                .take(2)
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    if out != expected.as_bytes() {
        return Err(format!(
            "micro {}: correctness gate FAILED (stdout {} bytes != expected {} bytes, sha {expected_sha})",
            m.name,
            out.len(),
            expected.len()
        ));
    }
    Ok(expected_sha)
}

/// Native lane: run the compiled micro binary `iters` times (one warmup first). Returns the
/// per-run ms samples and the last run's stdout.
pub fn micro_native(
    oracle: &Oracle,
    m: &MicroWorkload,
    dir: &Path,
    iters: usize,
) -> Result<(SampleSet, Vec<u8>), String> {
    let artifact = micro_run_command(m);
    let mut samples = Vec::with_capacity(iters);
    let mut last_out = Vec::new();
    for i in 0..iters + 1 {
        let t = Instant::now();
        let (code, out, err) = crate::run_cmd(oracle, dir, &[artifact.as_str()]);
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if code != Some(0) {
            return Err(format!(
                "micro {}: native run failed (exit {:?}): {}",
                m.name,
                code,
                String::from_utf8_lossy(&err)
                    .lines()
                    .take(2)
                    .collect::<Vec<_>>()
                    .join(" | ")
            ));
        }
        last_out = out;
        if i > 0 {
            samples.push(ms);
        }
    }
    Ok((crate::stats(&samples), last_out))
}

/// Candidate lane: prepare the micro source once (no reparse on run), then run it `iters` times.
/// Returns (samples, last stdout, prepare ms, compat stamp).
pub fn micro_candidate(
    m: &MicroWorkload,
    dir: &Path,
    iters: usize,
) -> Result<(SampleSet, Vec<u8>, u64, String), String> {
    let cobol_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cobol");
    let source = std::fs::read_to_string(cobol_dir.join(m.source)).map_err(|e| e.to_string())?;
    // the same candidate-lane normalization as the corpus (multi-mode OPEN, UNTIL EXIT,
    // multi-pattern INSPECT, embedded subprogram headers)
    let source = crate::normalize_candidate_source(&source);
    let (prepared, timings) = gnucobol_rs::frontend::prepare_program_timed(
        &source,
        gnucobol_rs::dialect::Dialect::DEFAULT,
    )
    .map_err(|e| format!("micro {}: candidate prepare failed: {e}", m.name))?;
    let prepare_ms = timings.prepare_ms as u64;
    let mut samples = Vec::with_capacity(iters);
    let mut last_out = Vec::new();
    for i in 0..iters + 1 {
        let t = Instant::now();
        let out = crate::run_prepared_in_dir(&prepared, dir)?;
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        last_out = out;
        if i > 0 {
            samples.push(ms);
        }
    }
    Ok((
        crate::stats(&samples),
        last_out,
        prepare_ms,
        prepared.compat.to_string(),
    ))
}

/// One measured micro workload: gate first, then time both lanes and assert output agreement.
/// Returns the serializable entry for View D.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MicroEntry {
    pub name: String,
    pub description: String,
    pub iters: usize,
    pub native: SampleSet,
    pub candidate: SampleSet,
    pub candidate_prepare_ms: u64,
    pub expected_sha256: String,
    pub native_output_sha256: String,
    pub candidate_output_sha256: String,
    pub outputs_agree: bool,
    pub note: String,
}

pub fn measure_micro(
    oracle: &Oracle,
    work_root: &Path,
    m: &MicroWorkload,
    iters: usize,
) -> Result<MicroEntry, String> {
    let dir = micro_dir(work_root, m.name);
    prepare_dir(m, &dir)?;
    micro_compile(oracle, m, &dir)?;
    let expected_sha = micro_gate(oracle, m, &dir)?;
    let (native, native_out) = micro_native(oracle, m, &dir, iters)?;
    let (candidate, candidate_out, prepare_ms, _compat) = micro_candidate(m, &dir, iters)?;
    // candidate lane byte-exactness: never time-report a lane whose output is wrong
    if candidate_out != (m.expected)(m.iters).as_bytes() {
        return Err(format!(
            "micro {}: candidate output mismatch ({} bytes, expected {})",
            m.name,
            candidate_out.len(),
            (m.expected)(m.iters).len()
        ));
    }
    let native_sha = crate::sha256_hex(&native_out);
    let candidate_sha = crate::sha256_hex(&candidate_out);
    let outputs_agree = native_sha == expected_sha;
    Ok(MicroEntry {
        name: m.name.to_string(),
        description: m.description.to_string(),
        iters: m.iters,
        native,
        candidate,
        candidate_prepare_ms: prepare_ms,
        expected_sha256: expected_sha,
        native_output_sha256: native_sha.clone(),
        candidate_output_sha256: candidate_sha,
        outputs_agree,
        note: "correctness-gated: oracle stdout byte-exact before timing".to_string(),
    })
}
