<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.FILEIO.VERB.1 (court-casefile)

**Verdict: PASS** · 7/7 pass, 0 fail · crate `gnucobol-rs` 0.7.55

- **Oracle:** cobc WRITE/READ/REWRITE/DELETE attempted in the wrong OPEN/ACCESS mode (libcob/fileio.c)
- **Byte domain(s):** cob_write/read/read_next/rewrite/delete/start preconditions -> FILE STATUS (43/44/46/47/48/49/23) before the organization handler
- **Replay:** `bash lab/oracle/verb_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (7)
- a faithful port of the precondition layer of libcob/fileio.c cob_write/cob_read/cob_read_next/cob_rewrite/cob_delete/cob_start -- the FILE STATUS a verb returns before dispatching to its organization handler, proven byte-identical to the admitted libcob (verb_sweep 7/0): WRITE needs OPEN OUTPUT/EXTEND in sequential access (else OUTPUT/I-O) otherwise 48
- READ and READ NEXT need OPEN INPUT/I-O otherwise 47
- REWRITE and DELETE need OPEN I-O otherwise 49
- a SEQUENTIAL-access REWRITE or DELETE without a prior successful READ is 43
- a sequential READ past end-of-file is 46
- START on a RANDOM-access or non-INPUT/I-O file is 47
- a record outside record_min..record_max is 44

## Negative claims (9) — negative capability is the trust surface
- the organization dispatch itself (sealed separately by the LINESEQ/SEQ/RELATIVE courts)
- the compile-time START-on-RANDOM rejection (cobc rejects it before runtime)
- LINE SEQUENTIAL validate-71 at the verb layer
- CODE-SET conversion
- the variable_record size resolution
- EOP / exception side effects
- the indexed suppressed-key skip
- the fd read/write syscalls (declared OS boundary)
- lie prevented: a COBOL verb runs regardless of how the file was opened -- NO: WRITE to an INPUT file is 48, READ from an OUTPUT file is 47, REWRITE/DELETE outside I-O is 49, a sequential REWRITE/DELETE without a prior READ is 43, and a sequential READ past EOF is 46 -- each verb checks the open and access mode first

## Damage if overclaimed
skipping the mode preconditions would let an illegal verb corrupt or read a file the dialect would have rejected with a 4x status

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
