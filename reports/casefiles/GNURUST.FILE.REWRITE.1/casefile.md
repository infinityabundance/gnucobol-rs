<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.FILE.REWRITE.1 (court-casefile)

**Verdict: PASS** · 1/1 pass, 0 fail · crate `gnucobol-rs` 0.7.28

- **Oracle:** cobc OPEN I-O/REWRITE (libcob/fileio.c)
- **Byte domain(s):** OPEN I-O + REWRITE -> record overwritten in place (same length), others unchanged
- **Replay:** `bash lab/oracle/rewrite_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- the byte effect of OPEN I-O
- REWRITE on a fixed RECORD SEQUENTIAL file, matching cobc/libcob fileio.c (verified 1/0): REWRITE replaces the just-read record IN PLACE with the new content (space-padded to the record length), leaving every other record's bytes UNCHANGED -- rewriting records 0 and 2 of AAAABBBBCCCC yields X1X1BBBBZ3Z3. The in-place-update side of the file court (GNURUST.FILE.SEQUENTIAL.1 read, GNURUST.FILE.WRITE.1 write)

## Negative claims (7) — negative capability is the trust surface
- LINE SEQUENTIAL REWRITE
- length-changing rewrites
- DELETE
- indexed/relative organizations
- read-before-rewrite sequencing/status
- all dialects
- lie prevented: REWRITE rewrites the whole file -- NO: it overwrites ONLY the current record in place (same length); the bytes of every other record are untouched, so a REWRITE is a surgical single-record update not a file rewrite

## Damage if overclaimed
assuming REWRITE can change a record's length (it cannot for fixed records) or rewrites surrounding records corrupts a sequential file

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
