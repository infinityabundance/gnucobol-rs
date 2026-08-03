//! Shared data model for the `GNURUST.CCVS85.2/.3/.4` differential court.
//!
//! Every indexed CCVS85 unit finishes in exactly one [`FinalClassification`]; the structured
//! per-unit record carries orthogonal oracle/candidate/comparison fields so distinct failure
//! classes (oracle rejection vs candidate rejection vs harness limitation vs timeout vs
//! nondeterminism vs output mismatch vs exit-status mismatch ...) are never conflated.

use serde::{Deserialize, Serialize};

/// The committed `GNURUST.CCVS85.1` custody facts this court re-verifies before doing anything.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Custody {
    pub compressed_sha256: String,
    pub compressed_bytes: u64,
    pub decompressed_sha256: String,
    pub decompressed_bytes: u64,
    pub decompressed_lines: u64,
    pub unit_count: usize,
    pub header_by_kind: std::collections::BTreeMap<String, usize>,
}

/// One split unit from the corpus spine (index/kind/name/line-range), mirroring the committed
/// `reports/ccvs85/corpus-index.json` entries.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UnitIndexEntry {
    pub index: usize,
    pub kind: String,
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
}

/// The materialized form of one indexed unit (GNURUST.CCVS85.2 manifest entry).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MaterializedUnit {
    pub unit_index: usize,
    pub kind: String,
    /// The CCVS85 name (for a `SUBRTN` header this is the SUBPROGRAM's name).
    pub name: String,
    /// The raw `*HEADER,...` line, preserved for provenance.
    pub header_raw: String,
    /// For `*HEADER,COBOL,<MAIN>,SUBRTN,<SUB>` units: the main program this subprogram binds to.
    pub main_program: Option<String>,
    /// For `*HEADER,COBOL,<MAIN>,SUBRTN,<SUB>` units: the subprogram name (== `name`).
    pub subprogram: Option<String>,
    /// Stable, filesystem-safe path relative to the materialized root.
    pub source_path: String,
    /// SHA-256 of the materialized file bytes (original, unmodified).
    pub source_sha256: String,
    /// Path of the site-adapted execution copy (the file the oracle/candidate compile), relative
    /// to the materialized root (under `adapted/`).
    pub adapted_path: String,
    /// SHA-256 of the site-adapted copy (see [`crate::corpus::SITE_ADAPTATION`]).
    pub adapted_sha256: String,
    /// 1-based line range in the decompressed corpus.
    pub start_line: usize,
    pub end_line: usize,
    /// `PROGRAM-ID` names found in the unit source.
    pub program_ids: Vec<String>,
    /// `COPY <name>.` references that resolve to a CLBRY unit.
    pub copy_dependencies: Vec<String>,
    /// `COPY <name>.` references with no CLBRY unit in the corpus (dependency gap).
    pub missing_copybooks: Vec<String>,
    /// DATA* units this unit consumes (fed on stdin at run time).
    pub data_dependencies: Vec<String>,
    /// Whether this unit is a standalone executable candidate (COBOL, not a SUBRTN-only unit).
    pub is_executable_candidate: bool,
}

/// Outcome of one subprocess invocation (compile or run), with raw evidence files preserved.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Invocation {
    pub command: Vec<String>,
    pub cwd: String,
    pub environment: Vec<String>,
    pub exit_code: Option<i32>,
    pub signal: Option<i32>,
    pub timed_out: bool,
    pub duration_ms: u64,
    pub stdout_path: Option<String>,
    pub stderr_path: Option<String>,
    /// sha256 of the captured stdout bytes ("" when absent).
    pub stdout_sha256: String,
    pub stderr_sha256: String,
    /// Files the invocation produced (e.g. the compiled binary, the report file) as
    /// (relative path, sha256, bytes) triples.
    pub artifacts: Vec<Artifact>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Artifact {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
}

/// The oracle (real GnuCOBOL 3.2) side of one unit.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct OracleSide {
    /// compile: pass | reject | error | skipped
    pub compile: String,
    pub compile_invocation: Option<Invocation>,
    /// run: pass | fail | timeout | skipped | not-applicable
    pub run: String,
    pub run_invocation: Option<Invocation>,
    /// The CCVS85 report file (PRINT-FILE) bytes sha256, when produced.
    pub report_sha256: String,
    /// Parsed CCVS85 verdict counts when the report has a parseable summary.
    pub verdict_counts: Option<VerdictCounts>,
}

