<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.SCREENIO.DISPLAY.3 — SCREEN SECTION multi-field DISPLAY -- the general ncurses mvcur (inter-field moves), native terminal bytes

**Verdict: PASS** · replay `PASS=21 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.SCREENIO.DISPLAY.3` |
| court | SCREEN SECTION multi-field DISPLAY -- the general ncurses mvcur (inter-field moves), native terminal bytes |
| crate_version | `0.8.42` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | a multi-field DISPLAY -> the exact ncurses inter-field cursor-movement + field byte stream, for non-overlapping layouts |
| replay command | `bash lab/oracle/screenio_multi_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- overlapping same-row layouts (later field left of an earlier one) -- those need the curses refresh line-diff (clr_eol erase), a follow-on court
- 3+ field interactions beyond the swept pairs
- colour/attribute SGR (monochrome attributes are sealed in GNURUST.SCREENIO.ATTR.1), numeric-edited/JUSTIFIED display, ACCEPT input
- any terminal other than TERM=xterm or ncurses other than the admitted 6.6 (terminfo-dependent)

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
