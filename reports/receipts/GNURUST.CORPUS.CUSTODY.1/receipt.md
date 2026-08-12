<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.CORPUS.CUSTODY.1 — valid-COBOL corpus custody -- every family report directory exists and the pre-change repository state was frozen (preflight-repository-state.json + before-state.json + integration-design.md)

**Verdict: PASS** · replay `PASS=1 FAIL=0 custody`

| field | value |
|-------|-------|
| campaign | `GNURUST.CORPUS.CUSTODY.1` |
| court | valid-COBOL corpus custody -- every family report directory exists and the pre-change repository state was frozen (preflight-repository-state.json + before-state.json + integration-design.md) |
| crate_version | `0.8.57` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | committed corpus evidence files under reports/valid-corpus/ |
| replay command | `bash lab/oracle/../valid-corpus/corpus_court_sweep.sh custody` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- no validity claim: custody only proves the evidence tree exists and was frozen

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
