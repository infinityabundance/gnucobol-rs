<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.12 (court-casefile)

**Verdict: PASS** · 52 sweep + fuzz · crate `gnucobol-rs` 0.7.28

- **Oracle:** cobc SET final bytes
- **Byte domain(s):** parent field-storage bytes
- **Replay:** `bash lab/oracle/set_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- canonical parent bytes for SET condition TO TRUE (first VALUE / range lower bound) matching cobc
- output satisfies eval_88

## Negative claims (5) — negative capability is the trust surface
- SET TO FALSE
- FALSE clause
- expressions
- execution
- lie prevented: 'SET TRUE writes anything satisfying the 88' — it writes cobc's canonical bytes

## Damage if overclaimed
a wrong SET writes a status the program never intended

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
