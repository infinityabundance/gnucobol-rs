<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.2 (court-casefile)

**Verdict: PASS** · 13152/13152 pass, 0 fail · crate `gnucobol-rs` 0.8.52

- **Oracle:** libcob cob_move (runtime harness)
- **Byte domain(s):** field-storage bytes
- **Replay:** `bash lab/oracle/sweep.sh 0`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- COMP-3/zoned/display field-storage bytes + cob_move result bytes

## Negative claims (4) — negative capability is the trust surface
- arithmetic
- edited pictures
- any other type pair
- lie prevented: 'a MOVE result is close enough' — every byte of the receiving field is proven

## Damage if overclaimed
a wrong MOVE result posted as a real value corrupts a record silently

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
