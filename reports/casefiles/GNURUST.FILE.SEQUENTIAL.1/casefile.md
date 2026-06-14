<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.FILE.SEQUENTIAL.1 (court-casefile)

**Verdict: PASS** · 10/10 pass, 0 fail · crate `gnucobol-rs` 0.7.50

- **Oracle:** cobc OPEN INPUT/READ NEXT (libcob/fileio.c, program-shape)
- **Byte domain(s):** sequential READ record buffer bytes + file status
- **Replay:** `bash lab/oracle/seqfile_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (7)
- OPEN INPUT + repeated READ NEXT RECORD
- AT END over a flat byte buffer at a declared fixed record length, for ORGANIZATION RECORD SEQUENTIAL (fixed binary: record_len chunks status 00
- the FD buffer PERSISTS so a short final record overlays only the available bytes and LEAVES the prior record's tail
- the initial buffer is low-values 0x00) and LINE SEQUENTIAL (newline-delimited: short line space-padded 00
- a line longer than the record delivered record_len bytes at a time with status 06 until the remainder fits 00
- a trailing newline yields NO empty record
- a mid-file empty line IS a record), with file status 00/06/10 -- matching cobc/libcob byte-for-byte

## Negative claims (7) — negative capability is the trust surface
- indexed/relative/VSAM
- WRITE/REWRITE/START/DELETE
- OPEN I-O/EXTEND
- file-status codes beyond 00/06/10
- record locking
- Procedure Division control flow
- lie prevented: 'a sequential READ just gives you the record' -- a short final RECORD SEQUENTIAL read LEAKS the prior record's tail bytes (not spaces, not zeros), a long LINE SEQUENTIAL line is split across reads with status 06, and the initial buffer is low-values; the bytes and status both matter

## Damage if overclaimed
misreading a short final record (leaked prior-record tail) or a chunked long line, or branching on an unproven file status, silently corrupts a batch file pipeline

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
