<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.GNUCOBOL-RUNTIME-MATH.2 (court-casefile)

**Verdict: PASS** · reports/gnucobol-runtime-tests/* · crate `gnucobol-rs` 0.8.54

- **Oracle:** the TESTSUITE.1 baseline
- **Byte domain(s):** math-correctness.{json,md} + math-performance.{json,csv,md} + raw-samples/
- **Replay:** `bash lab/oracle/gnucobol-testsuite/run-docker.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (4)
- the math subset is recomputed from the SAME full-suite ledger as every other test after the rerun
- the machine invariant sum(math classifications) == math test count == 323 (unique ids, ids subset of the suite) holds
- the generator fails on any violation
- performance is re-reported only for correctness-matched programs

## Negative claims (3) — negative capability is the trust surface
- no performance claim from end-to-end interpreter-vs-native timing
- no equivalence claim outside the tested environment
- lie prevented: 'the 22/21 wrapper-option discrepancy was cosmetic' is the lie this prevents -- the reconciliation is machine-enforced

## Damage if overclaimed
claiming math parity from a non-reconciling ledger

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
