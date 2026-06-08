<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.9 (court-casefile)

**Verdict: PASS** · 96 of 288 PIC sweep + fuzz · crate `gnucobol-rs` 0.2.6

- **Oracle:** cobc -C attr witness
- **Byte domain(s):** generated-C cob_field_attr + LENGTH OF
- **Replay:** `cobc -C attr witness`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (4)
- P scaling field model (digits/scale/size) matching cobc
- trailing digits=9s
- P scale=-P, leading digits=9s scale=9s
- P

## Negative claims (4) — negative capability is the trust surface
- V+P
- P at both ends
- VALUE/MOVE on P field
- lie prevented: 'P-scaling is just digits' — the asymmetric P digit/scale rule is exact

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
