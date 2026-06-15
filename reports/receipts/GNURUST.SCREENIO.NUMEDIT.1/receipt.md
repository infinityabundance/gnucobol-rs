<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.SCREENIO.NUMEDIT.1 — SCREEN SECTION numeric-edited field DISPLAY (zero-suppression / sign / CR-DB positioning) -- native terminal bytes

**Verdict: PASS** · replay `PASS=14 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.SCREENIO.NUMEDIT.1` |
| court | SCREEN SECTION numeric-edited field DISPLAY (zero-suppression / sign / CR-DB positioning) -- native terminal bytes |
| crate_version | `0.7.80` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | a numeric-edited positioned single-field DISPLAY (edited PIC FROM a numeric source) -> the exact ncurses byte stream that skips leading blanks + writes the edited run, on the admitted terminal |
| replay command | `bash lab/oracle/screenio_numedit_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- the numeric editing itself (the move.c court; this proves the screen POSITIONING of an already-edited image)
- the `*` check-protection zero-fill rule (a separate edited.rs follow-on)
- a long (5+) trailing/interior blank run that ncurses cursor-skips rather than space-fills
- MULTIPLE edited fields in one DISPLAY (the general multi-field line-diff); numeric-edited colour/attributes; ACCEPT-side numeric-edited input
- any terminal other than TERM=xterm or ncurses other than the admitted 6.6

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
