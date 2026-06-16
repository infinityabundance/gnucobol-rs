<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.SCREENIO.DISPLAY.3 (court-casefile)

**Verdict: PASS** · 21/21 pass, 0 fail · crate `gnucobol-rs` 0.7.83

- **Oracle:** cobc SCREEN SECTION DISPLAY of multiple positioned fields (libcob/screenio.c via ncurses 6.6 mvcur), captured under a pty with TERM=xterm
- **Byte domain(s):** a multi-field SCREEN SECTION DISPLAY (an ordered list of LINE/COLUMN fields) -> the exact ncurses cursor-movement + field byte stream, for non-overlapping layouts, on the admitted terminal
- **Replay:** `bash lab/oracle/screenio_multi_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (4)
- a from-scratch, dependency-free Rust reproduction of ncurses's GENERAL mvcur cursor optimization (screenio.rs mvcur + horiz_candidates + display_and_stop) -- WITHOUT linking ncurses -- proven byte-identical to GnuCOBOL's terminal output for a MULTI-FIELD DISPLAY, where the move between fields starts from a non-home origin (screenio_multi_sweep 21/0 over forward / row-change / near / far position pairs). mvcur enumerates the strategies ncurses considers -- (1) keep the column and move vertically (VPA \e[<r>d, or cuu1 \e[A for up-one) then horizontally from the current column
- (2) carriage-return to column 1, move vertically, then horizontally from 1
- (3) home \e[H for the exact (1,1) target
- (4) direct cursor-address CUP -- and emits the shortest, with the local/CR strategies winning a byte-count tie over CUP. The horizontal sub-move space-fills for <=4 columns, uses column-address HPA \e[<c>G for a 5..7-column advance, backspaces for a left move, and otherwise defers to CUP. The same general mvcur subsumes the from-home first-field move (DISPLAY.2, still 70/0) and the post-field move to the pause prompt (which sits one row below the LAST field, not the lowest).

## Negative claims (7) — negative capability is the trust surface
- overlapping same-row layouts where a later field sits LEFT of an earlier field's cells -- those trigger ncurses's REFRESH LINE-DIFF (clr_eol \e[K erase of the now-stale earlier cells via the curses TransformLine algorithm), a separate mechanism that is a declared follow-on court (NOT the cursor-move model sealed here)
- 3+ field interactions beyond the swept pairs
- color / attribute SGR
- numeric-edited / JUSTIFIED display
- ACCEPT input
- any terminal other than TERM=xterm or ncurses other than the admitted 6.6 (terminfo-dependent)
- lie prevented: multi-field screen layout needs ncurses linked -- NO (for non-overlapping layouts): the inter-field mvcur byte choice is deterministic for a fixed terminal and reproduced exactly across the swept pairs, no ncurses linked. The curses refresh line-diff (overlapping erase) is honestly carved out as a separate unsealed mechanism

## Damage if overclaimed
claiming 'multi-field DISPLAY works' would hide that overlapping same-row layouts (needing the clr_eol refresh diff), attributes, and ACCEPT are unported, and that only non-overlapping pairs on the admitted terminal are proven

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
