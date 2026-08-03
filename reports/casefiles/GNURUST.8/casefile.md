<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.8 (court-casefile)

**Verdict: PASS** · 392 sweep + 3M fuzz · crate `gnucobol-rs` 0.8.51

- **Oracle:** cobc DISPLAY of group (raw bytes)
- **Byte domain(s):** record-storage bytes
- **Replay:** `bash lab/oracle/value_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- 01 record initial WORKING-STORAGE bytes from VALUE matching cobc

## Negative claims (5) — negative capability is the trust surface
- VALUE under OCCURS/REDEFINES
- non-fitting literals
- no-VALUE records
- figuratives beyond ZERO/SPACE
- lie prevented: 'VALUE init is trivial' — the initial record bytes match cobc WORKING-STORAGE

## Damage if overclaimed
a wrong VALUE image mis-seeds an uninitialized record read as real data

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
