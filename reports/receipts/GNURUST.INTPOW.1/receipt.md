<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.INTPOW.1 — integer exponentiation cob_s32_pow/cob_s64_pow

**Verdict: FAIL** · replay `no-result`

| field | value |
|-------|-------|
| campaign | `GNURUST.INTPOW.1` |
| court | integer exponentiation cob_s32_pow/cob_s64_pow |
| crate_version | `0.8.16` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | (base, power, width) -> integer result (wrapping) |
| replay command | `bash lab/oracle/pow_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- decimal ** (cob_decimal_pow is intrinsic.c)
- fractional exponents
- 0 ** negative (fails closed instead of SIGFPE)

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
