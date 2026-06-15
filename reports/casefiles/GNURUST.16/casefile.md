<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.16 (court-casefile)

**Verdict: PASS** · 92/92 pass, 0 fail · crate `gnucobol-rs` 0.7.70

- **Oracle:** cobc MOVE numeric -> edited, DISPLAY edited bytes
- **Byte domain(s):** edited DISPLAY field bytes -> value + text
- **Replay:** `bash lab/oracle/edited_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- edited DISPLAY field bytes -> value + presentation text for Z 9 , . -
- (16a) and $ * CR DB B 0 / (16b), slot-based

## Negative claims (5) — negative capability is the trust surface
- report writer
- locale/currency
- EBCDIC edited
- edited arithmetic/VALUE
- lie prevented: 'presentation string and numeric value are the same field truth' — they are separated

## Damage if overclaimed
reading an edited presentation string as the numeric value double-counts or mis-totals

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
