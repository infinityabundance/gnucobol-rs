<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.SCREENIO.COLOR.1 (court-casefile)

**Verdict: PASS** · 11/11 pass, 0 fail · crate `gnucobol-rs` 0.8.47

- **Oracle:** cobc SCREEN SECTION DISPLAY of a field with FOREGROUND-COLOR/BACKGROUND-COLOR (libcob/screenio.c via ncurses 6.6 start_color/init_pair/wclear), captured under a pty with TERM=xterm
- **Byte domain(s):** a colour-attributed positioned single-field DISPLAY (LINE/COLUMN + FOREGROUND-COLOR/BACKGROUND-COLOR, LINE>=2) -> the exact ncurses whole-screen colour-repaint byte stream, on the admitted terminal
- **Replay:** `bash lab/oracle/screenio_color_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- a from-scratch, dependency-free Rust reproduction of the FULL ncurses colour-repaint byte stream for a single colour-attributed SCREEN SECTION DISPLAY field -- WITHOUT linking ncurses -- proven byte-identical to GnuCOBOL (screenio_color_sweep 11/0 + an offline 628-capture R>=2 x column x 8x8-colour grid). Two facts are pinned: (1) the COBOL->curses colour permutation -- COBOL orders the colour bits (blue,green,red), curses (red,green,blue), so a COBOL colour maps to its curses colour by REVERSING the low three bits (COBOL 1 blue->4, 4 red->1, 6 brown->3), the fg SGR = 30+curses, bg = 40+curses (curses_color)
- (2) the default pair (fg=7,bg=0 = white-on-black = pair 0) needs no SGR and triggers NO repaint, falling back to the plain positioned-write stream. A non-default colour forces ncurses's wclear + top-down TransformLine repaint: a colour-injected init prologue (the pair SGR before \e[H\e[2J), then VPA(R+1)+reset+\e[J (erase to bottom), the rows above the field cleared top-down (\e[H\e[K then \e[<r>d\e[K), the field-row positioning (space-fill when col-1<=4 else CUP+\e[1K+space), the coloured field, reset+\e[K, and the shared mvcur prompt move (screenio.rs color_display_and_stop + curses_color).

## Negative claims (7) — negative capability is the trust surface
- the LINE==1 single-row-screen colour case (a different \e[A-based positioning, a documented edge)
- MULTIPLE colour fields in one DISPLAY (the general doupdate/TransformLine line-diff across an arbitrary screen delta -- the overlapping same-row clr_eol case)
- combined colour+monochrome attributes
- colour on numeric-edited displays
- ACCEPT-side colour
- any terminal other than TERM=xterm or ncurses other than the admitted 6.6 (terminfo-dependent)
- lie prevented: the colour-repaint path was the explicit non-claim of GNURUST.SCREENIO.ATTR.1; this seals the SINGLE-field colour repaint byte-exactly (no ncurses linked) while honestly carving out the general multi-field line-diff and the LINE==1 edge as still unsealed

## Damage if overclaimed
claiming 'screen colour works' would hide that only a single colour field on LINE>=2 is proven -- the general multi-field doupdate line-diff, the LINE==1 case, numeric-edited/ACCEPT colour, and non-admitted terminals are not

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
