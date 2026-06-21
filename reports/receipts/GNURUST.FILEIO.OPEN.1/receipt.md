<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.FILEIO.OPEN.1 — file runtime OPEN/CLOSE + lifecycle

**Verdict: PASS** · replay `PASS=2 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.FILEIO.OPEN.1` |
| court | file runtime OPEN/CLOSE + lifecycle |
| crate_version | `0.8.22` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | OPEN/WRITE/READ NEXT/CLOSE over a CobFile -> on-disk file image bytes + open/close FILE STATUS (00/05/31/35/38/41/42) |
| replay command | `bash lab/oracle/open_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- indexed/BDB and SORT open/close (substrate boundary)
- record/file locking beyond no-op success
- OPEN EXTEND positioning details
- file-sharing modes
- concat (multi-file) input
- the EXTFH open path
- cob_cache_file / file_cache list management
- the actual fd open/close syscalls (declared OS boundary)

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
