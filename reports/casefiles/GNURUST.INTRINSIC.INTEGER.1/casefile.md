<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.INTRINSIC.INTEGER.1 (court-casefile)

**Verdict: PASS** · 20/20 pass, 0 fail · crate `gnucobol-rs` 0.7.48

- **Oracle:** cobc FUNCTION INTEGER/INTEGER-PART (libcob/intrinsic.c)
- **Byte domain(s):** FUNCTION INTEGER(x)=floor / INTEGER-PART(x)=trunc
- **Replay:** `bash lab/oracle/integer_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- FUNCTION INTEGER(x) returns the greatest integer NOT GREATER THAN x (FLOOR) and FUNCTION INTEGER-PART(x) returns the integer part TRUNCATED toward zero, matching cobc/libcob (verified 20/0): they AGREE for positives and exact integers but DIFFER for negatives with a fractional part -- INTEGER(-3.7)=-4 (floor) vs INTEGER-PART(-3.7)=-3 (truncate). The pair of implemented rounding intrinsics, split from GNURUST.INTRINSIC.ATLAS.1

## Negative claims (5) — negative capability is the trust surface
- non-numeric arguments
- i128 out-of-range magnitudes
- INTEGER and INTEGER-PART interchangeable
- all dialects
- lie prevented: INTEGER just drops the decimals -- NO: INTEGER is FLOOR not truncation, so INTEGER(-3.7)=-4 (one LESS than INTEGER-PART(-3.7)=-3); using INTEGER where INTEGER-PART is meant biases negatives downward

## Damage if overclaimed
swapping INTEGER and INTEGER-PART silently off-by-one on every negative non-integer in date/age/allocation math

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
