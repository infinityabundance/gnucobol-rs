<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.INTRINSIC.NUMVAL-C.1 — FUNCTION NUMVAL-C currency parse

**Verdict: PASS** · replay `PASS=10 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.INTRINSIC.NUMVAL-C.1` |
| court | FUNCTION NUMVAL-C currency parse |
| crate_version | `0.8.41` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | FUNCTION NUMVAL-C(currency string) -> value (strip $ + thousands commas) |
| replay command | `bash lab/oracle/numvalc_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- non-default currency symbol (2-arg form)
- DECIMAL-POINT IS COMMA / locale comma-decimal
- national/UTF-8
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
