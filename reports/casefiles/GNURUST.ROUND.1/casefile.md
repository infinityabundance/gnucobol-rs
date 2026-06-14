<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.ROUND.1 (court-casefile)

**Verdict: PASS** · round sweep 6720/0 (DISPLAY + packed receivers, seven byte-producing modes, kept digits 0-9) + unit matrix + fuzz + Kani · crate `gnucobol-rs` 0.7.45

- **Oracle:** libcob cob_add with the COB_STORE_<mode> opt, DISPLAY and packed receivers (cob_decimal_do_round + cob_add_bcd)
- **Byte domain(s):** value + target scale + ROUNDED mode + receiver path (cob_decimal / packed BCD) -> stored field bytes
- **Replay:** `bash lab/oracle/round_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- the stored bytes of a value narrowed to a smaller scale under each ROUNDED MODE IS setting, on BOTH store paths: the cob_decimal path (COMPUTE / MOVE / DISPLAY receiver, cob_decimal_do_round numeric.c:1936) AND the packed cob_add_bcd path (ADD/SUBTRACT into a COMP-3 receiver, numeric.c:2826+). Modes: NEAREST-AWAY-FROM-ZERO (the default ROUNDED), AWAY-FROM-ZERO, NEAREST-EVEN (banker's), NEAREST-TOWARD-ZERO, TOWARD-GREATER (ceiling), TOWARD-LESSER (floor), PROHIBITED (size error on a dropped non-zero digit), and TRUNCATION -- byte-for-byte vs cobc. The two paths agree except NEAREST-EVEN, which the BCD path resolves away-from-zero (no to-even)
- the port maps it accordingly (ROUND.2)

## Negative claims (4) — negative capability is the trust surface
- bignum values beyond i128 (38-digit COMPUTE intermediates)
- floating-point COMP-1/COMP-2 rounding
- intermediate-result rounding rules
- lie prevented: 'ROUNDED always means round-half-up' -- NO, COBOL has eight distinct ROUNDED MODE IS settings; NEAREST-EVEN rounds 2.5 to 2 while the default rounds 2.5 to 3, and ceiling/floor follow +/-infinity not magnitude

## Damage if overclaimed
a wrong rounding mode silently mis-rounds money: NEAREST-EVEN vs the default differ on every exact tie, so a misapplied mode corrupts totals and interest across a whole ledger

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
