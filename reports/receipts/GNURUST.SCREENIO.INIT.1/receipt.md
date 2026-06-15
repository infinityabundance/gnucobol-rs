<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.SCREENIO.INIT.1 — SCREEN SECTION init/teardown framing + positioned DISPLAY (native terminal bytes)

**Verdict: PASS** · replay `PASS=1 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.SCREENIO.INIT.1` |
| court | SCREEN SECTION init/teardown framing + positioned DISPLAY (native terminal bytes) |
| crate_version | `0.7.81` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | a positioned SCREEN SECTION DISPLAY -> the exact ncurses terminal byte stream, on the admitted terminal |
| replay command | `bash lab/oracle/screenio_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- the ncurses mvcur cursor-cost model is sealed separately (GNURUST.SCREENIO.DISPLAY.2)
- multi-field layout, color/attribute SGR, numeric-edited/JUSTIFIED display
- ACCEPT input / key handling / cursor navigation
- any terminal other than TERM=xterm or any ncurses build other than the admitted 6.6 (the byte stream is terminfo-dependent)

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
