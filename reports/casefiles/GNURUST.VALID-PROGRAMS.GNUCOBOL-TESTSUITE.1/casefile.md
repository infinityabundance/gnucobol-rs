<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.VALID-PROGRAMS.GNUCOBOL-TESTSUITE.1 (court-casefile)

**Verdict: PASS** · 1/1 pass, 0 fail · crate `gnucobol-rs` 0.8.56

- **Oracle:** GnuCOBOL 3.2.0 (admitted lab/oracle build) + the committed corpus evidence under reports/valid-corpus/
- **Byte domain(s):** reports/valid-corpus/gnucobol-testsuite/*
- **Replay:** `bash lab/oracle/../valid-corpus/corpus_court_sweep.sh valid-testsuite`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- the GnuCOBOL Autotest suite is classified at AT_CHECK-step level (valid-programs.json, discovered-steps.json, invalid-programs.json, mixed-groups.json, dependency-graph.json, stable-current-drift.json, summary.md)

## Negative claims (3) — negative capability is the trust surface
- no real-world generalization claim from upstream tests alone
- screen/curses steps are skipped under the no-terminal oracle profile
- lie prevented: every number in the report is aggregated from the committed per-family evidence; this court re-verifies the evidence tree, it never re-measures or invents values

## Damage if overclaimed
claiming the candidate generalizes because the upstream suite passes would overstate evidence from the candidate's own development source

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
