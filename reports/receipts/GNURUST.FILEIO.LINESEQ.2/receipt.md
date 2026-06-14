<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.FILEIO.LINESEQ.2 — line-sequential READ config matrix

**Verdict: PASS** · replay `PASS=12 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.FILEIO.LINESEQ.2` |
| court | line-sequential READ config matrix |
| crate_version | `0.7.55` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | OPEN INPUT + READ NEXT (LINE SEQUENTIAL) under COB_LS_VALIDATE/NULLS/SPLIT -> record bytes + FILE STATUS (00/04/06/09/10) |
| replay command | `bash lab/oracle/lineseq_read_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- multi-file open_next chain
- CODE-SET conversion (sort_collating)
- COB_LS_VALIDATE>1 printable check (COB_EXPERIMENTAL)
- COB_LS_NULLS error-recovery after status 71
- lineseq_rewrite
- record/relative/indexed organizations
- the fd/FILE* reads (declared OS boundary)

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
