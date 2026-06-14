<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.12B (court-casefile)

**Verdict: PASS** · set-false sweep 37/0 + fuzz · crate `gnucobol-rs` 0.7.53

- **Oracle:** cobc SET ... TO FALSE final bytes
- **Byte domain(s):** parent field-storage bytes
- **Replay:** `bash lab/oracle/set_false_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- canonical parent bytes for SET condition-name TO FALSE (the WHEN SET TO FALSE IS literal) matching cobc
- output does not satisfy eval_88

## Negative claims (4) — negative capability is the trust surface
- condition-name expressions
- collating-sensitive ranges
- Procedure Division execution
- lie prevented: 'SET FALSE writes any non-satisfying bytes' -- it writes cobc's canonical WHEN SET TO FALSE IS literal

## Damage if overclaimed
a wrong SET FALSE writes a status the program never intended

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
