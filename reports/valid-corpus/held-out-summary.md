# Held-out evaluation (Phase 10.3)

PURE MEASUREMENT: the held-out set was never used for implementation tuning and this report feeds nothing back into the candidate.

Every probe/run is bounded at 2s per file.

Every probe/run is bounded at 2s per file.

| measure | count |
|---|---|
| files | 101 |
| parse ok | 43 |
| check ok | 24 |
| run ok | 16 |
| crashed | 0 |
| timed out | 0 |

First-failure buckets:
- check: 19
- none: 16
- parse: 55
- preprocess: 3
- run: 8

See `held-out-results.json` for the per-file rows. This report is a pure
measurement; the held-out set is never used to modify the candidate.
