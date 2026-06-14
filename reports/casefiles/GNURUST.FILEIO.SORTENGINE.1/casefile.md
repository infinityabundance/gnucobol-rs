<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.FILEIO.SORTENGINE.1 (court-casefile)

**Verdict: PASS** · 1/1 pass, 0 fail · crate `gnucobol-rs` 0.7.64

- **Oracle:** cobc SORT ... USING/GIVING record order (libcob/fileio.c in-memory cobsort engine)
- **Byte domain(s):** a sequence of submitted records + SORT keys -> the retrieved (sorted) record sequence
- **Replay:** `bash lab/oracle/sortengine_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (4)
- a faithful 1:1 port of libcob/fileio.c's in-memory sort engine -- struct cobsort + cob_file_sort_init/init_key/submit/process/retrieve, cob_sort_queues, cob_new_item, cob_file_sort_using/giving/giving_internal, cob_free_list/cob_file_sort_close -- proven to produce byte-identical output ORDER to the admitted libcob's SORT GIVING file (sortengine_sweep 1/0 over a 16-record, full-key-duplicate, mixed ASCENDING/DESCENDING dataset). The C singly-linked queues (with the empty free-list) are modelled as an arena of items linked by next indices
- submit pushes each one-element block onto the shorter of queue[0]/queue[1]
- cob_sort_queues repeatedly merges adjacent runs across a 4-queue ping-pong (end_of_block delimiting runs) until one sorted run remains
- retrieve drains it. The full-key tie is broken by the per-record unique insertion counter, so the engine is a STABLE permutation. Cross-checked against the GNURUST.FILEIO.SORT.1 stable order and exercised across several merge rounds

## Negative claims (7) — negative capability is the trust surface
- the temp-file spill path (switch_to_file/files_used/cob_get_sort_tempfile/cob_read_item/cob_write_block/cob_create_tmpfile, taken only above COB_SORT_MEMORY -- the declared OS boundary)
- the EXTFH variants (cob_file_sort_using_extfh/giving_extfh)
- numeric-key ordering (a declared composition with GNURUST.NUMCMP.1)
- INPUT/OUTPUT PROCEDURE (RELEASE/RETURN)
- MERGE-sequence checking
- the actual fd tempfile syscalls
- lie prevented: a SORT is just any sort that puts records in key order -- NO: libcob uses a specific 4-queue natural merge whose equal-key tiebreak is the input order (the unique counter), so the output is the STABLE permutation, byte-reproducible record-for-record, not merely key-ordered

## Damage if overclaimed
claiming the whole sort engine when only the in-memory path is proven would hide the temp-file-spill and EXTFH paths, which a large SORT (above COB_SORT_MEMORY) actually takes

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
