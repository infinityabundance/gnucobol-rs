<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.DIALECT.RUNTIME.ATLAS.1 — observed dialect-runtime divergence atlas

**Verdict: PASS** · replay `PASS=7 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.DIALECT.RUNTIME.ATLAS.1` |
| court | observed dialect-runtime divergence atlas |
| crate_version | `0.7.38` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | cross-dialect divergence: stored zoned-sign bytes (invariant) vs DISPLAY presentation sign placement (leading/trailing camps) vs compile-acceptance of extensions |
| replay command | `bash lab/oracle/dialect_runtime_atlas_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- non-default dialect implementation
- vendor-compiler parity (IBM/MF/MVS/BS2000/RM/GCOS)
- a -std mode being the vendor dialect itself
- complete enumeration of every cross-dialect divergence
- decoding the DISPLAY presentation form back to a value
- screen-routed dialects (acu/realia)
- runtime portability across platforms

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
