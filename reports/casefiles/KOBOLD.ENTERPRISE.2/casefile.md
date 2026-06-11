<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.ENTERPRISE.2 (court-casefile)

**Verdict: PASS** · kobold-attest (external rust, ed25519) selftest (6 states) + kobold-attest check over all casefiles -> reports/signing/verification-report.json · crate `kobold-data-shim` kobold-attest 0.1.0 (lab tool)

- **Oracle:** ed25519 (ed25519-compact, pinned) over the DSSE PAE; selftest proves all states
- **Byte domain(s):** DSSE PAE over the in-toto payload, verified by ed25519 under a configured key
- **Replay:** `ed25519 (ed25519-compact, pinned) over the DSSE PAE; selftest proves all states`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (1)
- optional cryptographic verification of generated DSSE/in-toto artifacts by the Rust `kobold-attest` tool (ed25519): unsigned is an EXPLICIT honest state (unsigned_no_key_configured), signed verifies under a configured key, and tampered payload / tampered signature / wrong key / payload mismatch each yield a DISTINCT status (no fake green)

## Negative claims (7) — negative capability is the trust surface
- regulatory compliance
- production approval
- customer acceptance
- long-term key custody/rotation/revocation
- identity trust beyond the configured key
- complete supply-chain assurance beyond listed materials
- lie prevented: 'it has a signature, so it is approved/compliant/trusted' -- ENTERPRISE.2 proves only that a configured key signed the exact payload; unsigned stays explicit and honest

## Damage if overclaimed
a signature sold as compliance or production approval manufactures false assurance; a fake-green verifier would pass tampered evidence

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
