<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.VALUE.NEGZERO.EDGE.1 (court-casefile)

**Verdict: PASS** · 8/8 pass, 0 fail · crate `gnucobol-rs` 0.7.38

- **Oracle:** cobc VALUE initial image (cobc/typeck.c + libcob packed/zoned encode)
- **Byte domain(s):** VALUE-image negative-zero sign across usage x literal-form x scale; oracle rule + the locked gnucobol-rs divergence set (COMP-3 integer-form only)
- **Replay:** `bash lab/oracle/edge_negzero_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (4)
- characterized the negative-zero VALUE-image sign rule across the full matrix (usage x literal-form x sign x scale) under the gnucobol-3.2.0 oracle, sweep 11/0, harvested by GNURUST.LINEAGE.CORPUS.20M (1056 hits): cobc's rule is SHAPE-SENSITIVE ON THE LITERAL FORM, not the field scale -- COMP-3 + integer-form literal (no decimal point) CANONICALIZES negative-zero to the POSITIVE packed sign nibble 0C, COMP-3 + decimal-form literal (has a '.') PRESERVES negative 0D even in a V99 field, DISPLAY always PRESERVES negative as overpunch 0x70, signed binary stores all-zero with no sign nibble, and unsigned VALUE -0 is a compile error
- this BOUNDS GNURUST.8 (which did not test negative-zero) and LOCKS the exact gnucobol-rs divergence: value_image diverges from cobc ONLY for COMP-3 packed integer-form negative-zero (oracle 0C vs rust 0D) -- the 4 cells comp3-int-0/-00/-000 + comp3v99-int0 -- while DISPLAY and decimal-form COMP-3 MATCH
- the well-scoped PATCH WAS THEN APPLIED (value_image, init::encode_numeric packed branch: canonicalize the packed sign nibble for integer-form zero ONLY, scale==0) -- the 4 COMP-3 integer cells flipped known_diverge->match, value_image is now BYTE-EXACT with cobc, the divergence set is EMPTY, and the court now LOCKS PARITY (an under-fix re-diverges a COMP-3 integer cell, an over-fix diverges a DISPLAY/decimal cell -- either turns it RED)
- value_sweep stays 392/0 and set_sweep 52/0 (no regression)

## Negative claims (7) — negative capability is the trust surface
- negative-zero semantics GENERALLY (the patch fixes VALUE INITIALIZATION only -- the COMP-3 integer-form packed sign nibble
- a blanket parse_num canonicalization was tried first and REVERTED because it would wrongly canonicalize the decimal-form and DISPLAY cells)
- negative-zero under arithmetic/MOVE/COMPUTE (the arithmetic -0 path is GNURUST.13, separate and deliberate)
- figurative ZERO
- other usages/dialects
- the broader signed-zero space beyond VALUE initialization
- lie prevented: 'just canonicalize negative zero' is WRONG and was proven so (the blanket parse_num fix regressed value_sweep 391/392) -- the rule is shape-sensitive ON THE LITERAL FORM: cobc canonicalizes -0 to positive ONLY for COMP-3 integer-form literals, while PRESERVING the sign for decimal-form COMP-3 and for ALL DISPLAY; so the divergence is a narrow, exactly-located edge (4 cells), not a blanket bug, and GNURUST.8 is bounded accordingly

## Damage if overclaimed
a blanket negative-zero fix corrupts the decimal-form and DISPLAY cells (which are correct); treating this as a broad VALUE bug overstates a 4-cell edge; conflating it with arithmetic -0 breaks GNURUST.13's deliberate sign-on-zero

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
