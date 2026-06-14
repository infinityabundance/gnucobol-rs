<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.TABLE.PERFORM.SLICE.1 (court-casefile)

**Verdict: PASS** · 3/3 pass, 0 fail · crate `gnucobol-rs` 0.7.64

- **Oracle:** cobc PERFORM VARYING + subscript (cobc/typeck.c + codegen.c)
- **Byte domain(s):** PERFORM VARYING I over a 1-based OCCURS table accumulating TABLE(I)
- **Replay:** `bash lab/oracle/table_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (3)
- PERFORM VARYING over a subscripted OCCURS table -- the core of COBOL table processing -- executed to oracle-identical accumulation (verified 3/0: whole-table sum, filtered count, strided BY-2 sum). PERFORM VARYING I FROM a BY b UNTIL I > limit accumulates TABLE(I), where TABLE(I) is 1-BASED (element I at base
- (I-1)*elem_size). An optional per-element IF TABLE(I) <op> literal gates the accumulation. Deepens the execution slices with subscript access
- reuses the witnessed PERFORM VARYING (test-before, I ends one past the limit)

## Negative claims (9) — negative capability is the trust surface
- multi-dimensional/nested OCCURS
- OCCURS DEPENDING ON
- subscript out-of-bounds
- INDEXED BY / SEARCH / SET
- signed/packed/V elements
- numeric SIZE ERROR
- non-sum bodies
- all dialects
- lie prevented: COBOL subscripts are 0-based like C -- NO: TABLE(I) is 1-BASED, so TABLE(1) is the FIRST element (at offset 0) and TABLE(0) is out of range; an off-by-one in the subscript reads the wrong element or past the table

## Damage if overclaimed
using 0-based subscript arithmetic on a COBOL table reads the wrong element (or adjacent storage), corrupting every table-driven total or lookup

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
