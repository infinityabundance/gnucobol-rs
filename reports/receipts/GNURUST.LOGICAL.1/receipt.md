<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.LOGICAL.1 — bit-logical B-AND/B-OR/B-XOR/B-NOT + shifts

**Verdict: FAIL** · replay `no-result`

| field | value |
|-------|-------|
| campaign | `GNURUST.LOGICAL.1` |
| court | bit-logical B-AND/B-OR/B-XOR/B-NOT + shifts |
| crate_version | `0.7.75` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | (value0, value1) -> u64 bitwise via |value| mod 2^64 |
| replay command | `bash lab/oracle/logical_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- fractional operands (scale dropped)
- operands beyond 64 bits (truncated)
- cob_logical_left_c/right_c size-bounded variants

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
