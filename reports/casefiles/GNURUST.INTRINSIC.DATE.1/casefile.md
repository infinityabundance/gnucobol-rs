<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.INTRINSIC.DATE.1 (court-casefile)

**Verdict: PASS** · 30/30 pass, 0 fail · crate `gnucobol-rs` 0.7.59

- **Oracle:** cobc FUNCTION INTEGER-OF-DATE/DATE-OF-INTEGER/INTEGER-OF-DAY/DAY-OF-INTEGER (libcob/intrinsic.c)
- **Byte domain(s):** YYYYMMDD/YYYYDDD <-> integer day number (proleptic Gregorian, 1601-01-01=1)
- **Replay:** `bash lab/oracle/date_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- FUNCTION INTEGER-OF-DATE / DATE-OF-INTEGER / INTEGER-OF-DAY / DAY-OF-INTEGER match cobc/libcob across a spread of dates incl leap days and century boundaries (verified 30/0 + a 200000-day round-trip unit test): COBOL integer dates count days in the PROLEPTIC GREGORIAN calendar from a fixed epoch where 1601-01-01 = day 1 (INTEGER-OF-DATE(20240229)=154557 a leap day
- the four functions are mutual inverses). These are DETERMINISTIC pure calendar math, distinct from the environment-sensitive CURRENT-DATE/WHEN-COMPILED which stay refused. The date intrinsic court on the spine

## Negative claims (6) — negative capability is the trust surface
- validation of malformed dates
- environment-sensitive CURRENT-DATE/WHEN-COMPILED
- business date arithmetic / Y2K windowing
- non-Gregorian calendars
- all dialects
- lie prevented: COBOL dates use a Unix or 1900 epoch -- NO: the integer date counts days from 1600-12-31 so 1601-01-01 is 1, and it is PROLEPTIC GREGORIAN (no Julian-calendar cutover), so a wrong epoch or calendar offsets every converted date

## Damage if overclaimed
using a Unix/1900 epoch or a Julian-calendar model shifts every INTEGER-OF-DATE/DATE-OF-INTEGER result, corrupting date arithmetic in a ported program

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
