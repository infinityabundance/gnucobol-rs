//! The versioned admission schema: `gnurust-valid-cobol-program-v1`.
//!
//! A program is never "valid COBOL" in the abstract: validity is represented relative to an
//! explicit profile (oracle identity, dialect, source format, encoding, compiler options,
//! copybook paths, defines, runtime configuration, platform). Every extracted unit receives
//! exactly one [`Classification`]; no unit may remain unclassified at completion.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SCHEMA: &str = "gnurust-valid-cobol-program-v1";

/// The three top-level corpus classes. They are reported separately and never merged into one
/// headline count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CorpusClass {
    /// GnuCOBOL native testsuite, CCVS85, official manual examples, programs shipped with
    /// GnuCOBOL: feature implementation + exact differential behaviour + first-failure
    /// attribution + stable/current compatibility.
    UpstreamSemantic,
    /// Official contribution collections, the Open Mainframe course, vetted public
    /// repositories, X-COBOL: realistic structure, dialect breadth, project-level
    /// dependencies, generalization (with a frozen held-out subset).
    ExternalValid,
    /// Purpose-built scalable programs + runtime-operation microbenchmarks.
    Performance,
}

/// Which admitted source family a unit came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SourceFamily {
    GnucobolTestsuite,
    Ccvs85,
    GnucobolManual,
    GnucobolExtras,
    OmpCourse,
    Xcobol,
    Bench,
}

/// Origin custody: a source must be reproducible from origin + immutable revision + expected
/// hash + extraction rules + licence decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Origin {
    pub kind: OriginKind,
    pub url: String,
    pub revision: String,
    /// Path of the source within the repository/archive (empty = root).
    #[serde(default)]
    pub source_path: String,
    /// SHA-256 of the archive/bundle the revision was admitted from, when applicable.
    #[serde(default)]
    pub archive_sha256: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OriginKind {
    Git,
    Archive,
    Other,
}

/// Licence decision. Unknown or conflicting licences quarantine the unit (never published).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Licence {
    /// SPDX expression (e.g. "GPL-3.0-or-later", "MIT", "LicenseRef-Public-Domain").
    pub spdx_expression: String,
    pub redistribution_allowed: bool,
    #[serde(default)]
    pub notice_paths: Vec<String>,
    /// Decision text: who decided, when, based on what evidence.
    #[serde(default)]
    pub decision: String,
    pub reviewed: bool,
}

/// Source byte facts. Original bytes are preserved separately (see [`crate::bytes`]); this is the
/// decoded/analysed identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceInfo {
    /// Content-addressed files (blob SHA-256), in dependency order.
    pub files: Vec<String>,
    pub main_file: String,
    #[serde(default)]
    pub copybooks: Vec<String>,
    #[serde(default)]
    pub modules: Vec<String>,
    pub encoding: String,
    pub line_endings: String,
    pub source_format: String,
    /// SHA-256 of the main file's original bytes.
    pub content_sha256: String,
}

/// The profile under which a unit is valid. Validity collapses across dialects is forbidden.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidityProfile {
    pub oracle: String,
    #[serde(default)]
    pub oracle_sha256: Option<String>,
    pub dialect: String,
    #[serde(default)]
    pub compiler_options: Vec<String>,
    #[serde(default)]
    pub include_paths: Vec<String>,
    #[serde(default)]
    pub defines: BTreeMap<String, String>,
    #[serde(default)]
    pub environment: BTreeMap<String, String>,
    #[serde(default)]
    pub runtime_configuration: BTreeMap<String, String>,
    pub platform: String,
}

/// The oracle contract: what the admitted oracle is expected to do under the profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct OracleResult {
    pub compile_exit: i32,
    #[serde(default)]
    pub warnings_expected: bool,
    pub run_required: bool,
    #[serde(default)]
    pub run_exit: Option<i32>,
    #[serde(default)]
    pub stdout_sha256: Option<String>,
    #[serde(default)]
    pub stderr_sha256: Option<String>,
    /// generated-file path (relative to the run dir) -> blob SHA-256.
    #[serde(default)]
    pub generated_files: BTreeMap<String, String>,
    pub deterministic: bool,
}

