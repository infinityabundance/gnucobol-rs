<!-- GENERATED from receipt.json by lab/receipt/run.py — DO NOT EDIT BY HAND.
     Regenerate: python3 lab/receipt/run.py generate -->
# GNURUST.SIZE.ERROR.1 — arithmetic SIZE ERROR truncation + condition

**Verdict: PASS** · replay `PASS=12 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.SIZE.ERROR.1` |
| court | arithmetic SIZE ERROR truncation + condition |
| crate_version | `0.7.26` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | overflow -> low-order truncated store (no ON SIZE ERROR) + size-error condition |
| replay command | `bash lab/oracle/size_error_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- the arithmetic itself (GNURUST.7/13/19)
- ROUNDED
- intermediate-result precision
- SIZE ERROR on MOVE
- floating-point receivers
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `python3 lab/receipt/run.py generate`.
