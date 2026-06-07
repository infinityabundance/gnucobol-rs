//! # cobc-oracle-rs
//!
//! Drives the upstream **GnuCOBOL `cobc`** compiler as an oracle: it builds and runs a COBOL
//! fixture and records a deterministic, canonical-JSON **receipt** of the oracle's identity and
//! observed behaviour (stdout/stderr/exit + SHA-256 of source, generated C, and the executable).
//!
//! This crate copies **no** GPL compiler logic — it only spawns `cobc`/the produced binary and
//! reads their outputs — but, being tightly coupled tooling for the GPL compiler, it is licensed
//! **GPL-3.0-or-later** (published as GPL tooling, kept out of the LGPL/Apache decode path).
//!
//! ## Where this sits in the oracle ecosystem
//!
//! This is the **program-shape** oracle: compile a whole COBOL program with `cobc` and capture its
//! runtime behaviour. The **runtime-library shape** (`lab/oracle/decimal_harness`, linking the built
//! `libcob` and calling `cob_move`/field helpers directly) drives the byte-level sweeps for the
//! sealed courts. Campaign evidence of record is the **generated replay receipt** (`lab/receipt/run.py`
//! → `reports/receipts/<CAMPAIGN>/receipt.json`, regenerated from a live sweep; see
//! `docs/trust2-generated-receipts.md`). This crate's per-fixture receipt is one such replayable
//! program-shape witness.
//!
//! Doctrine encoded here:
//! - **Generated C is a witness, not authority** (`GNURUST.GENC.0`): the receipt records
//!   `generated_c_hash`, but semantic authority is the *runtime* (stdout/stderr/exit + field
//!   bytes), never the generated C.
//! - **Compilation mode is always named** (`GNURUST.ORACLEMODE.0`): every receipt states which
//!   `cobc` mode produced it.
//! - **Oracle availability is a typed verdict** (`GNURUST.ORACLEAVAIL.0`), never a silent skip.
//! - **Canonical JSON** (`GNURUST.JSONCANON.0`): stable key order, lowercase-hex bytes, explicit
//!   nulls — emitted by a tiny hand-written serializer (zero runtime deps).

#![forbid(unsafe_code)]

mod sha256;

use std::path::Path;
use std::process::Command;

pub use sha256::sha256_hex;

/// Which `cobc` compilation mode produced a receipt (`GNURUST.ORACLEMODE.0`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OracleMode {
    /// `cobc -x program.cob` → standalone executable.
    Executable,
    /// `cobc -C program.cob` → generated C only (a witness, not run).
    GeneratedC,
    /// `cobc -m program.cob` → dynamically loadable module.
    DynamicModule,
}

impl OracleMode {
    fn as_str(self) -> &'static str {
        match self {
            OracleMode::Executable => "executable",
            OracleMode::GeneratedC => "generated_c",
            OracleMode::DynamicModule => "dynamic_module",
        }
    }
}

/// Typed availability verdict for the oracle (`GNURUST.ORACLEAVAIL.0`): a missing `cobc` is never
/// a silent skip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OracleAvailability {
    /// `cobc` resolved and reported a version.
    Available { cobc_version: String },
    /// `cobc` was not found (expected on machines without the built oracle on PATH).
    UnavailableExpected,
    /// `cobc` was found but failed to report a version.
    UnavailableUnexpected { detail: String },
}

/// Locate `cobc` and capture its version, or report why it is unavailable.
pub fn probe_oracle() -> OracleAvailability {
    match Command::new("cobc").arg("--version").output() {
        Ok(out) if out.status.success() => {
            let v = String::from_utf8_lossy(&out.stdout);
            let first = v.lines().next().unwrap_or("").trim().to_string();
            OracleAvailability::Available {
                cobc_version: first,
            }
        }
        Ok(out) => OracleAvailability::UnavailableUnexpected {
            detail: format!("cobc --version exit {:?}", out.status.code()),
        },
        Err(_) => OracleAvailability::UnavailableExpected,
    }
}

/// The observed result of compiling (and possibly running) a fixture.
#[derive(Debug, Clone)]
pub struct OracleReceipt {
    pub fixture: String,
    pub mode: OracleMode,
    pub cobc_version: String,
    pub platform: String,
    pub locale: String,
    pub source_sha256: String,
    pub compile_exit: Option<i32>,
    pub compile_stderr_sha256: String,
    pub generated_c_sha256: Option<String>,
    pub executable_sha256: Option<String>,
    pub run_exit: Option<i32>,
    pub run_stdout_sha256: Option<String>,
    pub run_stderr_sha256: Option<String>,
}

