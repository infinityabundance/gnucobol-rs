<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.METHODOLOGY.PARSER.1 (court-casefile)

**Verdict: PASS** · docs/methodology/parser-front-end-provenance.md + reports/methodology/parser-provenance.json · crate `gnucobol-rs` 0.8.54

- **Oracle:** n/a (historical/process documentation)
- **Byte domain(s):** the provenance records + the commit history they cite
- **Replay:** `n/a (historical/process documentation)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (1)
- docs/methodology/parser-front-end-provenance.md + reports/methodology/parser-provenance.json reconstruct the parser history from the committed record (origin commit 9357a7cac 'from-scratch (NOT cobc-derived)', 150+ follow-on commits, oracle-differential development), and state explicitly that the tooling and consulted-materials history is UNKNOWN -- so strict clean-room process separation cannot be independently verified, and the documentation qualifies the term accordingly

## Negative claims (3) — negative capability is the trust surface
- strict clean-room separation is NOT claimed
- no claim that cobc source was never consulted (no evidence either way)
- lie prevented: 'the parser is strict clean-room' is the lie this prevents -- the honest position is independently written per the author's claim

## Damage if overclaimed
a strict clean-room claim would be legally load-bearing and unverifiable from the record

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
