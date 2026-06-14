<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.19 (court-casefile)

**Verdict: PASS** · 736/736 pass, 0 fail · crate `gnucobol-rs` 0.7.56

- **Oracle:** cobc DIVIDE GIVING (program-shape)
- **Byte domain(s):** DIVIDE GIVING receiver field bytes
- **Replay:** `bash lab/oracle/divide_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- DIVIDE a BY b GIVING c + a INTO b GIVING c receiver bytes (DISPLAY/COMP-3, signed/scaled/narrowing, truncate
- ROUNDED) matching cobc

## Negative claims (9) — negative capability is the trust surface
- divide-by-zero / ON SIZE ERROR
- REMAINDER
- COMPUTE / expression evaluation
- procedure control flow
- float
- binary/edited receivers
- other rounding modes
- business correctness
- lie prevented: 'DIVIDE is ordinary decimal division and a mathematically correct quotient is enough' -- receiver scale, truncation toward zero, ROUNDED, sign-of-zero, and receiver bytes all matter

## Damage if overclaimed
a divide result with wrong scale/rounding/truncation/sign/receiver-bytes misstates interest, fees, rates, allocations, or balances while looking plausible

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
