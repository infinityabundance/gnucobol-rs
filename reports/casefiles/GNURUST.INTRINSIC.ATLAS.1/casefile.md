<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.INTRINSIC.ATLAS.1 (court-casefile)

**Verdict: PASS** · 19/19 pass, 0 fail · crate `gnucobol-rs` 0.7.73

- **Oracle:** cobc FUNCTION intrinsics (libcob/intrinsic.c)
- **Byte domain(s):** declared intrinsic + input -> observed result (deterministic) or shape (env-sensitive)
- **Replay:** `bash lab/oracle/intrinsic_atlas_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (3)
- observed GnuCOBOL results for a declared set of 15 high-use intrinsics under the gnucobol-3.2.0-default witness, each bound to its input and verified by the sweep (19/0): LENGTH/BYTE-LENGTH (byte length), NUMVAL/NUMVAL-C (numeric parse), INTEGER (FLOOR: INTEGER(-3.7)=-4) vs INTEGER-PART (TRUNCATE toward zero: -3), MOD (result takes the DIVISOR sign: MOD(-17,5)=+3) vs REM (DIVIDEND sign: REM(-17,5)=-2), UPPER-CASE/LOWER-CASE/REVERSE, ORD/CHAR (1-based, ORD('A')=66). 13 are deterministic candidate-courts
- CURRENT-DATE
- WHEN-COMPILED are environment-sensitive and admitted as a 21-char SHAPE only, never the value. OBSERVED court (the pure kernel implements none of these yet)

## Negative claims (7) — negative capability is the trust surface
- all intrinsics (a declared subset)
- CURRENT-DATE/WHEN-COMPILED values
- locale/collation in case/ordinal
- national/UTF-8
- that a candidate-court is implemented
- all dialects
- lie prevented: 'intrinsics are obvious' -- INTEGER is FLOOR not truncation (so INTEGER(-3.7)=-4 not -3), MOD takes the DIVISOR sign while REM takes the DIVIDEND sign (so MOD(-17,5)=+3 but REM(-17,5)=-2), ORD/CHAR are 1-based, and CURRENT-DATE/WHEN-COMPILED have only an admitted SHAPE not a value

## Damage if overclaimed
using INTEGER where INTEGER-PART is meant (or MOD/REM interchangeably) on negative values flips results; trusting a CURRENT-DATE value bakes wall-clock time into evidence

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
