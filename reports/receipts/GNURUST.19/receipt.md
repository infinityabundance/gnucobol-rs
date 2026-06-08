<!-- GENERATED from receipt.json by lab/receipt/run.py — DO NOT EDIT BY HAND.
     Regenerate: python3 lab/receipt/run.py generate -->
# GNURUST.19 — DIVIDE receiving-field bytes

**Verdict: PASS** · replay `PASS=736 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.19` |
| court | DIVIDE receiving-field bytes |
| crate_version | `0.7.1` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | DIVIDE GIVING receiver field bytes (DISPLAY/COMP-3) |
| replay command | `bash lab/oracle/divide_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- divide-by-zero / ON SIZE ERROR
- REMAINDER
- COMPUTE / expression evaluation
- procedure control flow
- float
- binary/edited receivers
- business correctness

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `python3 lab/receipt/run.py generate`.
