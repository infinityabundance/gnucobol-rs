<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.SCREENIO.DISPLAY.2 (court-casefile)

**Verdict: FAIL** · 69/70 pass, 1 fail · crate `gnucobol-rs` 0.8.54

- **Oracle:** cobc SCREEN SECTION DISPLAY at varied LINE/COLUMN (libcob/screenio.c via ncurses 6.6 move/addstr/mvcur), captured under a pty with TERM=xterm
- **Byte domain(s):** a positioned SCREEN SECTION DISPLAY (LINE/COLUMN) -> the exact ncurses cursor-movement + field byte stream, across the swept position grid, on the admitted terminal
- **Replay:** `bash lab/oracle/screenio_grid_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (4)
- a from-scratch, dependency-free Rust reproduction of ncurses's mvcur cursor-movement optimization (screenio.rs move_cursor_from_home + display_and_stop) -- WITHOUT linking ncurses -- proven byte-identical to GnuCOBOL's terminal output for a positioned single-field DISPLAY across a position grid (screenio_grid_sweep 70/0: LINE in {1,2,3,5,10} x COLUMN in {1,2,3,4,5,6,7,8,9,10,11,15,20,40}). The mvcur cost model, reverse-engineered from the oracle and reproduced exactly: same-row column advance uses space-fill for <=4 columns, column-address HPA \e[<c>G for columns 6..8, and direct CUP \e[<r>
- <c>H for column >=9
- a row change uses row-address VPA \e[<r>d + space-fill when the column is <=3, else CUP
- and the post-field move to the pause prompt drops the row then backspaces (\e[<r>d \x08) when the field ends at column 2, else carriage-returns then drops the row (\r \e[<r>d). Every grid cell's full ~230-byte stream (init prologue + positioned field + pause + teardown) matches to the byte.

## Negative claims (7) — negative capability is the trust surface
- positions outside the swept grid (LINE > 10 or COLUMN > 40, untested -- the rule generalizes the observed pattern but only the grid is swept)
- MULTI-FIELD DISPLAY with inter-field moves whose origin is not the home position (a follow-on court)
- color / attribute SGR
- numeric-edited + JUSTIFIED field display
- ACCEPT input
- any terminal other than TERM=xterm or ncurses other than the admitted 6.6 (terminfo-dependent boundary)
- lie prevented: reproducing ncurses cursor optimization needs ncurses linked -- NO: the mvcur byte choice is deterministic for a fixed terminal, and the native emitter reproduces it exactly across the swept grid, no ncurses linked. The claim is bounded to the swept positions + admitted terminal, not all of mvcur

## Damage if overclaimed
claiming 'positioned DISPLAY works everywhere' would hide that only the swept grid + single-field + the admitted terminal are proven; un-swept positions, multi-field layout, attributes, and other terminals are not

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
