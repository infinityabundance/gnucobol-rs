<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.VALID-PROGRAMS.ACCURACY.1 (court-casefile)

**Verdict: PASS** · 1/1 pass, 0 fail · crate `gnucobol-rs` 0.8.57

- **Oracle:** GnuCOBOL 3.2.0 (admitted lab/oracle build) + the committed corpus evidence under reports/valid-corpus/
- **Byte domain(s):** reports/valid-corpus/accuracy.json + per-family accuracy reports
- **Replay:** `bash lab/oracle/../valid-corpus/corpus_court_sweep.sh accuracy`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- accuracy.json records the raw-byte accuracy dimensions per family (compile status, execution status, report bytes sha256, raw stdout/stderr, generated files, return status)

## Negative claims (3) — negative capability is the trust surface
- output normalization is never reported as raw-byte parity
- warning-text parity is not semantic correctness
- lie prevented: every number in the report is aggregated from the committed per-family evidence; this court re-verifies the evidence tree, it never re-measures or invents values

## Damage if overclaimed
reporting normalized output as byte parity would misstate the comparison

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
