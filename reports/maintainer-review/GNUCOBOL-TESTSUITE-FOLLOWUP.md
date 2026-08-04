# GnuCOBOL test-suite integration — maintainer-facing follow-up

> Evidence packet (GNURUST.GNUCOBOL-TESTSUITE.{1,2,3} + runtime/math + methodology). Read time ≈ 5–10 minutes.
> Every number below is a MEASURED result from the isolated Docker court; nothing is projected.

## 1. What was implemented

- A first-class `cobc-rs` compatibility driver (`crates/cobc-rs/`) driven by an explicit
  option-policy registry (translated / accepted-equivalent / accepted-proven-no-op /
  rejected-unsupported / rejected-ambiguous — never a silent "ignore unknown flags").
- A truthful artifact model: `cobc-rs -x -o prog` writes a **launcher symlink + JSON manifest +
  expanded source** — an interpreter launch artifact, **never** a native COBOL executable.
- The GnuCOBOL 3.2 native Autotest suite runs with `COBC=cobc-rs` through its own machinery
  (`make localcheck`, `GNUCOBOL_TEST_LOCAL=1`, atlocal bootstrap, `TESTSUITEFLAGS=--jobs=12`).
- Host-side evidence pipeline: census → classify → determinism → receipts → privacy gate →
  option-compatibility doc → runtime/math subset.
- Methodology/provenance docs: `docs/methodology/{libcob-rust-port,parser-front-end-provenance}.md`
  + machine records under `reports/methodology/`.

## 2. Exact GnuCOBOL source identity

