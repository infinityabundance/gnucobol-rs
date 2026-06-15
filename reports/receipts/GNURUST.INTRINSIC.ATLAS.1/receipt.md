<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.INTRINSIC.ATLAS.1 — observed intrinsic-function atlas

**Verdict: PASS** · replay `PASS=19 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.INTRINSIC.ATLAS.1` |
| court | observed intrinsic-function atlas |
| crate_version | `0.7.67` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | declared intrinsic + input -> observed result (deterministic) or shape (env-sensitive) |
| replay command | `bash lab/oracle/intrinsic_atlas_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- not all intrinsics
- environment-sensitive values (CURRENT-DATE/WHEN-COMPILED)
- locale/collation
- national/UTF-8
- status is not an implementation
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
