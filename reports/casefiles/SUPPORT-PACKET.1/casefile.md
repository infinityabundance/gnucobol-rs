<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — SUPPORT-PACKET.1 (court-casefile)

**Verdict: PASS** · lab/support/run.py generate/check (10 committed artifacts + runtime pointers; gated fresh) · crate `kobold-data-shim` governance (gnucobol-rs lab)

- **Oracle:** the existing generated artifacts (re-gather equality)
- **Byte domain(s):** existing generated artifacts -> one manifest + index (by reference + sha)
- **Replay:** `the existing generated artifacts (re-gather equality)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (2)
- a single generated bundle gathers the EXISTING governance artifacts (STATUS, claim-ladder, negative capabilities, the casefile index + each casefile's SARIF/in-toto/DSSE, the DSSE verification report, release packets, the size-error atlas, the truth-boundary doc, crate versions) by reference + sha256, and POINTS at the runtime/operator artifacts (bench/scale receipts, bank-reconcile/diff reports, redaction/date/currency/sentinel manifests) without embedding them
- it introduces no new evidence and the bundle is re-gather-stable

## Negative claims (7) — negative capability is the trust surface
- new truth/evidence
- certification
- regulatory compliance
- production approval
- customer acceptance
- a live/current state (it is a snapshot)
- lie prevented: 'here is the support packet, therefore the system is certified/compliant/approved' -- SUPPORT-PACKET.1 only collects existing generated artifacts and claims nothing new

## Damage if overclaimed
a bundle sold as certification/compliance/approval manufactures assurance that no court provides

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
