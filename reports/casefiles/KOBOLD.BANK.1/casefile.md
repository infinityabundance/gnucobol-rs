<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.BANK.1 (court-casefile)

**Verdict: PASS** · recon/banking H/D/T balanced + tampered; tests/banking.rs 3 green · crate `kobold-data-shim` kobold 0.6.3

- **Oracle:** composed gnucobol-rs courts (COMP-3/display/layout) + declared profile
- **Byte domain(s):** banking batch file -> per-variant records + control-total reconciliation
- **Replay:** `composed gnucobol-rs courts (COMP-3/display/layout) + declared profile`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (2)
- variant routing by a declared discriminator (H/D/T) + declared-vs-observed control totals (count/debit/credit)
- balanced file reconciles, tampered trailer fails with a finding, unknown record type fails closed

## Negative claims (8) — negative capability is the trust surface
- posting truth
- ledger truth
- business truth
- debit/credit polarity from a numeric sign
- a balanced file being a correct file
- a trailer match being ledger acceptance
- auto-detected variants
- lie prevented: 'a balanced file is a correct, accepted posting' -- KOBOLD.BANK.1 proves declared==observed totals only, never posting/ledger/business truth

## Damage if overclaimed
treating a balanced file as a correct, accepted posting moves money on an unverified batch

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
