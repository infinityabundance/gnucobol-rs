<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.FILEIO.SORT.1 — SORT/MERGE record comparison

**Verdict: PASS** · replay `PASS=1 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.FILEIO.SORT.1` |
| court | SORT/MERGE record comparison |
| crate_version | `0.8.33` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | SORT keys (offset/size/direction) over records -> the sorted record order (GIVING file bytes) |
| replay command | `bash lab/oracle/sort_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- numeric keys (composition with GNURUST.NUMCMP.1)
- the SORT engine tempfile merge / queues / chunking
- INPUT/OUTPUT PROCEDURE (RELEASE/RETURN)
- MERGE of multiple files
- DUPLICATES IN ORDER beyond the stable tiebreak
- the EXTFH sort path
- the fd tempfile syscalls (declared OS boundary)

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
