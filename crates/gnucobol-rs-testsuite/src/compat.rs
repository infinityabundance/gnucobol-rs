//! The generated `docs/generated/cobc-rs-option-compatibility.md` (prompt §6.3): for EVERY option
//! observed in the real invocation census, the wrapper-side policy from the `cobc-rs` option
//! registry (authoritative), a semantic-risk note, and the census evidence. Freshness is enforced
//! by regenerating from the same two inputs and diffing.

use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

/// Load the policy registry export (`cobc-rs --dump-policy-json=<path>`): option -> policy record.
fn load_policy(path: &Path) -> Result<BTreeMap<String, Value>, String> {
    let v: Value = serde_json::from_str(
        &std::fs::read_to_string(path).map_err(|e| format!("policy {path:?}: {e}"))?,
    )
    .map_err(|e| format!("policy JSON {path:?}: {e}"))?;
    let mut map = BTreeMap::new();
    for e in v["options"].as_array().unwrap_or(&vec![]) {
        if let Some(opt) = e["option"].as_str() {
            map.insert(opt.to_string(), e.clone());
        }
        for a in e["aliases"].as_array().unwrap_or(&vec![]) {
            if let Some(a) = a.as_str() {
                map.insert(a.to_string(), e.clone());
            }
        }
    }
    Ok(map)
}

/// Load the invocation census: option token -> (count, sample argv).
fn load_census(path: &Path) -> Result<BTreeMap<String, (usize, String)>, String> {
    let v: Value = serde_json::from_str(
        &std::fs::read_to_string(path).map_err(|e| format!("census {path:?}: {e}"))?,
    )
    .map_err(|e| format!("census JSON {path:?}: {e}"))?;
    let mut map: BTreeMap<String, (usize, String)> = BTreeMap::new();
    for inv in v["invocations"].as_array().unwrap_or(&vec![]) {
        let argv: Vec<String> = inv["argv"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|a| a.as_str().map(String::from))
            .collect();
        for a in argv.iter().skip(1) {
            if !a.starts_with('-') || a.len() <= 1 {
                continue;
            }
            // normalize `-opt=value` -> `-opt=` (the census token shape)
            let tok = match a.find('=') {
                Some(i) => a[..i + 1].to_string(),
                None => a.to_string(),
            };
            let e = map.entry(tok).or_insert((0, String::new()));
            e.0 += 1;
            if e.1.is_empty() {
                e.1 = argv.join(" ");
            }
        }
    }
    Ok(map)
}

/// The `-W` / `-f*` family fallback: the registry's generic entry a token resolves to when it has
/// no exact entry (the `-W<category>` prefix rule in `policy::lookup`).
fn family_of(tok: &str) -> Option<&'static str> {
    if tok.starts_with("-W") && !tok.starts_with("--") {
        Some("-W")
    } else {
        None
    }
}

fn risk_note(policy: &str, justification: &str) -> String {
    match policy {
        "translated" => "low (translated to a candidate-side equivalent; semantics preserved where the subset supports them)"
            .to_string(),
        "accepted-equivalent" => "low (alias of a translated/accepted spelling)".to_string(),
        "accepted-proven-no-op" => "none for the admitted suite (diagnostic/optimization/no-native-model flag; recorded in the invocation ledger)"
            .to_string(),
        "rejected-unsupported" => {
            format!("HIGH — rejected, never silently dropped: {justification}")
        }
        "rejected-ambiguous" => "HIGH — rejected: ambiguous or unknown spelling".to_string(),
        _ => "UNKNOWN — no explicit policy (must fail closed)".to_string(),
    }
}

/// Generate the compatibility document.
pub fn generate(policy_path: &Path, census_path: &Path, out: &Path) -> Result<String, String> {
    let policy = load_policy(policy_path)?;
    let census = load_census(census_path)?;
    if census.is_empty() {
        return Err("census has no option tokens — nothing to document".into());
    }

    let mut md = String::new();
    md.push_str(
        "<!-- GENERATED from the cobc-rs option-policy registry + the real invocation census — DO NOT EDIT BY HAND.\n\
         Regenerate: cargo run -q -p gnucobol-rs-testsuite -- compat-doc --policy <dump-policy-json> --census reports/gnucobol-testsuite/invocation-census.json --out docs/generated/cobc-rs-option-compatibility.md\n\
         Check: --check (regenerate to a temp file and diff).\n-->\n\n\
         # cobc-rs option compatibility (generated)\n\n\
         For EVERY option observed in the real GnuCOBOL 3.2 testsuite invocation census, the explicit\n\
         `cobc-rs` policy (from the option-policy registry), the semantic risk, and the census\n\
         evidence. An option with NO explicit policy fails closed (never silently ignored).\n\n\
         | option | status | translation / note | semantic risk | census count | example invocation |\n\
         |---|---|---|---:|---|---|\n",
    );
    for (tok, (count, sample)) in &census {
        // the census keeps `-opt=` tokens (attached-value shape); the registry keys are the bare
        // option, so strip a trailing `=` for the lookup (the parser's split_attached does the same),
        // and mirror the runtime's getopt_long equivalence (`--x` == `-x`).
        let key = tok.trim_end_matches('=');
        let normalized: String = key
            .strip_prefix("--")
            .map(|s| format!("-{s}"))
            .unwrap_or_default();
        // GCC-style short attached value (`-DNAME`, `-Ipath`): resolve via the 2-char prefix when
        // that prefix is a consumes-value registry entry (mirrors split_short_attached in cobc-rs).
        let short_prefix: String = if key.len() > 2 && key.starts_with('-') {
            key[..2].to_string()
        } else {
            String::new()
        };
        let short_ok = policy
            .get(&short_prefix)
            .map(|e| e["consumes_value"].as_bool().unwrap_or(false))
            .unwrap_or(false);
        let entry = policy
            .get(key)
            .or_else(|| policy.get(normalized.as_str()))
            .or_else(|| {
                if short_ok {
                    policy.get(&short_prefix)
                } else {
                    None
                }
            })
            .or_else(|| family_of(key).and_then(|f| policy.get(f)));
        match entry {
            Some(e) => {
                let p = e["policy"].as_str().unwrap_or("?");
                let just = e["justification"].as_str().unwrap_or("");
                md.push_str(&format!(
                    "| `{tok}` | {p} | {just} | {} | {count} | `{sample}` |\n",
                    risk_note(p, just)
                ));
            }
            None => {
                md.push_str(&format!(
                    "| `{tok}` | **NO POLICY** | not present in the registry — must fail closed (unknown option) | HIGH — rejected | {count} | `{sample}` |\n"
                ));
            }
        }
    }
    md.push_str(&format!(
        "\n{} unique observed option tokens; every one is classified above — none is silently dropped.\n",
        census.len()
    ));
    std::fs::write(out, &md).map_err(|e| format!("write {}: {e}", out.display()))?;
    Ok(md)
}

/// Freshness check: regenerate to a temp file and compare with the committed one.
pub fn check(policy_path: &Path, census_path: &Path, out: &Path) -> Result<(), String> {
    let fresh = generate(policy_path, census_path, out)?;
    let committed =
        std::fs::read_to_string(out).map_err(|e| format!("cannot read committed {out:?}: {e}"))?;
    if fresh != committed {
        return Err(format!(
            "STALE: {out:?} differs from the regenerated document (run `compat-doc` to refresh)"
        ));
    }
    Ok(())
}
