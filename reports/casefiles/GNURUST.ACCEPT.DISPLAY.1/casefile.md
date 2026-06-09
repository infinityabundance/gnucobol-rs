<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.ACCEPT.DISPLAY.1 (court-casefile)

**Verdict: PASS** · 7/7 pass, 0 fail · crate `gnucobol-rs` 0.7.23

- **Oracle:** cobc DISPLAY/ACCEPT (libcob/termio.c)
- **Byte domain(s):** DISPLAY operand concatenation + newline; ACCEPT field move bytes
- **Replay:** `bash lab/oracle/accept_display_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- DISPLAY emits its operands' bytes CONCATENATED with no separator + one trailing newline (a literal -> its text, an alphanumeric field -> its space-padded bytes, an unsigned 9(n) field -> its zoned digit bytes), and ACCEPT field FROM CONSOLE moves one input line into the field left-justified, space-padded, truncated to the field width -- matching cobc/libcob byte-for-byte. Emitted text is admitted as evidence

## Negative claims (6) — negative capability is the trust surface
- DISPLAY of signed numeric (GnuCOBOL prefixes +/-) or V-scaled/edited numeric (reformats)
- DISPLAY UPON / WITH NO ADVANCING
- ACCEPT FROM DATE/TIME/environment/screen
- device/console specifics
- all dialects
- lie prevented: DISPLAY is not just a print -- it CONCATENATES operands with NO separator and adds exactly one newline; ACCEPT MOVEs the line (pad/truncate) not a free read; and DISPLAY of a SIGNED numeric reformats (+/- prefix), which is a non-claim here

## Damage if overclaimed
assuming DISPLAY separates operands (it does not) or that ACCEPT preserves an over-length line (it truncates) corrupts emitted/captured text in a ported program

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
