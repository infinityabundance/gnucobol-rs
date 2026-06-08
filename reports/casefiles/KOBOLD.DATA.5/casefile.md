<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.DATA.5 (composition-casefile)

**Verdict: PASS** · account-cp500 corpus byte-stable, 0 unsupported · crate `kobold-data-shim` kobold 0.6.2

- **Oracle:** gnucobol-rs GNURUST.17 (sealed)
- **Byte domain(s):** raw EBCDIC zoned field bytes -> value
- **Replay:** `gnucobol-rs GNURUST.17 (sealed)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (2)
- cp500 EBCDIC zoned numeric DISPLAY fields decode via GNURUST.17 in the corpus
- audit names the court

## Negative claims (5) — negative capability is the trust surface
- cp037
- edited-numeric under cp500
- binary/packed via this path
- mixed/auto-detect encoding
- lie prevented: 'EBCDIC numeric decodes like ASCII zoned' -- the C/D/F sign zones differ

## Damage if overclaimed
auto-detecting the code page mis-decodes the whole EBCDIC file

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
