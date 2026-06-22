<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.COMMON.NUMCHECK.1 — Not-numeric runtime diagnostic + field-type explanation

**Verdict: PASS** · replay `PASS=2 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.COMMON.NUMCHECK.1` |
| court | Not-numeric runtime diagnostic + field-type explanation |
| crate_version | `0.8.44` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | a non-numeric value reaching arithmetic -> the exact not-numeric diagnostic message bytes (verified vs both oracles 3.1.2 + 3.2) |
| replay command | `bash lab/oracle/numeric_check_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- the cob_is_numeric validity decision itself (this court reproduces the MESSAGE, taking the verdict as input)
- the libcob: prefix framing + the abort/exit
- NATIONAL/UTF-16 byte widths in the escaping

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
