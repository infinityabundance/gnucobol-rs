# GnuCOBOL test-suite integration — maintainer-facing follow-up (boundary-reduction work)

> Evidence packet (GNURUST.GNUCOBOL-TESTSUITE.{1,2,3,4,BOUNDARY-REDUCTION} + GNURUST.MODULE.* +
> GNURUST.COBC-RS.* + GNURUST.GNUCOBOL-RUNTIME-MATH.{1,2}). Read time ≈ 5–10 minutes.
> Every number below is a MEASURED result from the isolated Docker court (two fresh passes,
> determinism-identical); nothing is projected.

## 1. What was implemented (after Simon's guidance)

- **First-failure attribution corrected.** The v0.8.54 "module-model" bucket (407 tests) was
  dominated by `$COBCRUN_DIRECT ./prog` runs whose real first failure was a cobrun
  parse/check/runtime diagnostic — the module-model check shadowed them. The classifier now
  attributes each failure to its TRUE boundary (same diagnostic, same class, on the syntax-only
  and the run path). This is re-measurement, not reclassification-for-counts: every move has raw
  evidence.
- **An interpreted module lifecycle.** `cobc-rs -m` is silent (cobc is silent); `cobcrun-rs`
  resolves modules through `-M <dir>` (trailing slash appended, GnuCOBOL semantics), cwd and
  `COB_LIBRARY_PATH`, passes program arguments to `ACCEPT FROM COMMAND-LINE`, emits cobcrun-shaped
  diagnostics, loads runtime config files (`-c <cfg>` / `COB_RUNTIME_CONFIG` / `COB_CONFIG_DIR`)
  with libcob-shaped errors and recursive-include detection, and `--runtime-conf` reflects applied
  values, environment priority and `${...}` expansion.
- **CALL/CANCEL semantics.** CALL across separately compiled modules (sibling resolution +
  EXTERNAL sharing), CANCEL resets persisted WORKING-STORAGE, and CANCELing the active
  non-INITIAL program raises the libcob-shaped fatal `attempt to CANCEL active program` (exit 1).
- **Parser/check feature families** (measured, coherent, complete): cobc-exact DISPLAY of numeric /
  E-notation / binary / hex literals; level-78 named constants and the `01 ... CONSTANT [GLOBAL]`
  clause; USAGE BINARY-INT / FLOAT-SHORT / FLOAT-LONG / FLOAT-DOUBLE / FLOAT-EXTENDED / HANDLE.
