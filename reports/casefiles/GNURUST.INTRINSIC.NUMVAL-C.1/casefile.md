<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.INTRINSIC.NUMVAL-C.1 (court-casefile)

**Verdict: PASS** · 10/10 pass, 0 fail · crate `gnucobol-rs` 0.7.31

- **Oracle:** cobc FUNCTION NUMVAL-C (libcob/intrinsic.c)
- **Byte domain(s):** FUNCTION NUMVAL-C(currency string) -> value
- **Replay:** `bash lab/oracle/numvalc_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- FUNCTION NUMVAL-C(s) for the narrow admitted form parses to the same value as cobc/libcob: like NUMVAL but first STRIPS the default currency symbol $ and thousands-separator commas, then applies the NUMVAL sign/space/decimal rules (verified 10/0): NUMVAL-C of $1,234.56 = 1234.56, NUMVAL-C of $1,234.56CR = -1234.56. Implemented as intrinsic_numval after removing $ and comma -- completing the NUMVAL family, split from GNURUST.INTRINSIC.ATLAS.1

## Negative claims (5) — negative capability is the trust surface
- non-default currency symbol (2-arg form)
- DECIMAL-POINT IS COMMA / locale comma-decimal
- national/UTF-8
- all dialects
- lie prevented: NUMVAL and NUMVAL-C are the same -- NO: NUMVAL-C additionally strips the currency symbol and thousands commas (NUMVAL would NOT parse a $1,234.56 string), while still honoring the trailing-sign/CR/DB negative convention

## Damage if overclaimed
using NUMVAL on currency-formatted input (or assuming a non-$ symbol/European decimal) misparses monetary amounts

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
