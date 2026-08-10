//! Materialized program packages from the GnuCOBOL Autotest suite.
//!
//! One [`StepPackage`] per `AT_CHECK` step, carrying everything needed to replay that step
//! outside the monolithic harness: the group's source files, the expanded command, the oracle
//! contract (expected status/stdout/stderr), the environment, and the identity. The expansion
//! table mirrors the suite's own `atlocal` definitions (`FLAGS`, `COMPILE_ONLY`, ...) so a step
//! replays with the same compiler options under which it is accepted.

use super::at::AtGroup;

/// The expected-output contract of a step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expected {
    /// The check does not compare this stream (`ignore` in AT_CHECK terms).
    Ignore,
    /// Exact expected bytes.
    Text(String),
}

/// A fully materialized, replayable step package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepPackage {
    /// Stable identity, path-independent: `gnucobol-testsuite-3.2/<file>/<group>/<step>`.
    pub identity: String,
    /// `stable-3.2` or `current`.
    pub lane: String,
    pub source_file: String,
    pub group_line: usize,
    pub group_title: String,
    /// 1-based step index within the group.
    pub step_index: usize,
    pub check_line: usize,
    /// Raw command from the `.at` source (with `$MACRO`s).
    pub command: String,
    /// Command with the suite macros expanded (replayable under `sh -c`).
    pub expanded_command: String,
    /// Expected exit status (`None` = any).
    pub status_expected: Option<i32>,
    pub stdout_expected: Expected,
    pub stderr_expected: Expected,
    /// Group source files in creation order: `(filename, content)`.
    pub files: Vec<(String, String)>,
    pub skip_conditions: Vec<String>,
    pub xfail_conditions: Vec<String>,
    /// `AT_CAPTURE_FILE` names (generated-file expectations).
    pub capture_files: Vec<String>,
    pub dialect: String,
    pub source_format: String,
    pub oracle: String,
    pub unknown_macros: Vec<String>,
}

/// The suite's atlocal flags: `-debug -Wall -fdiagnostics-plain-output -fno-diagnostics-show-option`.
pub const SUITE_FLAGS: &str =
    "-debug -Wall -fdiagnostics-plain-output -fno-diagnostics-show-option";

/// Expand the suite's shell-variable command macros. Unknown `$VAR`s are left as-is and the
/// caller decides (fail-closed on commands whose macros are not in the table).
pub fn expand_command(raw: &str, lane_flags: &str) -> String {
    let mut out = String::with_capacity(raw.len() + 32);
    let bytes = raw.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' {
            // read the variable name
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                j += 1;
            }
            let name = &raw[i + 1..j];
            let replacement: Option<String> = match name {
                "COBC" => Some("cobc".to_string()),
                "COBCRUN" => Some("cobcrun".to_string()),
                "COMPILE" => Some(format!("cobc -x {lane_flags}")),
                "COMPILE_ONLY" => Some(format!("cobc -fsyntax-only {lane_flags} -Wno-unsupported")),
                "COMPILE_MODULE" => Some(format!("cobc -m {lane_flags}")),
                "COMPILE_LISTING" => Some(format!(
                    "cobc -fsyntax-only {lane_flags} -Wno-unsupported -fttitle=GnuCOBOL_V.R.P -fno-ttimestamp"
                )),
                "COBCRUN_DIRECT" => Some(String::new()),
                "GREP" => Some("grep".to_string()),
                "SED" => Some("sed".to_string()),
                "AWK" => Some("awk".to_string()),
                "COB_EXE_EXT" => Some(String::new()),
                "COB_OBJECT_EXT" => Some(".o".to_string()),
                "PATHSEP" => Some(":".to_string()),
                "COB_HAS_CURSES" => Some("no".to_string()),
                "COB_HAS_ISAM" => Some("no".to_string()),
                "COB_ENV" => Some(String::new()),
                "" => {
                    // a bare `$` at end of input: keep it
                    out.push('$');
                    i += 1;
                    continue;
                }
                _ => {
                    // unknown variable: keep verbatim (the caller may still run it; the
                    // step is flagged when the command cannot be expanded safely)
                    out.push_str(&raw[i..j]);
                    i = j;
                    continue;
                }
            };
            match replacement {
                Some(rep) => out.push_str(&rep),
                None => out.push_str(&raw[i..j]),
            }
            i = j;
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

