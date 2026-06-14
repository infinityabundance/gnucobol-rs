<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.FILE.FILTER.SLICE.1 — filter (conditional) read-loop

**Verdict: PASS** · replay `PASS=4 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.FILE.FILTER.SLICE.1` |
| court | filter (conditional) read-loop |
| crate_version | `0.7.60` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | read-loop with a per-record IF gating the accumulation -> resulting WORKING-STORAGE |
| replay command | `bash lab/oracle/file_filter_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- compound conditions (AND/OR)
- signed/packed numeric filter
- per-record mutation/transform (only accumulate)
- multi-branch EVALUATE filter
- indexed/relative
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
