<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.FLOAT.1 — float fields COMP-1/COMP-2 + FLOAT-DECIMAL-16/34, both directions

**Verdict: FAIL** · replay `no-result`

| field | value |
|-------|-------|
| campaign | `GNURUST.FLOAT.1` |
| court | float fields COMP-1/COMP-2 + FLOAT-DECIMAL-16/34, both directions |
| crate_version | `0.7.80` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | decimal value <-> COMP-1/COMP-2/FLOAT-DECIMAL field bytes (truncate-toward-zero) |
| replay command | `bash lab/oracle/float_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- float arithmetic (composition; separate)
- cob_cmp_float epsilon comparison
- long double / FP_BIN32/64/128 usages
- Inf/NaN beyond decode-refusal

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
