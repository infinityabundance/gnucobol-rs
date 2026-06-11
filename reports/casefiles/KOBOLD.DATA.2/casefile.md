<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.DATA.2 (composition-casefile)

**Verdict: PASS** · 3 families x 120 incl binary fields, byte-stable · crate `kobold-data-shim` kobold-data-shim 0.3.0

- **Oracle:** sealed GNURUST courts (no new oracle)
- **Byte domain(s):** JSON + audit bytes
- **Replay:** `sealed GNURUST courts (no new oracle)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (1)
- COMP/COMP-5/COMP-X binary fields decoded end-to-end in the reconciliation corpus (composes GNURUST.14)

## Negative claims (3) — negative capability is the trust surface
- binary arithmetic
- business truth
- lie prevented: 'binary fields are text under any encoding' — they are raw storage passthrough

## Damage if overclaimed
reading a binary field as text under an encoding corrupts keys and amounts

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
