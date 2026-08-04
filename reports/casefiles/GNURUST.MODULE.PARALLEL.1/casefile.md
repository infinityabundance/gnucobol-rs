<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.MODULE.PARALLEL.1 (court-casefile)

**Verdict: PASS** · crates/cobc-rs/tests/module_courts.rs (one_hundred_parallel_modules_with_colliding_basenames_stay_isolated) · crate `gnucobol-rs` 0.8.55

- **Oracle:** deterministic per-directory expectation (each dir's own output)
- **Byte domain(s):** per-directory stdout correctness under concurrency
- **Replay:** `bash lab/oracle/gnucobol-testsuite/run-docker.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- 100 concurrent cobc-rs/cobcrun-rs invocations with colliding source basenames in separate directories each see their OWN module (atomic manifest writes, no cross-test leakage, no shared mutable state)

## Negative claims (2) — negative capability is the trust surface
- no claim about concurrency inside a single program's execution
- lie prevented: 'parallel tests corrupt each other's modules' is the lie this prevents

## Damage if overclaimed
claiming concurrency safety without the stress court

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
