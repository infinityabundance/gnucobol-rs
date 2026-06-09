<!-- GENERATED from receipt.json by lab/receipt/run.py — DO NOT EDIT BY HAND.
     Regenerate: python3 lab/receipt/run.py generate -->
# GNURUST.15 — cp500 EBCDIC DISPLAY decode

**Verdict: PASS** · replay `PASS=256 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.15` |
| court | cp500 EBCDIC DISPLAY decode |
| crate_version | `0.7.11` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | raw EBCDIC field bytes -> decoded text |
| replay command | `bash lab/oracle/ebcdic_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- cp037 / other code pages
- numeric EBCDIC zoned sign
- national/DBCS
- collation
- mixed/auto-detect encoding

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `python3 lab/receipt/run.py generate`.
