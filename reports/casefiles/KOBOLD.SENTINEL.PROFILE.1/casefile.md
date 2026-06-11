<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.SENTINEL.PROFILE.1 (court-casefile)

**Verdict: PASS** · tests/sentinel.rs (3: declared raw_hex+decoded_value markers match as evidence-only, undeclared zeroes not inferred, missing declared field fails closed) · crate `kobold-data-shim` kobold 0.6.4

- **Oracle:** the declared profile (deterministic match of decoded value / raw bytes)
- **Byte domain(s):** declared sentinel rules -> matched-marker evidence per field
- **Replay:** `the declared profile (deterministic match of decoded value / raw bytes)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (3)
- declared byte/value patterns (raw_hex or decoded_value) are labeled as named markers for named fields and recorded as EVIDENCE ONLY
- an undeclared sentinel-looking value is never inferred (undeclared_inference:false)
- nullness/date/missingness/business-status/account-state/default-meaning stay claimed:false

## Negative claims (8) — negative capability is the trust surface
- LOW-VALUES=null
- HIGH-VALUES=max-date
- SPACES=missing
- ZEROES=absent
- zero-date=a date
- a marker=business status/account state
- an undeclared marker
- lie prevented: 'this field is LOW-VALUES, so it is null / HIGH-VALUES, so it is the max date / spaces, so it is missing' -- SENTINEL.PROFILE.1 records DECLARED markers as evidence only and refuses every meaning

## Damage if overclaimed
treating a sentinel as null/date/missing silently mis-handles closed accounts, open dates, and absent data across a whole file

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
