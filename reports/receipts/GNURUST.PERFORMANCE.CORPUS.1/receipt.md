<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.PERFORMANCE.CORPUS.1 — performance corpus -- corpus throughput (View E: 10 workloads x 4 scales, one pass per lane) with peak memory and raw samples retained; unfavorable results are never discarded

**Verdict: PASS** · replay `PASS=1 FAIL=0 performance`

| field | value |
|-------|-------|
| campaign | `GNURUST.PERFORMANCE.CORPUS.1` |
| court | performance corpus -- corpus throughput (View E: 10 workloads x 4 scales, one pass per lane) with peak memory and raw samples retained; unfavorable results are never discarded |
| crate_version | `0.8.56` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | performance/views.json (View E) + raw/view_e.json |
| replay command | `bash lab/oracle/../valid-corpus/corpus_court_sweep.sh performance` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- no equivalence between compiled-native and interpreted runtime work

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
