<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.INTRINSIC.MOD-REM.1 (court-casefile)

**Verdict: PASS** · 20/20 pass, 0 fail · crate `gnucobol-rs` 0.7.36

- **Oracle:** cobc FUNCTION MOD/REM (libcob/intrinsic.c)
- **Byte domain(s):** FUNCTION MOD/REM(integer a, b) -> value
- **Replay:** `bash lab/oracle/modrem_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- FUNCTION MOD(a,b) and FUNCTION REM(a,b) for integer operands match cobc/libcob across the full sign matrix (verified 20/0): MOD = a - b*floor(a/b) so the result takes the DIVISOR sign (MOD(-17,5)=3, MOD(17,-5)=-3), REM = a - b*trunc(a/b) = a%b so the result takes the DIVIDEND sign (REM(-17,5)=-2, REM(17,-5)=2). The third pair of implemented intrinsics, split from GNURUST.INTRINSIC.ATLAS.1

## Negative claims (5) — negative capability is the trust surface
- non-integer operands
- MOD/REM by zero
- MOD and REM interchangeable
- all dialects
- lie prevented: MOD and REM are the same -- NO: on negatives they DIFFER (MOD takes the DIVISOR sign, REM the DIVIDEND sign), so MOD(-17,5)=3 but REM(-17,5)=-2; using one for the other flips the sign of a modulo result

## Damage if overclaimed
swapping MOD and REM (or assuming a language's % semantics) silently flips the sign of remainders in interest/allocation/check-digit logic on negative values

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
