<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.ACCEPT.DISPLAY.1 — DISPLAY emitted text + ACCEPT field bytes

**Verdict: PASS** · replay `PASS=7 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.ACCEPT.DISPLAY.1` |
| court | DISPLAY emitted text + ACCEPT field bytes |
| crate_version | `0.8.33` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | DISPLAY operand concatenation + newline; ACCEPT field move bytes |
| replay command | `bash lab/oracle/accept_display_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- DISPLAY of signed/V/edited numeric (reformats)
- DISPLAY UPON / WITH NO ADVANCING
- ACCEPT FROM DATE/TIME/environment/screen
- device/console specifics
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
