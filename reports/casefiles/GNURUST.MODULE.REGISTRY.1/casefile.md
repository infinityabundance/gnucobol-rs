<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.MODULE.REGISTRY.1 (court-casefile)

**Verdict: PASS** · crates/cobc-rs/tests/module_courts.rs + reports/gnucobol-testsuite/module-lifecycle-census.{json,md} · crate `gnucobol-rs` 0.8.54

- **Oracle:** GnuCOBOL 3.2 cobcrun semantics as observed in the admitted suite's own module tests
- **Byte domain(s):** module artifacts (launcher, manifest, expanded source), cobcrun stdout/stderr/exit, module search order
- **Replay:** `bash lab/oracle/gnucobol-testsuite/run-docker.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (3)
- cobc-rs -m writes a silent launcher+manifest+expanded-source artifact (never a native .so)
- cobcrun-rs resolves modules through the -M directory (trailing slash appended, GnuCOBOL semantics), the working directory and COB_LIBRARY_PATH, passes program arguments to ACCEPT FROM COMMAND-LINE, and emits cobcrun-shaped diagnostics (missing PROGRAM name / invalid module argument / cannot find module)
- exercised end-to-end by crates/cobc-rs/tests/module_courts.rs (GNURUST.MODULE.* courts) and by the suite rerun

## Negative claims (4) — negative capability is the trust surface
- no native shared-object semantics
- no ABI compatibility with real cobcrun
- module state is interpreted, not a loaded DSO
- lie prevented: '-m produces a GnuCOBOL module' is the lie this prevents

## Damage if overclaimed
presenting an interpreted-module manifest as a shared object would break the no-native-artifact boundary

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
