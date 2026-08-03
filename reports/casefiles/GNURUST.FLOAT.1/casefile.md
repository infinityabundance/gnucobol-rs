<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.FLOAT.1 (court-casefile)

**Verdict: PASS** · float_sweep 1476/0 (all four usages x both directions x signs/scales/magnitudes) + oracle-pinned unit bit-patterns · crate `gnucobol-rs` 0.8.53

- **Oracle:** libcob cob_move via cob_decimal_get/set_double + get/set_ieee64dec/ieee128dec (numeric.c; constants coblocal.h:165-198)
- **Byte domain(s):** decimal value <-> COMP-1/COMP-2/FLOAT-DECIMAL-16/FLOAT-DECIMAL-34 field bytes, both directions
- **Replay:** `bash lab/oracle/float_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (5)
- the field bytes and decimal<->float conversions of all four GnuCOBOL float usages, both directions, byte-identical to cob_move: COMP-2 (IEEE-754 double) -- a decimal is converted by TRUNCATING TOWARD ZERO to the nearest representable double (libcob goes through a 2048-bit GMP mpf then mpf_get_d, which rounds toward zero
- an inexact decimal lands 1 ULP below a correctly-rounded parse), reproduced via a round-to-nearest parse + exact big-integer comparison + step-toward-zero
- COMP-1 (IEEE-754 single) -- (float)double: truncate-to-double THEN round-to-nearest-float (the C cast), NOT a second truncation
- float -> decimal -- the exact double value floor-truncated to the receiver scale (floor(|v| * 10^scale), low digits)
- FLOAT-DECIMAL-16/34 (IEEE-754-2008 decimal64/decimal128, BID encoding) -- sign bit + combination field + extended-exponent form + binary-integer significand (<=16/<=34 digits, biases 398/6176), encode canonicalizes by stripping trailing zeros then truncating low digits to fit

## Negative claims (7) — negative capability is the trust surface
- float ARITHMETIC (cob_add over float receivers -- a separate composition)
- cob_cmp_float epsilon comparison
- long double (COMP-2 extended) and binary FP_BIN32/64/128 usages
- Inf/NaN handling beyond decode-refusal
- locale-dependent display of floats
- the mpf 2048-bit intermediate itself (only its observable truncate-toward-zero result)
- lie prevented: 'decimal-to-float conversion rounds to nearest like every modern parser' -- NO, libcob TRUNCATES TOWARD ZERO through GMP mpf_get_d, so 0.1 in a COMP-2 is 1 ULP BELOW Rust's/C's correctly-rounded 0.1; assuming round-to-nearest silently diverges on most inexact decimals

## Damage if overclaimed
a correctly-rounded conversion writes a different COMP-2 byte image than GnuCOBOL for most non-representable decimals -- checksums, comparisons, and downstream arithmetic diverge one ULP at a time

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
