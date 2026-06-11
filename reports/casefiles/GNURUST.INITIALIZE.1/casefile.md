<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.INITIALIZE.1 (court-casefile)

**Verdict: PASS** · 6/6 pass, 0 fail · crate `gnucobol-rs` 0.7.32

- **Oracle:** cobc INITIALIZE (program-shape, sentinel-prefill + REDEFINES dump)
- **Byte domain(s):** INITIALIZE record -> changed/preserved receiver bytes
- **Replay:** `bash lab/oracle/initialize_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- the bytes a plain INITIALIZE <record> changes or preserves, matching cobc/libcob byte-for-byte: each ELEMENTARY item gets its category default -- X(n) spaces (0x20), numeric DISPLAY 9/S9 '0' digits (0x30, no sign overpunch on +0), COMP-3 packed zero with sign nibble C (signed) / F (unsigned), binary COMP/COMP-5/COMP-X zero bytes -- while FILLER is PRESERVED, a REDEFINES redefiner is SKIPPED (only the base definition is initialized), every OCCURS element is initialized, and a VALUE clause is NOT restored (the category default wins). Proven from a sentinel prefill (MOVE ALL) so changed-vs-preserved is visible

## Negative claims (8) — negative capability is the trust surface
- full Procedure Division execution
- INITIALIZE REPLACING/TO VALUE/WITH FILLER
- numeric-edited/JUSTIFIED/BLANK WHEN ZERO
- ODO runtime active count
- active REDEFINES view
- all dialects
- business defaults
- lie prevented: 'INITIALIZE just zeroes/blanks everything and restores VALUE' -- it PRESERVES FILLER and redefiners, sets category defaults NOT the VALUE clause, uses sign nibble C(signed)/F(unsigned) for packed, and '0' digits (not low-values) for numeric DISPLAY

## Damage if overclaimed
assuming INITIALIZE cleared a FILLER/redefiner, restored a VALUE, or used low-values/spaces wrongly silently corrupts a record before it is written or compared

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
