<!-- GENERATED from receipt.json by lab/receipt/run.py — DO NOT EDIT BY HAND.
     Regenerate: python3 lab/receipt/run.py generate -->
# GNURUST.ACCEPT.DISPLAY.2 — DISPLAY of signed/V-scaled numeric

**Verdict: PASS** · replay `PASS=8 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.ACCEPT.DISPLAY.2` |
| court | DISPLAY of signed/V-scaled numeric |
| crate_version | `0.7.18` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | DISPLAY numeric: signed +/- prefix + V decimal point |
| replay command | `bash lab/oracle/accept_display2_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- numeric-edited PICs (Z/,/*/$/CR/DB -> GNURUST.16)
- BLANK WHEN ZERO
- JUSTIFIED / floating-point USAGE
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `python3 lab/receipt/run.py generate`.
