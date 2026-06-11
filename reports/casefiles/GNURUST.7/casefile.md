<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.7 (court-casefile)

**Verdict: PASS** · within 5400 arith sweep + 8M fuzz · crate `gnucobol-rs` 0.7.34

- **Oracle:** libcob cob_add/cob_sub/cob_mul
- **Byte domain(s):** receiving-field storage bytes
- **Replay:** `bash lab/oracle/arith_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- result bytes matching cob_add/cob_sub/cob_mul (cob_decimal path), truncate + nearest-away

## Negative claims (5) — negative capability is the trust surface
- DIVIDE
- other rounding modes
- ON SIZE ERROR
- >38-digit bignum
- lie prevented: 'rounding/sign are details' — receiving bytes match cob_add/sub/mul exactly

## Damage if overclaimed
a wrong arithmetic result written back mis-states a balance or total

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
