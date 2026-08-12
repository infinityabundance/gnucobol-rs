<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.VALID-PROGRAMS.HELD-OUT.1 — valid-program corpus -- the held-out evaluation exists and states it was never used for implementation tuning (held-out-results.json)

**Verdict: PASS** · replay `PASS=1 FAIL=0 held-out`

| field | value |
|-------|-------|
| campaign | `GNURUST.VALID-PROGRAMS.HELD-OUT.1` |
| court | valid-program corpus -- the held-out evaluation exists and states it was never used for implementation tuning (held-out-results.json) |
| crate_version | `0.8.57` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | held-out-results.json |
| replay command | `bash lab/oracle/../valid-corpus/corpus_court_sweep.sh held-out` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- no held-out claim after the set has been used for implementation tuning

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
