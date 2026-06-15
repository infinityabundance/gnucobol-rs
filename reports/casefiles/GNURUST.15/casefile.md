<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.15 (court-casefile)

**Verdict: PASS** · 256/256 pass, 0 fail · crate `gnucobol-rs` 0.7.69

- **Oracle:** libcob cob_load_collation (ebcdic500_ascii8bit)
- **Byte domain(s):** raw EBCDIC field bytes -> decoded text
- **Replay:** `bash lab/oracle/ebcdic_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- raw EBCDIC alphanumeric DISPLAY bytes -> text under the admitted cp500 table

## Negative claims (6) — negative capability is the trust surface
- cp037/other code pages
- numeric EBCDIC zoned sign
- national/DBCS
- collation
- binary/packed conversion
- lie prevented: 'EBCDIC conversion can be record-wide' — it is per-DISPLAY-field; binary/packed raw

## Damage if overclaimed
a wrong EBCDIC decode silently garbles names, codes, and statuses

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
