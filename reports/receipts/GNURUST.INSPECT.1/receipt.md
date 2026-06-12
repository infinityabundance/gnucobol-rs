<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.INSPECT.1 — INSPECT byte effects + tally bytes

**Verdict: PASS** · replay `PASS=12 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.INSPECT.1` |
| court | INSPECT byte effects + tally bytes |
| crate_version | `0.7.39` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | INSPECT TALLYING/REPLACING/CONVERTING target bytes + tally receiver bytes |
| replay command | `bash lab/oracle/inspect_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- full Procedure Division execution
- locale/case-folding
- regex/pattern semantics
- national/UTF-8 multibyte
- unadmitted multi-clause ordering
- business validation
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