- **Diagnostic dimensions.** Each test now carries `semantic_diagnostic_verdict` (REJECT = the
  candidate correctly rejected invalid source) and `diagnostic_shape_parity` (MATCH / DIFFERS vs
  the oracle's expected stderr) — exact cobc wording is NOT required for a correct rejection.

## 2. Exact GnuCOBOL source identity

- GnuCOBOL 3.2 tarball, sha256 `8ecc77d0a4c9401618b8b99adf2050adef14767916767c54bb42341f0ab504fb`
  (the repository's admitted oracle source; never a distribution package).
- Configure: `./configure --prefix=/work/oracle/prefix --with-db BDB_CFLAGS=-I/usr/include/db5.3
  BDB_LIBS=-ldb-5.3 CFLAGS="-O2 -std=gnu17 -fsigned-char"` — stock, identical in every tree.

## 3. Exact commands (unambiguous candidate identity)

```sh
# baseline (real admitted cobc, in-tree)
( cd <baseline-tree> && make check TESTSUITEFLAGS="--jobs=12" )

# candidate (conceptual form — the candidate binaries are cobc-rs and cobcrun-rs)
( cd <candidate-tree>/tests && env COBC=/work/candidate-bin/cobc-rs COBCRUN=/work/candidate-bin/cobcrun-rs \
    make localcheck TESTSUITEFLAGS="--jobs=12" )

# candidate (real form — the harness requires PATH entries named `cobc`/`cobcrun`;
# candidate-bin/cobc is a symlink to cobc-rs and candidate-bin/cobcrun a symlink to cobcrun-rs;
# both targets + sha256 are recorded in no-delegation.json — NEVER the oracle binaries)
( cd <candidate-tree>/tests && env COBC=/work/candidate-bin/cobc COBCRUN=/work/candidate-bin/cobcrun \
    make localcheck TESTSUITEFLAGS="--jobs=12" )

# one-command replay (public, configurable storage)
bash lab/gnucobol-testsuite/run-docker.sh
```

## 4. Baseline totals (real admitted cobc, in-tree, this environment)

- **1242 pass · 9 skip · 31 xfail · 0 fail · 0 timeout** (1282 groups reconciled; re-run fresh for
  every candidate measurement — the expectations never changed).

## 5. Candidate totals (before → after)

| boundary | v0.8.54 | now | change |
|---|---:|---:|---|
| OBSERVABLE_MATCH | 173 | **193** | +20 |
| module-model unsupported | 407 | **4** | −403 (re-attributed + implemented) |
| candidate check/parse rejects | 439 | **683** | the honest re-attribution; 9 became matches, 7 became runtime-fails |
| candidate runtime fails | 2 | **136** | the launcher-run failures now attributed to runtime |
| candidate unsupported | 22 | 21 | −1 |
| wrapper-option unsupported | 173 | 180 | +7 honest re-attribution (`-fbinary-size` etc. surface first) |
| wrapper-invocation malformed | 26 | 25 | −1 |
| timeout / nondeterministic / not-reached | 0 | **0** | unchanged |

All 1282 groups reconcile exactly; two fresh passes are classification-identical.

## 6. Module boundary (407 → 4)

Measured transitions (per-test in `boundary-reduction.json` / `classification-transitions.json`):
- 9 → OBSERVABLE_MATCH (the genuine module tests now pass: `cobcrun -M`, CANCEL, runtime config).
- 238 → check-reject, 30 → parse-reject, 127 → runtime-fail: honest first-failure re-attribution.
- 4 remain module-model (cobcrun-side errors not yet matched, e.g. parts of 0040/0044).

Module semantics now supported (GNURUST.MODULE.{REGISTRY,CALL,CANCEL,SEARCH,PARALLEL}.1):
- silent `-m` launcher+manifest artifacts (never native `.so`);
- `cobcrun -M <dir> <program> [args]`, cwd + COB_LIBRARY_PATH search;
- cobcrun error messages + exit codes (`missing PROGRAM name`, `invalid module argument ''`,
  `cannot find module`);
- CALL across separately compiled modules with EXTERNAL sharing;
- CANCEL state reset + active-program fatal;
- 100-way parallel same-basename isolation;
- tampered-manifest refusal (self-hash).

Remaining module boundaries: no native shared-object semantics, no ABI compatibility with real
cobcrun, no dlopen — documented as non-claims.

## 7. Parser/check boundary (decomposed + first families)

Census (`parser-reject-census.{json,md}`, `parser-feature-frequency.csv`,
`parser-feature-dependency-graph.json`): 683 rejects by phase — checker 406, grammar 115,
data-layout 98, semantic-check 33, name-resolution 31. Implemented families:
- DISPLAY literals (numeric / sign / E-notation / B'…' / BX'…' as decimal) — cobc-exact;
- level-78 named constants + `01 … CONSTANT [GLOBAL] …`;
- USAGE BINARY-INT / FLOAT-SHORT / FLOAT-LONG / FLOAT-DOUBLE / FLOAT-EXTENDED / HANDLE.
The remaining rejects are deliberately NOT chased as one-off grammar productions: they are a
scatter of distinct constructs (diagnostic-shape tests on intentionally invalid sources,
individual dialect forms), each with a documented reason code.

## 8. Wrapper-option boundary (173 → 180, decomposed)

Census (`unsupported-option-census.{json,md}`): 180 tests by option — 82 dialect/extension flags
(`-fttitle` 35, `-fnotrunc` 9, `-fodoslide` 6, …), 35 listing (`-t-`/`-ftsymbols`/`-Xref`), 5
native-code modes (`-C`, `-c`), plus `-j`/`-b`/long options. No unknown semantic option is
silently discarded; native-code modes remain an honest typed boundary (no fake C/assembly/object
files); every accepted no-op appears in the invocation ledger; the policy registry is
machine-reconciled against the invocation census.

## 9. Math subset (exactly 323, machine-enforced)

The ledger confirms (and the generator FAILS unless this holds):
`sum(math classification totals) == 323 == unique math ids`, ids ⊆ suite, one classification per
test. Current distribution: **101 OBSERVABLE_MATCH · 126 check-reject · 28 wrapper-option ·
7 parse-reject · 56 runtime-fail · 1 unsupported · 3 oracle-skip · 1 oracle-xfail** (= 323).
The v0.8.54 prose discrepancy (a stale "21 wrapper-option / 1 malformed" claim vs the ledger's
22/0) is corrected: the ledger is the only source, and a freshness test prevents prose drift.

## 10. Performance (strict caveats — unchanged methodology)

Performance is reported ONLY for programs proven output-identical on both sides, in three separate
views; no "Rust is faster than GnuCOBOL" claim is made or implied. The execution models differ
(native compile+run vs reparse+interpret); per-workload measured statements only.

## 11. GnuCOBOL-side observations

The baseline is fully green (0 real failures) under the stock configuration; 9 skips and 31 xfails
are the suite's own declared conditions. See `upstream-observations.md`. One observation from this
work: `COBCRUN_DIRECT` is empty in the suite's atlocal (`$COBCRUN_DIRECT ./prog` == `./prog`), and
`COB_RUNTIME_CONFIG` is exported to `<srcdir>/config/runtime_empty.cfg` in every mode while
`COB_CONFIG_DIR` is exported only outside local mode — the candidate honors both.

## 12. `cobc-rs` option coverage

`docs/generated/cobc-rs-option-compatibility.md` maps EVERY option in the real invocation census to
an explicit policy (translated / accepted-proven-no-op / rejected-unsupported); only the suite's
intentional-unknown options show NO POLICY and fail closed. The doc is freshness-gated.

## 13. How to reproduce

```sh
export GNURUST_GNUCOBOL_TEST_DOCKER_ROOT=/path/on-a-large-filesystem/gnucobol-rs
export GNURUST_GNUCOBOL_TEST_BASE_IMAGE=/path/to/ubuntu-minimal-image.tar
bash lab/gnucobol-testsuite/run-docker.sh
```

## 14. Raw evidence links

- `reports/gnucobol-testsuite/raw/` — baseline + candidate `testsuite.log`, per-group dirs, census,
  execve trace; `boundary-reduction-baseline.json` binds the v0.8.54 snapshot (commit, identities,
  ledger + raw hashes); `boundary-reduction.{json,md}` and `classification-transitions.{json,md}`
  are the before/after ledger.
- `reports/gnucobol-testsuite/{module-lifecycle-census,parser-reject-census,
  unsupported-option-census}.{json,md}` + `no-delegation.json` + `determinism.json`.
- `reports/receipts/GNURUST.GNUCOBOL-TESTSUITE.{1,2,3,4}/` + the GNURUST.MODULE.* and
  GNURUST.COBC-RS.* receipts + `reports/casefiles/` (160 forensic casefiles).

## 15. Methodology documents

- [`docs/methodology/libcob-rust-port.md`](../../docs/methodology/libcob-rust-port.md) — the
  runtime is a faithful LGPL-3.0-or-later **derivative** of the admitted `libcob` sources (not
  clean-room).
- [`docs/methodology/parser-front-end-provenance.md`](../../docs/methodology/parser-front-end-provenance.md) —
  the parser is **independently written per the author's committed claim**; strict clean-room is
  NOT independently verifiable (tooling history UNKNOWN).

## 16. Questions where further guidance would be valuable

- The diagnostic-shape parity dimension (MATCH / DIFFERS) is now recorded per test; should the
  suite's diagnostic tests (intentionally invalid sources) be reported as a separate headline
  count, or kept inside check-reject with the verdict dimension?
- The remaining 4 module-model tests need either a fuller cobcrun config-fidelity surface or an
  explicit boundary — worth a dedicated follow-up court?
- Should the listing flags (`-t-`, `-ftsymbols`, `-Xref`, 35 tests) get a first-class candidate
  listing format (honest banner, never byte-claimed as cobc's), or stay a typed boundary?
