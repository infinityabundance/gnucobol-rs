<!-- GENERATED from receipt.json by lab/receipt/run.py — DO NOT EDIT BY HAND.
     Regenerate: python3 lab/receipt/run.py generate -->
# GNURUST.17 — cp500 EBCDIC zoned-decimal numeric DISPLAY decode

**Verdict: PASS** · replay `PASS=120 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.17` |
| court | cp500 EBCDIC zoned-decimal numeric DISPLAY decode |
| crate_version | `0.7.3` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | raw EBCDIC zoned field bytes -> value |
| replay command | `bash lab/oracle/ebcdic_num_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- cp037 / other code pages
- edited-numeric under cp500
- binary/packed via this path
- mixed/auto-detect encoding
- national/DBCS

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `python3 lab/receipt/run.py generate`.
