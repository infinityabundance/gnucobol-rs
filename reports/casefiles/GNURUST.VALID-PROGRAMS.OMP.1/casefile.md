<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.VALID-PROGRAMS.OMP.1 (court-casefile)

**Verdict: PASS** · 1/1 pass, 0 fail · crate `gnucobol-rs` 0.8.56

- **Oracle:** GnuCOBOL 3.2.0 (admitted lab/oracle build) + the committed corpus evidence under reports/valid-corpus/
- **Byte domain(s):** reports/valid-corpus/omp/programs.json + inventory.json
- **Replay:** `bash lab/oracle/../valid-corpus/corpus_court_sweep.sh valid-omp`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- the OMP COBOL Programming Course repository is fully inventoried (30 COBOL programs, 43 JCL, 226 images, 48 docs, 3 data, 5 support) with complete solutions separated from exercises and platform dependencies typed

## Negative claims (3) — negative capability is the trust surface
- platform-service dependencies (z/OS datasets, JCL, DB2, CICS, VSAM, LE) are typed boundaries, never parser failures
- starter exercises with intentionally missing code are not valid complete programs
- lie prevented: every number in the report is aggregated from the committed per-family evidence; this court re-verifies the evidence tree, it never re-measures or invents values

## Damage if overclaimed
describing platform-service failures as parser failures would misattribute the candidate's boundary

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
