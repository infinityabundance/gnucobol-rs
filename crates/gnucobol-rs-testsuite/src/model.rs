//! Data model for the GNURUST.GNUCOBOL-TESTSUITE.{1,2,3} court: per-test records, the orthogonal
//! oracle/candidate/comparison model, and the required classifications (prompt §3.2/§3.3).

use serde::{Deserialize, Serialize};

/// A single test group's outcome in one side's run (baseline or candidate).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TestStatus {
    Pass,
    Fail,
    Skip,
    Xfail,
    Xpass,
}

impl TestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TestStatus::Pass => "PASS",
            TestStatus::Fail => "FAIL",
            TestStatus::Skip => "SKIP",
            TestStatus::Xfail => "XFAIL",
            TestStatus::Xpass => "XPASS",
        }
    }
}

/// One test group, as recorded by the generated Autotest `testsuite` in `testsuite.log`
/// (line `N. <title> (<at-file:line>): <msg>`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRecord {
    /// Ordinal test number (1..1346 for the GnuCOBOL 3.2 suite).
    pub number: usize,
    /// The `AT_SETUP` title.
    pub title: String,
    /// `AT_SETUP` source location (`<file>.at:<line>`), e.g. `run_fundamental.at:1564`.
    pub at_source: String,
    pub status: TestStatus,
    /// The trailing message detail: `ok (0.6s)`, `FAILED (<check-line>)`, `skipped (<reason>)`, …
    pub detail: String,
    /// Wall time of the group when reported, seconds (0 when absent).
    pub seconds: f64,
}

/// A single observed compiler/tool invocation from the baseline census recorder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invocation {
    pub t: String,
    pub cwd: String,
    #[serde(default)]
    pub tool: String,
    pub argv: Vec<String>,
    pub env: serde_json::Map<String, serde_json::Value>,
}

/// Option classification categories (prompt §0.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OptionCategory {
    Semantic,
    Dialect,
    SourceFormat,
    IncludeCopybook,
    Preprocessor,
    OutputSelection,
    CompileLinkMode,
    RuntimeModule,
    Diagnostic,
    OptimizationDebug,
    TestHarness,
    Unknown,
}

/// The orthogonal per-test result model (prompt §3.2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResultRow {
    pub test_id: String,
    pub number: usize,
    pub title: String,
    pub group: String,
    pub oracle: StatusView,
    pub wrapper: WrapperView,
    pub candidate: CandidateView,
    pub comparison: ComparisonView,
    pub primary_classification: String,
    pub reason_code: String,
    /// Diagnostic tests (Phase-3.6 dimension): whether the CANDIDATE rejected the source the oracle
    /// also rejected (`REJECT`) or accepted it (`ACCEPT`), or whether execution semantics apply
    /// (`EXEC`). A rejection is the SEMANTICALLY correct verdict even when the message differs.
    pub semantic_diagnostic_verdict: String,
    /// Diagnostic tests (Phase-3.6 dimension): whether the candidate's stderr byte-matches the
    /// oracle's expected stderr for the failing check (`MATCH`), differs (`DIFFERS`), or the check
    /// did not compare stderr (`N/A`). Exact cobc wording is NOT required for a correct rejection;
    /// this dimension reports the shape separately.
    pub diagnostic_shape_parity: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatusView {
    pub compile: String,
    pub run: String,
    pub verdict: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WrapperView {
    pub argument_translation: String,
    pub artifact_generation: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CandidateView {
    pub preprocess: String,
    pub parse: String,
    pub check: String,
    pub prepare: String,
    pub run: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComparisonView {
    pub stdout: String,
    pub stderr: String,
    pub exit_status: String,
    pub files: String,
}

/// The stable summary counts (reconciled over every indexed test).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Summary {
    pub total_tests: usize,
    pub oracle: OracleTotals,
    pub candidate: CandidateTotals,
    pub comparison: ComparisonTotals,
    pub wrapper: WrapperTotals,
    pub first_failure: std::collections::BTreeMap<String, usize>,
    pub reason_codes: std::collections::BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OracleTotals {
    pub pass: usize,
    pub fail: usize,
    pub skip: usize,
    pub xfail: usize,
    pub xpass: usize,
    pub timeout: usize,
    pub infra_error: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CandidateTotals {
    pub preprocess_reject: usize,
    pub parse_reject: usize,
    pub check_reject: usize,
    pub layout_reject: usize,
    pub unsupported: usize,
    pub module_model_unsupported: usize,
    pub runtime_fail: usize,
    pub timeout: usize,
    pub nondeterministic: usize,
    pub passed: usize,
    pub skipped: usize,
    pub not_reached: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComparisonTotals {
    pub observable_match: usize,
    pub stdout_mismatch: usize,
    pub stderr_mismatch: usize,
    pub exit_status_mismatch: usize,
    pub generated_file_mismatch: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WrapperTotals {
    pub option_translated: usize,
    pub option_noop: usize,
    pub option_unsupported: usize,
    pub invocation_malformed: usize,
    pub artifact_error: usize,
}
