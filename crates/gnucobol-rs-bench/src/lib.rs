//! gnucobol-rs-bench — purpose-designed scalable COBOL performance corpus (Phase 8).
//!
//! Ten workload families, each with small/medium/large/stress scales, deterministic Rust data
//! generators ([`gen`]), independently computed expected outputs ([`expected`]), and a runner
//! that validates each workload byte-exactly against the host GnuCOBOL oracle BEFORE any timing
//! (spec 8.3: a benchmark enters the corpus only after correctness passes).

pub mod expected;
pub mod gen;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

/// One workload: its COBOL sources, the generator+expected pair, and the run command.
pub struct Workload {
    pub name: &'static str,
    pub description: &'static str,
    /// COBOL sources (relative to the crate's `cobol/` dir), in dependency order.
    pub sources: &'static [&'static str],
    /// The generated input filename (written before compile/run).
    pub input_file: &'static str,
    /// The output filename the program writes.
    pub output_file: &'static str,
    /// Expected-output calculator: (input, expected, note).
    pub expected: fn(usize, &str) -> (String, String, String),
    /// Run command template after compilation (the artifact name).
    pub run_artifact: &'static str,
    /// Whether the workload needs its module subprograms compiled alongside.
    pub modules: bool,
}

pub const WORKLOADS: &[Workload] = &[
    Workload {
        name: "payroll",
        description: "packed-decimal payroll: COMP-3 totals, tax, rounding, report",
        sources: &["payroll.cob"],
        input_file: "payroll.dat",
        output_file: "payroll.out",
        expected: expected::payroll,
        run_artifact: "./payroll",
        modules: false,
    },
    Workload {
        name: "invoice",
        description: "invoice and account processing: decimal multiplication, discounts, taxes",
        sources: &["invoice.cob"],
        input_file: "invoice.dat",
        output_file: "invoice.out",
        expected: expected::invoice,
        run_artifact: "./invoice",
        modules: false,
    },
    Workload {
        name: "seqfile",
        description: "sequential-file batch: validation, aggregation, file-status",
        sources: &["seqfile.cob"],
        input_file: "seqfile.dat",
        output_file: "seqfile.out",
        expected: expected::seqfile,
        run_artifact: "./seqfile",
        modules: false,
    },
    Workload {
        name: "tables",
        description: "table processing: OCCURS, SORT, SEARCH ALL, aggregation",
        sources: &["tables.cob"],
        input_file: "tables.dat",
        output_file: "tables.out",
        expected: expected::tables,
        run_artifact: "./tables",
        modules: false,
    },
    Workload {
        name: "strings",
        description: "string processing: STRING, UNSTRING, INSPECT, refmod",
        sources: &["strings.cob"],
        input_file: "strings.dat",
        output_file: "strings.out",
        expected: expected::strings,
        run_artifact: "./strings",
        modules: false,
    },
    Workload {
        name: "float",
        description: "floating-point: COMP-1/COMP-2, SIZE ERROR",
        sources: &["floatwork.cob"],
        input_file: "float.dat",
        output_file: "float.out",
        expected: expected::floatwork,
        run_artifact: "./float",
        modules: false,
    },
    Workload {
        name: "report",
        description: "report generation: grouping, subtotals, grand total",
        sources: &["report.cob"],
        input_file: "report.dat",
        output_file: "report.out",
        expected: expected::reportwork,
        run_artifact: "./report",
        modules: false,
    },
    Workload {
        name: "relative",
        description: "relative-file: insert, lookup, update, delete, traversal",
        sources: &["relative.cob"],
        input_file: "relative.dat",
        output_file: "relative.out",
        expected: expected::relative,
        run_artifact: "./relative",
        modules: false,
    },
    Workload {
        name: "modules",
        description: "module workload: dynamic CALL, EXTERNAL data, CANCEL, reload",
        sources: &["modcall.cob", "calcm.cob"],
        input_file: "modules.dat",
        output_file: "modules.out",
        expected: expected::modules,
        run_artifact: "./modcall",
        modules: true,
    },
    Workload {
        name: "mixed",
        description:
            "mixed business workflow: file input, validation, tables, module calls, report",
        sources: &["mixed.cob", "mixmod.cob"],
        input_file: "mixed.dat",
        output_file: "mixed.out",
        expected: expected::mixed,
        run_artifact: "./mixed",
        modules: true,
    },
];

pub fn workload(name: &str) -> Option<&'static Workload> {
    WORKLOADS.iter().find(|w| w.name == name)
}

/// The resolved host oracle environment (same pinned layout as the corpus crate).
pub struct Oracle {
    pub prefix: PathBuf,
    pub cobc: PathBuf,
}

