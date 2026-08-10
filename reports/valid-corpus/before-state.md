# Valid-COBOL corpus task — before-state freeze (Phase 0.2)

Frozen immediately after the Phase-0.2 gate repair commit `69523663a` (gates green).

## Canonical gates

| gate | result |
|---|---|
| `cargo test --workspace` | **PASS** — 25 test binaries, 0 failures (after repairing the pre-existing stale `cob_intr_rows.rs` example) |
| `lab/check-docs.sh` | **PASS** — receipts, forensic casefiles, FUNCTION-EVIDENCE, COBOL/FILE-PARITY, support packet, and root `config/` regenerated mechanically; plus the documented `@no32` drift exemption for `p77_compile_step` |
| `lab/verify-sealed-courts.sh` | **PASS** — 107 green, 0 red (KOBOLD shim SKIP = the `.kobold` sibling crate is absent from this workspace; environmental) |
| `lab/gnucobol-testsuite/run-docker.sh` | not re-run here (multi-hour isolated-Docker run); two-pass determinism + no-delegation + privacy evidence present and fresh; full re-run at Phase 13 |
| `lab/ccvs85/run-docker.sh` | not re-run here; `GNURUST.CCVS85.2/.3/.4` evidence gate **PASSED** inside the sealed-courts gate; full re-run at Phase 13 |

## What this phase changed (all derived or documented, nothing historical)

- Repaired the stale `crates/gnucobol-rs/examples/cob_intr_rows.rs` (CHAR/ORD gained a
  collation argument in the `6f4f95fd7` sync; the example had not been updated and broke the
  workspace build).
- `lab/oracle/cobol_frontend_sweep.sh`: added a per-file, reason-requiring `@no32` exemption for
  a **documented** stable-3.2-vs-current oracle drift (`SOURCE_DATE_EPOCH` MODULE-DATE off-by-one,
  upstream `946f3e638`; the port deliberately targets current upstream). `p77_compile_step.cob`
  declares the reason. The exemption never suppresses an unexplained mismatch.
- Regenerated derived evidence via their documented commands:
  - receipts (`xtask receipt generate`) — `GNURUST.FRONTEND.1`, `GNURUST.ELITE-REPLAY.2`
  - forensic casefiles (`kobold-courts casefile generate`) — `GNURUST.FRONTEND.1`
  - `FUNCTION-EVIDENCE.md` + `reports/port-index/evidence.json` (1138/1138 evidenced)
  - `COBOL-PARITY.md`, `FILE-PARITY.md`
  - `reports/support-packet/support-packet.json`
  - root `config/*.conf` re-copied from `crates/gnucobol-rs/config`
- Re-worded one frontend-test comment that tripped the doc-gate placeholder scan.

## Frozen reference (the baseline every later phase reconciles against)

- **Suite**: 1,282 tests; determinism two-pass identical; no-delegation isolated; top classes:
  CANDIDATE_CHECK_REJECT 652, OBSERVABLE_MATCH 193, WRAPPER_OPTION_UNSUPPORTED 180,
  CANDIDATE_RUNTIME_FAIL 136, ORACLE_XFAIL 31, CANDIDATE_PARSE_REJECT 31,
  WRAPPER_INVOCATION_MALFORMED 25, CANDIDATE_UNSUPPORTED 21, ORACLE_SKIP 9,
  CANDIDATE_MODULE_MODEL_UNSUPPORTED 4.
- **CCVS85**: 512 units; RAW_OUTPUT_MATCH 27, OUTPUT_MISMATCH 42, RUST_REJECT_PARSE 36,
  RUST_REJECT_UNSUPPORTED 190, GENERATED_FILE_MISMATCH 8, ORACLE_RUN_FAIL 64, etc.; evidence gate PASS.
- **Math**: 323 tests; reconciliation invariant machine-enforced.
- **Oracle**: GnuCOBOL 3.2.0 (`lab/oracle/prefix`); current upstream pinned at `5568b8fc770f`
  (`lab/admit/gnucobol-upstream-current`).
- **Candidate**: `gnucobol-rs 0.8.55` interpreter (`cobc-rs` 0.1.0, `cobrun` example).

Machine-readable copy: `reports/valid-corpus/before-state.json`.
