<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.SCREENIO.ATTR.1 — SCREEN SECTION monochrome display attributes (HIGHLIGHT/LOWLIGHT/UNDERLINE/BLINK/REVERSE) -- native terminal bytes

**Verdict: PASS** · replay `PASS=10 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.SCREENIO.ATTR.1` |
| court | SCREEN SECTION monochrome display attributes (HIGHLIGHT/LOWLIGHT/UNDERLINE/BLINK/REVERSE) -- native terminal bytes |
| crate_version | `0.8.7` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | an attributed positioned DISPLAY -> the exact ncurses SGR-wrapped field byte stream, on the admitted terminal |
| replay command | `bash lab/oracle/screenio_attr_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- COLOUR attributes (FOREGROUND-COLOR/BACKGROUND-COLOR) -- the whole-screen colour repaint, sealed in GNURUST.SCREENIO.COLOR.1
- combined attributes, attributes on multi-field/numeric-edited displays, ACCEPT-side attributes
- any terminal other than TERM=xterm or ncurses other than the admitted 6.6 (terminfo-dependent)

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