/// The gnucobol-rs candidate side of one unit.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CandidateSide {
    /// prepare: accept | reject-unsupported | reject-parse | reject-layout | reject-runtime-boundary | bound-to-main | dependency-blocked | error
    pub prepare: String,
    pub prepare_invocation: Option<Invocation>,
    /// The raw cobrun exit code (0 = RETURN-CODE 0, n = RETURN-CODE n, 2 = fail-closed reject,
    /// 124 = timeout, 128+sig = signal).
    pub prepare_invocation_rc: Option<i32>,
    /// run: pass | fail | timeout | not-run | not-applicable
    pub run: String,
    pub run_invocation: Option<Invocation>,
    /// The candidate's captured stdout sha256.
    pub stdout_sha256: String,
}

/// The comparison of one unit's oracle vs candidate observable results.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ComparisonSide {
    /// match | mismatch | not_comparable
    pub raw_stdout: String,
    /// match | mismatch | not_comparable
    pub canonical_stdout: String,
    /// match | mismatch | not_comparable
    pub generated_files: String,
    /// match | mismatch | not_comparable
    pub exit_status: String,
    /// CCVS85 summary-count comparison when both reports parsed.
    pub verdict_counts: String,
    /// The oracle's parsed counts (recorded on both sides of the comparison).
    pub oracle_counts: Option<VerdictCounts>,
    pub candidate_counts: Option<VerdictCounts>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct VerdictCounts {
    pub passed: u64,
    pub failed: u64,
    pub deleted: u64,
    pub inspect: u64,
    pub informational: u64,
    /// The raw lines the counts were extracted from (for audit).
    pub source_lines: Vec<String>,
}

/// The primary classification. Exactly one per indexed unit.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FinalClassification {
    NonExecutableLibrary,
    NonExecutableData,
    OracleCompilePass,
    OracleCompileReject,
    OracleCompileError,
    OracleRunPass,
    OracleRunFail,
    OracleTimeout,
    RustAcceptAndRun,
    RustAcceptButRuntimeFail,
    RustRejectUnsupported,
    RustRejectParse,
    RustRejectLayout,
    RustRejectRuntimeBoundary,
    RustTimeout,
    RawOutputMatch,
    CanonicalOutputMatch,
    OutputMismatch,
    ExitStatusMismatch,
    GeneratedFileMismatch,
    HarnessBlocked,
    DependencyBlocked,
    InfrastructureError,
}

/// The full per-unit court record (one row of `comparison-results.json`).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UnitResult {
    pub unit_index: usize,
    pub kind: String,
    pub name: String,
    pub source_path: String,
    pub source_sha256: String,
    pub oracle: OracleSide,
    pub candidate: CandidateSide,
    pub comparison: ComparisonSide,
    pub final_classification: FinalClassification,
    pub reason_code: String,
    /// When the two determinism passes disagree, this is set with both classifications.
    pub nondeterministic: bool,
    pub determinism: Option<DeterminismNote>,
    /// The first cobc error line (compile reject) / first cobrun message (reject), for bucketing.
    pub first_failure_line: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeterminismNote {
    pub pass_a: String,
    pub pass_b: String,
}

/// Summary counts (must reconcile: every indexed unit accounted for exactly once).
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Summary {
    pub units_total: usize,
    pub units_by_kind: std::collections::BTreeMap<String, usize>,
    pub executable_candidates: usize,
    pub non_executable_library: usize,
    pub non_executable_data: usize,
    pub oracle_compile_pass: usize,
    pub oracle_compile_reject: usize,
    pub oracle_compile_error: usize,
    pub oracle_run_pass: usize,
    pub oracle_run_fail: usize,
    pub oracle_timeout: usize,
    pub candidate_accepted: usize,
    pub candidate_unsupported: usize,
    pub candidate_parse_fail: usize,
    pub candidate_runtime_fail: usize,
    pub candidate_timeout: usize,
    pub raw_output_match: usize,
    pub canonical_output_match: usize,
    pub output_mismatch: usize,
    pub exit_status_mismatch: usize,
    pub generated_file_mismatch: usize,
    pub harness_blocked: usize,
    pub dependency_blocked: usize,
    pub infrastructure_error: usize,
    pub nondeterministic: usize,
    pub by_final_classification: std::collections::BTreeMap<String, usize>,
    pub by_reason_code: std::collections::BTreeMap<String, usize>,
    pub by_section: std::collections::BTreeMap<String, usize>,
    pub oracle_candidate_pair: std::collections::BTreeMap<String, usize>,
}

