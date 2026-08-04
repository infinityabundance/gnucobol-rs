<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.4 (court-casefile)

**Verdict: PASS** · 32 sweep + 4M+ fuzz · crate `gnucobol-rs` 0.8.55

- **Oracle:** cobc -C f_M witness + LENGTH OF
- **Byte domain(s):** generated-C field offset+size
- **Replay:** `bash lab/oracle/layout_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- item byte offsets / group sizes / fixed OCCURS / REDEFINES(<=target) / FILLER matching cobc

## Negative claims (3) — negative capability is the trust surface
- SYNCHRONIZED
- REDEFINES larger than target
- lie prevented: 'offsets are obvious' — layout matches cobc's emitted offsets, not intuition

## Damage if overclaimed
a wrong offset shifts an entire record, mis-attributing values to the wrong fields

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