/// Guess the source format of the group's main file from its bytes. Evidence-based and honest:
/// a col-7 indicator or a digit-filled sequence area is fixed format; code starting at column 1
/// is free format (cobc 3.2 auto-detects it); a 7-space-indented source is what cobc's default
/// treats as fixed. The replay always uses cobc's own default, exactly like the suite, so the
/// guess never changes the compile profile.
pub fn guess_source_format(content: &str) -> String {
    let first = content.lines().next().unwrap_or("");
    let chars: Vec<char> = first.chars().collect();
    if chars.len() >= 7 {
        let indicator = chars[6];
        if matches!(indicator, '*' | '/' | 'D' | '$') {
            return "fixed".to_string();
        }
        if chars[..6].iter().any(|c| c.is_ascii_digit()) {
            return "fixed".to_string();
        }
    }
    // code starting in column 1 (no indent): free format
    if first.trim_start().is_empty() || first.len() == first.trim_start().len() {
        "free".to_string()
    } else {
        // 7-space-indented: cobc's default (fixed) treatment
        "fixed".to_string()
    }
}

impl StepPackage {
    /// The oracle-contract validity decision (spec 2.3): a step is a valid path when the
    /// upstream contract expects a successful compile or run (expected status 0); a valid
    /// compile-with-expected-warning when status 0 and the expected stderr mentions warnings.
    pub fn contract_class(&self) -> StepClass {
        match self.status_expected {
            None => StepClass::ContractAnyStatus,
            Some(0) => {
                if matches!(&self.stderr_expected, Expected::Text(t) if t.to_lowercase().contains("warning"))
                {
                    StepClass::ValidWithExpectedWarning
                } else {
                    StepClass::Valid
                }
            }
            Some(_) => StepClass::InvalidExpectedReject,
        }
    }

    /// Whether the expanded command is a compile-only / compile / run shape (used to split
    /// `VALID_EXECUTABLE` vs `VALID_COMPILE_ONLY` from the contract alone, before replay).
    pub fn command_shape(&self) -> CommandShape {
        let c = self.expanded_command.trim();
        if c.starts_with("cobc -fsyntax-only") {
            CommandShape::CompileOnly
        } else if c.starts_with("cobc -x") || c.starts_with("cobc -m") {
            CommandShape::Compile
        } else if c.starts_with("cobcrun ") || c.starts_with("./") || c.starts_with(".\\") {
            CommandShape::Run
        } else if c.contains("RUN_PROG_MANUAL") {
            CommandShape::ScreenRun
        } else {
            CommandShape::Shell
        }
    }
}

/// The upstream-contract step classification (never inferred from candidate behaviour).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepClass {
    /// Expected status 0: a valid compile/run path.
    Valid,
    /// Expected status 0 with warnings in the expected stderr.
    ValidWithExpectedWarning,
    /// Expected nonzero status: the suite declares the source invalid.
    InvalidExpectedReject,
    /// The contract accepts any status.
    ContractAnyStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandShape {
    CompileOnly,
    Compile,
    Run,
    ScreenRun,
    Shell,
}

/// Build one step package for `(group, step_index)`.
pub fn build_step(
    lane: &str,
    oracle: &str,
    source_file: &str,
    group: &AtGroup,
    step_index: usize,
    group_no: usize,
) -> StepPackage {
    let check = &group.checks[step_index];
    let expanded = expand_command(&check.command, SUITE_FLAGS);
    let main_content = group
        .data_files
        .iter()
        .find(|d| d.filename == "prog.cob" || d.filename.ends_with(".cob"))
        .map(|d| d.content.as_str())
        .unwrap_or_default();
    let source_format = if main_content.is_empty() {
        "unknown".to_string()
    } else {
        guess_source_format(main_content)
    };
    StepPackage {
        identity: format!(
            "gnucobol-testsuite-{lane}/{}/group-{group_no:04}/step-{:03}",
            source_file.replace('/', "-"),
            step_index + 1
        ),
        lane: lane.to_string(),
        source_file: source_file.to_string(),
        group_line: group.line,
        group_title: group.title.clone(),
        step_index: step_index + 1,
        check_line: check.line,
        command: check.command.clone(),
        expanded_command: expanded,
        status_expected: parse_status(&check.status),
        stdout_expected: parse_expected(&check.stdout),
        stderr_expected: parse_expected(&check.stderr),
        files: group
            .data_files
            .iter()
            .map(|d| (d.filename.clone(), d.content.clone()))
            .collect(),
        skip_conditions: group.skip.clone(),
        xfail_conditions: group.xfail.clone(),
        capture_files: group.capture_files.clone(),
        dialect: "default".to_string(),
        source_format,
        oracle: oracle.to_string(),
        unknown_macros: group.unknown_macros.clone(),
    }
}