impl Oracle {
    pub fn host_default() -> Result<Oracle, String> {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root = manifest
            .ancestors()
            .nth(2)
            .ok_or_else(|| "workspace root".to_string())?;
        let prefix = root.join("lab/oracle/prefix");
        let cobc = prefix.join("bin/cobc");
        if !cobc.exists() {
            return Err(format!("host oracle not built: {}", cobc.display()));
        }
        Ok(Oracle { prefix, cobc })
    }

    pub fn env(&self) -> Vec<(String, String)> {
        vec![
            (
                "PATH".into(),
                format!(
                    "{}:{}",
                    self.prefix.join("bin").display(),
                    std::env::var("PATH").unwrap_or_default()
                ),
            ),
            (
                "LD_LIBRARY_PATH".into(),
                self.prefix.join("lib").display().to_string(),
            ),
            (
                "COB_CONFIG_DIR".into(),
                self.prefix
                    .join("share/gnucobol/config")
                    .display()
                    .to_string(),
            ),
            ("LC_ALL".into(), "C".into()),
            ("LANG".into(), "C".into()),
            ("TZ".into(), "UTC0".into()),
        ]
    }
}

/// One validation + timing result.
#[derive(Debug, Clone, serde::Serialize)]
pub struct BenchResult {
    pub workload: String,
    pub scale: String,
    pub records: usize,
    pub input_sha256: String,
    pub expected_sha256: String,
    pub oracle_compile_ms: u64,
    pub oracle_run_ms: u64,
    pub output_sha256: String,
    pub byte_exact: bool,
    pub note: String,
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest;
    let mut h = sha2::Sha256::new();
    h.update(bytes);
    let mut s = String::with_capacity(64);
    for b in h.finalize() {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn run_cmd(oracle: &Oracle, cwd: &Path, argv: &[&str]) -> (Option<i32>, Vec<u8>, Vec<u8>) {
    let mut cmd = Command::new(argv[0]);
    cmd.args(&argv[1..]).current_dir(cwd);
    cmd.env_clear();
    for (k, v) in oracle.env() {
        cmd.env(k, v);
    }
    match cmd.output() {
        Ok(o) => (o.status.code(), o.stdout, o.stderr),
        Err(_) => (None, Vec::new(), Vec::new()),
    }
}

/// Validate one workload at one scale against the host oracle; byte-exact output required
/// before any benchmark claim. Returns the result.
pub fn validate(w: &Workload, scale: &str, work_root: &Path) -> Result<BenchResult, String> {
    let oracle = Oracle::host_default()?;
    let n = gen::record_count(scale);
    let dir = work_root.join(format!("{}-{}", w.name, scale));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    // generate the input + expected
    let (input, expected, note) = (w.expected)(n, scale);
    std::fs::write(dir.join(w.input_file), &input).map_err(|e| e.to_string())?;
    let expected_sha = sha256_hex(expected.as_bytes());

    // copy the COBOL sources
    let cobol_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cobol");
    for s in w.sources {
        let src = std::fs::read(cobol_dir.join(s)).map_err(|e| e.to_string())?;
        std::fs::write(dir.join(s), src).map_err(|e| e.to_string())?;
    }

    // compile
    let artifact = w.run_artifact.trim_start_matches("./");
    let mut argv: Vec<String> = vec![
        oracle.cobc.display().to_string(),
        "-x".to_string(),
        "-O2".to_string(),
        "-o".to_string(),
        artifact.to_string(),
    ];
    for s in w.sources {
        argv.push(s.to_string());
    }
    let argv_refs: Vec<&str> = argv.iter().map(|s| s.as_str()).collect();
    let t0 = Instant::now();
    let (code, _out, err) = run_cmd(&oracle, &dir, &argv_refs);
    let compile_ms = t0.elapsed().as_millis() as u64;
    if code != Some(0) {
        return Err(format!(
            "{} @ {}: oracle compile failed (exit {:?}): {}",
            w.name,
            scale,
            code,
            String::from_utf8_lossy(&err)
                .lines()
                .take(3)
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }

    // run
    let t1 = Instant::now();
    let (code, _out, err) = run_cmd(&oracle, &dir, &[w.run_artifact]);
    let run_ms = t1.elapsed().as_millis() as u64;
    if code != Some(0) {
        return Err(format!(
            "{} @ {}: oracle run failed (exit {:?}): {}",
            w.name,
            scale,
            code,
            String::from_utf8_lossy(&err)
                .lines()
                .take(3)
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }

    // compare byte-exact
    let actual = std::fs::read(dir.join(w.output_file)).map_err(|e| e.to_string())?;
    let actual_str = String::from_utf8_lossy(&actual).into_owned();
    let expected_str = expected.trim_end_matches('\n');
    let actual_str_t = actual_str.trim_end_matches('\n');
    let byte_exact = actual_str_t == expected_str;
    let note = if byte_exact {
        if note.is_empty() {
            "byte-exact match".to_string()
        } else {
            note
        }
    } else {
        format!(
            "OUTPUT MISMATCH (expected {} bytes, got {} bytes)",
            expected_str.len(),
            actual_str_t.len()
        )
    };
    Ok(BenchResult {
        workload: w.name.to_string(),
        scale: scale.to_string(),
        records: n,
        input_sha256: sha256_hex(input.as_bytes()),
        expected_sha256: expected_sha,
        oracle_compile_ms: compile_ms,
        oracle_run_ms: run_ms,
        output_sha256: sha256_hex(&actual),
        byte_exact,
        note,
    })
}

// ---- Phase 9 measurement views ----------------------------------------------------------

/// Raw timing samples for one lane of one view.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SampleSet {
    pub samples_ms: Vec<f64>,
    pub median_ms: f64,
    pub min_ms: f64,
    pub iqr_ms: f64,
    pub p95_ms: f64,
}

fn stats(samples: &[f64]) -> SampleSet {
    let mut s = samples.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let med = s[s.len() / 2];
    let min = s[0];
    let q1 = s[s.len() / 4];
    let q3 = s[s.len() * 3 / 4];
    let p95 = s[(((s.len() as f64) * 0.95) as usize).min(s.len() - 1)];
    SampleSet {
        samples_ms: s,
        median_ms: med,
        min_ms: min,
        iqr_ms: q3 - q1,
        p95_ms: p95,
    }
}

/// One measured run of the candidate (prepared) lane: prepare (front-end) once, then run the
/// prepared program `iters` times without reparsing (spec 9.6). Returns per-run ms samples and
/// the prepared-program identity.
pub fn candidate_prepared(
    w: &Workload,
    scale: &str,
    dir: &Path,
    iters: usize,
) -> Result<(SampleSet, Vec<u8>, u64, String), String> {
    // the main source = first .cob source
    let cobol_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cobol");
    let main = w.sources[0];
    let source = std::fs::read_to_string(cobol_dir.join(main)).map_err(|e| e.to_string())?;
    let t0 = Instant::now();
    let prepared =
        gnucobol_rs::frontend::prepare_program(&source, gnucobol_rs::dialect::Dialect::DEFAULT)
            .map_err(|e| format!("candidate prepare failed: {e}"))?;
    let prepare_ms = t0.elapsed().as_millis() as u64;
    // warmup + measured runs (the input file must be in the current dir for file I/O)
    std::env::set_current_dir(dir).map_err(|e| e.to_string())?;
    let mut samples = Vec::with_capacity(iters);
    let mut last_out = Vec::new();
    for i in 0..iters + 1 {
        let t = Instant::now();
        let (out, _printer, _rc) = prepared
            .run(false)
            .map_err(|e| format!("candidate run failed: {e}"))?;
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        last_out = out;
        if i > 0 {
            samples.push(ms);
        }
    }
    Ok((
        stats(&samples),
        last_out,
        prepare_ms,
        prepared.compat.to_string(),
    ))
}

/// Native lane: run the compiled binary `iters` times. Returns per-run ms samples.
pub fn native_runs(
    oracle: &Oracle,
    w: &Workload,
    dir: &Path,
    iters: usize,
) -> Result<(SampleSet, Vec<u8>), String> {
    let mut samples = Vec::with_capacity(iters);
    let mut last_out = Vec::new();
    for i in 0..iters + 1 {
        let t = Instant::now();
        let (code, out, err) = run_cmd(oracle, dir, &[w.run_artifact]);
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if code != Some(0) {
            return Err(format!(
                "native run failed (exit {:?}): {}",
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
    Ok((stats(&samples), last_out))
}

/// Validate every workload at every scale. Returns the results keyed by workload.
pub fn validate_all(work_root: &Path) -> Result<BTreeMap<String, Vec<BenchResult>>, String> {
    let mut out: BTreeMap<String, Vec<BenchResult>> = BTreeMap::new();
    for w in WORKLOADS {
        let mut results = Vec::new();
        for scale in ["small", "medium", "large", "stress"] {
            match validate(w, scale, work_root) {
                Ok(r) => {
                    if !r.byte_exact {
                        return Err(format!(
                            "{} @ {}: correctness gate failed before benchmarking: {}",
                            w.name, scale, r.note
                        ));
                    }
                    results.push(r);
                }
                Err(e) => return Err(e),
            }
        }
        out.insert(w.name.to_string(), results);
    }
    Ok(out)
}
