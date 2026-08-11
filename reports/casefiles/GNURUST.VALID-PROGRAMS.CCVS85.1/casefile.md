<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.VALID-PROGRAMS.CCVS85.1 (court-casefile)

**Verdict: PASS** · 1/1 pass, 0 fail · crate `gnucobol-rs` 0.8.56

- **Oracle:** GnuCOBOL 3.2.0 (admitted lab/oracle build) + the committed corpus evidence under reports/valid-corpus/
- **Byte domain(s):** reports/valid-corpus/ccvs85/programs.json (512 units) + the single GNURUST.CCVS85 evidence system
- **Replay:** `bash lab/oracle/../valid-corpus/corpus_court_sweep.sh valid-ccvs85`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- every CCVS85 unit is classified and the 512 units reconcile (programs.json)
- valid executable units have complete packages (source, COPY libraries, inputs, expected report output)

## Negative claims (4) — negative capability is the trust surface
- no NIST certification
- no COBOL-85 conformance claim
- accuracy dimensions are recorded, not verdicts
- lie prevented: every number in the report is aggregated from the committed per-family evidence; this court re-verifies the evidence tree, it never re-measures or invents values

## Damage if overclaimed
presenting CCVS85 replay as certification would certify a suite designed for conformance testing, not certification

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
