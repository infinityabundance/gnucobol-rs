<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.GNUCOBOL-TESTSUITE.2 (court-casefile)

**Verdict: PASS** · crates/cobc-rs (the driver) + crates/gnucobol-rs-testsuite (the harness) + lab/gnucobol-testsuite · crate `gnucobol-rs` 0.8.57

- **Oracle:** none used during the candidate phase (isolation is mechanically enforced); the baseline is TESTSUITE.1
- **Byte domain(s):** per-test candidate outcomes (parse/check/prepare/run), raw stdout+stderr, candidate census, execve trace, launcher manifests
- **Replay:** `bash lab/gnucobol-testsuite/run-docker.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- the SAME generated suite runs with COBC=cobc-rs (the option-policy-driven compatibility driver) and COBCRUN=cobc-rs through the suite's own `make localcheck` + atlocal bootstrap, producing truthful launcher+manifest artifacts (never native COBOL executables), with the mechanical no-delegation proof (candidate PATH stripped of the oracle, oracle prefix absent in the container, cobrun/cobc-rs link no libcob -- ldd+readelf, plus an execve trace of candidate artifacts), the candidate invocation ledger, and every test accounted for
- NO suite-pass claim

## Negative claims (4) — negative capability is the trust surface
- no parity claim (that is TESTSUITE.3)
- no claim that the launcher is a native COBOL executable
- no claim that rejected options preserve semantics
- lie prevented: 'cobc-rs is a drop-in cobc' is the lie this prevents -- the artifacts are interpreter launch manifests and the boundaries are explicit

## Damage if overclaimed
treating the launcher as a native executable would misrepresent an interpreter as codegen

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
