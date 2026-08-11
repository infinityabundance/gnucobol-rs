<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.VALID-PROGRAMS.HELD-OUT.1 (court-casefile)

**Verdict: PASS** · 1/1 pass, 0 fail · crate `gnucobol-rs` 0.8.56

- **Oracle:** GnuCOBOL 3.2.0 (admitted lab/oracle build) + the committed corpus evidence under reports/valid-corpus/
- **Byte domain(s):** reports/valid-corpus/held-out-results.json
- **Replay:** `bash lab/oracle/../valid-corpus/corpus_court_sweep.sh held-out`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- the held-out evaluation (101 files) ran the candidate under a hard wall bound with 0 crashes and 0 timeouts, and the report states the held-out set was never used for implementation tuning

## Negative claims (3) — negative capability is the trust surface
- no held-out claim after the set has been used for implementation tuning
- parse/check/run success on held-out files is not language conformance
- lie prevented: every number in the report is aggregated from the committed per-family evidence; this court re-verifies the evidence tree, it never re-measures or invents values

## Damage if overclaimed
claiming held-out generalization after tuning on the held-out set would be circular

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
