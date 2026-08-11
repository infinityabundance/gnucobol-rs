<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.ACCEPT.DISPLAY.2 (court-casefile)

**Verdict: PASS** · 8/8 pass, 0 fail · crate `gnucobol-rs` 0.8.56

- **Oracle:** cobc DISPLAY of numeric (libcob/termio.c)
- **Byte domain(s):** DISPLAY numeric: signed +/- prefix + V decimal point
- **Replay:** `bash lab/oracle/accept_display2_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- DISPLAY reformats a numeric field, matching cobc/libcob byte-for-byte (verified 8/0): a SIGNED field (S9) gets a leading sign char (- negative, + positive, and + for ZERO -> S9(3)=0 displays as +000), and a V-scaled field gets a . inserted at the implied decimal point (9(3)V99=12.34 -> 012.34, S9(3)V99=-12.34 -> -012.34). Seals the signed/V DISPLAY formatting that GNURUST.ACCEPT.DISPLAY.1 deferred

## Negative claims (5) — negative capability is the trust surface
- numeric-edited PICs (Z/,/*/$/CR/DB -- that is GNURUST.16)
- BLANK WHEN ZERO
- JUSTIFIED / floating-point USAGE
- all dialects
- lie prevented: DISPLAY shows the stored bytes -- NO: for a SIGNED numeric it prints a +/- SIGN CHARACTER (not the overpunched last byte) so the width grows by one, and a positive/zero value shows +; a V-scaled field shows a . that is not stored

## Damage if overclaimed
assuming DISPLAY of a signed field equals its stored zoned bytes (it does not -- it de-overpunches to a sign prefix) mis-sizes and mis-reads captured DISPLAY output

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
