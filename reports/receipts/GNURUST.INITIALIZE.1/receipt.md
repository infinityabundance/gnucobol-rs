<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.INITIALIZE.1 — INITIALIZE receiver byte effects

**Verdict: PASS** · replay `PASS=6 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.INITIALIZE.1` |
| court | INITIALIZE receiver byte effects |
| crate_version | `0.8.52` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | INITIALIZE record -> changed/preserved receiver bytes (elementary/group/FILLER/REDEFINES/OCCURS/VALUE) |
| replay command | `bash lab/oracle/initialize_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- full Procedure Division execution
- INITIALIZE REPLACING/TO VALUE/WITH FILLER
- numeric-edited/JUSTIFIED/BLANK WHEN ZERO
- ODO runtime active count
- active REDEFINES view
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
