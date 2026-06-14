<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.NUMCMP.1 (court-casefile)

**Verdict: PASS** · numcmp_sweep 1024/0 (DISPLAY/PACKED x scales 0-3 x signs x magnitudes incl. -0) + unit cases · crate `gnucobol-rs` 0.7.46

- **Oracle:** libcob cob_numeric_cmp (numeric.c) over real cob_fields (cmp_harness)
- **Byte domain(s):** two numeric field byte images -> the -1/0/1 ordering verdict
- **Replay:** `bash lab/oracle/numcmp_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (3)
- the signed -1/0/1 verdict of comparing two numeric fields, byte/verdict-identical to libcob cob_numeric_cmp across DISPLAY x PACKED receivers, differing scales (aligned), and all sign combinations -- the first 1:1 port on the new pure-Rust GMP-mpz subset (gmp::Mpz) and cob_decimal layer (cob_decimal_set_field -> align_decimal/shift_decimal -> cob_decimal_cmp)
- the libcob fast paths (bcd compare, integer compare via cob_get_llint) all yield the same verdict as the general decimal comparison reproduced here
- float operands route to the float (f64) comparison

## Negative claims (5) — negative capability is the trust surface
- float-vs-float epsilon comparison edge cases (cob_cmp_float specifics)
- the optimized cob_cmp_packed/cmp_uint/cmp_llint fast-path code (verified only that the verdict matches)
- negative scale packed compare
- National/DBCS comparison
- lie prevented: 'comparing two COBOL numerics is just comparing their raw bytes' -- NO, fields of different usage/scale must be decoded to a common decimal and scale-aligned first; a byte compare gives the wrong verdict whenever usages or scales differ

## Damage if overclaimed
a wrong comparison flips IF/EVALUATE branches, mis-orders SORT keys, and breaks SEARCH ALL binary search -- silent control-flow and data corruption

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
