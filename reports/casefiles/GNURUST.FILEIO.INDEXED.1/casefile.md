<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.FILEIO.INDEXED.1 (court-casefile)

**Verdict: PASS** · 18/18 pass, 0 fail · crate `gnucobol-rs` 0.7.78

- **Oracle:** cobc ORGANIZATION INDEXED RECORD KEY (libcob/fileio.c indexed_* handlers)
- **Byte domain(s):** INDEXED operations (WRITE/READ/READ NEXT/START/REWRITE/DELETE + record/file locks) over a primary key -> FILE STATUS + record bytes + key order
- **Replay:** `bash lab/oracle/indexed_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (6)
- a faithful port of the COBOL-observable behaviour of fileio.c's INDEXED handlers (indexed_open/indexed_write/indexed_write_internal/indexed_read/indexed_read_next/indexed_start/indexed_start_internal/indexed_rewrite/indexed_delete/indexed_delete_internal/indexed_file_delete/indexed_close) over a primary RECORD KEY, proven to match the admitted libcob's FILE STATUS + record bytes + key order (indexed_sweep 18/0): WRITE indexes a record by its primary key and rejects a duplicate primary key with 22 (and a non-ascending key under SEQUENTIAL access with 21)
- a random READ by key returns 00 + the record or 23 (not found)
- READ NEXT walks the keys in ascending order (10 at end)
- START positions the cursor by an =/</<=/>/>= condition (00, a following READ NEXT returns that record) or 23 when no key satisfies it
- REWRITE replaces an existing record (00) or returns 21 (ISAM KEY_INVALID) when the primary key is absent
- DELETE removes a record (00) or returns 23. Verified end-to-end against a fixed cobc ORGANIZATION INDEXED script (write incl duplicate, random read hit/miss, read-next sweep, START =/>/>=/<, rewrite, delete). The record/file LOCKING sub-layer (lock_record/unlock_record/test_record_lock/lock_file/unlock_file) is a faithful port of the NOWAIT grant/deny status contract: a record locked by another open is 51, a file 61, unit-tested with two contending opens over one lock environment

## Negative claims (8) — negative capability is the trust surface
- the on-disk index file format (BDB/VBISAM/EXTFH -- the declared OS boundary, the model is in-memory)
- ALTERNATE RECORD KEYs and WITH DUPLICATES (only the primary key is the court)
- READ PREVIOUS / descending traversal
- the actual cross-process BDB lock environment (52 deadlock and 30 permanent-error are BDB-internal)
- variable-length indexed records
- the bdb_*/EXTFH substrate functions
- concurrent multi-process contention beyond the in-process model
- lie prevented: an INDEXED file is just a keyed map -- NO: WRITE of a duplicate primary key is 22 (not an overwrite), REWRITE of an absent key is 21 (not 23), READ NEXT is strictly key-ordered, START positions by a relational condition, and a record locked by another open is 51

## Damage if overclaimed
treating the in-memory model as the whole INDEXED runtime would hide the on-disk index format, alternate keys, and real cross-process locking that operational indexed files depend on

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
