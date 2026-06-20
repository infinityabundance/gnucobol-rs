<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.SCREENIO.DISPLAY.2 — SCREEN SECTION positioned DISPLAY -- the ncurses mvcur cursor-cost model (native terminal bytes)

**Verdict: PASS** · replay `PASS=70 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.SCREENIO.DISPLAY.2` |
| court | SCREEN SECTION positioned DISPLAY -- the ncurses mvcur cursor-cost model (native terminal bytes) |
| crate_version | `0.8.17` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | a positioned DISPLAY (LINE/COLUMN) -> the exact ncurses cursor-movement + field byte stream, across the swept position grid |
| replay command | `bash lab/oracle/screenio_grid_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- positions outside the swept LINE x COLUMN grid
- multi-field DISPLAY is sealed separately (GNURUST.SCREENIO.DISPLAY.3)
- color/attribute SGR, numeric-edited/JUSTIFIED display, ACCEPT input
- any terminal other than TERM=xterm or ncurses other than the admitted 6.6 (terminfo-dependent)

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
