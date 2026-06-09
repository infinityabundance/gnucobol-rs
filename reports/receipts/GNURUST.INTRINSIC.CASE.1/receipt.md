<!-- GENERATED from receipt.json by lab/receipt/run.py — DO NOT EDIT BY HAND.
     Regenerate: python3 lab/receipt/run.py generate -->
# GNURUST.INTRINSIC.CASE.1 — FUNCTION UPPER-CASE/LOWER-CASE/REVERSE

**Verdict: PASS** · replay `PASS=10 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.INTRINSIC.CASE.1` |
| court | FUNCTION UPPER-CASE/LOWER-CASE/REVERSE |
| crate_version | `0.7.25` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | ASCII case fold + byte reversal |
| replay command | `bash lab/oracle/case_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- locale/national case folding
- non-ASCII bytes folded
- multibyte REVERSE
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `python3 lab/receipt/run.py generate`.
