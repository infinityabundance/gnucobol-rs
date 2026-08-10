# Valid-COBOL corpus system — integration design (Phase 0.3)

This document maps the existing machinery and specifies how the corpus subsystem integrates
with it. It is the written architecture required by the Phase-0 completion gate: extend existing
abstractions wherever technically coherent; never create a disconnected second evidence system.

## 1. Existing machinery inventory (mapped)

| system | location | role | reuse in the corpus system |
|---|---|---|---|
| `gnucobol-rs-testsuite` crate | `crates/gnucobol-rs-testsuite/` | Autotest suite lane: `autotest.rs` (testsuite.log + group-log parse), `census.rs` (JSONL census), `classify.rs` (oracle/candidate/comparison), `model.rs`, `math.rs`, `receipts.rs`, `gate.rs`, `option_census.rs`, `reject_census.rs` | **Phase 2 base**: its `autotest.rs` M4/AT_CHECK awareness is the seed for the syntax-aware step extractor; its `model.rs` classification vocabulary is extended with the corpus-class/validity-profile fields |
| `gnucobol-rs-ccvs85` crate | `crates/gnucobol-rs-ccvs85/` | CCVS85 court: `corpus.rs` (custody, decompress, unit index), `oracle.rs` (compile/run), `candidate.rs`, `compare.rs`, `runner.rs`, `receipts.rs` | **Phase 3 base**: its materialization is the single CCVS85 copy; the corpus system references it, never duplicates |
| `gnucobol-rs-port-index` crate | `crates/gnucobol-rs-port-index/` | C↔Rust symbol parity, FUNCTION-EVIDENCE | consulted for candidate phase boundaries (parse/check) |
| oracle sweep corpus | `lab/corpus/frontend/` + `lab/oracle/*_sweep.sh` | sealed-court sweeps against host GnuCOBOL 3.2.0 | the `@std/@env/@format/@clock/@no312/@no32` per-program marker scheme is adopted for corpus profiles |
| opencbs real-program corpus | `lab/corpus/opencbs/repo` | real-program replay (`GNURUST.ELITE-REPLAY.2`) | a Corpus-B seed; its 39-match baseline is preserved |
| admitted GnuCOBOL source | `lab/admit/gnucobol-3.2/` (stable 3.2.0) and `lab/admit/gnucobol-upstream-current/` (current pin `5568b8fc770f`) | both Autotest suites + manual sources + config trees | **Phase 2 + Phase 4 sources** |
| CCVS85 artifact | `lab/corpus/ccvs85/newcob.val.Z` | NIST CCVS85 source archive | **Phase 3 source** (already under custody) |
| xtask receipt/casefile/ladder | `xtask/src/{receipt,trust4,ladder,support}.rs` + `kobold-courts` | TRUST.2/TRUST.4/claim-ladder/SUPPORT-PACKET | every new corpus court registers a receipt + casefile + claim-ladder entry through these |
| isolated Docker court infra | `lab/ccvs85/run-docker.sh`, `lab/gnucobol-testsuite/run-docker.sh`, `lab/docker/` | rootless isolated daemon, pinned base image, preflight, privacy gate, determinism double-run | `lab/valid-corpus/run-docker.sh` + `lab/performance/run-docker.sh` reuse this verbatim (daemon bootstrap, base-image cache, evidence copy-back, sanitizer) |
| `lab/gnucobol-upstream-current/` atlas | `gen_atlas.py` + `atlas_overrides.json` | upstream commit atlas | unchanged; corpus admission records upstream revisions already pinned there |

## 2. New components

### 2.1 `crates/gnucobol-rs-corpus/` — the corpus subsystem (Phase 1)

Public crate + installed binary `gnucobol-rs-corpus`. Pure Rust (no shell-only implementation).

Modules (implemented):

- `store.rs` — content-addressed store under `GNURUST_COBOL_CORPUS_ROOT` (XDG fallback):
  `blobs/<sha256>`, `manifests/`, `origins/`, `licences/`, `packages/`, `expected/`, `evidence/`,
  `raw/`. Addresses archives, git bundles, files, copybooks, inputs, oracle/candidate binaries,
  expected outputs by hash. Rejects hash mismatches. `open_at` allows embedders/tests to avoid
  process-wide environment state.
- `schema.rs` — versioned admission schema `gnurust-valid-cobol-program-v1` (the task's
  equivalent JSON: program_id, corpus_class, source_family, origin, licence, source, validity
  profile, oracle, candidate, classification, admission_state) + the classification enum
  (all valid classes + typed rejections; no `UNKNOWN` remains at completion).
- `state.rs` — the admission state machine
  `DISCOVERED → CUSTODY_VERIFIED → LICENCE_VERIFIED → DEPENDENCIES_RESOLVED →
  ORACLE_COMPILE_VERIFIED → ORACLE_RUN_VERIFIED → DETERMINISM_VERIFIED → ADMITTED`
  plus typed rejection transitions. No source jumps discovered → admitted.
- `bytes.rs` — byte preservation (original / decoded / normalized / transformed kept separate;
  encoding, BOM, line endings, sequence/indicator/text/identification areas, tab positions
  recorded).
- `dedup.rs` — exact hash, normalized-source hash, whitespace-insensitive hash, structural
  (identifier-normalized) hash, near-duplicate token-set similarity, repository-level grouping.
- `origin.rs` — fetch specifications (git revision, archive hash, extraction rules) and the
  `check-updates` drift reporter.