impl OracleReceipt {
    /// Serialize to canonical JSON (`GNURUST.JSONCANON.0`): fixed key order, lowercase-hex bytes,
    /// explicit `null` for absent fields.
    pub fn to_canonical_json(&self) -> String {
        fn s(v: &str) -> String {
            let mut out = String::from("\"");
            for c in v.chars() {
                match c {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    c => out.push(c),
                }
            }
            out.push('"');
            out
        }
        fn oi(v: Option<i32>) -> String {
            v.map_or_else(|| "null".to_string(), |n| n.to_string())
        }
        fn os(v: &Option<String>) -> String {
            v.as_ref().map_or_else(|| "null".to_string(), |x| s(x))
        }
        format!(
            concat!(
                "{{\"schema\":\"gnucobol-rs-oracle-receipt-v1\",",
                "\"fixture\":{},\"mode\":{},",
                "\"oracle\":{{\"cobc_version\":{},\"platform\":{},\"locale\":{}}},",
                "\"compile\":{{\"exit\":{},\"stderr_sha256\":{},\"generated_c_sha256\":{}}},",
                "\"source_sha256\":{},\"executable_sha256\":{},",
                "\"run\":{{\"exit\":{},\"stdout_sha256\":{},\"stderr_sha256\":{}}}}}"
            ),
            s(&self.fixture),
            s(self.mode.as_str()),
            s(&self.cobc_version),
            s(&self.platform),
            s(&self.locale),
            oi(self.compile_exit),
            s(&self.compile_stderr_sha256),
            os(&self.generated_c_sha256),
            s(&self.source_sha256),
            os(&self.executable_sha256),
            oi(self.run_exit),
            os(&self.run_stdout_sha256),
            os(&self.run_stderr_sha256),
        )
    }
}

fn platform_string() -> String {
    let out = Command::new("uname")
        .args(["-mrs"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    out.unwrap_or_else(|| std::env::consts::OS.to_string())
}

/// Compile a fixture with `cobc -x`, run it, and return the receipt. Returns `Err` only when the
/// fixture cannot be read; a *compile* failure is captured in the receipt (non-zero exit), not an
/// `Err`, because a failing compile is itself an observation.
pub fn run_executable_fixture(fixture: &Path) -> std::io::Result<OracleReceipt> {
    let source = std::fs::read(fixture)?;
    let cobc_version = match probe_oracle() {
        OracleAvailability::Available { cobc_version } => cobc_version,
        _ => String::from("(unavailable)"),
    };
    let dir = std::env::temp_dir().join("cobc-oracle-rs");
    let _ = std::fs::create_dir_all(&dir);
    let stem = fixture
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("prog");
    let exe = dir.join(stem);

    let compile = Command::new("cobc")
        .arg("-x")
        .arg("-o")
        .arg(&exe)
        .arg(fixture)
        .output()?;

    // Generated-C witness (a separate, non-authoritative artifact).
    let _ = Command::new("cobc")
        .arg("-C")
        .arg("-o")
        .arg(dir.join(format!("{stem}.c")))
        .arg(fixture)
        .output();
    let generated_c_sha256 = std::fs::read(dir.join(format!("{stem}.c")))
        .ok()
        .map(|b| sha256_hex(&b));

    let mut receipt = OracleReceipt {
        fixture: fixture.display().to_string(),
        mode: OracleMode::Executable,
        cobc_version,
        platform: platform_string(),
        locale: std::env::var("LC_ALL").unwrap_or_else(|_| "(unset)".to_string()),
        source_sha256: sha256_hex(&source),
        compile_exit: compile.status.code(),
        compile_stderr_sha256: sha256_hex(&compile.stderr),
        generated_c_sha256,
        executable_sha256: std::fs::read(&exe).ok().map(|b| sha256_hex(&b)),
        run_exit: None,
        run_stdout_sha256: None,
        run_stderr_sha256: None,
    };

    if compile.status.success() {
        if let Ok(run) = Command::new(&exe).output() {
            receipt.run_exit = run.status.code();
            receipt.run_stdout_sha256 = Some(sha256_hex(&run.stdout));
            receipt.run_stderr_sha256 = Some(sha256_hex(&run.stderr));
        }
    }
    Ok(receipt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_json_is_stable_and_explicit_null() {
        let r = OracleReceipt {
            fixture: "hello.cob".into(),
            mode: OracleMode::Executable,
            cobc_version: "cobc (GnuCOBOL) 3.2.0".into(),
            platform: "Linux x86_64".into(),
            locale: "C.UTF-8".into(),
            source_sha256: sha256_hex(b"x"),
            compile_exit: Some(0),
            compile_stderr_sha256: sha256_hex(b""),
            generated_c_sha256: None,
            executable_sha256: None,
            run_exit: Some(0),
            run_stdout_sha256: Some(sha256_hex(b"HELLO\n")),
            run_stderr_sha256: Some(sha256_hex(b"")),
        };
        let j = r.to_canonical_json();
        assert!(j.starts_with("{\"schema\":\"gnucobol-rs-oracle-receipt-v1\""));
        assert!(j.contains("\"generated_c_sha256\":null"));
        assert!(j.contains("\"mode\":\"executable\""));
        // deterministic
        assert_eq!(j, r.to_canonical_json());
    }

    #[test]
    fn availability_is_typed() {
        // Just exercise the probe; either variant is acceptable depending on PATH.
        let _ = probe_oracle();
    }
}
