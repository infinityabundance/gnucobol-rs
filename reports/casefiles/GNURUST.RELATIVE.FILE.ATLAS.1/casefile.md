<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.RELATIVE.FILE.ATLAS.1 (court-casefile)

**Verdict: PASS** · 3/3 pass, 0 fail · crate `gnucobol-rs` 0.7.40

- **Oracle:** cobc RELATIVE file I/O (libcob/fileio.c)
- **Byte domain(s):** RELATIVE random access by record number + status (23 empty slot)
- **Replay:** `bash lab/oracle/relative_file_atlas_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (3)
- observed GnuCOBOL RELATIVE-file random access (3/0): READ/WRITE address a record by its 1-based RELATIVE record number (slot) -- a WRITE at slot 3 then READ at slot 3 returns it
- reading an UNWRITTEN slot returns status 23 (not found)
- records sit at FIXED POSITIONS by relative number (NOT key-sorted like indexed). OBSERVED court: gnucobol-rs implements no relative file I/O (the on-disk slotted format is backend-specific) -- this MAPS the surface

## Negative claims (7) — negative capability is the trust surface
- relative file execution
- the on-disk slotted format
- sequential/dynamic access modes
- REWRITE/DELETE/START
- indexed files
- all dialects
- lie prevented: a relative file is records in order with a number -- NO: records sit at FIXED SLOTS by relative number (an unwritten slot reads status 23, gaps are real), distinct from indexed (key-sorted) and sequential (insertion order); and gnucobol-rs runs no relative I/O

## Damage if overclaimed
treating relative slots as a dense sequential file ignores empty-slot gaps (status 23) and the by-number addressing

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
