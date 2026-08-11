<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.VALID-PROGRAMS.XCOBOL.1 (court-casefile)

**Verdict: PASS** · 1/1 pass, 0 fail · crate `gnucobol-rs` 0.8.56

- **Oracle:** GnuCOBOL 3.2.0 (admitted lab/oracle build) + the committed corpus evidence under reports/valid-corpus/
- **Byte domain(s):** reports/valid-corpus/xcobol/programs.json + partitions.json + robustness.json + licence-quarantine.json
- **Replay:** `bash lab/oracle/../valid-corpus/corpus_court_sweep.sh valid-xcobol`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- the X-COBOL dataset (DOI 10.5281/zenodo.7968845) is under immutable custody with structural classification, repository-level licence quarantine, frozen development/validation/held-out partitions and large-scale robustness measurement

## Negative claims (3) — negative capability is the trust surface
- unknown-licence source stays quarantined (REFERENCE_ONLY) and is not published
- near-duplicate families are not independent evidence
- lie prevented: every number in the report is aggregated from the committed per-family evidence; this court re-verifies the evidence tree, it never re-measures or invents values

## Damage if overclaimed
claiming generalization from an unfrozen or contaminated held-out set would invalidate every downstream claim

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
