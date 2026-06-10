# VALUE.NEGATIVE_ZERO.COMP3.INTEGER_CANONICALIZES_POSITIVE

Discovered by GNURUST.LINEAGE.CORPUS.20M differential burn; shrunk + confirmed under per-call oracle isolation.

- oracle (cobc) bytes: `0c`
- rust value_image bytes: `0d`
- court target: GNURUST.8/2/14
- candidate court: GNURUST.VALUE.NEGZERO.EDGE.1

Shape-sensitive (NOT a blanket negative-zero rule). Siblings:
- VALUE.NEGATIVE_ZERO.COMP3.INTEGER_CANONICALIZES_POSITIVE
- VALUE.NEGATIVE_ZERO.COMP3.SCALED_PRESERVES_NEGATIVE
- VALUE.NEGATIVE_ZERO.DISPLAY.PRESERVES_NEGATIVE_OVERPUNCH

A blanket canonicalization patch was attempted and REVERTED: it regressed value_sweep (391/392) and the scaled/display siblings, proving the rule is shape-sensitive.
