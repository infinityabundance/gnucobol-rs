<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.ROUND.1 (court-casefile)

**Verdict: PASS** · round sweep 672/0 (seven byte-producing modes) + unit matrix + fuzz · crate `gnucobol-rs` 0.7.35

- **Oracle:** libcob cob_add with the COB_STORE_<mode> opt (cob_decimal_do_round)
- **Byte domain(s):** value + target scale + ROUNDED mode -> stored field bytes
- **Replay:** `bash lab/oracle/round_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- the stored bytes of a value narrowed to a smaller scale under each ROUNDED MODE IS setting on the cob_decimal store path (COMPUTE / MOVE / DISPLAY receiver): NEAREST-AWAY-FROM-ZERO (the default ROUNDED), AWAY-FROM-ZERO, NEAREST-EVEN (banker's), NEAREST-TOWARD-ZERO, TOWARD-GREATER (ceiling), TOWARD-LESSER (floor), PROHIBITED (size error on a dropped non-zero digit), and TRUNCATION -- a faithful port of cob_decimal_do_round (numeric.c:1936) matching cobc byte-for-byte

## Negative claims (4) — negative capability is the trust surface
- ADD/SUBTRACT directly INTO a packed field (the cob_add_bcd nibble-rounding path, numeric.c:2907, resolves NEAREST-EVEN ties differently -- that is GNURUST.13's surface, a loud non-claim here)
- bignum values beyond i128
- floating-point COMP-1/COMP-2 rounding
- lie prevented: 'ROUNDED always means round-half-up' -- NO, COBOL has eight distinct ROUNDED MODE IS settings; NEAREST-EVEN rounds 2.5 to 2 while the default rounds 2.5 to 3, and ceiling/floor follow +/-infinity not magnitude

## Damage if overclaimed
a wrong rounding mode silently mis-rounds money: NEAREST-EVEN vs the default differ on every exact tie, so a misapplied mode corrupts totals and interest across a whole ledger

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
