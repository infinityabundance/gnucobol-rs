<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.FILEIO.RELATIVE.1 (court-casefile)

**Verdict: PASS** · 4/4 pass, 0 fail · crate `gnucobol-rs` 0.7.71

- **Oracle:** cobc OPEN/WRITE/READ/REWRITE/DELETE RELATIVE RANDOM+SEQUENTIAL (libcob/fileio.c)
- **Byte domain(s):** OPEN + WRITE/READ/REWRITE/DELETE/START (RELATIVE) -> slot array bytes (8-byte LE header + record_max) + FILE STATUS (00/22/23/24/10)
- **Replay:** `bash lab/oracle/relative_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (6)
- a faithful port of libcob/fileio.c relative_write/relative_read/relative_read_next/relative_rewrite/relative_delete/relative_start, proven byte-identical to the admitted libcob (relative_sweep 4/0): an ORGANIZATION IS RELATIVE file is an array of fixed slots of relsize = sizeof(record->size)+record_max bytes, each = an 8-byte NATIVE-ENDIAN size header (>0 active, 0 empty/deleted) + record_max data
- a record at relative key N lives at slot (N-1). Keyed WRITE writes the slot (zero-filling gaps), status 22 on an occupied slot, 24 on key<1
- READ/REWRITE/DELETE address slot (key-1) -> 23 on an empty slot, 24 on key<1
- DELETE tombstones by zeroing the header and LEAVES the data
- sequential READ NEXT skips empty/deleted slots and is EOF (10) at end
- START positions by EQ/GE/GT/LE/LT/FIRST/LAST. Verified file bytes + per-op FILE STATUS + a READ NEXT scan

## Negative claims (8) — negative capability is the trust surface
- variable-length relative records
- the READ FIRST/LAST/PREVIOUS directions of read_next
- COB_READ_MASK option bits
- the SEQUENTIAL-access RELATIVE KEY write-back on append
- OPEN EXTEND / file-sharing and record locking
- the EXTFH external file handler path
- the actual fd lseek/read/write syscalls (declared OS boundary)
- lie prevented: a relative file is a packed array of records -- NO: each record is a fixed slot prefixed by an 8-byte native-endian length header, a DELETE only zeroes that header (the data bytes survive), an empty/deleted slot READs as status 23, and writing key N zero-fills the gap up to slot N-1

## Damage if overclaimed
mis-sizing the slot header (width/endianness) or treating a deleted slot as absent vs zero-data shifts or corrupts every relative record's addressing

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
