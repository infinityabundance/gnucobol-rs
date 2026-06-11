<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.CORPUS.2 (court-casefile)

**Verdict: PASS** · tests/corpus2.rs (6 bucket tests) + recon/corpus2-manifest.json · crate `kobold-data-shim` kobold 0.6.3

- **Oracle:** the sealed courts' own fail-closed behavior (FILE.1/BANK.1/BANK.2/DB2HOST.1/RECON.2/operator dirty-mode)
- **Byte domain(s):** adversarial fixtures -> expected fail-closed findings across all courts
- **Replay:** `the sealed courts' own fail-closed behavior (FILE.1/BANK.1/BANK.2/DB2HOST.1/RECON.2/operator dirty-mode)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (2)
- hostile fixtures across 5 buckets (file/container, storage, banking, database, transform) each produce the expected fail-closed finding/behavior
- no hostile fixture silently decodes as clean

## Negative claims (5) — negative capability is the trust surface
- production representativeness
- customer-data coverage
- business correctness
- exhaustive adversarial coverage
- lie prevented: 'the courts handle the happy path, so they handle real (dirty, hostile, mis-routed) data' -- CORPUS.2 proves each hostile shape fails closed with a named finding

## Damage if overclaimed
trusting the stack on a happy-path corpus alone misses the mis-routed/mis-counted/null/dirty records that cause real banking incidents

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
