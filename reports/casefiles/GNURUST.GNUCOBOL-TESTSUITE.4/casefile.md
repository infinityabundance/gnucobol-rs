<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.GNUCOBOL-TESTSUITE.4 (court-casefile)

**Verdict: PASS** · reports/gnucobol-testsuite/* + lab/gnucobol-testsuite/run-docker.sh · crate `gnucobol-rs` 0.8.57

- **Oracle:** the TESTSUITE.1 baseline (re-run, unchanged expectations)
- **Byte domain(s):** the re-measured ledgers + raw rerun evidence + the boundary-reduction transitions
- **Replay:** `bash lab/oracle/gnucobol-testsuite/run-docker.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (5)
- the FULL suite reruns with the new cobc-rs/cobcrun-rs after each major boundary reduction
- all 1282 test groups still reconcile exactly
- the before/after ledger (boundary-reduction) attributes every v0.8.54 classification to its measured after-state
- no-delegation remains mechanically green
- the math subset is regenerated from the same ledger

## Negative claims (6) — negative capability is the trust surface
- no claim that a boundary reduction equals a pass
- no claim of full suite parity
- no COBOL conformance certification
- no claim that a transition was measured without raw evidence
- no claim that the rerun changed the oracle expectations
- lie prevented: 'the three boundaries were reduced' is the claim this court measures -- without the rerun it is projection

## Damage if overclaimed
reporting projected unlocks as measured results

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
