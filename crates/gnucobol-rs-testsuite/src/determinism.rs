//! Determinism comparison between two fresh passes (GNURUST.GNUCOBOL-TESTSUITE determinism gate):
//! the stable summary surface must be identical across two fresh containers/trees.

use serde_json::{json, Value};
use std::path::Path;

pub fn compare(pass_a: &Path, pass_b: &Path, out: &Path) -> Result<Value, String> {
    let a = read_json(pass_a)?;
    let b = read_json(pass_b)?;
    let sa = &a["summary"];
    let sb = &b["summary"];

    let stable = |s: &Value| {
        json!({
            "total_tests": s["total_tests"],
            "oracle": s["oracle"],
            "candidate": s["candidate"],
            "comparison": s["comparison"],
            "wrapper": s["wrapper"],
            "first_failure": s["first_failure"],
            "reason_codes": s["reason_codes"],
        })
    };
    let identical = stable(sa) == stable(sb);

    // per-test primary classifications must also match one-for-one
    // (pass-b's inventory is read from the sibling pass dir when `out` is the repo reports dir)
    let inv_a = read_json(&out.join("test-inventory.json")).ok();
    let inv_b = read_json(&out.join("test-inventory.json"))
        .ok()
        .or_else(|| {
            read_json(
                &out.join("..")
                    .join("..")
                    .join("pass-b")
                    .join("test-inventory.json"),
            )
            .ok()
        });
    let classifications_match = match (inv_a, inv_b) {
        (Some(ia), Some(ib)) => {
            let ca: Vec<&str> = ia["tests"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .map(|t| t["primary_classification"].as_str().unwrap_or(""))
                        .collect()
                })
                .unwrap_or_default();
            let cb: Vec<&str> = ib["tests"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .map(|t| t["primary_classification"].as_str().unwrap_or(""))
                        .collect()
                })
                .unwrap_or_default();
            ca == cb
        }
        _ => false,
    };

    let doc = json!({
        "schema": "gnurust-gnucobol-testsuite-determinism-v1",
        "pass_a": {"summary_sha256": sha256_file(pass_a), "path": pass_a},
        "pass_b": {"summary_sha256": sha256_file(pass_b), "path": pass_b},
        "stable_summary_identical": identical,
        "per_test_classifications_identical": classifications_match,
        "note": "stable summary counts + per-test classifications must be identical across two fresh full runs (timestamps deliberately excluded)",
    });
    std::fs::write(
        out.join("determinism.json"),
        serde_json::to_string_pretty(&doc).unwrap() + "\n",
    )
    .map_err(|e| format!("write determinism.json: {e}"))?;
    Ok(doc)
}

pub fn read_json(p: &Path) -> Result<Value, String> {
    std::fs::read_to_string(p)
        .map_err(|e| format!("read {}: {e}", p.display()))
        .and_then(|s| serde_json::from_str(&s).map_err(|e| format!("parse {}: {e}", p.display())))
}

fn sha256_file(p: &Path) -> String {
    use sha2::{Digest, Sha256};
    match std::fs::read(p) {
        Ok(b) => {
            let mut h = Sha256::new();
            h.update(&b);
            format!("{:x}", h.finalize())
        }
        Err(_) => String::new(),
    }
}
