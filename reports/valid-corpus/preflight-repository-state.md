# Valid-COBOL corpus task — preflight repository state (Phase 0.1)

Captured immediately after pushing `c35d6c2b9..214cd9cac` to `origin/main`.

## Repository identity

| fact | value |
|---|---|
| HEAD | `214cd9cac126d0a7ec8a4f2182ce9a0b7c1145bc` (`atlas: record Phase-2 evidence for 7b324f50e + 277a07c2e + 23f850352`) |
| `origin/main` | `214cd9cac126d0a7ec8a4f2182ce9a0b7c1145bc` — synced (0 behind, 0 ahead) |
| branch | `main` |
| remote | `https://github.com/infinityabundance/gnucobol-rs.git` |
| latest tag | `v0.8.55` |
| worktree | clean (0 modified tracked files); pre-existing untracked `tf1`, `tf2`, two `cob-prof-prog-*.csv` (predate this task) |
| unpushed | 0 |

## Versioning

| crate | version |
|---|---|
| `gnucobol-rs` | 0.8.55 |
| `cobc-rs` | 0.1.0 |
| `gnucobol-rs-testsuite` | 0.1.0 |
| `gnucobol-rs-ccvs85` | 0.1.0 |
| `gnucobol-rs-port-index` | 0.1.0 |
| `gnucobol-rs-ffi` | 0.1.0 |
| `cobc-oracle-rs` | 0.0.2 |

Claimed published (from the committed release evidence): `gnucobol-rs 0.8.55`,
`gnucobol-rs-testsuite 0.1.0`, `gnucobol-rs-ccvs85 0.1.0`, `gnucobol-rs-port-index 0.1.0`.
crates.io is unreachable from the sandbox; re-verified at Phase 14.

## Identities

- GitHub: `https://github.com/infinityabundance/gnucobol-rs`
- Codeberg: not recorded in the repository (synchronization is a Phase-14 release step)

## Evidence counts

- receipts: 115 (`reports/receipts/`)
- casefiles: 160 (`reports/casefiles/`)

## Current suite summary (GnuCOBOL testsuite lane)

1,282 tests, reconciled:

| primary classification | count |
|---|---|
| CANDIDATE_CHECK_REJECT | 652 |
| OBSERVABLE_MATCH | 193 |
| WRAPPER_OPTION_UNSUPPORTED | 180 |
| CANDIDATE_RUNTIME_FAIL | 136 |
| ORACLE_XFAIL | 31 |
| CANDIDATE_PARSE_REJECT | 31 |
| WRAPPER_INVOCATION_MALFORMED | 25 |
| CANDIDATE_UNSUPPORTED | 21 |
| ORACLE_SKIP | 9 |
| CANDIDATE_MODULE_MODEL_UNSUPPORTED | 4 |

Determinism: two-pass per-test classifications identical.

## Current CCVS85 summary

512 units:

| final classification | count |
|---|---|
| RUST_REJECT_UNSUPPORTED | 190 |
| NON_EXECUTABLE_LIBRARY | 119 |
| ORACLE_RUN_FAIL | 64 |
| OUTPUT_MISMATCH | 42 |
| RUST_REJECT_PARSE | 36 |
| RAW_OUTPUT_MATCH | 27 |
| ORACLE_COMPILE_REJECT | 18 |
| GENERATED_FILE_MISMATCH | 8 |
| ORACLE_COMPILE_ERROR | 3 |
| NON_EXECUTABLE_DATA | 2 |
| HARNESS_BLOCKED / RUST_REJECT_RUNTIME_BOUNDARY / ORACLE_TIMEOUT | 1 each |

## Current math summary

323 math tests (subset of the 1,282-suite inventory; reconciliation invariant machine-enforced).
Distribution by `.at` source: run_functions 126, run_fundamental 114, data_packed 23,
data_binary 18, syn_literals 16, syn_value 13, data_display 9, syn_multiply 3, data_pointer 1.

## Current performance evidence

- `reports/gnucobol-runtime-tests/math-performance.{json,csv,md}` (views A/B, N=200 after 20
  warmups, monotonic ms, pinned environment, raw samples under `raw-samples/`).
- Perf receipts `GNURUST.PERFORM.SLICE.1`, `GNURUST.TABLE.PERFORM.SLICE.1`.

## 0.2 gate note (upfront)

`cargo test --workspace` currently fails on a **pre-existing** stale example
(`crates/gnucobol-rs/examples/cob_intr_rows.rs`, broken by the CHAR/ORD collating sync
`6f4f95fd7` — intrinsic signatures gained a collation argument; the example was not updated;
fails with the SD sync stashed, so it predates this task). Phase 0.2 repairs it so the canonical
gate reproduces.

Machine-readable copy: `reports/valid-corpus/preflight-repository-state.json`.
