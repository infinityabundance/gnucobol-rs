<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.VALUE.NEGZERO.EDGE.1 — negative-zero VALUE sign edge (oracle-characterized)

**Verdict: PASS** · replay `PASS=8 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.VALUE.NEGZERO.EDGE.1` |
| court | negative-zero VALUE sign edge (oracle-characterized) |
| crate_version | `0.7.46` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | VALUE-image negative-zero sign matrix; oracle rule + locked gnucobol-rs divergence (COMP-3 integer-form) |
| replay command | `bash lab/oracle/edge_negzero_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- the fix (characterizes + locks; patch is separate, not applied)
- arithmetic/MOVE negative-zero (GNURUST.13)
- figurative ZERO
- other usages/dialects
- broader signed-zero space

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
