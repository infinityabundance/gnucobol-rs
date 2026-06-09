<!-- GENERATED from receipt.json by lab/receipt/run.py — DO NOT EDIT BY HAND.
     Regenerate: python3 lab/receipt/run.py generate -->
# GNURUST.INTRINSIC.ORD-CHAR.1 — FUNCTION ORD/CHAR 1-based ordinal/char

**Verdict: PASS** · replay `PASS=15 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.INTRINSIC.ORD-CHAR.1` |
| court | FUNCTION ORD/CHAR 1-based ordinal/char |
| crate_version | `0.7.17` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | ORD(c)=byte+1 (1-based) / CHAR(n)=byte(n-1) |
| replay command | `bash lab/oracle/ordchar_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- non-default collating sequence
- national/UTF-8
- CHAR(n) out of 1..256
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `python3 lab/receipt/run.py generate`.
