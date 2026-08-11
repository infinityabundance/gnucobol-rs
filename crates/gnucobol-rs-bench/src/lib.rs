//! gnucobol-rs-bench — purpose-designed scalable COBOL performance corpus (Phase 8).
//!
//! Ten workload families, each with small/medium/large/stress scales, deterministic Rust data
//! generators ([`gen`]), independently computed expected outputs ([`expected`]), and a runner
//! that validates each workload byte-exactly against the host GnuCOBOL oracle BEFORE any timing
//! (spec 8.3: a benchmark enters the corpus only after correctness passes).

pub mod expected;
pub mod gen;
pub mod micro;
pub mod views;

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

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest;
    let mut h = sha2::Sha256::new();
    h.update(bytes);
    let mut s = String::with_capacity(64);
    for b in h.finalize() {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

pub(crate) fn run_cmd(
    oracle: &Oracle,
    cwd: &Path,
    argv: &[&str],
) -> (Option<i32>, Vec<u8>, Vec<u8>) {
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

pub(crate) fn stats(samples: &[f64]) -> SampleSet {
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

/// The candidate source for a workload: all COBOL sources concatenated, so module subprograms
/// are contained programs (the prepared run's CALL resolution is in-process; external CALL is a
/// declared boundary). Single-source workloads are unchanged.
///
/// The concatenated source is passed through [`normalize_candidate_source`]: the sealed
/// candidate runtime lacks two idioms every corpus program uses, so the candidate lane
/// normalizes them to semantically identical forms (the oracle compiles the corpus sources
/// unchanged; only the candidate's prepared copy is normalized):
/// - multi-mode `OPEN INPUT A OUTPUT B.` -> `OPEN INPUT A.` + `OPEN OUTPUT B.` (the runtime's
///   OPEN only handles a single mode);
/// - `PERFORM UNTIL EXIT` (loop-forever idiom) -> `PERFORM UNTIL 1 = 2` (the runtime collects
///   an empty condition, since EXIT is a statement verb).
///
/// Every applied rewrite is recorded in the returned [`CandidateAdaptation`] (spec 5.3: the
/// original and transformed hashes + the rewrite list), so reports never claim pristine parity
/// for a transformed lane.
pub fn candidate_source(w: &Workload) -> Result<CandidateAdaptation, String> {
    let cobol_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cobol");
    let mut original = String::new();
    for (i, s) in w.sources.iter().enumerate() {
        let text = std::fs::read_to_string(cobol_dir.join(s)).map_err(|e| e.to_string())?;
        if i == 0 {
            original.push_str(&text);
        } else {
            // Subprogram sources: drop their `IDENTIFICATION DIVISION.` header so the
            // concatenated stream is `... main body tokens [PROGRAM-ID. sub ...]` -- the
            // runtime's parse splits programs at PROGRAM-ID, and a trailing IDENTIFICATION
            // DIVISION would land inside the main program's token range.
            for line in text.lines() {
                if line.trim().eq_ignore_ascii_case("IDENTIFICATION DIVISION.") {
                    continue;
                }
                original.push_str(line);
                original.push('\n');
            }
        }
        if !original.ends_with('\n') {
            original.push('\n');
        }
    }
    let original_hash = sha256_hex(original.as_bytes());
    let (source, rewrites) = normalize_candidate_source_detailed(&original);
    Ok(CandidateAdaptation {
        original_hash,
        transformed_hash: sha256_hex(source.as_bytes()),
        rewrites,
        source,
    })
}

/// The candidate lane's adapted copy of a workload's sources (spec 5.3 ledger). `source` is the
/// transformed text the candidate prepares; `original_hash`/`transformed_hash` bind the
/// adaptation; `rewrites` names every transformation actually applied. The oracle ALWAYS
/// compiles the original corpus sources unchanged -- the candidate lane is the only adapted
/// copy, and every view gates its output byte-exact against the independent expectation before
/// any timing is reported (a mis-normalization fails the gate loudly, never silently).
pub struct CandidateAdaptation {
    pub original_hash: String,
    pub transformed_hash: String,
    pub rewrites: Vec<String>,
    pub source: String,
}

/// See [`candidate_source`]: semantic-preserving line rewrites for the candidate lane. Lines
/// whose shape is unexpected are left untouched (best effort; a mis-normalization would fail
/// the correctness gate loudly, never silently).
pub(crate) fn normalize_candidate_source(source: &str) -> String {
    normalize_candidate_source_detailed(source).0
}

/// [`normalize_candidate_source`] plus the list of rewrite classes actually applied (the
/// spec-5.3 adaptation ledger: every transformation the candidate lane applies is recorded, and
/// the oracle always compiles the ORIGINAL sources unchanged -- the candidate lane is the only
/// adapted copy, and every view gates its output byte-exact against the independent
/// expectation before any timing is reported).
pub(crate) fn normalize_candidate_source_detailed(source: &str) -> (String, Vec<String>) {
    let mut applied: Vec<String> = Vec::new();
    let is_mode = |t: &str| matches!(t, "INPUT" | "OUTPUT" | "EXTEND" | "I-O");
    // Statement verbs / scope enders that begin a NEW statement line: the INSPECT rewrite
    // collects continuation lines only while they do NOT start one of these.
    let new_stmt = |w: &str| {
        matches!(
            w,
            "MOVE"
                | "SET"
                | "INITIALIZE"
                | "INSPECT"
                | "STRING"
                | "UNSTRING"
                | "ADD"
                | "SUBTRACT"
                | "MULTIPLY"
                | "DIVIDE"
                | "COMPUTE"
                | "DISPLAY"
                | "IF"
                | "PERFORM"
                | "STOP"
                | "CONTINUE"
                | "ACCEPT"
                | "GO"
                | "EVALUATE"
                | "SEARCH"
                | "CALL"
                | "GOBACK"
                | "EXIT"
                | "CANCEL"
                | "OPEN"
                | "CLOSE"
                | "READ"
                | "WRITE"
                | "REWRITE"
                | "DELETE"
                | "START"
                | "UNLOCK"
                | "SORT"
                | "MERGE"
                | "RELEASE"
                | "RETURN"
                | "END-PERFORM"
                | "END-IF"
                | "END-READ"
                | "END-STRING"
                | "END-UNSTRING"
                | "END-COMPUTE"
                | "END-INSPECT"
                | "ELSE"
        )
    };
    let lines: Vec<&str> = source.lines().collect();
    let mut out = String::with_capacity(source.len() + 256);
    let mut i = 0;
    // A second (or later) program in one source: its `IDENTIFICATION DIVISION.` precedes its
    // PROGRAM-ID, so it would land inside the PREVIOUS program's token range. Drop those
    // headers (the parse splits programs at PROGRAM-ID).
    let mut seen_program_id = false;
    while i < lines.len() {
        let line = lines[i];
        let indent_len = line.len() - line.trim_start().len();
        let indent = &line[..indent_len];
        let trimmed = line.trim_start();
        let upper = trimmed.to_ascii_uppercase();
        if upper.starts_with("PROGRAM-ID") {
            seen_program_id = true;
        }
        if seen_program_id && upper == "IDENTIFICATION DIVISION." {
            if !applied.iter().any(|r| r == "drop-embedded-program-header") {
                applied.push("drop-embedded-program-header".to_string());
            }
            i += 1;
            continue;
        }
        // `PERFORM UNTIL EXIT` -- GnuCOBOL's loop-forever idiom; the candidate needs a real
        // (always-false) condition.
        if upper == "PERFORM UNTIL EXIT" {
            if !applied.iter().any(|r| r == "perform-until-exit->1=2") {
                applied.push("perform-until-exit->1=2".to_string());
            }
            out.push_str(indent);
            out.push_str("PERFORM UNTIL 1 = 2\n");
            i += 1;
            continue;
        }
        // `INSPECT t TALLYING c FOR ALL "a" "b" ...` -- the candidate takes ONE pattern per
        // FOR; split into one INSPECT per pattern (each adds to the same counter). The
        // statement may span continuation lines and need not end with a period before the
        // next statement verb (COBOL sentence rules).
        if upper.starts_with("INSPECT ") {
            let start_i = i;
            let mut statement = upper.clone();
            i += 1;
            while i < lines.len() && !statement.ends_with('.') {
                let next = lines[i].trim_start().to_ascii_uppercase();
                if next.is_empty() || new_stmt(next.split_whitespace().next().unwrap_or("")) {
                    break;
                }
                statement.push(' ');
                statement.push_str(&next);
                i += 1;
            }
            let mut rewritten = false;
            if let Some(alli) = statement.split_whitespace().position(|w| w == "ALL") {
                let words: Vec<&str> = statement.split_whitespace().collect();
                let patterns: Vec<&str> = words[alli + 1..]
                    .iter()
                    .filter(|w| w.len() >= 2 && w.starts_with('\"') && w.ends_with('\"'))
                    .map(|w| *w)
                    .collect();
                if patterns.len() >= 2 {
                    rewritten = true;
                    if !applied.iter().any(|r| r == "inspect-split-per-pattern") {
                        applied.push("inspect-split-per-pattern".to_string());
                    }
                    // No trailing period: the corpus bodies are sentence-per-iteration (periods
                    // inside a PERFORM body would end the block in the sealed runtime).
                    for p in &patterns {
                        out.push_str(indent);
                        out.push_str("INSPECT ");
                        out.push_str(&words[1..alli].join(" "));
                        out.push_str(" ALL ");
                        out.push_str(p);
                        out.push('\n');
                    }
                }
            }
            if rewritten {
                continue;
            }
            // not rewritten: re-emit the original statement lines verbatim
            for l in &lines[start_i..i] {
                out.push_str(l);
                out.push('\n');
            }
            continue;
        }
        let mut rewritten: Option<String> = None;
        if upper.starts_with("OPEN ") && trimmed.trim_end().ends_with('.') {
            let tokens: Vec<String> = upper
                .split_whitespace()
                .skip(1) // the OPEN verb
                .map(|t| t.trim_end_matches('.').to_string())
                .collect();
            let mode_count = tokens.iter().filter(|t| is_mode(t.as_str())).count();
            if tokens.len() >= 2 && is_mode(&tokens[0]) && mode_count > 1 {
                let mut pairs: Vec<(String, String)> = Vec::new();
                let mut mode = String::new();
                for t in tokens {
                    if is_mode(&t) {
                        mode = t;
                    } else if !t.is_empty() {
                        pairs.push((mode.clone(), t));
                    }
                }
                if !pairs.is_empty() {
                    if !applied.iter().any(|r| r == "open-split-per-mode") {
                        applied.push("open-split-per-mode".to_string());
                    }
                    let mut text = String::new();
                    for (m, f) in &pairs {
                        text.push_str(indent);
                        text.push_str("OPEN ");
                        text.push_str(m);
                        text.push(' ');
                        text.push_str(f);
                        text.push_str(".\n");
                    }
                    rewritten = Some(text);
                }
            }
        }
        match rewritten {
            Some(text) => out.push_str(&text),
            None => {
                out.push_str(line);
                out.push('\n');
            }
        }
        i += 1;
    }
    (out, applied)
}

/// Prepare a workload's candidate program (full front-end once; spec 9.6).
pub fn prepare_candidate(w: &Workload) -> Result<gnucobol_rs::frontend::PreparedProgram, String> {
    let adaptation = candidate_source(w)?;
    gnucobol_rs::frontend::prepare_program(
        &adaptation.source,
        gnucobol_rs::dialect::Dialect::DEFAULT,
    )
    .map_err(|e| format!("candidate prepare failed: {e}"))
}

/// Run a prepared program with the working directory set to `dir` (file I/O needs the input in
/// cwd), restoring the previous cwd afterwards. Returns the program's stdout.
pub fn run_prepared_in_dir(
    prepared: &gnucobol_rs::frontend::PreparedProgram,
    dir: &Path,
) -> Result<Vec<u8>, String> {
    let prev = std::env::current_dir().map_err(|e| e.to_string())?;
    std::env::set_current_dir(dir).map_err(|e| e.to_string())?;
    let res = prepared
        .run(false)
        .map(|(out, _printer, _rc)| out)
        .map_err(|e| format!("candidate run failed: {e}"));
    let _ = std::env::set_current_dir(&prev);
    res
}

/// One measured run of the candidate (prepared) lane: prepare (front-end) once, then run the
/// prepared program `iters` times without reparsing (spec 9.6). Returns per-run ms samples and
/// the prepared-program identity.
pub fn candidate_prepared(
    w: &Workload,
    _scale: &str,
    dir: &Path,
    iters: usize,
) -> Result<(SampleSet, Vec<u8>, u64, String), String> {
    // the candidate source = all COBOL sources concatenated (module subprograms are contained)
    let adaptation = candidate_source(w)?;
    let t0 = Instant::now();
    let prepared = gnucobol_rs::frontend::prepare_program(
        &adaptation.source,
        gnucobol_rs::dialect::Dialect::DEFAULT,
    )
    .map_err(|e| format!("candidate prepare failed: {e}"))?;
    let prepare_ms = t0.elapsed().as_millis() as u64;
    // warmup + measured runs (the input file must be in the current dir for file I/O)
    let mut samples = Vec::with_capacity(iters);
    let mut last_out = Vec::new();
    for i in 0..iters + 1 {
        let t = Instant::now();
        let out = run_prepared_in_dir(&prepared, dir)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn oracle_available() -> bool {
        Oracle::host_default().is_ok()
    }

    /// (a) `stats` is stable: sorted median/min/IQR/p95 on a known sample, with raw retention.
    #[test]
    fn stats_is_stable_on_known_sample() {
        let s = stats(&[5.0, 1.0, 4.0, 2.0, 3.0]);
        assert_eq!(s.samples_ms, vec![1.0, 2.0, 3.0, 4.0, 5.0]); // raw samples retained, sorted
        assert_eq!(s.median_ms, 3.0);
        assert_eq!(s.min_ms, 1.0);
        assert_eq!(s.iqr_ms, 2.0); // q1 = 2.0, q3 = 4.0
        assert_eq!(s.p95_ms, 5.0); // floor(5 * 0.95) = 4
    }

    /// `stats` on an even-length sample (median = upper middle, matching the Phase-8 formula).
    #[test]
    fn stats_even_length_uses_upper_middle_median() {
        let s = stats(&[1.0, 2.0, 3.0, 4.0]);
        assert_eq!(s.median_ms, 3.0);
        assert_eq!(s.min_ms, 1.0);
        assert_eq!(s.iqr_ms, 2.0); // q1 = s[1] = 2.0, q3 = s[3] = 4.0
    }

    /// (c) the `measure` subcommand rejects an unknown view name.
    #[test]
    fn measure_rejects_unknown_view_name() {
        assert!(views::parse_view("view-x").is_err());
        assert!(views::parse_view("bogus").is_err());
        let args = vec!["view-x".to_string()];
        assert!(views::parse_measure_args(&args).is_err());
    }

    /// `measure` accepts defaults and `--iters`.
    #[test]
    fn measure_accepts_defaults_and_iters() {
        let args = vec!["--iters".to_string(), "3".to_string()];
        let a = views::parse_measure_args(&args).expect("defaults");
        assert!(a.view.is_none());
        assert!(a.workload.is_none());
        assert_eq!(a.iters, 3);
        let a2 = views::parse_measure_args(&["view-c".to_string(), "payroll".to_string()])
            .expect("view-c payroll");
        assert_eq!(a2.view, Some(views::View::C));
        assert_eq!(a2.workload.as_deref(), Some("payroll"));
    }

    /// (b) View C lane: repeated prepared runs are byte-identical, and native + candidate
    /// outputs agree byte-exactly for one small workload. Oracle-guarded (skips when the host
    /// oracle is not built -- e.g. CI without `lab/oracle/prefix/bin/cobc`).
    #[test]
    fn candidate_and_native_outputs_agree_small() {
        if !oracle_available() {
            eprintln!("skipping: host oracle not built (lab/oracle/prefix/bin/cobc)");
            return;
        }
        let work_root = std::env::temp_dir().join("gnucobol-rs-bench-test-c");
        let w = workload("payroll").expect("payroll workload");
        let r = validate(w, "small", &work_root).expect("validate payroll small");
        assert!(r.byte_exact, "oracle gate: {}", r.note);
        let dir = work_root.join("payroll-small");

        let (set1, _, prepare_ms, compat) =
            candidate_prepared(w, "small", &dir, 3).expect("candidate_prepared");
        assert_eq!(compat, "prepared-v1");
        assert!(prepare_ms > 0);
        let cand1 = std::fs::read(dir.join(w.output_file)).expect("candidate output");

        let (set2, _, _, _) =
            candidate_prepared(w, "small", &dir, 3).expect("candidate_prepared again");
        let cand2 = std::fs::read(dir.join(w.output_file)).expect("candidate output 2");
        assert_eq!(
            cand1, cand2,
            "repeated prepared runs must produce byte-identical output"
        );

        let oracle = Oracle::host_default().expect("oracle");
        let (nset, _) = native_runs(&oracle, w, &dir, 3).expect("native_runs");
        let nat = std::fs::read(dir.join(w.output_file)).expect("native output");
        assert_eq!(
            nat, cand2,
            "native and candidate outputs must agree byte-exactly"
        );
        for s in [set1, set2, nset] {
            assert!(s.median_ms >= 0.0 && s.min_ms >= 0.0);
        }
    }
}
