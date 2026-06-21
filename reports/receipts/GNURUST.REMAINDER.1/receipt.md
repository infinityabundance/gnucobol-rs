<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.REMAINDER.1 — DIVIDE REMAINDER receiving-field bytes

**Verdict: PASS** · replay `PASS=768 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.REMAINDER.1` |
| court | DIVIDE REMAINDER receiving-field bytes |
| crate_version | `0.8.23` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | DIVIDE GIVING quotient + REMAINDER receiver field bytes (DISPLAY/COMP-3) |
| replay command | `bash lab/oracle/remainder_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- ON SIZE ERROR / NOT ON SIZE ERROR control flow
- divide-by-zero (fail-closed)
- COMPUTE / expression evaluation
- procedure control flow
- float
- binary/edited receivers
- business correctness

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
