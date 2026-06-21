<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.DECLARATIVES.ATLAS.1 — observed DECLARATIVES / USE error-handler atlas

**Verdict: PASS** · replay `PASS=5 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.DECLARATIVES.ATLAS.1` |
| court | observed DECLARATIVES / USE error-handler atlas |
| crate_version | `0.8.22` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | DECLARATIVES/USE runtime control: which op fires the handler, per-file binding, FILE STATUS visibility inside, resume-after-handler |
| replay command | `bash lab/oracle/declaratives_atlas_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- executing a declarative
- USE FOR DEBUGGING
- non-file (arithmetic) exceptions
- GLOBAL declaratives across nested programs
- multi-declarative precedence ordering
- resume-vs-terminate for non-file exceptions
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
