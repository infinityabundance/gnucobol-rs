<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.FILEIO.SEQ.1 (court-casefile)

**Verdict: PASS** · 9/9 pass, 0 fail · crate `gnucobol-rs` 0.7.53

- **Oracle:** cobc OPEN/WRITE/READ RECORD SEQUENTIAL (fixed + RECORD IS VARYING) under COB_VARSEQ_FORMAT (libcob/fileio.c)
- **Byte domain(s):** OPEN OUTPUT/INPUT + WRITE/READ (RECORD SEQUENTIAL) -> file bytes (fixed: full record; variable: cob_varseq_type prefix + data) + FILE STATUS
- **Replay:** `bash lab/oracle/seqrec_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (3)
- a faithful port of libcob/fileio.c sequential_write + sequential_read + set_sequential_variable_length (+ sequential_rewrite), proven byte-identical to the admitted libcob (seqrec_sweep 9/0): a FIXED record (record_min==record_max) writes the full record area with no delimiter and reads it back in record_max chunks (a short final read leaves the prior record's tail, EOF=10)
- a VARIABLE-LENGTH record (RECORD IS VARYING) is framed by a cob_varseq_type length prefix -- COB_VARSEQ_FORMAT 0/default = BE16(size)+0x0000 (4 bytes), 1 = BE32 (4 bytes), 2 = native LE32 (4 bytes), 3 = BE16 (2 bytes) -- all four verified on both WRITE (raw prefix bytes) and READ (status+size round-trip). The RECORD SEQUENTIAL org sibling of the line-sequential courts
- extends GNURUST.FILE.SEQUENTIAL.1/WRITE.1 with the variable-length framing

## Negative claims (7) — negative capability is the trust surface
- WRITE ADVANCING (cob_seq_write_opt)
- CODE-SET conversion
- the over-long record bytes_to_skip seek-past
- multi-file concatenation
- relative/indexed organizations
- the fd read/write/lseek syscalls (declared OS boundary)
- lie prevented: variable-length records are just data -- NO: each is framed by a length prefix whose width AND byte order depend on COB_VARSEQ_FORMAT (2 or 4 bytes; big-endian, little-endian, or BE16+NUL pad), and a fixed short-final read leaks the prior record's tail rather than zero/space filling

## Damage if overclaimed
mis-framing the varseq prefix (width/endianness) shifts every variable record; assuming a short final fixed record is space-filled corrupts the tail bytes

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
