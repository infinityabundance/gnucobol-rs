<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.RECON.1 (court-casefile)

**Verdict: PASS** · 3 families / 360 records, byte-stable, CLI==lib · crate `kobold-data-shim` kobold-data-shim 0.2.0

- **Oracle:** the sealed GNURUST courts (no new oracle)
- **Byte domain(s):** JSON bytes + audit-receipt bytes (composing field/record/text-word/predicate domains)
- **Replay:** `the sealed GNURUST courts (no new oracle)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (1)
- end-to-end decode of fixed-record data -> byte-stable JSONL + audit receipt + explicit unsupported, over sealed courts only

## Negative claims (5) — negative capability is the trust surface
- transformed-record write-back
- EBCDIC
- edited PIC
- line-sequential containers
- lie prevented: 'decoded JSON is business truth' — it is record truth over sealed courts only

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