fn parse_status(s: &str) -> Option<i32> {
    match s.trim() {
        "" | "ignore" => None,
        other => other.parse::<i32>().ok(),
    }
}

/// Parse an AT_CHECK expected stream argument. BYTE-EXACT: `ignore` means the stream is not
/// checked; `[]` (empty) means the stream must be EMPTY; any other text is compared exactly
/// (including trailing newlines) -- Autotest semantics. Never trimmed.
fn parse_expected(s: &str) -> Expected {
    match s {
        "ignore" => Expected::Ignore,
        other => Expected::Text(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_suite_macros() {
        assert_eq!(
            expand_command("$COMPILE_ONLY prog.cob", SUITE_FLAGS),
            format!("cobc -fsyntax-only {SUITE_FLAGS} -Wno-unsupported prog.cob")
        );
        assert_eq!(
            expand_command("$COBCRUN_DIRECT ./prog", SUITE_FLAGS),
            " ./prog"
        );
        assert_eq!(
            expand_command("$COBCRUN prog a b", SUITE_FLAGS),
            "cobcrun prog a b"
        );
        assert_eq!(
            expand_command("COB_EXIT_WAIT=0 $COBCRUN_DIRECT ./prog", SUITE_FLAGS),
            "COB_EXIT_WAIT=0  ./prog"
        );
        // unknown vars are preserved verbatim (flagged by the caller)
        assert_eq!(expand_command("$MYSTERY x", SUITE_FLAGS), "$MYSTERY x");
    }

    #[test]
    fn contract_classes() {
        let mk = |status: &str, stderr: &str| StepPackage {
            identity: "t".into(),
            lane: "stable-3.2".into(),
            source_file: "f.at".into(),
            group_line: 1,
            group_title: "g".into(),
            step_index: 1,
            check_line: 2,
            command: "c".into(),
            expanded_command: "cobc -x prog.cob".into(),
            status_expected: parse_status(status),
            stdout_expected: Expected::Ignore,
            stderr_expected: parse_expected(stderr),
            files: vec![],
            skip_conditions: vec![],
            xfail_conditions: vec![],
            capture_files: vec![],
            dialect: "default".into(),
            source_format: "fixed".into(),
            oracle: "GnuCOBOL 3.2.0".into(),
            unknown_macros: vec![],
        };
        assert_eq!(mk("0", "").contract_class(), StepClass::Valid);
        assert_eq!(
            mk("0", "warning: something").contract_class(),
            StepClass::ValidWithExpectedWarning
        );
        assert_eq!(
            mk("1", "").contract_class(),
            StepClass::InvalidExpectedReject
        );
        assert_eq!(
            mk("ignore", "").contract_class(),
            StepClass::ContractAnyStatus
        );
    }

    #[test]
    fn command_shapes() {
        let p = StepPackage {
            identity: "t".into(),
            lane: "s".into(),
            source_file: "f".into(),
            group_line: 1,
            group_title: "g".into(),
            step_index: 1,
            check_line: 1,
            command: String::new(),
            expanded_command: "cobc -fsyntax-only -debug prog.cob".into(),
            status_expected: Some(0),
            stdout_expected: Expected::Ignore,
            stderr_expected: Expected::Ignore,
            files: vec![],
            skip_conditions: vec![],
            xfail_conditions: vec![],
            capture_files: vec![],
            dialect: "default".into(),
            source_format: "fixed".into(),
            oracle: "o".into(),
            unknown_macros: vec![],
        };
        assert_eq!(p.command_shape(), CommandShape::CompileOnly);
    }

    #[test]
    fn format_guess_fixed_vs_free() {
        let fixed = "000100 IDENTIFICATION DIVISION.\n000200 PROGRAM-ID. T.\n";
        assert_eq!(guess_source_format(fixed), "fixed");
        let fixed_comment = "000100*comment\n";
        assert_eq!(guess_source_format(fixed_comment), "fixed");
        // 7-space indent: cobc's default fixed treatment
        let indented = "       IDENTIFICATION DIVISION.\n";
        assert_eq!(guess_source_format(indented), "fixed");
        // code starting at column 1: free format (cobc auto-detects)
        let free = "IDENTIFICATION DIVISION.\nPROGRAM-ID. T.\n";
        assert_eq!(guess_source_format(free), "free");
    }
}
