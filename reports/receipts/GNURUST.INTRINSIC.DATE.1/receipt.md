<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.INTRINSIC.DATE.1 — date-conversion intrinsics

**Verdict: PASS** · replay `PASS=30 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.INTRINSIC.DATE.1` |
| court | date-conversion intrinsics |
| crate_version | `0.7.45` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | INTEGER-OF-DATE/DATE-OF-INTEGER/INTEGER-OF-DAY/DAY-OF-INTEGER (proleptic Gregorian, 1601-01-01=1) |
| replay command | `bash lab/oracle/date_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- invalid-date validation
- environment-sensitive CURRENT-DATE/WHEN-COMPILED
- business date arithmetic/Y2K windowing
- non-Gregorian calendars
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
