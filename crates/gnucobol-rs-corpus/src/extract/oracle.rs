//! Host-oracle replay of suite step packages (pinned environment).
//!
//! Runs a step's expanded command under `sh -c` in a scratch directory containing the group's
//! source files, with the pinned host-oracle environment (PATH, LD_LIBRARY_PATH,
//! COB_CONFIG_DIR, locale, TZ). The oracle is the admitted host GnuCOBOL 3.2.0 build under
//! `lab/oracle/prefix`; its identity and hash are recorded in every report.

use crate::extract::package::StepPackage;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The resolved host oracle (GnuCOBOL 3.2.0 built under `lab/oracle/prefix`).
#[derive(Debug, Clone)]
pub struct OracleEnv {
    pub label: String,
    pub prefix: PathBuf,
    pub cobc: PathBuf,
    pub cobcrun: PathBuf,
    pub ld_library_path: PathBuf,
    pub config_dir: PathBuf,
}

impl OracleEnv {
    /// Resolve the host oracle relative to the workspace root (two levels above this crate).
    pub fn host_default() -> Result<OracleEnv, String> {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root = manifest
            .ancestors()
            .nth(2)
            .ok_or_else(|| "cannot resolve workspace root".to_string())?;
        let prefix = root.join("lab").join("oracle").join("prefix");
        let cobc = prefix.join("bin").join("cobc");
        if !cobc.exists() {
            return Err(format!(
                "host oracle not built ({}); run the oracle build first",
                cobc.display()
            ));
        }
        Ok(OracleEnv {
            label: "GnuCOBOL 3.2.0 (host lab/oracle/prefix)".to_string(),
            prefix: prefix.clone(),
            cobc,
            cobcrun: prefix.join("bin").join("cobcrun"),
            ld_library_path: prefix.join("lib"),
            config_dir: prefix.join("share").join("gnucobol").join("config"),
        })
    }

    /// The pinned environment for oracle child processes.
    pub fn env(&self) -> Vec<(String, String)> {
        let path = format!(
            "{}:{}",
            self.prefix.join("bin").display(),
            std::env::var("PATH").unwrap_or_default()
        );
        vec![
            ("PATH".to_string(), path),
            (
                "LD_LIBRARY_PATH".to_string(),
                self.ld_library_path.display().to_string(),
            ),
            (
                "COB_CONFIG_DIR".to_string(),
                self.config_dir.display().to_string(),
            ),
            ("COB_HAS_CURSES".to_string(), "no".to_string()),
            ("COB_HAS_ISAM".to_string(), "no".to_string()),
            ("LC_ALL".to_string(), "C".to_string()),
            ("LANG".to_string(), "C".to_string()),
            ("TZ".to_string(), "UTC0".to_string()),
        ]
    }
}

/// The raw outcome of one executed step.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct StepOutcome {
    pub exit: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    /// Set when the command could not be executed (e.g. unsupported macro left in the command).
    pub exec_error: Option<String>,
    /// Set when the step was skipped (evaluated skip/xfail condition).
    pub skipped: bool,
    pub skip_reason: String,
}

/// Run one step's expanded command in `workdir` with the oracle env. `extra_env` carries
/// group-context variables (e.g. `COB_HAS_CURSES=no`) and per-command prefixes already applied.
pub fn run_step(
    oracle: &OracleEnv,
    workdir: &Path,
    expanded_command: &str,
    extra_env: &[(String, String)],
) -> StepOutcome {
    // Fail-closed: a command that still contains an unexpanded `$VAR` outside our table cannot
    // be replayed faithfully -- record it instead of running something we do not understand.
    if expanded_command.contains('$') && !command_is_shell_safe(expanded_command) {
        return StepOutcome {
            exit: None,
            stdout: Vec::new(),
            stderr: Vec::new(),
            exec_error: Some(format!(
                "unexpanded variable(s) in command: {expanded_command}"
            )),
            skipped: false,
            skip_reason: String::new(),
        };
    }
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(expanded_command);
    cmd.current_dir(workdir);
    cmd.env_clear();
    for (k, v) in oracle.env() {
        cmd.env(k, v);
    }
    for (k, v) in extra_env {
        cmd.env(k, v);
    }
    // Hard cap: no step may hang the replay (spec: no input hangs the candidate; the oracle
    // must be bounded too). `timeout` is coreutils on this host.
    let mut wrapped = Command::new("timeout");
    wrapped
        .arg("120")
        .arg(cmd.get_program())
        .args(cmd.get_args())
        .current_dir(workdir)
        .env_clear();
    for (k, v) in cmd.get_envs() {
        if let Some(v) = v {
            wrapped.env(k, v);
        }
    }
    let out = wrapped.output();
    match out {
        Ok(o) => StepOutcome {
            exit: o.status.code(),
            stdout: o.stdout,
            stderr: o.stderr,
            exec_error: None,
            skipped: false,
            skip_reason: String::new(),
        },
        Err(e) => StepOutcome {
            exit: None,
            stdout: Vec::new(),
            stderr: Vec::new(),
            exec_error: Some(format!("cannot execute: {e}")),
            skipped: false,
            skip_reason: String::new(),
        },
    }
}