/// Phase-attributed candidate outcome. Exactly one first failure per profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CandidateResult {
    /// "ok" or a typed failure diagnostic.
    pub preprocess: String,
    pub parse: String,
    pub resolve: String,
    pub layout: String,
    pub check: String,
    pub prepare: String,
    pub run: String,
    /// `(phase, diagnostic)` of the FIRST failing phase, when any.
    #[serde(default)]
    pub first_failure: Option<(String, String)>,
}

/// Exactly one classification per unit (no `UNKNOWN` may remain at completion).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Classification {
    // ---- valid classes ------------------------------------------------------
    ValidExecutableProgram,
    ValidCompileOnlyProgram,
    ValidModuleProgram,
    ValidFunctionProgram,
    ValidCopybook,
    ValidConfigurationProgram,
    ValidNativeArtifactProgram,
    // ---- invalid / diagnostic classes --------------------------------------
    InvalidExpectedReject,
    InvalidExpectedWarning,
    DiagnosticShapeOnly,
    // ---- non-program classes -----------------------------------------------
    SourceFragment,
    GeneratedSource,
    CopybookOnly,
    JclOnly,
    DataOnly,
    MissingDependency,
    UnknownDialect,
    UnsupportedEncoding,
    LicenceRestricted,
    LicenceUnknown,
    Nondeterministic,
    Duplicate,
    NearDuplicate,
    Quarantined,
}

impl Classification {
    /// Whether this classification is an ADMITTED valid program class.
    pub fn is_valid_program(self) -> bool {
        matches!(
            self,
            Classification::ValidExecutableProgram
                | Classification::ValidCompileOnlyProgram
                | Classification::ValidModuleProgram
                | Classification::ValidFunctionProgram
                | Classification::ValidConfigurationProgram
        )
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Classification::ValidExecutableProgram => "VALID_EXECUTABLE_PROGRAM",
            Classification::ValidCompileOnlyProgram => "VALID_COMPILE_ONLY_PROGRAM",
            Classification::ValidModuleProgram => "VALID_MODULE_PROGRAM",
            Classification::ValidFunctionProgram => "VALID_FUNCTION_PROGRAM",
            Classification::ValidCopybook => "VALID_COPYBOOK",
            Classification::ValidConfigurationProgram => "VALID_CONFIGURATION_PROGRAM",
            Classification::ValidNativeArtifactProgram => "VALID_NATIVE_ARTIFACT_PROGRAM",
            Classification::InvalidExpectedReject => "INVALID_EXPECTED_REJECT",
            Classification::InvalidExpectedWarning => "INVALID_EXPECTED_WARNING",
            Classification::DiagnosticShapeOnly => "DIAGNOSTIC_SHAPE_ONLY",
            Classification::SourceFragment => "SOURCE_FRAGMENT",
            Classification::GeneratedSource => "GENERATED_SOURCE",
            Classification::CopybookOnly => "COPYBOOK_ONLY",
            Classification::JclOnly => "JCL_ONLY",
            Classification::DataOnly => "DATA_ONLY",
            Classification::MissingDependency => "MISSING_DEPENDENCY",
            Classification::UnknownDialect => "UNKNOWN_DIALECT",
            Classification::UnsupportedEncoding => "UNSUPPORTED_ENCODING",
            Classification::LicenceRestricted => "LICENCE_RESTRICTED",
            Classification::LicenceUnknown => "LICENCE_UNKNOWN",
            Classification::Nondeterministic => "NONDETERMINISTIC",
            Classification::Duplicate => "DUPLICATE",
            Classification::NearDuplicate => "NEAR_DUPLICATE",
            Classification::Quarantined => "QUARANTINED",
        }
    }

    pub fn parse(s: &str) -> Option<Classification> {
        Some(match s {
            "VALID_EXECUTABLE_PROGRAM" => Classification::ValidExecutableProgram,
            "VALID_COMPILE_ONLY_PROGRAM" => Classification::ValidCompileOnlyProgram,
            "VALID_MODULE_PROGRAM" => Classification::ValidModuleProgram,
            "VALID_FUNCTION_PROGRAM" => Classification::ValidFunctionProgram,
            "VALID_COPYBOOK" => Classification::ValidCopybook,
            "VALID_CONFIGURATION_PROGRAM" => Classification::ValidConfigurationProgram,
            "VALID_NATIVE_ARTIFACT_PROGRAM" => Classification::ValidNativeArtifactProgram,
            "INVALID_EXPECTED_REJECT" => Classification::InvalidExpectedReject,
            "INVALID_EXPECTED_WARNING" => Classification::InvalidExpectedWarning,
            "DIAGNOSTIC_SHAPE_ONLY" => Classification::DiagnosticShapeOnly,
            "SOURCE_FRAGMENT" => Classification::SourceFragment,
            "GENERATED_SOURCE" => Classification::GeneratedSource,
            "COPYBOOK_ONLY" => Classification::CopybookOnly,
            "JCL_ONLY" => Classification::JclOnly,
            "DATA_ONLY" => Classification::DataOnly,
            "MISSING_DEPENDENCY" => Classification::MissingDependency,
            "UNKNOWN_DIALECT" => Classification::UnknownDialect,
            "UNSUPPORTED_ENCODING" => Classification::UnsupportedEncoding,
            "LICENCE_RESTRICTED" => Classification::LicenceRestricted,
            "LICENCE_UNKNOWN" => Classification::LicenceUnknown,
            "NONDETERMINISTIC" => Classification::Nondeterministic,
            "DUPLICATE" => Classification::Duplicate,
            "NEAR_DUPLICATE" => Classification::NearDuplicate,
            "QUARANTINED" => Classification::Quarantined,
            _ => return None,
        })
    }
}

