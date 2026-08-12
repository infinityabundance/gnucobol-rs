<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.CORPUS.DEDUP.1 (court-casefile)

**Verdict: PASS** · 1/1 pass, 0 fail · crate `gnucobol-rs` 0.8.57

- **Oracle:** GnuCOBOL 3.2.0 (admitted lab/oracle build) + the committed corpus evidence under reports/valid-corpus/
- **Byte domain(s):** deduplication.json + xcobol/dedup.json
- **Replay:** `bash lab/oracle/../valid-corpus/corpus_court_sweep.sh dedup`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- deduplication.json records exact + near-duplicate evidence
- grouping is repository-level so the development/validation/held-out partitions never split a repository

## Negative claims (3) — negative capability is the trust surface
- no independent-program count that includes duplicates
- near-duplicate thresholds are recorded, not universal
- lie prevented: every number in the report is aggregated from the committed per-family evidence; this court re-verifies the evidence tree, it never re-measures or invents values

## Damage if overclaimed
counting duplicate programs as independent evidence would inflate generalization claims

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
