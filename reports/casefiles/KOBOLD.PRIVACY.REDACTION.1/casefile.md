<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.PRIVACY.REDACTION.1 (court-casefile)

**Verdict: PASS** · tests/privacy.rs (3: redact+tokenize+allow keep hashes/provenance, deterministic-token stable-not-reversible, unlisted-fails-closed) · crate `kobold-data-shim` kobold 0.6.3

- **Oracle:** the declared redaction policy (deterministic; verified by hash preservation)
- **Byte domain(s):** decoded record + declared policy -> redacted evidence (hashes/provenance preserved)
- **Replay:** `the declared redaction policy (deterministic; verified by hash preservation)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (2)
- declared field-level redaction for generated evidence: values are withheld (redact) or tokenized (deterministic, scope-stable) while value_sha256/raw_sha256, offset/size, court identity, and audit structure are preserved
- unlisted fields fail closed under a deny-unlisted policy

## Negative claims (7) — negative capability is the trust surface
- anonymization
- regulatory compliance (GDPR/PCI)
- reversibility
- safe public release of customer data
- a token being an identity
- a hash being the business value
- lie prevented: 'redacted = anonymized = safe to publish'  -- PRIVACY.REDACTION.1 withholds declared values and preserves auditability, but claims no anonymization, compliance, reversibility, or public-release safety

## Damage if overclaimed
publishing a 'redacted' casefile as anonymized/compliant can re-identify customers and breach regulation

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
