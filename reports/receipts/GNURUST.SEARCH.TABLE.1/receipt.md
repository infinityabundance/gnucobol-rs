<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.SEARCH.TABLE.1 — SEARCH / SEARCH ALL table lookup

**Verdict: PASS** · replay `PASS=6 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.SEARCH.TABLE.1` |
| court | SEARCH / SEARCH ALL table lookup |
| crate_version | `0.7.33` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | the 1-based landing index of SEARCH (serial, forward-from-index) / SEARCH ALL (binary on ascending key) |
| replay command | `bash lab/oracle/search_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- multi-key/DESCENDING keys
- alphanumeric/signed/V keys
- VARYING/AT END/WHEN control flow (only the landing index)
- SEARCH ALL on an unsorted table
- OCCURS DEPENDING ON
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
