<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.SCALE.1 (court-casefile)

**Verdict: PASS** · kobold-scale (100m smoke + 1g admitted receipt; parity-gated; ~bounded RSS) + reports/SCALE-1-*.json · crate `kobold-data-shim` kobold-bench

- **Oracle:** the scalar reconcile output (pinned per-size baseline)
- **Byte domain(s):** multi-GB synthetic corpus -> streamed scalar/rayon decode timing gated on identical output hash
- **Replay:** `the scalar reconcile output (pinned per-size baseline)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (7)
- streams a declared synthetic mixed fixed-record corpus (DISPLAY
- COMP-3
- COMP
- COMP-X
- LEVEL-88) through the sealed reconcile pipeline in fixed reconcile-blocks with BOUNDED memory
- scalar and Rayon use the same block unit so their output hashes are byte-identical, and Rayon timing is admitted only after that match + a pinned baseline
- records wall time, throughput, peak RSS, temp disk, and the POSTING.1 hash chain

## Negative claims (7) — negative capability is the trust surface
- customer-workload performance
- production SLA
- AWS cost
- mainframe equivalence
- universal throughput
- that a synthetic corpus is a representative business corpus
- lie prevented: 'it does 2.5M records/sec, so it will handle your production batch on AWS' -- SCALE.1 measures a declared synthetic corpus on one host, parity-gated, and refuses every production/cost/representativeness generalization

## Damage if overclaimed
a synthetic scale number sold as production/AWS capacity sets false batch-window and cost expectations for a real migration

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
