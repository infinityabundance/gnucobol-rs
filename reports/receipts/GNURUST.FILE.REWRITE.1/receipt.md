<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.FILE.REWRITE.1 — sequential REWRITE in-place update

**Verdict: PASS** · replay `PASS=1 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.FILE.REWRITE.1` |
| court | sequential REWRITE in-place update |
| crate_version | `0.8.32` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | OPEN I-O + REWRITE -> record overwritten in place (same length), others unchanged |
| replay command | `bash lab/oracle/rewrite_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- LINE SEQUENTIAL REWRITE
- length-changing rewrites
- DELETE
- indexed/relative
- read-before-rewrite sequencing
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
