<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.FILEIO.SORTENGINE.1 — SORT/MERGE in-memory engine (4-queue natural merge)

**Verdict: PASS** · replay `PASS=1 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.FILEIO.SORTENGINE.1` |
| court | SORT/MERGE in-memory engine (4-queue natural merge) |
| crate_version | `0.8.51` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | a sequence of submitted records + SORT keys -> the retrieved (sorted) record sequence |
| replay command | `bash lab/oracle/sortengine_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- the temp-file spill path (switch_to_file/files_used, above COB_SORT_MEMORY -- declared OS boundary)
- the EXTFH variants (cob_file_sort_using_extfh/giving_extfh)
- numeric-key ordering (a declared composition with GNURUST.NUMCMP.1)
- INPUT/OUTPUT PROCEDURE (RELEASE/RETURN)
- MERGE-sequence checking
- the actual fd tempfile syscalls

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
