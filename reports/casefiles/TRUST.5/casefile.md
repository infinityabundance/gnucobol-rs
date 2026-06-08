<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — TRUST.5 (court-casefile)

**Verdict: PASS** · lab/trust5/run.py audit/check (A=16 B=28 C=5 D=0 F=0; gates F-empty + view no-new-truth + audit freshness) · crate `kobold-data-shim` governance (gnucobol-rs lab)

- **Oracle:** the claim-ladder + casefiles + receipts + negative-capabilities (re-audit equality)
- **Byte domain(s):** every court -> {class, can_fail proof, no-new-truth, neg>=pos}; F-set must be empty
- **Replay:** `the claim-ladder + casefiles + receipts + negative-capabilities (re-audit equality)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (2)
- classifies every claim-ladder court (A hard oracle/byte, B composed fail-closed, C view, D staged, F ceremonial) and gates that NO court is class F: each court must be generated from named evidence, carry a concrete drift/mutation detector (oracle sweep, fixture, or regenerate-and-compare), create no new truth (every VIEW court has a no-new-truth refusal), have damage_if_overclaimed + negatives>=positives
- the audit itself re-derives and fails on stale/hand-edit

## Negative claims (5) — negative capability is the trust surface
- certification
- that class is a quality score
- a live state (snapshot)
- new truth
- lie prevented: 'all these courts are real' -- TRUST.5 forces each court to PROVE it can fail (corrupt its evidence -> a gate goes red), and refuses to let a prose-only court count as a court

## Damage if overclaimed
presenting the audit as certification/compliance, or its class letters as a quality score, manufactures assurance the courts do not provide

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
