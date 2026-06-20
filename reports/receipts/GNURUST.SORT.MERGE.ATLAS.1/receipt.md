<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.SORT.MERGE.ATLAS.1 — observed SORT/MERGE atlas

**Verdict: PASS** · replay `PASS=2 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.SORT.MERGE.ATLAS.1` |
| court | observed SORT/MERGE atlas |
| crate_version | `0.8.14` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | SORT reordering byte-effect: ASCENDING/DESCENDING KEY, USING/GIVING over an SD work file |
| replay command | `bash lab/oracle/sort_merge_atlas_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- SORT execution
- INPUT/OUTPUT PROCEDURE (RELEASE/RETURN)
- MERGE
- multiple keys
- sort stability
- custom collating sequence
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
