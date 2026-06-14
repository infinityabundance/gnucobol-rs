<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.ROUND.1 — ROUNDED MODE IS (all eight rounding modes, cob_decimal + packed BCD paths)

**Verdict: FAIL** · replay `no-result`

| field | value |
|-------|-------|
| campaign | `GNURUST.ROUND.1` |
| court | ROUNDED MODE IS (all eight rounding modes, cob_decimal + packed BCD paths) |
| crate_version | `0.7.49` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | value + target scale + ROUNDED mode + receiver path -> stored field bytes |
| replay command | `bash lab/oracle/round_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- bignum values beyond i128 (38-digit intermediates)
- floating-point COMP-1/COMP-2 rounding
- PROHIBITED size error returns a typed error, not bytes

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
