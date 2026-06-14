<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.INDEXED.FILE.ATLAS.1 (court-casefile)

**Verdict: PASS** · 9/9 pass, 0 fail · crate `gnucobol-rs` 0.7.50

- **Oracle:** cobc INDEXED file I/O (libcob/fileio.c + ISAM backend)
- **Byte domain(s):** INDEXED keyed access: random READ by key, key-order retrieval, dup(22)/not-found(23) status, START, DELETE
- **Replay:** `bash lab/oracle/indexed_file_atlas_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (6)
- observed GnuCOBOL INDEXED-file keyed-access behavior under the gnucobol-3.2.0-default witness, verified by the sweep (9/0) -- the largest remaining gap cluster (START 238x
- DELETE 118x + indexed-org per GNURUST.PUBLIC.GAP.1): a random READ by RECORD KEY returns the record (status 00) or 23 not-found
- records are stored/retrieved in KEY ORDER (AAA,BBB,CCC) NOT insertion order (the index sorts)
- a duplicate primary key WRITE is rejected with status 22
- START KEY >= k positions for READ NEXT in key order
- DELETE removes the keyed record (00) and a later READ returns 23. OBSERVED court: gnucobol-rs does NOT implement indexed files -- the on-disk ISAM/BDB/VBISAM index format is backend-specific and outside the fixed-record evidence lane -- this MAPS the surface, it does no indexed I/O

## Negative claims (8) — negative capability is the trust surface
- the on-disk ISAM/BDB/VBISAM index format
- page checksum / atomicity
- alternate keys / DUPLICATES
- concurrent access / locking
- indexed file execution
- relative files
- all dialects
- lie prevented: an indexed file is records in insertion order with an index on the side -- NO: records are RETRIEVED IN KEY ORDER (AAA,BBB,CCC even though inserted CCC,BBB), a duplicate primary key is REJECTED (status 22), and the on-disk format is a BACKEND-SPECIFIC ISAM/BDB structure (not the fixed-record bytes) -- so KOBOLD's fixed-record evidence does NOT cover indexed files, and the on-disk index/page integrity is a separate (unbuilt) storage court

## Damage if overclaimed
treating an indexed file as a flat fixed-record file (insertion order, no key uniqueness) corrupts every keyed read; assuming the on-disk index is portable across backends silently breaks migration

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
