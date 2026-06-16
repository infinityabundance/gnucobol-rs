<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.LOGICAL.1 (court-casefile)

**Verdict: PASS** · logical_sweep 2400/0 (incl. negative operands, shift counts >= 64, full-width values) · crate `gnucobol-rs` 0.7.84

- **Oracle:** libcob cob_logical_and/or/xor/not/left/right (over cob_decimal)
- **Byte domain(s):** (value0, value1) -> u64 bitwise result via |value| mod 2^64
- **Replay:** `bash lab/oracle/logical_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- the bit operations libcob exposes as cob_logical_and/or/xor/not/left/right: each operand is reduced to the LOW 64 BITS OF ITS ABSOLUTE VALUE (mpz_get_ui ignores the sign, so B-AND(-1, 255) = 1, NOT 255), the C bitwise operator is applied over 64 bits, shifts take the count modulo 64, and the unsigned 64-bit result is the value -- byte-faithful to libcob

## Negative claims (5) — negative capability is the trust surface
- fractional operands (the scale is dropped, only the unscaled magnitude's low 64 bits are used)
- operands beyond 64 bits (truncated)
- cob_logical_left_c/right_c size-bounded variants
- boolean/bit data items as such
- lie prevented: 'B-AND(-1, x) = x because -1 is all-ones' -- NO, mpz_get_ui takes |-1| = 1, so the result is 1 & x; the sign is discarded, not sign-extended

## Damage if overclaimed
a two's-complement bit model diverges from GnuCOBOL on every negative operand and on shift counts at/above the word width

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
