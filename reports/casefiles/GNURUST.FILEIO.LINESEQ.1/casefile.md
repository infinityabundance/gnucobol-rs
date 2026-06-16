<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.FILEIO.LINESEQ.1 (court-casefile)

**Verdict: PASS** · 8/8 pass, 0 fail · crate `gnucobol-rs` 0.7.84

- **Oracle:** cobc OPEN OUTPUT/WRITE LINE SEQUENTIAL under COB_LS_* env (libcob/fileio.c)
- **Byte domain(s):** OPEN OUTPUT + WRITE (LINE SEQUENTIAL) under COB_LS_FIXED/NULLS/VALIDATE -> appended file bytes + FILE STATUS (00 / 71)
- **Replay:** `bash lab/oracle/lineseq_write_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (5)
- a faithful port of libcob/fileio.c lineseq_size + lineseq_write (the bytes a WRITE appends to an ORGANIZATION IS LINE SEQUENTIAL file + the FILE STATUS), proven byte-identical to the admitted libcob across the COB_LS_* runtime config matrix (lineseq_write_sweep 8/0): default COB_LS_VALIDATE=1 strips trailing spaces, writes raw
- LF, but rejects any record byte < 0x20 with status 71 (writing nothing)
- COB_LS_VALIDATE=0 passes control bytes raw
- COB_LS_NULLS=1 emits 0x00 before each byte < 0x20
- COB_LS_FIXED=1 writes the full unstripped record area. FORENSIC: fileio.c's IS_BAD_CHAR macro excludes BS/ESC/FF/SI/TAB, but those exclusions are DEAD in the compiled 3.2 GA build -- every byte 0x00-0x1F is rejected (verified 0x00-0xFF). The configured-line-mode WRITE that GNURUST.FILE.WRITE.1 left not_proven

## Negative claims (9) — negative capability is the trust surface
- WRITE ADVANCING (opt != 0, the cob_*_write_opt family + LINAGE)
- the Windows CR/LF text-mode path (cob_ls_uses_cr, a platform boundary, off on Unix)
- COB_LS_VALIDATE>1 printable-check (COB_EXPERIMENTAL)
- CODE-SET conversion
- variable-length records
- lineseq_read / lineseq_rewrite
- record/relative/indexed organizations
- the actual fd/FILE* syscalls (declared OS boundary)
- lie prevented: the COB_LS_* line modes are cosmetic -- NO: COB_LS_FIXED keeps trailing spaces (changes the record length), COB_LS_NULLS injects 0x00 bytes (changes the byte stream), and default COB_LS_VALIDATE rejects control bytes outright (status 71, nothing written) -- each config produces a materially different file, and the source IS_BAD_CHAR exclusions do not match the compiled binary

## Damage if overclaimed
assuming one line-sequential byte image across configs corrupts framing under COB_LS_FIXED/NULLS or silently drops records the default validate would reject

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
