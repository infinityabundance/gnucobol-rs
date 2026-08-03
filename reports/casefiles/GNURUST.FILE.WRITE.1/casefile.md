<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.FILE.WRITE.1 (court-casefile)

**Verdict: PASS** · 2/2 pass, 0 fail · crate `gnucobol-rs` 0.8.53

- **Oracle:** cobc OPEN OUTPUT/WRITE (libcob/fileio.c)
- **Byte domain(s):** OPEN OUTPUT + WRITE -> file bytes (RECORD SEQ full padded / LINE SEQ trailing-space-stripped + LF)
- **Replay:** `bash lab/oracle/write_seq_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- the file bytes produced by OPEN OUTPUT + repeated WRITE over fixed records, matching cobc/libcob fileio.c (verified 2/0): RECORD SEQUENTIAL writes the full fixed record_len-wide record area (space-padded) with NO delimiter
- LINE SEQUENTIAL STRIPS trailing spaces and appends a newline (an all-spaces record writes just a newline). The WRITE side of GNURUST.FILE.SEQUENTIAL.1 (the READ court)

## Negative claims (6) — negative capability is the trust surface
- variable-length records
- WRITE ADVANCING/BEFORE/AFTER
- REWRITE
- indexed/relative organizations
- all dialects (COB_LS_FIXED/COB_LS_NULLS line modes now sealed by GNURUST.FILEIO.LINESEQ.1)
- lie prevented: WRITE just dumps the record bytes -- NO: LINE SEQUENTIAL STRIPS trailing spaces and adds a newline (so an X(8) holding AB writes 3 bytes, and an all-spaces record writes just a newline), while RECORD SEQUENTIAL writes the FULL fixed padded record with no delimiter -- the two organizations produce different bytes for the same record

## Damage if overclaimed
assuming LINE SEQUENTIAL preserves trailing spaces (or that RECORD SEQUENTIAL adds a newline) corrupts every written record's framing

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
