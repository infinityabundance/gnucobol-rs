<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.FILEIO.OPEN.1 (court-casefile)

**Verdict: PASS** · 2/2 pass, 0 fail · crate `gnucobol-rs` 0.8.49

- **Oracle:** cobc OPEN/WRITE/READ/CLOSE incl. double-open, close-when-closed, CLOSE WITH LOCK, missing-input (libcob/fileio.c)
- **Byte domain(s):** OPEN/WRITE/READ NEXT/CLOSE over a CobFile -> the on-disk file image bytes + open/close FILE STATUS (00/05/31/35/38/41/42)
- **Replay:** `bash lab/oracle/open_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (5)
- a CobFile runtime that ties the sealed organization handlers into a working file: a faithful port of cob_open/cob_close/cob_pre_open/cob_unlock/cob_file_unlock/cob_unlock_file/cob_commit/cob_rollback/cob_delete_file, proven to match the admitted libcob's FILE STATUS and the on-disk file image (open_sweep 2/0, end-to-end against an OPEN OUTPUT/WRITE/READ/CLOSE LINE SEQUENTIAL program): cob_open loads the file image (real std::fs I/O) and sets status -- a file closed-with-lock is 38, an already-open file 41, an empty/badly-quoted filename 31, OPEN INPUT of a missing file 35 (or 05 when OPTIONAL), OPEN OUTPUT truncates
- cob_close flushes an output/I-O image and is 42 when the file is not open, leaving it Locked for CLOSE WITH LOCK
- WRITE/READ NEXT dispatch by organization to the sealed sequential_*/lineseq_*/relative_* handlers
- cob_unlock/cob_commit/cob_rollback release locks (no-op success without locking configured)
- cob_delete_file removes the file (00/30). The integration layer over GNURUST.FILEIO.LINESEQ/SEQ/RELATIVE

## Negative claims (9) — negative capability is the trust surface
- the indexed/BDB and SORT organizations' open/close (substrate boundary)
- record/file locking semantics beyond no-op success
- OPEN EXTEND positioning details
- the file-sharing modes
- concat (multi-file) input
- the EXTFH open path
- cob_cache_file / file_cache list management
- the actual fd open/close syscalls (declared OS boundary)
- lie prevented: OPEN/CLOSE always succeed -- NO: re-opening an open file is 41, closing a closed file is 42, opening a file that was CLOSE WITH LOCK is 38, a missing OPEN INPUT is 35 (05 if OPTIONAL), and an empty/bad-quoted name is 31; CLOSE of an output file flushes the written image to disk

## Damage if overclaimed
ignoring the OPEN/CLOSE status (41/42/38/35) lets a program write to an unopened file or silently re-open, corrupting the file or its sequence

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
