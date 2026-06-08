<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.PERF.1 (court-casefile)

**Verdict: PASS** · tests/perf1.rs (scalar==rayon across n=1..999 + posting chain) + kobold-bench2 --features rayon (parity-gated, ~2.9x on this host) · crate `kobold-data-shim` kobold 0.6.3 (rayon feature) + kobold-bench

- **Oracle:** the scalar reconcile output (parity baseline)
- **Byte domain(s):** parallel decode timing gated on byte-identical output to scalar
- **Replay:** `the scalar reconcile output (parity baseline)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (2)
- optional record-level Rayon (off-by-default `rayon` feature) emits byte-identical evidence to scalar across the workload -- same JSONL, audit, unsupported ledger, decode_output_sha256, and downstream posting hash chain
- the benchmark refuses Rayon timing unless the parity hash matches the scalar baseline

## Negative claims (7) — negative capability is the trust surface
- production SLA
- AWS throughput
- SIMD
- deterministic scheduling beyond identical emitted artifacts
- customer-workload representativeness
- any semantic change
- lie prevented: 'parallel is faster, so ship it' / 'faster means a different (better) answer' -- PERF.1 admits parallelism ONLY when every emitted artifact is byte-identical to scalar

## Damage if overclaimed
a parallel path that silently differs from scalar would corrupt evidence while looking faster; a benchmark sold as an SLA sets false capacity expectations

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
