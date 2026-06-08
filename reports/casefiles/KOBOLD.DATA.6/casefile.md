<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.DATA.6 (composition-casefile)

**Verdict: PASS** · account/payroll/insurance + account-cp500 corpus byte-stable, 0 unsupported · crate `kobold-data-shim` kobold 0.6.3

- **Oracle:** gnucobol-rs GNURUST.18 (sealed)
- **Byte domain(s):** COMP-6 field-storage bytes -> value
- **Replay:** `gnucobol-rs GNURUST.18 (sealed)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (3)
- unsigned COMP-6 fields decode via GNURUST.18 in the corpus
- audit names the court
- cp500 passthrough proven

## Negative claims (7) — negative capability is the trust surface
- signed COMP-6 (fails closed)
- COMP-6 arithmetic
- malformed COMP-6 bytes
- pre-3.2 behavior
- dialect portability
- EBCDIC conversion of COMP-6 bytes
- lie prevented: 'unsigned packed can reuse signed packed decoding with the sign nibble ignored' -- COMP-6 has no sign nibble and signed COMP-6 is really COMP-3

## Damage if overclaimed
treating signed COMP-6 as unsigned mis-reads it (GnuCOBOL makes it COMP-3)

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
