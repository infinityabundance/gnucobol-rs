<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.GNUCOBOL-TESTSUITE.3 (court-casefile)

**Verdict: PASS** · reports/gnucobol-testsuite/* (ledgers, summaries, raw logs, no-delegation, determinism) + reports/receipts/GNURUST.GNUCOBOL-TESTSUITE.{1,2,3}/ · crate `gnucobol-rs` 0.8.56

- **Oracle:** the TESTSUITE.1 baseline (real admitted cobc)
- **Byte domain(s):** per-test classification + reason codes + the raw baseline/candidate outputs they rest on
- **Replay:** `bash lab/gnucobol-testsuite/run-docker.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- every generated test group receives exactly one final classification (reconciled: total == sum of all classes), with the oracle x candidate outcome pairs, first-failure attribution, raw evidence preserved, deterministic two-pass comparison, and honest totals in this environment: 173 OBSERVABLE_MATCH, 439 candidate check/parse rejects, 407 module-model unsupported, 173 wrapper-option unsupported, 26 wrapper-malformed, 22 candidate unsupported, 2 runtime fails, 0 timeouts, 0 not-reached
- the runtime/math subset is reported separately (GNURUST.GNUCOBOL-RUNTIME-MATH.1)

## Negative claims (5) — negative capability is the trust surface
- OBSERVABLE_MATCH is the test's own AT_CHECK assertion outcome in this environment, not equivalence outside it
- no GnuCOBOL test-suite parity claim
- no COBOL conformance certification
- no performance claim (see the runtime-math performance view)
- lie prevented: 'the GnuCOBOL test-suite passes with gnucobol-rs' is the lie this prevents -- the honest surface is a classification, not a pass count

## Damage if overclaimed
presenting the classification as full suite parity would certify coverage the candidate does not have

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
