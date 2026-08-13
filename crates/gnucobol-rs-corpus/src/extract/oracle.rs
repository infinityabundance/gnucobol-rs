//! Host-oracle replay of suite step packages (pinned environment).
//!
//! Runs a step's expanded command under `sh -c` in a scratch directory containing the group's
//! source files, with the pinned host-oracle environment (PATH, LD_LIBRARY_PATH,
//! COB_CONFIG_DIR, locale, TZ). The oracle is the admitted GnuCOBOL 3.2.0 build; its identity
//! and hash are recorded in every report. The prefix resolves from `GNURUST_ORACLE_PREFIX`
//! (used by the isolated court containers, where the oracle lives on a bind mount built in the
//! container's toolchain image) and falls back to `lab/oracle/prefix` (the host-side build).

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
    /// Resolve the oracle prefix: `GNURUST_ORACLE_PREFIX` when set, else the workspace's
    /// `lab/oracle/prefix`.
    pub fn resolve_prefix(root: &Path) -> PathBuf {
        match std::env::var("GNURUST_ORACLE_PREFIX") {
            Ok(p) if !p.trim().is_empty() => PathBuf::from(p),
            _ => root.join("lab").join("oracle").join("prefix"),
        }
    }

    /// Resolve the host oracle relative to the workspace root (two levels above this crate).
    pub fn host_default() -> Result<OracleEnv, String> {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root = manifest
            .ancestors()
            .nth(2)
            .ok_or_else(|| "cannot resolve workspace root".to_string())?;
        let prefix = Self::resolve_prefix(&root);
        let cobc = prefix.join("bin").join("cobc");
        if !cobc.exists() {
            return Err(format!(
                "oracle not built ({}); set GNURUST_ORACLE_PREFIX or run the oracle build first",
                cobc.display()
            ));
        }
        Ok(OracleEnv {
            // The label is a stable identity (never the machine-specific prefix path: the
            // reports carry it in every row and must be reproducible across machines).
            label: "GnuCOBOL 3.2.0".to_string(),
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
    /// True when the recorded outcome is a RETRY after a timing-artifact first attempt
    /// (replay-timeout kill, or the compiler driver aborting on a signal such as SIGPIPE when a
    /// pipeline reader closed early). Deterministic outcomes are never retried, and the retry
    /// outcome is still compared byte-for-byte against the contract.
    pub retried: bool,
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
            retried: false,
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
    // Bounded capture: a runaway step may otherwise balloon into gigabytes of RAM and, worse,
    // make the recorded mismatch bytes nondeterministic (the TOTAL volume of a runaway loop
    // depends on kill timing while its LEADING bytes are stable). Capturing the first 16 MiB of
    // each stream bounds memory AND makes the recorded evidence reproducible: the child is left
    // to block on the full pipe, so the 120s timeout still kills it and the exit status is
    // unchanged.
    let cap = 16 * 1024 * 1024;
    let mut attempt = run_bounded(&mut wrapped, cap);
    let mut retried = false;
    if let Ok(o) = &attempt {
        if is_timing_artifact(o) {
            // A first attempt killed by the 120s replay wall clock (a step near the boundary
            // flips between 124 and completion), or whose stderr shows the compiler driver
            // aborting on a signal (the upstream `$COBC --help | head` step races on whether
            // the reader closes the pipe before the driver's final write, and a SIGPIPE abort
            // writes `cobc: aborting` to stderr), is a timing artifact, not a stable property
            // of the oracle: a single replay would flip the classification between runs. Retry
            // once; the retry outcome is still compared byte-for-byte against the contract (no
            // semantic check is weakened). Deterministic outcomes are never retried.
            retried = true;
            attempt = run_bounded(&mut wrapped, cap);
        }
    }
    match attempt {
        Ok(o) => StepOutcome {
            exit: o.status.code(),
            stdout: o.stdout,
            stderr: o.stderr,
            exec_error: None,
            skipped: false,
            skip_reason: String::new(),
            retried,
        },
        Err(e) => StepOutcome {
            exit: None,
            stdout: Vec::new(),
            stderr: Vec::new(),
            exec_error: Some(format!("cannot execute: {e}")),
            skipped: false,
            skip_reason: String::new(),
            retried,
        },
    }
}

/// Whether a first replay attempt is a timing artifact rather than a stable oracle property.
///
/// Two shapes are retried:
/// - `exit 124` — the replay wrapper's 120s wall clock killed the step; a step near the
///   boundary flips between timeout and completion depending on host load;
/// - stderr carrying the compiler/runtime driver's signal-abort line (`cobc: aborting` /
///   `libcob: aborting`) — emitted when the driver aborts on a signal such as SIGPIPE after a
///   pipeline reader (`head`) closed the pipe early. Whether the abort races depends on write
///   vs. read timing, so a single replay would classify the step differently across runs.
///
/// Everything else is deterministic and never retried.
fn is_timing_artifact(o: &std::process::Output) -> bool {
    if o.status.code() == Some(124) {
        return true;
    }
    let stderr = String::from_utf8_lossy(&o.stderr);
    stderr.lines().any(|l| l.trim_end().ends_with(": aborting"))
}

/// Spawn `cmd`, capture at most `cap` bytes of stdout + stderr each, then wait. One reader
/// thread per pipe (like `std::process::output`, avoiding lockstep throttling); each reader
/// keeps the child's pipe drained after its cap so the child runs at full speed and is killed
/// by the wrapper `timeout` exactly as an unbounded capture would (same exit status). The
/// leading `cap` bytes of a deterministic program are stable, so the captured evidence is
/// reproducible even for runaway steps whose TOTAL output volume is timing-dependent.
fn run_bounded(cmd: &mut Command, cap: usize) -> std::io::Result<std::process::Output> {
    use std::process::Stdio;
    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    let so = child.stdout.take().expect("stdout piped");
    let se = child.stderr.take().expect("stderr piped");
    let h1 = std::thread::spawn(move || read_capped(so, cap));
    let h2 = std::thread::spawn(move || read_capped(se, cap));
    let status = child.wait()?;
    let stdout = h1
        .join()
        .unwrap_or_else(|_| Ok(Vec::new()))
        .unwrap_or_default();
    let stderr = h2
        .join()
        .unwrap_or_else(|_| Ok(Vec::new()))
        .unwrap_or_default();
    Ok(std::process::Output {
        status,
        stdout,
        stderr,
    })
}

/// Read up to `cap` bytes, then keep draining (discarding) until EOF so a runaway writer is
/// never blocked by our capture and the wrapper timeout's kill semantics are preserved.
fn read_capped(mut r: impl std::io::Read, cap: usize) -> std::io::Result<Vec<u8>> {
    let mut out = Vec::with_capacity(cap.min(1 << 20));
    let mut buf = [0u8; 65536];
    loop {
        let n = r.read(&mut buf)?;
        if n == 0 {
            break;
        }
        if out.len() < cap {
            let take = (cap - out.len()).min(n);
            out.extend_from_slice(&buf[..take]);
        }
    }
    Ok(out)
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
    // A timeout kill (coreutils `timeout` -> 124) is a timing-dependent replay: the stderr the
    // killed program manages to flush before the pipe closes is a RACE (captured or not, byte
    // count varies), so the mismatch text must not embed it -- it would make the report
    // unreproducible. Record the deterministic facts (exit value, capped capture size) only.
    let killed_by_timeout = outcome.exit == Some(124);
    if let Some(expected) = pkg.status_expected {
        let actual = outcome.exit.unwrap_or(-1);
        if actual != expected {
            if killed_by_timeout {
                mismatches.push(format!(
                    "exit: expected {expected}, got 124 (killed by the 120s replay timeout)"
                ));
            } else {
                mismatches.push(format!(
                    "exit: expected {expected}, got {actual} (stderr: {})",
                    String::from_utf8_lossy(&outcome.stderr).trim_end()
                ));
            }
        }
    }
    if let crate::extract::package::Expected::Text(expected) = &pkg.stdout_expected {
        if killed_by_timeout {
            // a killed program's captured prefix is timing-dependent; the byte comparison
            // itself (even a transient match) is unreliable, so record the contract fact
            // deterministically and never a racy count.
            mismatches.push(format!(
                "stdout mismatch: expected {} bytes (the program ran until the replay timeout; \
                 the captured prefix is timing-dependent)",
                expected.len()
            ));
        } else if outcome.stdout != expected.as_bytes() {
            mismatches.push(format!(
                "stdout mismatch: expected {} bytes, got {} bytes",
                expected.len(),
                outcome.stdout.len()
            ));
        }
    }
    if let crate::extract::package::Expected::Text(expected) = &pkg.stderr_expected {
        if killed_by_timeout {
            // the stderr a killed program flushes before the pipe closes is a race; never emit
            // a count for it (the exit note already records the kill).
            mismatches.push(format!(
                "stderr mismatch: expected {} bytes (capture unreliable after a timeout kill)",
                expected.len()
            ));
        } else if outcome.stderr != expected.as_bytes() {
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
            retried: false,
        };
        assert!(compare_contract(&pkg, &ok).is_empty());
        let bad = StepOutcome {
            exit: Some(1),
            stdout: b"NO\n".to_vec(),
            stderr: Vec::new(),
            exec_error: None,
            skipped: false,
            skip_reason: String::new(),
            retried: false,
        };
        let m = compare_contract(&pkg, &bad);
        assert_eq!(m.len(), 2, "{m:?}");
    }

    #[test]
    fn timing_artifact_detection() {
        use std::os::unix::process::ExitStatusExt;
        // timeout kill
        let t = std::process::Output {
            status: std::process::ExitStatus::from_raw(124 << 8),
            stdout: Vec::new(),
            stderr: Vec::new(),
        };
        assert!(is_timing_artifact(&t));
        // SIGPIPE-abort stderr (the exact bytes the upstream `$COBC --help | head` step emits
        // when the reader closes the pipe before the driver's final write)
        let s = std::process::Output {
            status: std::process::ExitStatus::from_raw(0 << 8),
            stdout: Vec::new(),
            stderr: b"\nunknown (signal)\n\ncobc: aborting\n".to_vec(),
        };
        assert!(is_timing_artifact(&s));
        // a runtime abort line is the same class
        let r = std::process::Output {
            status: std::process::ExitStatus::from_raw(0 << 8),
            stdout: Vec::new(),
            stderr: b"libcob: aborting\n".to_vec(),
        };
        assert!(is_timing_artifact(&r));
        // clean and ordinary-diagnostic outcomes are never retried
        let c = std::process::Output {
            status: std::process::ExitStatus::from_raw(0 << 8),
            stdout: Vec::new(),
            stderr: Vec::new(),
        };
        assert!(!is_timing_artifact(&c));
        let d = std::process::Output {
            status: std::process::ExitStatus::from_raw(1 << 8),
            stdout: Vec::new(),
            stderr: b"prog.cob:10: error: syntax error\n".to_vec(),
        };
        assert!(!is_timing_artifact(&d));
    }

    #[test]
    fn run_step_retries_a_signal_abort_once() {
        // A `cobc: aborting`-shaped first attempt (the SIGPIPE-race signature) must be re-run
        // once and the retry outcome recorded with `retried = true`. The retry outcome is still
        // the raw bytes of the second attempt -- no semantic check is weakened.
        let oracle = OracleEnv {
            label: "test".into(),
            prefix: PathBuf::from("/nonexistent-prefix"),
            cobc: PathBuf::from("/nonexistent-prefix/bin/cobc"),
            cobcrun: PathBuf::from("/nonexistent-prefix/bin/cobcrun"),
            ld_library_path: PathBuf::from("/nonexistent-prefix/lib"),
            config_dir: PathBuf::from("/nonexistent-prefix/share/gnucobol/config"),
        };
        let out = run_step(&oracle, Path::new("/"), "echo 'x: aborting' >&2", &[]);
        assert!(
            out.retried,
            "signal-abort-shaped first attempt must be retried"
        );
        assert_eq!(out.exit, Some(0));
        assert_eq!(out.stderr, b"x: aborting\n");
    }

    #[test]
    fn run_step_does_not_retry_clean_outcomes() {
        let oracle = OracleEnv {
            label: "test".into(),
            prefix: PathBuf::from("/nonexistent-prefix"),
            cobc: PathBuf::from("/nonexistent-prefix/bin/cobc"),
            cobcrun: PathBuf::from("/nonexistent-prefix/bin/cobcrun"),
            ld_library_path: PathBuf::from("/nonexistent-prefix/lib"),
            config_dir: PathBuf::from("/nonexistent-prefix/share/gnucobol/config"),
        };
        let out = run_step(&oracle, Path::new("/"), "echo clean", &[]);
        assert!(!out.retried);
        assert_eq!(out.exit, Some(0));
        assert_eq!(out.stdout, b"clean\n");
    }
}
