<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.SCREENIO.COLOR.1 — SCREEN SECTION colour DISPLAY (FOREGROUND-COLOR/BACKGROUND-COLOR) -- the whole-screen ncurses repaint, native terminal bytes

**Verdict: PASS** · replay `PASS=11 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.SCREENIO.COLOR.1` |
| court | SCREEN SECTION colour DISPLAY (FOREGROUND-COLOR/BACKGROUND-COLOR) -- the whole-screen ncurses repaint, native terminal bytes |
| crate_version | `0.8.52` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | a colour-attributed positioned single-field DISPLAY (LINE>=2) -> the exact ncurses whole-screen colour-repaint byte stream, on the admitted terminal |
| replay command | `bash lab/oracle/screenio_color_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- the LINE==1 single-row-screen colour case (a different positioning edge)
- MULTIPLE colour fields in one DISPLAY -- the general doupdate/TransformLine line-diff across an arbitrary screen delta (overlapping same-row clr_eol)
- combined colour+monochrome attributes, colour on numeric-edited displays, ACCEPT-side colour
- any terminal other than TERM=xterm or ncurses other than the admitted 6.6 (terminfo-dependent)

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
