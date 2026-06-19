<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.SCREENIO.ATTR.1 (court-casefile)

**Verdict: PASS** · 10/10 pass, 0 fail · crate `gnucobol-rs` 0.8.8

- **Oracle:** cobc SCREEN SECTION DISPLAY ... HIGHLIGHT/LOWLIGHT/UNDERLINE/BLINK/REVERSE-VIDEO (libcob/screenio.c via ncurses 6.6 attrset), captured under a pty with TERM=xterm
- **Byte domain(s):** an attributed positioned DISPLAY (LINE/COLUMN + monochrome attribute) -> the exact ncurses SGR-wrapped field byte stream, on the admitted terminal
- **Replay:** `bash lab/oracle/screenio_attr_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (4)
- a from-scratch, dependency-free Rust reproduction of the SGR attribute sequences ncurses emits around an attributed SCREEN SECTION DISPLAY field -- WITHOUT linking ncurses -- proven byte-identical to GnuCOBOL (screenio_attr_sweep 10/0 over the 5 monochrome attributes x 2 positions). Each attribute wraps the field text in an SGR-on opener (\e(B \e[0
- <n>m \e[39
- 49m\e[37m\e[40m -- charset + set_attributes + default-colour restore) and a constant SGR-off closer (\e(B \e[m \e[39
- 49m\e[37m\e[40m), with the SGR parameter <n> = 1 HIGHLIGHT, 2 LOWLIGHT, 4 UNDERLINE, 5 BLINK, 7 REVERSE-VIDEO. The cursor advances only by the field length (the SGR sequences move nothing), so this composes exactly with the sealed mvcur positioning (screenio.rs ScreenAttr + sgr_on/sgr_off + display_and_stop).

## Negative claims (6) — negative capability is the trust surface
- COLOUR attributes (FOREGROUND-COLOR / BACKGROUND-COLOR) -- those trigger a whole-screen colour repaint (the terminfo bce path with an extra clear/\e[J/\e[K), a separate mechanism NOT sealed here
- COMBINED attributes (e.g. HIGHLIGHT UNDERLINE together)
- attributes on multi-field or numeric-edited displays
- ACCEPT-side prompt/secure attributes
- any terminal other than TERM=xterm or ncurses other than the admitted 6.6 (terminfo-dependent)
- lie prevented: screen display attributes need ncurses linked -- NO (for the monochrome attributes): each is a deterministic SGR on/off wrap reproduced exactly, no ncurses linked. The colour-repaint path is honestly carved out as unsealed

## Damage if overclaimed
claiming 'screen attributes work' would hide that COLOUR (the whole-screen repaint), combined attributes, and ACCEPT-side attributes are unported, and that only the admitted terminal is proven

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
