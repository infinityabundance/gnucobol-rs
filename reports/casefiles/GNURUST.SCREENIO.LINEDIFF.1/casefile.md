<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.SCREENIO.LINEDIFF.1 (court-casefile)

**Verdict: PASS** · 19/19 pass, 0 fail · crate `gnucobol-rs` 0.7.84

- **Oracle:** cobc two DISPLAY ... LINE r COLUMN c statements to the same row (libcob/screenio.c via ncurses doupdate), captured under a pty with TERM=xterm; DIFFERENTIALLY against GnuCOBOL 3.2 AND 3.1.2
- **Byte domain(s):** two same-row DISPLAY statements (col1/len1, col2/len2) -> the exact ncurses doupdate/TransformLine refresh byte stream, on the admitted terminal
- **Replay:** `bash lab/oracle/screenio_linediff_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (8)
- a from-scratch, dependency-free Rust reproduction of the ncurses doupdate/TransformLine line-diff for TWO same-row DISPLAY statements (the second overwriting/extending/overlapping the first) -- WITHOUT linking ncurses -- proven byte-identical to GnuCOBOL AND DIFFERENTIALLY byte-identical to the 3.1.2 second oracle (screenio_linediff_sweep 19/0, 3.1.2-matched 19/19
- plus an offline 297-case grid vs 3.2 with full 3.1.2 differential). This seals what was long documented as the hard non-claim (the overlapping same-row clr_eol line-diff). The second DISPLAY is diffed against the virtual screen and the cheapest reposition+write emitted (screenio::two_display_line_and_stop): the clr_eol trailing-erase (\e[K for >=2 blanked cells, a space for 1, nothing for 0)
- the EmitRange leading-cell rewrite (CR-to-col1 + rewrite unchanged cells when cheaper than addressing the first changed column)
- the backward/forward HPA distance threshold (repeated backspaces/spaces for distance <=4, column-address \e[<c>G for >=5)
- the cursor-uncertainty CUP (when the backward distance >=8 AND target column >=9 ncurses treats the prior column as unknown and forces a direct \e[<row>
- <col>H even though HPA is shorter)
- the rep run-length \e[<n>b for runs >=7
- candidate tie-break backspaces/spaces > CR > HPA > CUP. The CUP rule is the one non-cost rule -- verified with the real xterm terminfo NormalizedCost (hpa~5 < cup~8) so it cannot be a cost decision.

## Negative claims (11) — negative capability is the trust surface
- THREE or more same-row DISPLAYs (a DISPLAY's clear-to-EOL batches differently across 3+ refreshes -- the trailing erase is deferred when another write to the row follows)
- distant isolated survivors needing a jump + single space
- the MULTI-ROW diff (vertical mvcur between changed rows)
- insert/delete-character optimizations (ncurses ich/dch when the new line shifts content)
- scroll-region (DECSTBM) interactions within the diff
- attributes or colour on the diffed text (the bce repaint interacting with TransformLine)
- a numeric-edited or FROM field as the diffed content
- the LINE==1 single-row-screen diff
- any terminal other than TERM=xterm
- any ncurses other than the admitted 6.6
- lie prevented: the overlapping same-row refresh line-diff needs ncurses linked -- NO (for two same-row DISPLAYs): the doupdate/TransformLine reposition cost search + clr_eol + rep are deterministic bytes reproduced exactly and cross-checked against TWO GnuCOBOL versions, no ncurses linked. The 3+/distant/multi-row doupdate sub-cases are honestly carved out

## Damage if overclaimed
claiming 'the screen line-diff works' would hide that only the TWO-DISPLAY same-row refresh is proven -- 3+ displays, distant survivors, multi-row diffs, attributed text, and non-admitted terminals are not

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
