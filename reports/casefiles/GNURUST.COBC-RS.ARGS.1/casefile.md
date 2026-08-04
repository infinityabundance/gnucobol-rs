<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.COBC-RS.ARGS.1 (court-casefile)

**Verdict: PASS** · crates/cobc-rs/tests/cli.rs + docs/generated/cobc-rs-option-compatibility.md · crate `gnucobol-rs` 0.8.54

- **Oracle:** the real cobc/cobcrun invocation census of the admitted suite (behavioral reference for accepted spellings)
- **Byte domain(s):** the policy registry + the generated compatibility table + the argument-parsing integration tests
- **Replay:** `the real cobc/cobcrun invocation census of the admitted suite (behavioral reference for accepted spellings)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (2)
- the cobc-rs option-policy registry (crates/cobc-rs/src/policy.rs) assigns every option an explicit policy (translated / accepted-equivalent / accepted-proven-no-op / rejected-unsupported / rejected-ambiguous) and the generated option-compatibility table (docs/generated/cobc-rs-option-compatibility.md) maps EVERY option observed in the real invocation census to its policy, with the intentional unknowns (e.g. --thisoptiondoesntexist) failing closed
- integration tests cover short/long/attached/separated values, --compat modes, malformed invocations, and the getopt_long --x == -x equivalence

## Negative claims (3) — negative capability is the trust surface
- no claim that accepted no-op flags preserve semantics outside the admitted tests
- no full cobc CLI replacement
- lie prevented: 'cobc-rs ignores unknown flags like cobc' is the lie this prevents -- every option has an explicit policy

## Damage if overclaimed
silently dropping a semantic option would change program semantics without a record

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