- `cli.rs` — the phase-attribution vocabulary and the command engine: `discover fetch admit
  verify list classify run-oracle run-candidate compare report gate check-updates`, every
  command with `--json`. The oracle/candidate invocation pattern (compile+run with pinned env,
  sha256 outputs) is fed by the existing `gnucobol-rs-testsuite`/`cobc-oracle-rs` machinery —
  no second oracle implementation.
- `main.rs` — the installed binary `gnucobol-rs-corpus`.

### 2.2 `crates/gnucobol-rs-bench/` — the performance corpus (Phase 8)

Public crate + installed binary `gnucobol-rs-bench`.

- `gen/` — deterministic Rust data generators (seed, schema, record count, expected aggregate
  values, input hash; never the candidate generating its own expected output).
- `workloads/` — the ten required families (payroll, invoices, seq-file batch, relative/indexed,
  tables, strings, modules, float, reports, mixed business workflow), each with
  small/medium/large/stress scales.
- `measure/` — the five views (A end-to-end, B front-end, C repeated prepared vs native, D
  runtime-op microbenchmarks, E corpus throughput), monotonic timing, warmups, median/min/IQR/p95,
  raw sample retention, outlier policy, pinned CPU/container controls.
- `correctness/` — independent expected-output validation before any timing.

### 2.3 Phase-2 syntax-aware Autotest extractor

Extends `gnucobol-rs-testsuite/src/autotest.rs` (does not fork it): an `extract` module in the
corpus crate parses `AT_SETUP/AT_KEYWORDS/AT_DATA/AT_CHECK` with a small recursive-descent M4
layer (balanced quoting, multiline fields, macro expansion of the suite's own helper macros where
deterministic), classifying at `AT_CHECK`-step level and emitting program packages
(source files, copybooks, commands, env, expected outputs, generated-file expectations,
dialect/format, group + step number).

## 3. Data flow

```
discover (per source family, mandatory order)
  -> fetch (immutable revision + hash, content-addressed)
  -> admit (state machine; licence + custody gates)
  -> classify (exactly one class per unit)
  -> run-oracle (profile-relative; stable 3.2 AND current where both exist)
  -> run-candidate (phase-attributed first failure)
  -> compare (raw bytes; generated files; exit; file status)
  -> report (all reports under reports/valid-corpus/, reconciled totals)
  -> gate (freshness, custody, licence, dedup, held-out separation)
```

Every step writes JSON + raw evidence under `GNURUST_COBOL_CORPUS_ROOT`; committed reports under
`reports/valid-corpus/` carry only manifests, hashes, licences, summaries, patches, evidence
metadata — never gigabytes of third-party source.

## 4. Court/evidence integration

Each new court (`GNURUST.CORPUS.*`, `GNURUST.VALID-PROGRAMS.*`, `GNURUST.PERFORMANCE.*`) gets:

- a receipt via the xtask receipt registry (a sweep script under `lab/` returning
  `PASS=n FAIL=n`, registered so `xtask receipt generate` covers it);
- a forensic casefile via `kobold-courts casefile generate`;
- a claim-ladder entry via `xtask ladder`;
- a coverage + porting-ladder mapping (the existing GNURUST.COVERAGE.1 / PORTING-LADDER gates).

Negative claims (task §12.4) are added to `reports/negative-capabilities.json` + the docs model,
never weakening existing claims.

## 5. Docker lanes

`lab/valid-corpus/run-docker.sh` and `lab/performance/run-docker.sh` reuse the ccvs85/testsuite
court scripts: same preflight, isolated rootless daemon, pinned base image, two fresh passes,
determinism compare, privacy sanitizer, evidence copy-back. Corpus data lives under
`$GNURUST_COBOL_CORPUS_ROOT` on the storage drive; committed evidence is symbolic-only.

## 6. Reuse rules (no disconnected second system)

- CCVS85 packages come from `gnucobol-rs-ccvs85` materialization (single copy).
- Autotest parsing extends `gnucobol-rs-testsuite/src/autotest.rs`.
- Oracle invocation uses the existing pinned-env compile/run pattern + `cobc-oracle-rs`.
- Classification vocabulary is a superset of the testsuite `model.rs` enums.
- Receipts/casefiles/claim-ladder flow through the existing xtask + kobold-courts machinery.

## 7. Phase plan → components

| phase | deliverable | component |
|---|---|---|
| 1 | corpus crate + CLI + store + schema + state machine + dedup | `gnucobol-rs-corpus` |
| 2 | Autotest step extractor + packages + replay + drift | corpus `extract` module (extends testsuite autotest) |
| 3 | CCVS85 packages + accuracy/performance reports | references `gnucobol-rs-ccvs85` |
| 4 | manual examples | corpus `manual` extractor over the admitted texinfo |
| 5 | shipped programs + contributions | corpus `extras` admission |
| 6 | OMP course lane | corpus `omp` admission + platform typing |
| 7 | X-COBOL + large collections | corpus `xcobol` admission + quarantine + held-out split |
| 8 | performance workloads | `gnucobol-rs-bench` |
| 9 | unified measurement | bench measure + corpus classify/compare |
| 10 | generalization + metamorphic | corpus `held-out` + `mutation` modules |
| 11 | freshness tracking | corpus `check-updates` |
| 12 | reports/courts/docs | reports/valid-corpus + xtask registry |
| 13 | regression gates | new `lab/valid-corpus/*` + `lab/performance/*` + existing gates |
| 14 | release | existing release pipeline |
