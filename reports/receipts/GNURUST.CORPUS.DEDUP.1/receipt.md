<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.CORPUS.DEDUP.1 — valid-COBOL corpus deduplication -- deduplication.json records exact + near-duplicate evidence; grouping is repository-level so partitions never split a repo

**Verdict: PASS** · replay `PASS=1 FAIL=0 dedup`

| field | value |
|-------|-------|
| campaign | `GNURUST.CORPUS.DEDUP.1` |
| court | valid-COBOL corpus deduplication -- deduplication.json records exact + near-duplicate evidence; grouping is repository-level so partitions never split a repo |
| crate_version | `0.8.56` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | deduplication.json + the xcobol dedup report |
| replay command | `bash lab/oracle/../valid-corpus/corpus_court_sweep.sh dedup` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- no independent-program count that includes duplicates

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
