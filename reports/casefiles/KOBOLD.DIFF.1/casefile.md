<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.DIFF.1 (court-casefile)

**Verdict: PASS** · tests/diff.rs (6: identical-passes+target-not-oracle, field/missing/extra, finding-set+control+hash drift, admitted-oracle claims authority, deterministic, allowed_comparisons gate) · crate `kobold-data-shim` kobold 0.6.4

- **Oracle:** the DECLARED target (oracle_status gated; default not_oracle -- not an oracle)
- **Byte domain(s):** actual vs declared-expected artifact -> named structural-diff findings + SARIF
- **Replay:** `the DECLARED target (oracle_status gated; default not_oracle -- not an oracle)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (4)
- structural comparison of KOBOLD output against a DECLARED expected artifact over selected dimensions (field values, audit hashes, finding-id set, control totals): exact match passes
- field/missing/extra/finding-set/control-total/hash drift each emit a named finding
- deterministic
- the target is NEVER called an oracle unless oracle_status permits (default not_oracle)

## Negative claims (7) — negative capability is the trust surface
- business truth/correctness
- the target being an oracle (unless declared)
- validated system-of-record truth
- ledger acceptance
- settlement finality
- customer approval
- lie prevented: 'KOBOLD output matches the expected file, therefore the old system was correct' -- DIFF.1 proves equality to a DECLARED target under selected rules, nothing more

## Damage if overclaimed
a matched diff sold as correctness/approval ratifies a wrong legacy system or an unvalidated export as truth

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
