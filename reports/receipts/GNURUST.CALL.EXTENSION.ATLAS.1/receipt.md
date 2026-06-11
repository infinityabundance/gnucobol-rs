<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.CALL.EXTENSION.ATLAS.1 — observed CALL/linkage atlas

**Verdict: PASS** · replay `PASS=5 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.CALL.EXTENSION.ATLAS.1` |
| court | observed CALL/linkage atlas |
| crate_version | `0.7.32` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | CALL parameter passing (BY REFERENCE shares / BY CONTENT copies) + C$ extensions + CANCEL + ON EXCEPTION |
| replay command | `bash lab/oracle/call_atlas_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- subprogram execution
- dynamic linking/.so loading
- C$ extension implementation
- BY VALUE to a reference param
- recursion/reentrancy
- CANCEL state machine
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
