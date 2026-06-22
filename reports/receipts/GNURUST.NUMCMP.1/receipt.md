<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.NUMCMP.1 — numeric comparison cob_numeric_cmp (on the Mpz/cob_decimal 1:1 layer)

**Verdict: FAIL** · replay `no-result`

| field | value |
|-------|-------|
| campaign | `GNURUST.NUMCMP.1` |
| court | numeric comparison cob_numeric_cmp (on the Mpz/cob_decimal 1:1 layer) |
| crate_version | `0.8.43` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | two numeric field byte images -> -1/0/1 ordering verdict |
| replay command | `bash lab/oracle/numcmp_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- float-vs-float epsilon edges
- optimized cmp_packed/uint/llint fast-path code (verdict only)
- negative-scale packed compare
- National/DBCS comparison

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
