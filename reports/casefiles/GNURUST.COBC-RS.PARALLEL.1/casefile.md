<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.COBC-RS.PARALLEL.1 (court-casefile)

**Verdict: PASS** · crates/cobc-rs/tests/cli.rs (one_hundred_concurrent_invocations_colliding_basenames) · crate `gnucobol-rs` 0.8.54

- **Oracle:** n/a (concurrency contract of the wrapper itself)
- **Byte domain(s):** per-directory artifact correctness under concurrency
- **Replay:** `n/a (concurrency contract of the wrapper itself)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (2)
- the integration test one_hundred_concurrent_invocations_colliding_basenames runs 100 concurrent cobc-rs compiles with the SAME basename in different directories and verifies each artifact runs its OWN program (no cross-test leakage, deterministic per-directory manifests)
- the wrapper has no globally shared mutable output names or fixed temp files (atomic write-and-rename)

## Negative claims (3) — negative capability is the trust surface
- no claim about the GnuCOBOL suite's own parallel behavior
- only the wrapper's isolation
- lie prevented: 'the wrapper is not safe under parallel testsuite runs' is the defect this prevents

## Damage if overclaimed
a race in artifact generation would corrupt parallel test evidence

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
