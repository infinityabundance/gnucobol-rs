# ENTERPRISE.2 — signed attestation verification (generated; do not edit)

Verification of every casefile DSSE envelope against its in-toto payload, by the **Rust** `kobold-attest` tool (ed25519; no Python crypto). Regenerate with `kobold-attest report --root . --write`.

- signing mode: **unsigned**
- tool available: true  ·  selftest passed: true
- casefiles: 120
- status summary: `{"unsigned_no_key_configured":120}`

`unsigned_no_key_configured` is the **honest default** — not a failure. Set a signed policy + key to produce `signed_verified`. No regulatory/production/customer-acceptance/key-custody/supply-chain claim.
