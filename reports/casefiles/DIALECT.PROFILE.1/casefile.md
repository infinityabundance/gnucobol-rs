<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — DIALECT.PROFILE.1 (court-casefile)

**Verdict: PASS** · lab/dialect/run.py generate/check (profile self-consistent, -std binds the hash, witness matches live oracle) -> reports/dialect-profile/default.json · crate `kobold-data-shim` governance (gnucobol-rs lab)

- **Oracle:** the admitted GnuCOBOL 3.2.0 (lab/oracle/prefix) cobc/libcob
- **Byte domain(s):** the admitted oracle binaries + dialect -> a hashed witness profile bound into receipts/casefiles
- **Replay:** `the admitted GnuCOBOL 3.2.0 (lab/oracle/prefix) cobc/libcob`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (3)
- records the declared GnuCOBOL witness that produced a court's evidence: compiler+version (3.2.0), dialect/-std, source format, options, oracle identity (cobc/libcob sha256), and a stable profile_sha256 over the canonical content
- changing -std changes profile_sha256
- every oracle replay RECEIPT now references dialect_profile_id + dialect_profile_sha256 -> casefiles carry the witness

## Negative claims (7) — negative capability is the trust surface
- general COBOL behavior
- other dialects
- vendor parity
- runtime portability
- NIST conformance
- platform runtime
- lie prevented: 'COBOL behaves like this' -- DIALECT.PROFILE.1 makes every oracle-grounded claim name its exact GnuCOBOL version + dialect + binaries; dialect profile is EVIDENCE, not metadata

## Damage if overclaimed
presenting one GnuCOBOL build's behavior as 'COBOL' or vendor-equivalent misleads a migration about what was actually witnessed

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
