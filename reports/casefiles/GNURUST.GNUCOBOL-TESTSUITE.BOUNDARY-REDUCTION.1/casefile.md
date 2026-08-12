<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.GNUCOBOL-TESTSUITE.BOUNDARY-REDUCTION.1 (court-casefile)

**Verdict: PASS** · reports/gnucobol-testsuite/boundary-reduction-baseline.json + boundary-reduction.{json,md} + classification-transitions.{json,md} · crate `gnucobol-rs` 0.8.57

- **Oracle:** the v0.8.54 baseline record (reports/gnucobol-testsuite/boundary-reduction-baseline.json)
- **Byte domain(s):** boundary-reduction.{json,md} + classification-transitions.{json,md} + the raw rerun evidence
- **Replay:** `bash lab/oracle/gnucobol-testsuite/run-docker.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (4)
- the boundary-reduction baseline (commit 25fb3410b) is bound to the suite/oracle/candidate identity + ledger + raw-evidence hashes
- every v0.8.54 classification has a measured after-state from the rerun with its transition (MODULE_BOUNDARY_TO_MATCH / MODULE_BOUNDARY_TO_PARSER_REJECT / ...)
- no test is unaccounted
- transitions are measured, never projected

## Negative claims (5) — negative capability is the trust surface
- no claim that a transition is a pass (it is a re-measured classification)
- no claim that the v0.8.54 baseline was overwritten (it is preserved in boundary-reduction-baseline.json)
- no claim that the reduction is complete
- no claim that reclassification without raw evidence is acceptable
- lie prevented: '407 module tests are fixed' is the lie this prevents

## Damage if overclaimed
presenting reclassification as implementation would hide the real boundaries

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