/// Evaluate a skip/xfail condition (`test "$COB_HAS_CURSES" != "yes"`) against the oracle env.
pub fn condition_holds(oracle: &OracleEnv, condition: &str) -> Result<bool, String> {
    let out = Command::new("sh")
        .arg("-c")
        .arg(condition)
        .env_clear()
        .envs(oracle.env())
        .output()
        .map_err(|e| e.to_string())?;
    Ok(out.status.success())
}

/// Shell constructs we understand in suite commands (redirections, pipelines, `&&`/`||` chains,
/// env prefixes) do not require macro expansion; a leftover `$` in any other position is unsafe.
fn command_is_shell_safe(cmd: &str) -> bool {
    // `$` inside single quotes is literal shell text (e.g. awk programs) -- those are safe.
    let mut in_single = false;
    for c in cmd.chars() {
        match c {
            '\'' => in_single = !in_single,
            '$' if !in_single => return false,
            _ => {}
        }
    }
    true
}

/// Compare an actual outcome against a step's contract. Returns the verdict list of mismatches
/// (empty = the step replayed exactly as the oracle contract declares).
///
/// Autotest conventions: exit code 77 marks a runtime-skip (`test ... || exit 77`) -- the step
/// is skipped, never a mismatch.
pub fn compare_contract(pkg: &StepPackage, outcome: &StepOutcome) -> Vec<String> {
    let mut mismatches = Vec::new();
    if let Some(err) = &outcome.exec_error {
        mismatches.push(format!("exec error: {err}"));
        return mismatches;
    }
    if outcome.exit == Some(77) {
        // runtime skip (Autotest convention): not a failure, not a mismatch
        return mismatches;
    }
    if let Some(expected) = pkg.status_expected {
        let actual = outcome.exit.unwrap_or(-1);
        if actual != expected {
            mismatches.push(format!(
                "exit: expected {expected}, got {actual} (stderr: {})",
                String::from_utf8_lossy(&outcome.stderr).trim_end()
            ));
        }
    }
    if let crate::extract::package::Expected::Text(expected) = &pkg.stdout_expected {
        if outcome.stdout != expected.as_bytes() {
            mismatches.push(format!(
                "stdout mismatch: expected {} bytes, got {} bytes",
                expected.len(),
                outcome.stdout.len()
            ));
        }
    }
    if let crate::extract::package::Expected::Text(expected) = &pkg.stderr_expected {
        if outcome.stderr != expected.as_bytes() {
            mismatches.push(format!(
                "stderr mismatch: expected {} bytes, got {} bytes",
                expected.len(),
                outcome.stderr.len()
            ));
        }
    }
    mismatches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_safe_detects_leftover_vars() {
        assert!(!command_is_shell_safe("$MYSTERY x"));
        assert!(command_is_shell_safe("awk '{print $1}' f"));
        assert!(command_is_shell_safe("./prog"));
    }

    #[test]
    fn contract_compare() {
        let pkg = StepPackage {
            identity: "t".into(),
            lane: "s".into(),
            source_file: "f".into(),
            group_line: 1,
            group_title: "g".into(),
            step_index: 1,
            check_line: 1,
            command: "c".into(),
            expanded_command: "c".into(),
            status_expected: Some(0),
            stdout_expected: crate::extract::package::Expected::Text("OK\n".into()),
            stderr_expected: crate::extract::package::Expected::Ignore,
            files: vec![],
            skip_conditions: vec![],
            xfail_conditions: vec![],
            capture_files: vec![],
            dialect: "default".into(),
            source_format: "fixed".into(),
            oracle: "o".into(),
            unknown_macros: vec![],
        };
        let ok = StepOutcome {
            exit: Some(0),
            stdout: b"OK\n".to_vec(),
            stderr: Vec::new(),
            exec_error: None,
            skipped: false,
            skip_reason: String::new(),
        };
        assert!(compare_contract(&pkg, &ok).is_empty());
        let bad = StepOutcome {
            exit: Some(1),
            stdout: b"NO\n".to_vec(),
            stderr: Vec::new(),
            exec_error: None,
            skipped: false,
            skip_reason: String::new(),
        };
        let m = compare_contract(&pkg, &bad);
        assert_eq!(m.len(), 2, "{m:?}");
    }
}
