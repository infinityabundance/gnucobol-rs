<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.VALID-PROGRAMS.EXTRAS.1 (court-casefile)

**Verdict: PASS** · 1/1 pass, 0 fail · crate `gnucobol-rs` 0.8.56

- **Oracle:** GnuCOBOL 3.2.0 (admitted lab/oracle build) + the committed corpus evidence under reports/valid-corpus/
- **Byte domain(s):** reports/valid-corpus/extras/*
- **Replay:** `bash lab/oracle/../valid-corpus/corpus_court_sweep.sh valid-extras`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- OpenCBS COBOL Defects Benchmark Suite and other shipped/contributed programs are inventoried with licence decisions (extras programs.json + custody.json + metrics.json)

## Negative claims (3) — negative capability is the trust surface
- no pristine-parity claim from adapted programs
- adaptations are recorded with original/transformed hashes when any are applied
- lie prevented: every number in the report is aggregated from the committed per-family evidence; this court re-verifies the evidence tree, it never re-measures or invents values

## Damage if overclaimed
claiming shipped programs as candidate evidence before licensing/dependency resolution would misrepresent custody

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
