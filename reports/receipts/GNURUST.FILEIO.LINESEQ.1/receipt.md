<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.FILEIO.LINESEQ.1 — line-sequential WRITE config matrix

**Verdict: PASS** · replay `PASS=8 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.FILEIO.LINESEQ.1` |
| court | line-sequential WRITE config matrix |
| crate_version | `0.7.80` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | OPEN OUTPUT + WRITE (LINE SEQUENTIAL) under COB_LS_FIXED/NULLS/VALIDATE -> appended bytes + FILE STATUS (00/71) |
| replay command | `bash lab/oracle/lineseq_write_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- WRITE ADVANCING / LINAGE (opt != 0)
- Windows CR/LF text-mode (cob_ls_uses_cr)
- COB_LS_VALIDATE>1 printable check (COB_EXPERIMENTAL)
- CODE-SET conversion
- variable-length records
- lineseq_read / lineseq_rewrite
- record/relative/indexed organizations
- the fd/FILE* syscalls (declared OS boundary)

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
