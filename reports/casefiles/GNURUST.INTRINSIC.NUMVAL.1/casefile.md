<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.INTRINSIC.NUMVAL.1 (court-casefile)

**Verdict: PASS** · 14/14 pass, 0 fail · crate `gnucobol-rs` 0.7.12

- **Oracle:** cobc FUNCTION NUMVAL (libcob/intrinsic.c)
- **Byte domain(s):** FUNCTION NUMVAL(narrow string) -> parsed value (S9(8)V9(4) display)
- **Replay:** `bash lab/oracle/numval_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- FUNCTION NUMVAL(s) for the NARROW admitted form parses to the same value as cobc/libcob: optional leading/trailing spaces, an optional sign (leading +/-, or trailing +/-/CR/DB -- all of -, CR, DB mean NEGATIVE), digits, and an optional decimal point
- verified by moving NUMVAL into S9(8)V9(4) and matching the receiver display byte-for-byte. The second IMPLEMENTED intrinsic, split narrowly out of GNURUST.INTRINSIC.ATLAS.1

## Negative claims (6) — negative capability is the trust surface
- NUMVAL-C (currency/thousands)
- locale decimal/comma swap
- national/UTF-8
- malformed-input error semantics
- all dialects
- lie prevented: NUMVAL just reads the number -- a TRAILING sign and CR/DB both mean NEGATIVE (NUMVAL of 123.45 CR = -123.45), leading/trailing spaces are stripped, and a leading dot is allowed; currency/comma grouping is NUMVAL-C, NOT admitted here

## Damage if overclaimed
missing the trailing-sign/CR/DB negative convention flips the sign of a parsed amount; assuming NUMVAL strips commas mis-parses NUMVAL-C-shaped input

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
