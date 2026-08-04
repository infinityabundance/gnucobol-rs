<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.SCREENIO.ACCEPT.1 — SCREEN SECTION ACCEPT of an alphanumeric input field (prompt / reposition / echo / field-full) -- native terminal bytes

**Verdict: PASS** · replay `PASS=12 FAIL=0 (3.1.2 differential-matched=12)`

| field | value |
|-------|-------|
| campaign | `GNURUST.SCREENIO.ACCEPT.1` |
| court | SCREEN SECTION ACCEPT of an alphanumeric input field (prompt / reposition / echo / field-full) -- native terminal bytes |
| crate_version | `0.8.55` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | a SCREEN SECTION ACCEPT of a width-1..6 alphanumeric field + the printable input (<= width) -> the exact ncurses prompt/reposition/echo/field-full byte stream, on the admitted terminal |
| replay command | `bash lab/oracle/screenio_accept_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- width >= 7 fields (the `rep`-painted prompt + terminfo-internal post-rep reposition)
- OVERFLOW input (typing past the field width -- BEL + overwrite, a separate input-editing state machine)
- field editing keys (arrows / backspace-during-input / function keys), numeric/USING-validation fields, multi-field ACCEPT, ACCEPT colour/attributes
- any terminal other than TERM=xterm or ncurses other than the admitted 6.6 (terminfo-dependent)

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
