<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.INTRINSIC.NUMVAL.1 — FUNCTION NUMVAL numeric parse

**Verdict: PASS** · replay `PASS=14 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.INTRINSIC.NUMVAL.1` |
| court | FUNCTION NUMVAL numeric parse |
| crate_version | `0.7.51` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | FUNCTION NUMVAL(narrow numeric string) -> parsed value |
| replay command | `bash lab/oracle/numval_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- NUMVAL-C (currency/thousands)
- locale decimal/comma swap
- national/UTF-8
- malformed-input error semantics
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
