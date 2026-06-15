<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.FILEIO.LINESEQ.2 (court-casefile)

**Verdict: PASS** · 12/12 pass, 0 fail · crate `gnucobol-rs` 0.7.74

- **Oracle:** cobc OPEN INPUT/READ NEXT LINE SEQUENTIAL under COB_LS_* env (libcob/fileio.c)
- **Byte domain(s):** OPEN INPUT + READ NEXT (LINE SEQUENTIAL) under COB_LS_VALIDATE/NULLS/SPLIT -> record area bytes + FILE STATUS (00/04/06/09/10)
- **Replay:** `bash lab/oracle/lineseq_read_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (7)
- a faithful port of libcob/fileio.c lineseq_read (the record bytes
- FILE STATUS produced by READ ... NEXT RECORD over a LINE SEQUENTIAL file), proven byte-identical to the admitted libcob across the COB_LS_* matrix (lineseq_read_sweep 12/0): a line ends at \n, \r\n folds to \n, a lone \r is kept as data
- default COB_LS_VALIDATE flags an IS_BAD_CHAR byte with status 09 (line still read)
- COB_LS_NULLS decodes a 0x00-prefixed control byte
- a line longer than the record is split into 06+00 records (COB_LS_SPLIT on) or truncated to status 04 with the rest discarded (split off)
- short lines are space-filled (00), a trailing newline makes no empty record, a mid-file empty line is a record, EOF with no bytes is status 10. FORENSIC ASYMMETRY: IS_BAD_CHAR's BS/TAB/FF/SI/ESC exclusions are LIVE on READ (those five -> status 00) but DEAD on WRITE (every byte < 0x20 -> status 71)
- verified 0x00-0x1F on both paths. The READ side of GNURUST.FILEIO.LINESEQ.1

## Negative claims (8) — negative capability is the trust surface
- the multi-file open_next chain
- CODE-SET conversion (sort_collating)
- COB_LS_VALIDATE>1 printable-check (COB_EXPERIMENTAL)
- the COB_LS_NULLS error-recovery path after status 71
- lineseq_rewrite
- record/relative/indexed organizations
- the fd/FILE* reads (declared OS boundary)
- lie prevented: READ just splits on newline -- NO: \r\n folds but a lone \r is data, a long line splits (06) or truncates (04) by COB_LS_SPLIT, a bad control byte raises status 09 while the line is still delivered, and -- unlike WRITE -- TAB/BS/FF/SI/ESC are accepted (the IS_BAD_CHAR exclusions are live on read, dead on write)

## Damage if overclaimed
mis-handling the split (06 vs 04), the \r\n fold, or the read-vs-write bad-char asymmetry corrupts record framing or wrongly accepts/rejects control data

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
