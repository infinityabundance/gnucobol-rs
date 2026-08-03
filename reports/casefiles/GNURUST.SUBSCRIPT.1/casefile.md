<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.SUBSCRIPT.1 (court-casefile)

**Verdict: PASS** · 17/17 pass, 0 fail · crate `gnucobol-rs` 0.8.53

- **Oracle:** cobc DISPLAY TABLE(i[,j]) subscript
- **Byte domain(s):** OCCURS table bytes + subscripts -> element bytes
- **Replay:** `bash lab/oracle/subscript_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- 1-based direct table subscript TABLE(i) and multi-dimensional TABLE(i,j) element extraction (offset = sum (idx-1)*stride, innermost stride = element size) matching cobc in bounds

## Negative claims (6) — negative capability is the trust surface
- out-of-bounds subscript (fail-closed by design, NOT cobc's flag-dependent read)
- OCCURS DEPENDING ON variable length
- INDEXED BY index-names
- subscript arithmetic expressions
- signed/packed element decode
- lie prevented: 'COBOL subscripts are 0-based like C' -- NO, TABLE(1) is the FIRST element at offset 0; an off-by-one reads the wrong element

## Damage if overclaimed
a wrong subscript offset reads the wrong table element or adjacent storage, corrupting every table lookup

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
