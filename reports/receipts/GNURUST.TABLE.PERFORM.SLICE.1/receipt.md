<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.TABLE.PERFORM.SLICE.1 — table (OCCURS) PERFORM VARYING slice

**Verdict: PASS** · replay `PASS=3 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.TABLE.PERFORM.SLICE.1` |
| court | table (OCCURS) PERFORM VARYING slice |
| crate_version | `0.8.33` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | PERFORM VARYING I over a 1-based OCCURS table accumulating TABLE(I) |
| replay command | `bash lab/oracle/table_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- multi-dimensional/nested OCCURS
- OCCURS DEPENDING ON
- subscript out-of-bounds
- INDEXED BY / SEARCH / SET
- signed/packed/V elements
- numeric SIZE ERROR
- non-sum per-element bodies
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