/// The full admission record. `program_id` is a stable identity independent of local paths
/// (e.g. `ccvs85/NC107A`, `gnucobol-current/run-move/step-004/program-main`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgramRecord {
    pub schema: String,
    pub program_id: String,
    pub corpus_class: CorpusClass,
    pub source_family: SourceFamily,
    pub origin: Origin,
    pub licence: Licence,
    pub source: SourceInfo,
    pub validity_profile: ValidityProfile,
    pub oracle: OracleResult,
    pub candidate: CandidateResult,
    pub classification: Classification,
    /// Admission state machine position (see [`crate::state`]).
    pub admission_state: String,
    /// `ADMITTED` / typed rejection reason / discovered-step note.
    pub admission_note: String,
    /// Snapshot of the exact corpus-tool version that produced this record.
    pub tool_version: String,
}

impl ProgramRecord {
    /// Validate structural invariants of the schema. Returns a list of violations (empty = valid).
    pub fn validate(&self) -> Vec<String> {
        let mut errs = Vec::new();
        if self.schema != SCHEMA {
            errs.push(format!("schema mismatch: {}", self.schema));
        }
        if self.program_id.trim().is_empty() {
            errs.push("program_id is empty".into());
        }
        if self.source.main_file.trim().is_empty() {
            errs.push("source.main_file is empty".into());
        }
        if self.source.content_sha256.trim().is_empty() {
            errs.push("source.content_sha256 is empty".into());
        }
        if self.validity_profile.dialect.trim().is_empty() {
            errs.push("validity_profile.dialect is empty".into());
        }
        if self.validity_profile.oracle.trim().is_empty() {
            errs.push("validity_profile.oracle is empty".into());
        }
        // A valid program class requires an oracle contract: compile exit recorded and, for
        // executable programs, a run outcome.
        if self.classification.is_valid_program() {
            if !self.licence.reviewed {
                errs.push("valid program without a reviewed licence decision".into());
            }
            if self.oracle.compile_exit != 0 {
                errs.push("valid program with nonzero oracle compile exit".into());
            }
            if self.classification == Classification::ValidExecutableProgram
                && self.oracle.run_exit.is_none()
            {
                errs.push("valid executable program without an oracle run outcome".into());
            }
        }
        // No byte parity claims without recorded hashes.
        if self.oracle.stdout_sha256.is_some() && self.oracle.stdout_sha256.as_deref() == Some("") {
            errs.push("empty stdout_sha256 recorded".into());
        }
        errs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_record() -> ProgramRecord {
        ProgramRecord {
            schema: SCHEMA.to_string(),
            program_id: "ccvs85/NC107A".to_string(),
            corpus_class: CorpusClass::UpstreamSemantic,
            source_family: SourceFamily::Ccvs85,
            origin: Origin {
                kind: OriginKind::Archive,
                url: "https://www.itl.nist.gov/div897/ctg/soe/ccvs85/".to_string(),
                revision: "newcob.val.Z".to_string(),
                source_path: "NC107A.cob".to_string(),
                archive_sha256: Some("ab".repeat(32)),
            },
            licence: Licence {
                spdx_expression: "LicenseRef-Public-Domain".to_string(),
                redistribution_allowed: true,
                notice_paths: vec![],
                decision: "NIST CCVS85 is a public-domain government test suite".to_string(),
                reviewed: true,
            },
            source: SourceInfo {
                files: vec!["d".repeat(64)],
                main_file: "NC107A.cob".to_string(),
                copybooks: vec![],
                modules: vec![],
                encoding: "UTF-8".to_string(),
                line_endings: "LF".to_string(),
                source_format: "fixed".to_string(),
                content_sha256: "e".repeat(64),
            },
            validity_profile: ValidityProfile {
                oracle: "GnuCOBOL 3.2.0".to_string(),
                oracle_sha256: None,
                dialect: "cobol85".to_string(),
                compiler_options: vec!["-fixed".to_string()],
                include_paths: vec![],
                defines: BTreeMap::new(),
                environment: BTreeMap::new(),
                runtime_configuration: BTreeMap::new(),
                platform: "linux-x86_64".to_string(),
            },
            oracle: OracleResult {
                compile_exit: 0,
                warnings_expected: false,
                run_required: true,
                run_exit: Some(0),
                stdout_sha256: Some("f".repeat(64)),
                stderr_sha256: Some("g".repeat(64)),
                generated_files: BTreeMap::new(),
                deterministic: true,
            },
            candidate: CandidateResult::default(),
            classification: Classification::ValidExecutableProgram,
            admission_state: "ADMITTED".to_string(),
            admission_note: String::new(),
            tool_version: "0.1.0".to_string(),
        }
    }

    #[test]
    fn schema_validates_a_complete_record() {
        assert!(base_record().validate().is_empty());
    }

    #[test]
    fn schema_rejects_missing_identity() {
        let mut r = base_record();
        r.program_id = "".to_string();
        assert!(r.validate().iter().any(|e| e.contains("program_id")));
        r = base_record();
        r.schema = "gnurust-valid-cobol-program-v0".to_string();
        assert!(r.validate().iter().any(|e| e.contains("schema")));
    }

    #[test]
    fn schema_rejects_valid_class_without_reviewed_licence() {
        let mut r = base_record();
        r.licence.reviewed = false;
        let errs = r.validate();
        assert!(errs.iter().any(|e| e.contains("licence")), "{errs:?}");
    }

    #[test]
    fn schema_rejects_valid_executable_without_run_outcome() {
        let mut r = base_record();
        r.oracle.run_exit = None;
        assert!(r.validate().iter().any(|e| e.contains("run outcome")));
    }

    #[test]
    fn schema_rejects_valid_class_with_nonzero_compile_exit() {
        let mut r = base_record();
        r.oracle.compile_exit = 1;
        assert!(r.validate().iter().any(|e| e.contains("compile exit")));
    }

    #[test]
    fn all_classifications_round_trip() {
        for c in [
            Classification::ValidExecutableProgram,
            Classification::ValidCompileOnlyProgram,
            Classification::ValidModuleProgram,
            Classification::ValidFunctionProgram,
            Classification::ValidCopybook,
            Classification::ValidConfigurationProgram,
            Classification::ValidNativeArtifactProgram,
            Classification::InvalidExpectedReject,
            Classification::InvalidExpectedWarning,
            Classification::DiagnosticShapeOnly,
            Classification::SourceFragment,
            Classification::GeneratedSource,
            Classification::CopybookOnly,
            Classification::JclOnly,
            Classification::DataOnly,
            Classification::MissingDependency,
            Classification::UnknownDialect,
            Classification::UnsupportedEncoding,
            Classification::LicenceRestricted,
            Classification::LicenceUnknown,
            Classification::Nondeterministic,
            Classification::Duplicate,
            Classification::NearDuplicate,
            Classification::Quarantined,
        ] {
            assert_eq!(Classification::parse(c.as_str()), Some(c), "{}", c.as_str());
        }
        assert_eq!(Classification::parse("UNKNOWN"), None);
    }
}
