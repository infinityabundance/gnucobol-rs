<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.CLASS.1 — class conditions IS NUMERIC/ALPHABETIC

**Verdict: PASS** · replay `PASS=46 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.CLASS.1` |
| court | class conditions IS NUMERIC/ALPHABETIC |
| crate_version | `0.7.76` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | alphanumeric field bytes -> class-condition truth |
| replay command | `bash lab/oracle/class_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- user-defined CLASS names
- national/UTF-8/DBCS classes
- locale collating sequence

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
