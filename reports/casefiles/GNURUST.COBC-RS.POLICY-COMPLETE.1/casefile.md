<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.COBC-RS.POLICY-COMPLETE.1 (court-casefile)

**Verdict: PASS** · docs/generated/cobc-rs-option-compatibility.md (freshness-gated) · crate `gnucobol-rs` 0.8.56

- **Oracle:** the real invocation census (argv boundaries preserved)
- **Byte domain(s):** policy registry export + invocation census + the generated compatibility document
- **Replay:** `bash lab/oracle/gnucobol-testsuite/run-docker.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (3)
- every option in the real invocation census maps to an explicit policy (translated / accepted-proven-no-op / rejected-unsupported / rejected-ambiguous)
- the machine invariant 'observed options == explicit policy + intentional unknown-option tests + program args after the delimiter' holds
- no unknown semantic option is silently discarded

## Negative claims (5) — negative capability is the trust surface
- no claim that accepted no-op flags preserve semantics outside the admitted tests
- no claim that a rejected option was translated
- no claim that the allowlist covers dialects beyond GnuCOBOL 3.2
- no claim that an observed token with no policy was silently handled (it fails closed)
- lie prevented: 'cobc-rs ignores unknown flags safely' is the lie this prevents

## Damage if overclaimed
claiming policy completeness without the census reconciliation

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
