<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.11 (court-casefile)

**Verdict: PASS** · 103 sweep + 6M fuzz · crate `gnucobol-rs` 0.7.30

- **Oracle:** cobc IF truth matrix
- **Byte domain(s):** parent field-storage bytes -> boolean
- **Replay:** `bash lab/oracle/cond_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (3)
- condition-name truth vs parent bytes matching cobc IF (alnum padded compare
- numeric value compare
- THRU inclusive)

## Negative claims (5) — negative capability is the trust surface
- SET
- condition expressions
- collating-sensitive ranges
- Procedure Division execution
- lie prevented: '88 truth is a string compare' — padded/numeric/THRU semantics are exact

## Damage if overclaimed
a wrong LEVEL-88 truth mis-classifies account status (active/closed/delinquent)

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
