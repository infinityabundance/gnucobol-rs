<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.FILEIO.RELATIVE.1 — RELATIVE organization keyed + sequential access

**Verdict: PASS** · replay `PASS=4 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.FILEIO.RELATIVE.1` |
| court | RELATIVE organization keyed + sequential access |
| crate_version | `0.8.7` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | OPEN + WRITE/READ/REWRITE/DELETE/START (RELATIVE) -> 8-byte LE header + record_max slots + FILE STATUS (00/22/23/24/10) |
| replay command | `bash lab/oracle/relative_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- variable-length relative records
- READ FIRST/LAST/PREVIOUS directions
- COB_READ_MASK option bits
- SEQUENTIAL-access RELATIVE KEY write-back on append
- OPEN EXTEND / file-sharing and record locking
- the EXTFH external file handler path
- the fd lseek/read/write syscalls (declared OS boundary)

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
