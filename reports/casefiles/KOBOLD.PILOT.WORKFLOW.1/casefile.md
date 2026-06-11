<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.PILOT.WORKFLOW.1 (court-casefile)

**Verdict: PASS** · tests/pilot_workflow.rs (1: the chain wires, no cleartext leaks, packet complete + binds all artifacts) · crate `kobold-data-shim` kobold 0.7.1

- **Oracle:** the composed sealed courts (each its own authority) over synthetic bytes
- **Byte domain(s):** a synthetic extract -> the full court chain -> one hash-bound pilot packet
- **Replay:** `the composed sealed courts (each its own authority) over synthetic bytes`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (1)
- a declared synthetic/private-pilot-shaped banking extract flows end-to-end through EXTRACT.PROFILE.1, PRIVACY.REDACTION.1, BANK.1/2, BANK.RECONCILE.1, DIFF.1, TOOLING.EXPORT.1, and PILOT-PACKET.1, proving the workflow PLUMBING and evidence custody: the account id is tokenized before any artifact leaves the secure zone (no cleartext in the redaction, tooling export, or packet), the reconcile view is source-bound to the extract + redaction, the diff target is non-oracle, and the pilot packet hash-binds every produced artifact + the operator review-notes hash (complete:true, creates_new_truth:false)

## Negative claims (7) — negative capability is the trust surface
- customer-data coverage
- production readiness
- regulatory compliance
- business acceptance
- that the synthetic bytes are a real extract
- semantic validation of each court (their own seals are the authority)
- lie prevented: 'the pilot workflow ran, therefore we can decode/reconcile real customer data in production' -- PILOT.WORKFLOW.1 proves only the WIRING + custody over a DECLARED SYNTHETIC extract

## Damage if overclaimed
presenting a synthetic pilot-wiring run as customer-ready/production-ready/compliant manufactures readiness the courts do not provide, and could invite real customer data into an unproven flow

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
