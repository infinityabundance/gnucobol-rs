<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.DATA.3 (composition-casefile)

**Verdict: PASS** · account-cp500 family (120 rec) + ebcdic_never_touches_binary test · crate `kobold-data-shim` kobold-data-shim 0.4.0

- **Oracle:** sealed GNURUST courts
- **Byte domain(s):** JSON + audit bytes
- **Replay:** `sealed GNURUST courts`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (1)
- cp500 alphanumeric DISPLAY decoded end-to-end with binary/packed passthrough proven untouched (composes GNURUST.15)

## Negative claims (3) — negative capability is the trust surface
- numeric DISPLAY under cp500 (EBCDIC zoned sign)
- business truth
- lie prevented: 'EBCDIC touches all bytes' — binary/packed pass through untouched (proven)

## Damage if overclaimed
converting binary/packed bytes as EBCDIC text destroys numeric values

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
