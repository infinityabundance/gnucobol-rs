<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.SCREENIO.INIT.1 (court-casefile)

**Verdict: PASS** · 1/1 pass, 0 fail · crate `gnucobol-rs` 0.8.53

- **Oracle:** cobc SCREEN SECTION DISPLAY (libcob/screenio.c via ncurses 6.6), captured under a pty with TERM=xterm
- **Byte domain(s):** a positioned SCREEN SECTION DISPLAY (line/column + literal/FROM bytes) -> the exact terminal escape-sequence byte stream GnuCOBOL writes via ncurses, on the admitted terminal
- **Replay:** `bash lab/oracle/screenio_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- a from-scratch, dependency-free Rust reproduction of the terminal byte stream GnuCOBOL's SCREEN SECTION DISPLAY writes via ncurses -- WITHOUT linking ncurses (screenio.rs INIT_PROLOGUE/TEARDOWN_EPILOGUE/PAUSE_PROMPT + move_cursor + display_and_stop). GnuCOBOL drives ncurses initscr/move/addstr/refresh/endwin
- the literal bytes are ncurses's terminfo-optimized output, which this port emits exactly for the admitted terminal. Captured deterministically from the oracle under a pty (script(1)) and proven byte-identical (screenio_sweep 1/0): the canonical program `DISPLAY "X" LINE 2 COLUMN 3.` then `STOP RUN.` yields the exact 230-byte stream -- the smcup/scroll-region/charset/color init prologue, the positioned field (VPA \e[2d + space-fill to column 3 + the byte), the libcob "end of program, please press a key to exit " pause, and the rmcup teardown epilogue -- reproduced to the byte.

## Negative claims (8) — negative capability is the trust surface
- the ncurses mvcur cursor-cost model (CUP vs VPA vs CHA vs space-fill for arbitrary moves -- only the VPA+space-fill case is swept)
- multi-field layout + field ordering
- color / attribute SGR (BLANK/HIGHLIGHT/REVERSE/UNDERLINE/colors)
- numeric-edited + JUSTIFIED field display
- ACCEPT input / key handling / cursor navigation
- any terminal other than TERM=xterm
- any ncurses build other than the admitted 6.6 (the byte stream is terminfo-dependent -- this is the explicit terminal-dependence boundary)
- lie prevented: SCREEN SECTION needs ncurses linked -- NO (for the framing + a positioned DISPLAY): the terminal byte stream is deterministic for a fixed terminal, so a native Rust emitter reproduces it exactly, no ncurses linked. The honest scope is the admitted terminal; the bytes are terminfo-dependent and that dependence is declared, not hidden

## Damage if overclaimed
claiming 'SCREEN SECTION works' would hide that this is one positioned field on one terminal -- the cursor-cost model, attributes, multi-field layout, and ACCEPT input are unported, and the bytes do not transfer to other terminals/ncurses versions

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
