<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.SCREENIO.ACCEPT.2 (court-casefile)

**Verdict: PASS** · 11/11 pass, 0 fail · crate `gnucobol-rs` 0.7.82

- **Oracle:** cobc SCREEN SECTION ACCEPT with over-width input (libcob/screenio.c via ncurses field input), input fed then EOF, captured under a pty with TERM=xterm; DIFFERENTIALLY against GnuCOBOL 3.2 AND 3.1.2
- **Byte domain(s):** a SCREEN SECTION ACCEPT of a width-1..6 field + printable input LONGER than the field -> the exact ncurses BEL/overwrite overflow byte stream, on the admitted terminal
- **Replay:** `bash lab/oracle/screenio_accept2_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (3)
- a from-scratch, dependency-free Rust reproduction of the terminal byte stream when input EXCEEDS the ACCEPT field width -- WITHOUT linking ncurses -- proven byte-identical to GnuCOBOL AND DIFFERENTIALLY byte-identical to the 3.1.2 second oracle (screenio_accept2_sweep 11/0, 3.1.2-matched 11/11
- an offline 72-case width x col x overflow-input grid vs 3.2 + a 48-case 3.1.2 differential). Extends GNURUST.SCREENIO.ACCEPT.1: after the field fills and the cursor parks on the last cell, each excess key rings the bell `\a`
- if the key differs from the character currently shown in that cell it overwrites it (write the char, then a backspace to stay on the cell), otherwise only the bell is emitted (overwriting a cell with its own value is a no-op). The shown character starts as the last filled cell and updates on each overwrite (screenio::accept_field_and_stop overflow tail).

## Negative claims (7) — negative capability is the trust surface
- width >= 7 fields (the `rep`-painted prompt + terminfo-internal reposition)
- field editing keys (arrows / backspace-during-input / function keys)
- numeric or USING-validation fields
- multi-field ACCEPT
- ACCEPT colour/attributes
- any terminal other than TERM=xterm or ncurses other than the admitted 6.6
- lie prevented: ACCEPT overflow needs ncurses linked -- NO: the BEL + conditional-overwrite tail is a deterministic byte sequence reproduced exactly and cross-checked against TWO GnuCOBOL versions, no ncurses linked. It removes the overflow non-claim that GNURUST.SCREENIO.ACCEPT.1 had carved out

## Damage if overclaimed
claiming 'ACCEPT overflow works' would hide that only width<=6 alphanumeric overflow is proven -- the rep prompt, edit keys, numeric/USING validation, multi-field, and non-admitted terminals are not

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
