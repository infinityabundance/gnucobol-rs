<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.VALID-PROGRAMS.ACCURACY.1 — valid-program corpus -- accuracy.json records the raw-byte accuracy dimensions (compile status, execution status, report bytes, raw stdout/stderr, generated files, return status)

**Verdict: PASS** · replay `PASS=1 FAIL=0 accuracy`

| field | value |
|-------|-------|
| campaign | `GNURUST.VALID-PROGRAMS.ACCURACY.1` |
| court | valid-program corpus -- accuracy.json records the raw-byte accuracy dimensions (compile status, execution status, report bytes, raw stdout/stderr, generated files, return status) |
| crate_version | `0.8.57` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | accuracy.json |
| replay command | `bash lab/oracle/../valid-corpus/corpus_court_sweep.sh accuracy` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- output normalization is never reported as raw-byte parity

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
