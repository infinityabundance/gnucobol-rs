# Stable GnuCOBOL 3.2 — immutable before-state (Phase 0 freeze)

Bound at git commit **`c35d6c2b93577013e5257c4bf60e23975d34640e`** (tag `v0.8.55`), reproduced by two fresh rootless-Docker containers with two fresh candidate build trees (run `20260805T215552Z-c35d6c2b`).

## Suite identity

- test-inventory sha256: `858b1babd828b331550a28ab2019b87e4e75e3f0e20b206df4df005cf8202921`
- invocation-census sha256: `7ec5afabb36ff2cb18f50f6d9ff1be6040ace3cdaa9b7c7ff1abb70cafb15bd2`
- summary sha256: `d93791ee242ac48407e158ff36cdc099c7a31ff96bc22a568b804c5ad85cb7f7`
- parser-reject census sha256: `5f8a4ea9df17a6d3bc5dd72a71c354e4fef7b23e6207a0a189ef15ad7066b731`
- parser-census reconciliation sha256: `96b0cdeadb10fb76fb810343b3b65366dfa946e3ab5370e3bf05edbb67266688`
- raw evidence tree sha256: `f32a0b60c297988edb80df20714a634d8a697faf5e9cefdb1941f8743bacacc1` (5639 files)
- oracle: GnuCOBOL 3.2.0 (source `8ecc77d0…`, cobc `98dd2b10…`)
- candidate: cobc-rs/cobcrun-rs at `c35d6c2b93577013e5257c4bf60e23975d34640e` (crate 0.8.55)

## Classifications (1,282 groups, exactly reconciled)

| classification | count |
|---|---|
| OBSERVABLE_MATCH | 193 |
| CANDIDATE_CHECK_REJECT | 652 |
| CANDIDATE_PARSE_REJECT | 31 |
| CANDIDATE_RUNTIME_FAIL | 136 |
| WRAPPER_OPTION_UNSUPPORTED | 180 |
| WRAPPER_INVOCATION_MALFORMED | 25 |
| CANDIDATE_UNSUPPORTED | 21 |
| CANDIDATE_MODULE_MODEL_UNSUPPORTED | 4 |
| ORACLE_SKIP | 9 |
| ORACLE_XFAIL | 31 |

## Invariants

- math subset 323 invariant: **PASS**
- no-delegation: **PASS**
- privacy gate (symbolic storage aliases only): **PASS**
- determinism: stable summaries identical across passes, per-test classifications identical
- parser census: 683 first-failure groups (652 check + 31 parse), phases checker 406 / data-layout 98 / grammar 115 / name-resolution 31 / semantic-check 33; the 700-era stale Markdown family was regenerated (see `parser-census-reconciliation.md`)

## CCVS85 (512 units)

27 raw-output matches, 42 output mismatches, 8 generated-file mismatches, 1 harness-blocked; oracle 370 compile pass / 304 run pass. Determinism verified across fresh containers.