- GnuCOBOL 3.2 tarball, sha256 `8ecc77d0a4c9401618b8b99adf2050adef14767916767c54bb42341f0ab504fb`
  (the repository's admitted oracle source; never a distribution package).
- Configure: `./configure --prefix=/work/oracle/prefix --with-db BDB_CFLAGS=-I/usr/include/db5.3
  BDB_LIBS=-ldb-5.3 CFLAGS="-O2 -std=gnu17 -fsigned-char"` — stock (no `-fpermissive`, no compat
  `-Wno-*`), identical in the baseline and candidate trees.

## 3. Exact commands

```sh
# baseline (real admitted cobc, in-tree)
( cd <baseline-tree> && make check TESTSUITEFLAGS="--jobs=12" )

# candidate (COBC=cobc-rs through the native harness)
( cd <candidate-tree>/tests && env COBC=.../cobc COBCRUN=.../cobcrun \
    make localcheck TESTSUITEFLAGS="--jobs=12" )

# one-command replay (public, configurable storage)
bash lab/gnucobol-testsuite/run-docker.sh
```

## 4. Baseline totals (real admitted cobc, in-tree, this environment)

- **1242 pass · 9 skip · 31 xfail · 0 fail · 0 timeout** (1282 test groups reconciled). The baseline
  is fully green under the stock configuration; the 31 xfails are the suite's own expected failures.
- Full ledger: `reports/gnucobol-testsuite/summary.md`; raw logs: `reports/gnucobol-testsuite/raw/`.

## 5. Candidate totals (COBC=cobc-rs)

- **173 OBSERVABLE_MATCH** (the test's own AT_CHECK assertions held on both sides) ·
  **439 candidate check/parse rejects** (fail closed) · **407 module-model unsupported** (`-m` /
  `cobcrun -M` boundary) · **173 wrapper-option unsupported** (honest rejection of native-code
  modes and unmodeled flags) · **26 wrapper-malformed** · **22 candidate unsupported** ·
  **2 runtime fails** · **0 timeouts · 0 not-reached · 0 nondeterministic** — all 1282 accounted.

## 6. Runtime-test totals

- The runtime subset (files, sequential I/O, report writer, extensions) is classified inside the
  same 1282-test ledger; the math subset is reported separately (next section). Raw per-group logs
  are preserved under `reports/gnucobol-testsuite/raw/`.

## 7. Math-test totals

- **323 math tests** (data_binary / data_display / data_packed / data_pointer / run_fundamental /
  run_functions / syn_multiply / syn_value / syn_literals), classified from the SAME results as the
  whole suite (no favorable selection): **97 OBSERVABLE_MATCH · 52 check-reject · 147 module-model
  unsupported · 21 wrapper-option · 1 malformed · 1 unsupported · 3 oracle-skip · 1 oracle-xfail**.
  See `reports/gnucobol-runtime-tests/math-correctness.md`.

## 8. Correctness matches

- **173 of 1282 suite tests** and **97 of 323 math tests** produced identical observable results on
  both sides (the suite's own AT_CHECK assertions). 0 timeouts, 0 nondeterministic outcomes.

## 9. Performance findings (strict caveats)

- Performance is reported ONLY for programs proven output-identical on both sides, in three
  SEPARATE views (A: end-to-end workflow, observational; B: repeated per-run; C: runtime-operation
  microbenchmarks — a separately-designed court, not mixed into A/B). Measured on this pinned
  machine/container (AMD Ryzen 7 9800X3D, 16 cores), N=200 after 20 warmups:

  | program | View A native (compile+run, ms) | View A candidate (adapt+run, ms) | View B native (ms/run) | View B candidate (ms/run) |
  |---|---|---|---:|---:|
  | mixed_moves | 58 | 12 | 1.0 | 12.0 |
  | packed_math | 58 | 12 | 1.0 | 11.1 |
  | display_arith | 58 | 23 | 2.0 | 23.0 |
  | packed_loop | 57 | 17 | 1.0 | 17.0 |

- View A being faster for the candidate is compile-vs-parse work, not a runtime claim; View B shows
  the interpreter (reparse included) slower per run, as expected. No "Rust is faster than
  GnuCOBOL" claim is made or implied — the execution models differ and only per-workload measured
  statements are made. Full methodology: `reports/gnucobol-runtime-tests/math-performance.md`.

## 10. Largest candidate blockers

- **Module model (407 tests):** the suite's `$COBCRUN_DIRECT ./prog` / `-m` module lifecycle — the
  candidate's manifest-based module resolution covers `-x` artifacts; the dynamic-module model is a
  typed boundary (`CANDIDATE_MODULE_MODEL_UNSUPPORTED`).
- **Parser/check rejects (439):** the sealed front-end subset rejects constructs the suite uses
  (fail closed — never guessed).
- **Wrapper-option unsupported (173):** native-code modes (`-C`, `-S`, `-c`, listings) and
  unmodeled flags are rejected honestly.
- **Float SIZE-ERROR semantics:** the candidate's decimal-domain float arithmetic does not raise the
  IEEE-range SIZE ERROR at 2^127/2^1024 (a measured divergence, not a hang; three related
  non-termination defects found during bring-up are FIXED — see the CHANGELOG).

## 11. GnuCOBOL-side observations

- The baseline is fully green (0 real failures) under the stock configuration; 9 skips are the
  suite's own `AT_SKIP_IF` conditions (e.g. `COB_HAS_ISAM`/screen availability) and 31 xfails are
  suite-marked expected failures. See `reports/gnucobol-testsuite/upstream-observations.md`.

## 12. `cobc-rs` option coverage

- The generated `docs/generated/cobc-rs-option-compatibility.md` maps EVERY option observed in the
  real invocation census (~2111 cobc/cobcrun invocations) to an explicit policy (translated /
  accepted-proven-no-op / rejected-unsupported); only the suite's intentional-unknown options
  (e.g. `--thisoptiondoesntexist`) and module program-args show NO POLICY and fail closed.

## 13. How to reproduce

```sh
export GNURUST_GNUCOBOL_TEST_DOCKER_ROOT=/path/on/a-large-filesystem/gnucobol-rs
export GNURUST_GNUCOBOL_TEST_BASE_IMAGE=/path/to/ubuntu-minimal-image.tar
bash lab/gnucobol-testsuite/run-docker.sh
```

The replay uses a dedicated rootless Docker daemon whose mutable state is stored beneath
`$GNURUST_GNUCOBOL_TEST_DOCKER_ROOT`; the harness verifies it never touches the production socket
and that all daemon data/cache/temp/outputs stay beneath the configured root.

## 14. Raw evidence links

- `reports/gnucobol-testsuite/raw/` — baseline + candidate `testsuite.log`, per-group dirs, census
  JSONL, execve trace.
- `reports/gnucobol-testsuite/{invocation-census,oracle-results,candidate-results,
  comparison-results,summary,failure-buckets,no-delegation,determinism}.{json,md,csv}`.
- `reports/receipts/GNURUST.GNUCOBOL-TESTSUITE.{1,2,3}/` — replay receipts.

## 15. Methodology documents

- [`docs/methodology/libcob-rust-port.md`](../../docs/methodology/libcob-rust-port.md) — the
  runtime is a faithful LGPL-3.0-or-later **derivative** of the admitted `libcob` sources (not
  clean-room); statement-by-statement with upstream line citations; 100% symbol parity.
- [`docs/methodology/parser-front-end-provenance.md`](../../docs/methodology/parser-front-end-provenance.md) —
  the parser is **independently written per the author's committed from-scratch claim**; strict
  clean-room process separation is NOT independently verifiable from the committed record
  (tooling/consulted-materials history UNKNOWN).
- Machine records: `reports/methodology/{libcob-port-provenance,parser-provenance}.json`.

## 16. Questions where further guidance would be valuable

- Should `cobc-rs` grow a build-local **module registry** (so `-m` + `cobcrun -M <module>` resolve
  through the candidate model) beyond the current manifest-based resolution? The suite's module
  tests are currently classified `CANDIDATE_MODULE_MODEL_UNSUPPORTED` — an honest boundary, but a
  registry would move a large bucket to executed-and-compared.
- Diagnostic-shape parity: several suite tests assert cobc's EXACT stderr wording (e.g. "cobc:
  unrecognized option '…'"). Matching those shapes is a distinct compatibility surface from
  semantics — worth a dedicated court?
- The `-fno-diagnostics-show-option`-style flags are proven no-ops for the admitted suite. The
  allowlist is generated from the census; should it be re-derived per release?
