<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.RECON.2 (court-casefile)

**Verdict: PASS** · tests/recon2.rs (4: set+add compose, byte-stable, subtract, undeclared-fails-closed) · crate `kobold-data-shim` kobold 0.6.3

- **Oracle:** composed sealed courts (GNURUST.12 SET-88-TRUE, GNURUST.7 ADD/SUBTRACT)
- **Byte domain(s):** input record bytes -> declared-transform -> output record bytes + before/after decode
- **Replay:** `composed sealed courts (GNURUST.12 SET-88-TRUE, GNURUST.7 ADD/SUBTRACT)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (4)
- a declared sealed transform (SET 88 TRUE / ADD / SUBTRACT) takes input bytes to output bytes
- before+after decode + audit delta
- byte-stable
- undeclared transforms fail closed

## Negative claims (8) — negative capability is the trust surface
- Procedure Division execution
- production write-back
- file rewrite parity
- ledger acceptance
- business truth
- undeclared transforms
- side effects beyond the declared field
- lie prevented: 'a transformed record is a written-back, accepted, business-true record' -- RECON.2 proves only before/after bytes for a named sealed transform

## Damage if overclaimed
treating transform evidence as a production write-back or ledger acceptance mutates or posts data that was never committed

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
