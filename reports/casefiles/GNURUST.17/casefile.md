<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.17 (court-casefile)

**Verdict: PASS** · 120/120 pass, 0 fail · crate `gnucobol-rs` 0.8.50

- **Oracle:** cobc -fsign=EBCDIC via edited intermediary
- **Byte domain(s):** raw EBCDIC zoned field bytes -> value
- **Replay:** `bash lab/oracle/ebcdic_num_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- raw cp500 EBCDIC zoned-decimal numeric DISPLAY bytes -> value (cp500 translate + cob_get_sign_ebcdic overpunch sign)

## Negative claims (6) — negative capability is the trust surface
- cp037
- edited numeric under cp500
- binary/packed via this path
- mixed/auto-detect encodings
- national/DBCS
- lie prevented: 'EBCDIC numeric decodes like ASCII zoned' — the C/D/F sign zones differ

## Damage if overclaimed
a wrong zoned sign flips the sign of mainframe money

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
