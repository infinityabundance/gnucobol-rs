<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.PERFORMANCE.FRONTEND.1 — performance corpus -- front-end only: per-phase candidate timings (preprocess/lex/parse/resolution/layout/check/prepare) vs oracle compile, measured separately (phase-metrics.json)

**Verdict: PASS** · replay `PASS=1 FAIL=0 performance`

| field | value |
|-------|-------|
| campaign | `GNURUST.PERFORMANCE.FRONTEND.1` |
| court | performance corpus -- front-end only: per-phase candidate timings (preprocess/lex/parse/resolution/layout/check/prepare) vs oracle compile, measured separately (phase-metrics.json) |
| crate_version | `0.8.57` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | performance/phase-metrics.json + views.json (View B) |
| replay command | `bash lab/oracle/../valid-corpus/corpus_court_sweep.sh performance` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- no native-code performance claim without a native candidate path; View A is labelled 'unlike workflows'

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
