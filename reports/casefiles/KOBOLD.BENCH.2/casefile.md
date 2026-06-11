<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.BENCH.2 (court-casefile)

**Verdict: PASS** · kobold-bench2 (parity-gated; tampered baseline aborts, verified) + reports/BENCH-2-receipt.json · crate `kobold-data-shim` kobold-bench (path: gnucobol-rs 0.7 + kobold-data-shim 0.6)

- **Oracle:** the shim's own byte-stable reconcile output (parity baseline)
- **Byte domain(s):** end-to-end pipeline timing gated on byte-stable output hash
- **Replay:** `the shim's own byte-stable reconcile output (parity baseline)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (2)
- the full reconciliation pipeline (FILE.1 ingest+decode+LEVEL-88+audit) is measured in scalar mode ONLY after the output/audit hash matches the pinned baseline
- a mismatch aborts with no timing admitted

## Negative claims (5) — negative capability is the trust surface
- production performance
- AWS performance
- parallel/Rayon/SIMD throughput
- customer-workload representativeness
- lie prevented: 'it is fast, so it is correct' / 'a hot-path number is the product throughput' -- BENCH.2 admits timing only behind a parity gate and measures the full pipeline, not just cob_move

## Damage if overclaimed
a parallel or hot-path benchmark number sold as production throughput sets false capacity/SLA expectations for a real migration

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
