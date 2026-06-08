<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.DATE.PROFILE.1 (court-casefile)

**Verdict: PASS** · tests/date_profile.rs (3: valid/invalid-calendar/delegated-sentinel + PIC9-no-claim, undeclared zero-date flags sentinel-required, missing field fails closed) · crate `kobold-data-shim` kobold 0.6.4

- **Oracle:** the declared format + gregorian calendar validity (deterministic)
- **Byte domain(s):** declared date format -> per-field valid|invalid_format|invalid_calendar|declared_sentinel
- **Replay:** `the declared format + gregorian calendar validity (deterministic)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (3)
- a named field is validated against an EXPLICIT declared format (YYYYMMDD / YYDDD) on its RAW digit string (leading zeros preserved)
- sentinel values are delegated to SENTINEL.PROFILE.1 (a declared sentinel is not validated as a date)
- the strongest claim is format_valid_only

## Negative claims (9) — negative capability is the trust surface
- PIC shape = a date
- zero-date = null
- high-date = max/open date
- Y2K window
- business calendar
- settlement/maturity meaning
- currentness/timezone
- date arithmetic
- lie prevented: 'PIC 9(8) so it is a date / 00000000 so it is null / 99999999 so it is forever-open / a 2-digit year so it is 19xx' -- DATE.PROFILE.1 validates only DECLARED formats and refuses all date meaning

## Damage if overclaimed
misreading a numeric id as a date, or a zero/high sentinel as null/forever-open, mis-handles maturities, settlements, and closures across a whole portfolio

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
