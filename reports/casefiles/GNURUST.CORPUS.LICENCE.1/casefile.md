<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.CORPUS.LICENCE.1 (court-casefile)

**Verdict: PASS** · 1/1 pass, 0 fail · crate `gnucobol-rs` 0.8.56

- **Oracle:** GnuCOBOL 3.2.0 (admitted lab/oracle build) + the committed corpus evidence under reports/valid-corpus/
- **Byte domain(s):** licences.json + xcobol/licence-quarantine.json
- **Replay:** `bash lab/oracle/../valid-corpus/corpus_court_sweep.sh licence`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- licences.json records a licence decision for every admitted family
- unknown-licence X-COBOL repositories are quarantined (licence-quarantine.json, REFERENCE_ONLY) and never published

## Negative claims (3) — negative capability is the trust surface
- no redistribution claim for quarantined source
- licence review is recorded, not legal advice
- lie prevented: every number in the report is aggregated from the committed per-family evidence; this court re-verifies the evidence tree, it never re-measures or invents values

## Damage if overclaimed
claiming redistribution rights for quarantined source would misrepresent third-party licensing

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
