<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.SCREENIO.ACCEPT.1 (court-casefile)

**Verdict: PASS** · 12/12 pass, 0 fail · crate `gnucobol-rs` 0.7.79

- **Oracle:** cobc SCREEN SECTION ACCEPT of an alphanumeric USING field (libcob/screenio.c via ncurses field input), input fed then EOF, captured under a pty with TERM=xterm; DIFFERENTIALLY against GnuCOBOL 3.2 AND 3.1.2
- **Byte domain(s):** a SCREEN SECTION ACCEPT of a width-1..6 alphanumeric field + the printable input (<= width) -> the exact ncurses prompt/reposition/echo/field-full byte stream, on the admitted terminal
- **Replay:** `bash lab/oracle/screenio_accept_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (3)
- a from-scratch, dependency-free Rust reproduction of the terminal byte stream of a SCREEN SECTION ACCEPT of a single alphanumeric USING field -- WITHOUT linking ncurses -- proven byte-identical to GnuCOBOL, AND DIFFERENTIALLY byte-identical to the 3.1.2 second oracle (screenio_accept_sweep 12/0, 3.1.2-matched 12/12
- an offline 336-capture width x col x line x input grid vs 3.2 + a 56-case 3.1.2 differential). The stream is: position at the field start (shared mvcur), paint the field as W underscores (the ncurses input prompt), reposition to the field start (a same-row backward move -- HPA \e[<col>G / backspaces / CR+spaces, shortest-wins, HPA before backspaces before CR on a tie, screenio::accept_reposition), echo the typed characters (up to W), and -- if the input fills the field -- a single field-full backspace
- on EOF the implicit STOP RUN pause is skipped, straight to teardown (screenio::accept_field_and_stop).

## Negative claims (8) — negative capability is the trust surface
- width >= 7 fields (ncurses then paints the prompt with the `rep` capability `\e[<W-1>b` and the post-rep reposition is terminfo-internal + position-dependent)
- OVERFLOW input (typing past the field width bells `\a` + overwrites -- a separate input-editing state machine)
- field editing keys (arrows / backspace-during-input / function keys)
- numeric or USING-validation fields
- multi-field ACCEPT
- ACCEPT colour/attributes
- any terminal other than TERM=xterm or ncurses other than the admitted 6.6
- lie prevented: screen ACCEPT input needs ncurses linked -- NO (for the width<=6 non-overflow envelope): the prompt + reposition + echo + field-full are deterministic bytes reproduced exactly and cross-checked against TWO GnuCOBOL versions, no ncurses linked. The `rep` prompt, overflow editing, and field keys are honestly carved out

## Damage if overclaimed
claiming 'screen ACCEPT works' would hide that only single width<=6 non-overflow alphanumeric input is proven -- the rep prompt, overflow/edit keys, numeric/USING validation, multi-field, and non-admitted terminals are not

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
