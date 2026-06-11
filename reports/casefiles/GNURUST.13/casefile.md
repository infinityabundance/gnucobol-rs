<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.13 (court-casefile)

**Verdict: PASS** · within 5400 arith sweep + 8M fuzz · crate `gnucobol-rs` 0.3.3

- **Oracle:** libcob cob_add/cob_sub (cob_add_bcd path)
- **Byte domain(s):** receiving-field storage bytes
- **Replay:** `libcob cob_add/cob_sub (cob_add_bcd path)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (1)
- ADD/SUBTRACT into a PACKED receiver, receiving-field bytes matching libcob cob_add_bcd

## Negative claims (6) — negative capability is the trust surface
- DIVIDE
- SIZE ERROR
- other rounding modes
- bignum
- float
- lie prevented: 'packed add rounds like display' — cob_add_bcd keeps -0 on truncation

## Damage if overclaimed
a wrong packed sign/rounding flips the sign of money on truncation

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
