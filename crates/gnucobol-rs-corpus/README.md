# gnucobol-rs-corpus

Valid-COBOL corpus subsystem for `gnucobol-rs`: content-addressed admission, profile-relative
validity classification, phase-attributed candidate measurement, and reproducible reports.

## Validity definition

A program is never "valid COBOL" in the abstract. Validity is represented relative to an
explicit profile:

```text
VALID_FOR(oracle_identity, dialect, source_format, encoding, compiler_options,
          copybook_paths, definitions, runtime_configuration, platform)
```

The oracle contract (compile exit, run exit, stdout/stderr hashes, generated files,
determinism) is recorded per profile. The candidate is measured phase by phase
(`preprocess`, `parse`, `resolve`, `layout`, `check`, `prepare`, `run`) with exactly one
first-failure classification per profile. Candidate output is never used as expected output.

## Admission

Every unit walks the strictly ordered state machine:

```text
DISCOVERED
  -> CUSTODY_VERIFIED
  -> LICENCE_VERIFIED
  -> DEPENDENCIES_RESOLVED
  -> ORACLE_COMPILE_VERIFIED
  -> ORACLE_RUN_VERIFIED
  -> DETERMINISM_VERIFIED
  -> ADMITTED
```

or transitions to a typed rejection state. No source jumps from discovered to admitted, and
`--finalize` refuses to mark a record `ADMITTED` unless the whole chain was walked and (for
valid program classes) the oracle contract plus a reviewed licence are present. Original bytes
are stored content-addressed and are never overwritten; a normalized analysis representation is
kept separately.

## Storage

All large, mutable, downloaded, extracted, compiled and generated corpus data lives beneath
`GNURUST_COBOL_CORPUS_ROOT` (default: `$XDG_DATA_HOME/gnucobol-rs-corpus`). The repository
itself only carries manifests, fetch specs, hashes, licences, small admitted fixtures, patches
and evidence metadata.

## CLI

```text
gnucobol-rs-corpus discover <dir>
gnucobol-rs-corpus fetch <spec.json>
gnucobol-rs-corpus admit --id ID [steps...]
gnucobol-rs-corpus verify <id>
gnucobol-rs-corpus list
gnucobol-rs-corpus classify <id> <CLASS>
gnucobol-rs-corpus run-oracle [steps]
gnucobol-rs-corpus run-candidate [steps]
gnucobol-rs-corpus compare <id>
gnucobol-rs-corpus report
gnucobol-rs-corpus gate
gnucobol-rs-corpus check-updates
```

Every command supports `--json` for structured output. Admission step flags (see
`gnucobol-rs-corpus` usage text) map one-to-one onto the state machine; illegal jumps are
rejected with an error describing the strictly ordered chain.

`fetch` is offline-safe: the archive must already exist as a local file whose SHA-256 equals
the spec's `archive_sha256` (or already be in the store); network downloads are the job of the
family extractors (testsuite / CCVS85 / manual / OMP / X-COBOL) which run where the network is
reachable. Hash mismatches are always rejected.

## Integrity rules

- No `.cob` extension alone makes a unit valid; every unit gets exactly one typed
  classification and none may remain `UNKNOWN` at completion.
- Validity is never collapsed across dialects; the compiler options, copybook paths, defines,
  runtime configuration and platform under which a program is accepted are recorded.
- Exact and near duplicates are identified (five layers: exact bytes, normalized source,
  whitespace-insensitive, structural, token-set similarity) and never counted as independent
  evidence.
- Benchmarking never precedes correctness; the performance corpus is a separate class.
