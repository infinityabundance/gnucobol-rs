<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.BIGNUM.1 — MULTIPLY beyond i128 (exact 256-bit product)

**Verdict: FAIL** · replay `no-result`

| field | value |
|-------|-------|
| campaign | `GNURUST.BIGNUM.1` |
| court | MULTIPLY beyond i128 (exact 256-bit product) |
| crate_version | `0.7.67` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | overflowing i128 product -> receiver bytes (low-digit truncation + ROUNDED) |
| replay command | `bash lab/oracle/bignum_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- ADD/SUBTRACT upscale overflow
- DIVIDE overflow
- chained-COMPUTE single-expression GMP precision
- a single operand exceeding 38 digits

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
