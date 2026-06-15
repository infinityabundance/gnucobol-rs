<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.INTRINSIC.INTEGER.1 — FUNCTION INTEGER (floor) / INTEGER-PART (truncate)

**Verdict: PASS** · replay `PASS=20 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.INTRINSIC.INTEGER.1` |
| court | FUNCTION INTEGER (floor) / INTEGER-PART (truncate) |
| crate_version | `0.7.80` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | FUNCTION INTEGER(x)=floor / INTEGER-PART(x)=trunc-toward-zero |
| replay command | `bash lab/oracle/integer_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- non-numeric argument
- i128 out-of-range magnitudes
- INTEGER and INTEGER-PART interchangeable
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
