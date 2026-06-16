<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.SCREENIO.LINEDIFF.1 — multi-DISPLAY same-row refresh line-diff (ncurses doupdate/TransformLine) -- native terminal bytes

**Verdict: FAIL** · replay `PASS=19 FAIL=0 (3.1.2 differential-matched=19)`

| field | value |
|-------|-------|
| campaign | `GNURUST.SCREENIO.LINEDIFF.1` |
| court | multi-DISPLAY same-row refresh line-diff (ncurses doupdate/TransformLine) -- native terminal bytes |
| crate_version | `0.7.83` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | two same-row DISPLAY statements -> the exact ncurses doupdate/TransformLine refresh byte stream (reposition + changed run + clr_eol trailing-erase), on the admitted terminal |
| replay command | `bash lab/oracle/screenio_linediff_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- THREE or more same-row DISPLAYs (clear-to-EOL batches differently across 3+ refreshes)
- distant isolated survivors needing a jump + single space; the MULTI-ROW diff (vertical mvcur between changed rows)
- attributes/colour on the diffed text
- any terminal other than TERM=xterm or ncurses other than the admitted 6.6 (terminfo-dependent)

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
