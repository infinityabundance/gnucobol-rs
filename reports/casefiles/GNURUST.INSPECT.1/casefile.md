<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.INSPECT.1 (court-casefile)

**Verdict: PASS** · 12/12 pass, 0 fail · crate `gnucobol-rs` 0.7.66

- **Oracle:** cobc INSPECT (program-shape, target + count-redefines dump)
- **Byte domain(s):** INSPECT TALLYING/REPLACING/CONVERTING target bytes + tally receiver bytes
- **Replay:** `bash lab/oracle/inspect_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (5)
- the target bytes and tally receiver bytes of narrow INSPECT statements, matching cobc/libcob byte-for-byte: TALLYING FOR ALL (every NON-OVERLAPPING occurrence, left-to-right consume), LEADING (run from the region start), CHARACTERS (every position)
- REPLACING ALL/LEADING/FIRST (equal-length operands)
- CONVERTING (per-byte translation)
- and BEFORE/AFTER INITIAL region restriction (BEFORE absent = whole region, AFTER absent = empty). Scanning is left-to-right non-overlapping
- matching is exact bytes (no case folding)

## Negative claims (8) — negative capability is the trust surface
- full Procedure Division execution
- locale/case-folding
- regex/pattern semantics
- national/UTF-8 multibyte
- unadmitted multi-clause ordering
- business validation
- all dialects
- lie prevented: 'INSPECT counts/replaces are obvious' -- ALL counts NON-OVERLAPPING left-to-right (AAAAAA FOR ALL AA = 3 not 5), LEADING stops at the first non-match, BEFORE/AFTER restrict the region (BEFORE an absent delimiter = the whole field), matching is exact bytes with no case folding

## Damage if overclaimed
a wrong overlap/leading/region/case assumption in an INSPECT-based cleanup or validation transform silently mis-counts or mis-mutates a field

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
