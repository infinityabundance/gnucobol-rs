<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.DATA.4 (composition-casefile)

**Verdict: PASS** · account/payroll/insurance corpus byte-stable · crate `kobold-data-shim` kobold 0.6.0

- **Oracle:** gnucobol-rs GNURUST.16 (sealed)
- **Byte domain(s):** edited DISPLAY field bytes -> presentation + numeric
- **Replay:** `gnucobol-rs GNURUST.16 (sealed)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (2)
- edited DISPLAY fields decode in the corpus
- JSON keeps the presentation string, audit carries the GNURUST.16 numeric

## Negative claims (5) — negative capability is the trust surface
- numeric->edited formatting
- report writer
- locale/currency
- edited under EBCDIC
- lie prevented: 'presentation string and numeric value are the same field truth' -- they are separated

## Damage if overclaimed
replacing the edited presentation with its numeric (or vice versa) mis-reports money

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
