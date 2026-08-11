<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.VALID-PROGRAMS.MANUAL.1 — valid-program corpus -- every GnuCOBOL manual code block is classified in both lanes (stable-3.2 + current examples.json + snippets.json)

**Verdict: PASS** · replay `PASS=1 FAIL=0 valid-manual`

| field | value |
|-------|-------|
| campaign | `GNURUST.VALID-PROGRAMS.MANUAL.1` |
| court | valid-program corpus -- every GnuCOBOL manual code block is classified in both lanes (stable-3.2 + current examples.json + snippets.json) |
| crate_version | `0.8.56` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | gnucobol-manual reports |
| replay command | `bash lab/oracle/../valid-corpus/corpus_court_sweep.sh valid-manual` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- partial snippets and pseudocode are not counted as executable programs

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
