<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.18 — COMP-6 unsigned packed storage + MOVE

**Verdict: PASS** · replay `PASS=98 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.18` |
| court | COMP-6 unsigned packed storage + MOVE |
| crate_version | `0.8.31` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | field-storage + move-result bytes (COMP-6, unsigned packed) |
| replay command | `bash lab/oracle/comp6_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- signed COMP-6 (GnuCOBOL converts to COMP-3)
- COMP-6 arithmetic
- malformed packed bytes
- dialect portability (cobol85/2002 reject it)
- pre-3.2 behavior

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
