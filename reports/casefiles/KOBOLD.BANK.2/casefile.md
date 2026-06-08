<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.BANK.2 (court-casefile)

**Verdict: PASS** · tests/banking.rs (5: balanced+rate-not-summed, tampered, negative-D-still-debit, unknown-fails-closed, credit-routing) · crate `kobold-data-shim` kobold 0.6.3

- **Oracle:** composed gnucobol-rs courts + declared kobold-accounting-profile-v1
- **Byte domain(s):** declared accounting profile over banking detail records -> debit/credit totals
- **Replay:** `composed gnucobol-rs courts + declared kobold-accounting-profile-v1`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (4)
- debit/credit polarity from a declared source field + value tables only (never sign/CR-DB/name)
- only declared Amount-role fields are summed (identifiers/rates/codes never money)
- negative amount with declared D stays debit
- unknown polarity fails closed

## Negative claims (9) — negative capability is the trust surface
- polarity from numeric sign
- polarity from CR/DB presentation
- numeric role from PIC or field name
- transaction-code table unless declared
- account-type/normal-balance polarity
- reversal from negative sign
- currency minor-unit correctness
- posting/ledger/business truth
- lie prevented: 'a numeric field is money and its sign is its posting side' -- ACCOUNTING.PROFILE.1 sums only declared Amount fields and takes polarity only from the declared source

## Damage if overclaimed
summing a rate or account-id as money, or reading a sign as debit/credit, mis-states the ledger

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