impl FinalClassification {
    pub fn as_str(&self) -> &'static str {
        match self {
            FinalClassification::NonExecutableLibrary => "NON_EXECUTABLE_LIBRARY",
            FinalClassification::NonExecutableData => "NON_EXECUTABLE_DATA",
            FinalClassification::OracleCompilePass => "ORACLE_COMPILE_PASS",
            FinalClassification::OracleCompileReject => "ORACLE_COMPILE_REJECT",
            FinalClassification::OracleCompileError => "ORACLE_COMPILE_ERROR",
            FinalClassification::OracleRunPass => "ORACLE_RUN_PASS",
            FinalClassification::OracleRunFail => "ORACLE_RUN_FAIL",
            FinalClassification::OracleTimeout => "ORACLE_TIMEOUT",
            FinalClassification::RustAcceptAndRun => "RUST_ACCEPT_AND_RUN",
            FinalClassification::RustAcceptButRuntimeFail => "RUST_ACCEPT_BUT_RUNTIME_FAIL",
            FinalClassification::RustRejectUnsupported => "RUST_REJECT_UNSUPPORTED",
            FinalClassification::RustRejectParse => "RUST_REJECT_PARSE",
            FinalClassification::RustRejectLayout => "RUST_REJECT_LAYOUT",
            FinalClassification::RustRejectRuntimeBoundary => "RUST_REJECT_RUNTIME_BOUNDARY",
            FinalClassification::RustTimeout => "RUST_TIMEOUT",
            FinalClassification::RawOutputMatch => "RAW_OUTPUT_MATCH",
            FinalClassification::CanonicalOutputMatch => "CANONICAL_OUTPUT_MATCH",
            FinalClassification::OutputMismatch => "OUTPUT_MISMATCH",
            FinalClassification::ExitStatusMismatch => "EXIT_STATUS_MISMATCH",
            FinalClassification::GeneratedFileMismatch => "GENERATED_FILE_MISMATCH",
            FinalClassification::HarnessBlocked => "HARNESS_BLOCKED",
            FinalClassification::DependencyBlocked => "DEPENDENCY_BLOCKED",
            FinalClassification::InfrastructureError => "INFRASTRUCTURE_ERROR",
        }
    }
}

impl From<&str> for FinalClassification {
    fn from(s: &str) -> Self {
        match s {
            "NON_EXECUTABLE_LIBRARY" => FinalClassification::NonExecutableLibrary,
            "NON_EXECUTABLE_DATA" => FinalClassification::NonExecutableData,
            "ORACLE_COMPILE_PASS" => FinalClassification::OracleCompilePass,
            "ORACLE_COMPILE_REJECT" => FinalClassification::OracleCompileReject,
            "ORACLE_COMPILE_ERROR" => FinalClassification::OracleCompileError,
            "ORACLE_RUN_PASS" => FinalClassification::OracleRunPass,
            "ORACLE_RUN_FAIL" => FinalClassification::OracleRunFail,
            "ORACLE_TIMEOUT" => FinalClassification::OracleTimeout,
            "RUST_ACCEPT_AND_RUN" => FinalClassification::RustAcceptAndRun,
            "RUST_ACCEPT_BUT_RUNTIME_FAIL" => FinalClassification::RustAcceptButRuntimeFail,
            "RUST_REJECT_UNSUPPORTED" => FinalClassification::RustRejectUnsupported,
            "RUST_REJECT_PARSE" => FinalClassification::RustRejectParse,
            "RUST_REJECT_LAYOUT" => FinalClassification::RustRejectLayout,
            "RUST_REJECT_RUNTIME_BOUNDARY" => FinalClassification::RustRejectRuntimeBoundary,
            "RUST_TIMEOUT" => FinalClassification::RustTimeout,
            "RAW_OUTPUT_MATCH" => FinalClassification::RawOutputMatch,
            "CANONICAL_OUTPUT_MATCH" => FinalClassification::CanonicalOutputMatch,
            "OUTPUT_MISMATCH" => FinalClassification::OutputMismatch,
            "EXIT_STATUS_MISMATCH" => FinalClassification::ExitStatusMismatch,
            "GENERATED_FILE_MISMATCH" => FinalClassification::GeneratedFileMismatch,
            "HARNESS_BLOCKED" => FinalClassification::HarnessBlocked,
            "DEPENDENCY_BLOCKED" => FinalClassification::DependencyBlocked,
            "INFRASTRUCTURE_ERROR" => FinalClassification::InfrastructureError,
            _ => FinalClassification::InfrastructureError,
        }
    }
}

/// CCVS85 section prefix of a unit name (the first two characters) — used for the
/// by-section grouping (CM/DB/IC/NC/SQ/... etc.).
pub fn section_of(name: &str) -> String {
    let up = name.to_ascii_uppercase();
    let letters: String = up.chars().take_while(|c| c.is_ascii_alphabetic()).collect();
    letters.chars().take(2).collect()
}
