//! Process execution for the court: run a command with a wall-clock timeout, capture raw
//! stdout/stderr to evidence files, record exit status / signal / duration / produced artifacts.
//!
//! The timeout is a hard wall-clock bound (the GNU `timeout` utility is used so that the child and
//! its whole process group are killed, not just the direct child). The spec requires CPU *and*
//! wall-clock bounds; CPU bounding is applied by the caller through `ulimit -t` in the run wrapper
//! (recorded in the invocation environment).

use crate::corpus::sha256_hex;
use crate::model::Invocation;
use std::path::Path;
use std::time::Instant;

/// Execute `argv` in `cwd` with `env` (KEY=VALUE pairs), writing stdout/stderr to
/// `evidence_dir/stdout` and `evidence_dir/stderr`. `timeout_secs` bounds the wall clock
/// (0 = no timeout). Returns the recorded invocation.
pub fn run_invocation(
    argv: &[String],
    cwd: &Path,
    env: &[(String, String)],
    timeout_secs: u64,
    evidence_dir: &Path,
    stdin_bytes: Option<&[u8]>,
) -> Invocation {
    let mut inv = Invocation {
        command: argv.to_vec(),
        cwd: cwd.to_string_lossy().into_owned(),
        environment: env.iter().map(|(k, v)| format!("{k}={v}")).collect(),
        ..Default::default()
    };
    std::fs::create_dir_all(evidence_dir).ok();

    let start = Instant::now();
    let status = run_with_timeout(argv, cwd, env, timeout_secs, evidence_dir, stdin_bytes);
    inv.duration_ms = start.elapsed().as_millis() as u64;

    let stdout_path = evidence_dir.join("stdout");
    let stderr_path = evidence_dir.join("stderr");
    inv.stdout_sha256 = read_sha(&stdout_path);
    inv.stderr_sha256 = read_sha(&stderr_path);
    if stdout_path.exists() {
        inv.stdout_path = Some(stdout_path.to_string_lossy().into_owned());
    }
    if stderr_path.exists() {
        inv.stderr_path = Some(stderr_path.to_string_lossy().into_owned());
    }

    match status {
        Ok(code) => {
            inv.exit_code = Some(code);
            if code == 124 {
                inv.timed_out = true;
            }
        }
        Err(e) => {
            inv.error = Some(e.to_string());
        }
    }
    inv
}

/// Hash the contents of a file ("" when absent).
pub fn read_sha(path: &Path) -> String {
    match std::fs::read(path) {
        Ok(b) => sha256_hex(&b),
        Err(_) => String::new(),
    }
}

/// Run with a wall-clock timeout using the GNU `timeout` utility, capturing stdout/stderr to
/// files. `timeout` exit status: 124 = timed out; 125 = timeout itself failed; 126/127 = cannot
/// run. A killed-by-signal child reports 128+signum.
fn run_with_timeout(
    argv: &[String],
    cwd: &Path,
    env: &[(String, String)],
    timeout_secs: u64,
    evidence_dir: &Path,
    stdin_bytes: Option<&[u8]>,
) -> Result<i32, std::io::Error> {
    let stdout_f = std::fs::File::create(evidence_dir.join("stdout"))?;
    let stderr_f = std::fs::File::create(evidence_dir.join("stderr"))?;

    // Wrap with the GNU `timeout` utility so the whole process group is bounded (kill-after 5s
    // for stragglers). The recorded `Invocation.command` stays the real argv; the wrapper is
    // internal to the harness.
    let mut final_cmd = if timeout_secs > 0 {
        let mut c = std::process::Command::new("timeout");
        c.arg("-k").arg("5").arg(timeout_secs.to_string());
        c.args(argv);
        c
    } else {
        let mut c = std::process::Command::new(&argv[0]);
        c.args(&argv[1..]);
        c
    };

    final_cmd.current_dir(cwd);
    final_cmd.env_clear();
    for (k, v) in env {
        final_cmd.env(k, v);
    }
    final_cmd.stdout(stdout_f);
    final_cmd.stderr(stderr_f);
    if let Some(_bytes) = stdin_bytes {
        final_cmd.stdin(std::process::Stdio::piped());
    } else {
        final_cmd.stdin(std::process::Stdio::null());
    }

    let mut child = final_cmd.spawn()?;
    if let Some(bytes) = stdin_bytes {
        use std::io::Write;
        if let Some(mut si) = child.stdin.take() {
            let _ = si.write_all(bytes);
        }
    }
    let status = child.wait()?;
    Ok(status.code().unwrap_or_else(|| {
        // killed by signal: report 128 + signal (the shell convention)
        use std::os::unix::process::ExitStatusExt;
        let sig = status.signal().unwrap_or(0);
        128 + sig
    }))
}

/// Read a file's bytes (for comparisons).
pub fn read_bytes(path: &Path) -> Vec<u8> {
    std::fs::read(path).unwrap_or_default()
}

/// Read a file's contents lossy (for diagnostics / reason extraction).
pub fn read_lossy(path: &Path) -> String {
    String::from_utf8_lossy(&read_bytes(path)).into_owned()
}

/// The first non-empty line of a file (for failure bucketing).
pub fn first_line(path: &Path) -> String {
    read_lossy(path)
        .lines()
        .map(|l| l.trim_end().to_string())
        .find(|l| !l.trim().is_empty())
        .unwrap_or_default()
}
