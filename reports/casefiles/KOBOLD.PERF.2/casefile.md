<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.PERF.2 (court-casefile)

**Verdict: PASS** · tests/perf2.rs (profile-preserves-output + stages-populated; full-custody scalar==rayon) + bench2 perf2_stage_profile (bottleneck=per_record) · crate `kobold-data-shim` kobold 0.6.4

- **Oracle:** the scalar reconcile output (PERF.1 parity) + reconcile_profile == reconcile_encoded bytes
- **Byte domain(s):** per-stage timing of the reconcile pipeline; parallel record-local work, ordered aggregation
- **Replay:** `the scalar reconcile output (PERF.1 parity) + reconcile_profile == reconcile_encoded bytes`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (5)
- the reconcile pipeline is a 3-stage shape -- (1) parse/prepare once, (2) parallel record-local evidence, (3) ordered aggregation -- with reconcile_profile() exposing per-stage wall-clock (StageProfile parse_ns/record_ns/aggregate_ns) WITHOUT changing the emitted bytes
- profiling identifies the per-record stage as the bottleneck (~75%) that PERF.1's Rayon already parallelizes byte-identically, with aggregation (~25%) kept serial
- the full custody workload (reconcile
- POSTING.1 chain
- PRIVACY.REDACTION.1) is byte-identical scalar vs rayon

## Negative claims (7) — negative capability is the trust surface
- parallel posting-chain
- parallel order being optional
- a JSON fast-path as authority
- thread schedule as evidence
- float parallel totals
- a customer SLA
- lie prevented: 'more threads, so optimize everything in parallel' -- PERF.2 parallelizes ONLY record-local work, keeps custody/aggregation ordered and serial, and admits timing only after full evidence parity

## Damage if overclaimed
a parallelized custody/aggregation that diverges from scalar would corrupt evidence (reordered records, a different posting chain) while looking faster

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
