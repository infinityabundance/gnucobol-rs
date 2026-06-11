<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.CURRENCY.PROFILE.1 (court-casefile)

**Verdict: PASS** · tests/currency.rs (3: amount validates + code evidence-only, scale-mismatch + rate-not-money, missing field/code fail closed) · crate `kobold-data-shim` kobold 0.6.5

- **Oracle:** the declared profile + the decoded implied scale (deterministic)
- **Byte domain(s):** declared amount profile -> per-field scale-match evidence + currency-code evidence
- **Replay:** `the declared profile + the decoded implied scale (deterministic)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (3)
- a named numeric field declared role=amount is checked against an explicit declared scale (observed implied scale vs declared -> scale-match or finding) with an optional currency-code field preserved as EVIDENCE (never legal-tender truth)
- a non-amount role (rate/percent/id) is not admitted as money
- sign is NOT polarity (BANK.2 owns debit/credit)

## Negative claims (9) — negative capability is the trust surface
- V99 = money
- minor-unit inferred from PIC
- currency code = legal tender
- FX conversion
- rounding policy
- sign = polarity
- a rate = an amount
- a decoded amount = business value
- lie prevented: 'PIC V99 so it is money / there is a CCY field so it is legal tender / it is negative so it is a credit' -- CURRENCY.PROFILE.1 admits declared amount-scale evidence only and refuses every money meaning

## Damage if overclaimed
summing rates as money, assuming a 2-decimal minor unit on a 0/3-decimal currency, or reading a sign as a credit, mis-states totals across an entire ledger extract

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
