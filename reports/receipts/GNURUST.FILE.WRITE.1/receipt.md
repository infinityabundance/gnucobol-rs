<!-- GENERATED from receipt.json by lab/receipt/run.py — DO NOT EDIT BY HAND.
     Regenerate: python3 lab/receipt/run.py generate -->
# GNURUST.FILE.WRITE.1 — sequential WRITE byte effects

**Verdict: PASS** · replay `PASS=2 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.FILE.WRITE.1` |
| court | sequential WRITE byte effects |
| crate_version | `0.7.21` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | OPEN OUTPUT + WRITE -> file bytes (RECORD SEQ full padded / LINE SEQ trailing-space-stripped + LF) |
| replay command | `bash lab/oracle/write_seq_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- COB_LS_FIXED/COB_LS_NULLS line modes
- variable-length records
- WRITE ADVANCING/BEFORE/AFTER
- REWRITE
- indexed/relative organizations
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `python3 lab/receipt/run.py generate`.
