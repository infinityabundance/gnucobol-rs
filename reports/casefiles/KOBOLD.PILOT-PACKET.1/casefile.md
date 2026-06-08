<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.PILOT-PACKET.1 (court-casefile)

**Verdict: PASS** · tests/pilot.rs (3: binds artifacts + hashes review notes (no cleartext leak), changed artifact changes the packet, missing required flagged) · crate `kobold-data-shim` kobold 0.7.1

- **Oracle:** the pilot run's existing court artifacts (re-derive equality)
- **Byte domain(s):** a pilot run's existing court artifacts -> one hash-pinned packet + operator checklist
- **Replay:** `the pilot run's existing court artifacts (re-derive equality)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (2)
- a generated derived view over a pilot run's existing hash-pinned court artifacts (EXTRACT.PROFILE.1, the redaction policy, BANK.RECONCILE.1, DIFF.1, TOOLING.EXPORT.1, the SCALE receipt, DSSE verification), the copybook sha, an operator review checklist, and a review-notes hash (the notes are hashed, never embedded)
- derived_view true and creates_new_truth false, so a changed source artifact changes the packet hash and missing required artifacts (extract/redaction/bank_reconcile) are flagged complete:false

## Negative claims (10) — negative capability is the trust surface
- certification
- regulatory compliance
- production approval
- customer acceptance
- a live or current state (it is a snapshot)
- new truth or new evidence
- checklist completion (the checklist is a template, not proof it was done)
- review-notes content verification (only their hash is pinned)
- re-validation of the bundled artifacts (they are pinned, not re-run)
- lie prevented: 'here is the pilot packet, therefore the migration is approved/compliant/accepted' -- PILOT-PACKET.1 bundles EXISTING hash-pinned pilot artifacts and claims pilot evidence only

## Damage if overclaimed
a pilot bundle sold as certification/compliance/production-approval/customer-acceptance manufactures pilot-to-production assurance that no court provides

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
