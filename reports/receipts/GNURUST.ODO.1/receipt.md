<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.ODO.1 — OCCURS DEPENDING ON used length + bounded access

**Verdict: PASS** · replay `PASS=10 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.ODO.1` |
| court | OCCURS DEPENDING ON used length + bounded access |
| crate_version | `0.7.69` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | controlling value + table -> used length / active element bytes |
| replay command | `bash lab/oracle/odo_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- access beyond active count (fail-closed)
- nested/2-D ODO
- SET controlling mid-access
- signed/packed elements

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
