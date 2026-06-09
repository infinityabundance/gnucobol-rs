<!-- GENERATED from receipt.json by lab/receipt/run.py — DO NOT EDIT BY HAND.
     Regenerate: python3 lab/receipt/run.py generate -->
# GNURUST.FILE.REWRITE.1 — sequential REWRITE in-place update

**Verdict: PASS** · replay `PASS=1 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.FILE.REWRITE.1` |
| court | sequential REWRITE in-place update |
| crate_version | `0.7.18` |
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
> `python3 lab/receipt/run.py generate`.
