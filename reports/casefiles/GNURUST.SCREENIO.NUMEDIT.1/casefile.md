<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.SCREENIO.NUMEDIT.1 (court-casefile)

**Verdict: PASS** · 14/14 pass, 0 fail · crate `gnucobol-rs` 0.8.55

- **Oracle:** cobc SCREEN SECTION DISPLAY of a numeric-edited field FROM a numeric source (libcob/screenio.c via ncurses; the edited image from the move/edit engine), captured under a pty with TERM=xterm
- **Byte domain(s):** a numeric-edited positioned single-field DISPLAY (LINE/COLUMN + edited PIC FROM a numeric source) -> the exact ncurses byte stream that skips leading blanks + writes the edited run, on the admitted terminal
- **Replay:** `bash lab/oracle/screenio_numedit_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (3)
- a from-scratch, dependency-free Rust reproduction of how ncurses paints a NUMERIC-EDITED SCREEN SECTION field (an edited PIC -- ZZ,ZZ9.99 / -9(5).99 / 9(4).99CR / ZZ,ZZ9.99- ...) FROM a numeric source -- WITHOUT linking ncurses -- proven byte-identical to GnuCOBOL (screenio_numedit_sweep 14/0 across Z-suppression large/small/tiny, all-blank zero, fixed/floating sign, CR/DB positive+negative, a trailing fixed sign, and varied positions). The edited image is produced by the sealed move/edit engine (edited::encode_edited) and POSITIONED by screenio::display_edited_and_stop, which reproduces three new screen behaviours: leading blanks are SKIPPED (the cursor moves straight to col+first_nonblank via the shared mvcur cost model and writes from there)
- the written run goes to the field end (edited[first_nonblank..], short trailing-blank runs space-filled)
- an all-blank field (ZZZZ.ZZ of zero) writes NOTHING and parks the cursor at col+width. This court also surfaced + fixed a real numeric-edit bug: complete zero suppression (all-Z picture, no forced 9/*, value zero) must blank the ENTIRE field including the decimal point (edited::encode_edited).

## Negative claims (9) — negative capability is the trust surface
- the numeric editing itself (the move.c court
- this court proves the screen POSITIONING of an already-edited image)
- a pathological PIC with a long (5+) interior/trailing blank run that ncurses would cursor-skip rather than space-fill
- the `*` check-protection zero-fill rule (a separate edited.rs follow-on)
- MULTIPLE edited fields in one DISPLAY (the general multi-field line-diff)
- numeric-edited COLOUR/attributes
- ACCEPT-side numeric-edited input
- any terminal other than TERM=xterm or ncurses other than the admitted 6.6
- lie prevented: numeric-edited screen display needs ncurses linked -- NO: the edited image composes the sealed editor and the screen POSITIONING is a deterministic leading-blank-skip reproduced exactly, no ncurses linked. The editing rule itself stays the move.c court's claim; the `*`-fill and multi-field cases are honestly carved out

## Damage if overclaimed
claiming 'numeric-edited screen display works' would hide that only single-field Z-suppression positioning is proven -- star-fill, multi-field line-diff, edited colour/attributes, ACCEPT, and non-admitted terminals are not

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
