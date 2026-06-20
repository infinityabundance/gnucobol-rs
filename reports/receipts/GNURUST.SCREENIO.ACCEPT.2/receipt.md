<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.SCREENIO.ACCEPT.2 — SCREEN SECTION ACCEPT overflow input (typing past the field width: BEL + overwrite) -- native terminal bytes

**Verdict: FAIL** · replay `PASS=11 FAIL=0 (3.1.2 differential-matched=11)`

| field | value |
|-------|-------|
| campaign | `GNURUST.SCREENIO.ACCEPT.2` |
| court | SCREEN SECTION ACCEPT overflow input (typing past the field width: BEL + overwrite) -- native terminal bytes |
| crate_version | `0.8.12` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | a SCREEN SECTION ACCEPT of a width-1..6 field + printable input LONGER than the field -> the exact ncurses BEL/overwrite overflow byte stream, on the admitted terminal |
| replay command | `bash lab/oracle/screenio_accept2_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- width >= 7 fields (the `rep`-painted prompt + terminfo-internal reposition)
- field editing keys (arrows / backspace-during-input / function keys), numeric/USING-validation fields, multi-field ACCEPT, ACCEPT colour/attributes
- any terminal other than TERM=xterm or ncurses other than the admitted 6.6 (terminfo-dependent)

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
