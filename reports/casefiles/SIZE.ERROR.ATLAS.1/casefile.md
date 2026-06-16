<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — SIZE.ERROR.ATLAS.1 (court-casefile)

**Verdict: PASS** · 12/12 pass, 0 fail · crate `kobold-data-shim` 0.7.84

- **Oracle:** bash lab/oracle/size_error_sweep.sh
- **Byte domain(s):** observed receiver bytes (before/after) + size-error flag from the oracle
- **Replay:** `bash lab/oracle/size_error_atlas_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (5)
- OBSERVED GnuCOBOL 3.2 size-error behavior over ADD/SUBTRACT/MULTIPLY/DIVIDE + divide-by-zero, DISPLAY
- COMP-3 receivers, signed
- ROUNDED-carry: with a SENTINEL-prefilled receiver, records receiver-written-vs-preserved (before/after bytes) and size_error_signaled. Finding: plain ADD/SUB/MUL/ROUNDED overflow WRITES the truncated receiver
- plain divide-by-zero PRESERVES it
- ON SIZE ERROR always preserves + signals

## Negative claims (6) — negative capability is the trust surface
- ON SIZE ERROR / NOT ON SIZE ERROR control flow (not implemented)
- Procedure Division execution
- which branch runs
- receiver-write inference
- business-arithmetic correctness
- lie prevented: 'KOBOLD arithmetic handles ON SIZE ERROR like GnuCOBOL' -- SIZE.ERROR.ATLAS.1 only OBSERVES whether the receiver is written/preserved and whether size error is signaled; it implements no control flow

## Damage if overclaimed
assuming a money field was preserved (or written) on overflow when it was the opposite silently corrupts financial data

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
