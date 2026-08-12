# Phase 0 — Diagnostic-unblocked lane preflight (frozen state)

_schema: `gnurust-diag-unblocked-preflight-v1`_

## 0.1 Repository identity

| item | value |
|---|---|
| git commit (HEAD) | `d197c7abf3cdf8616361d3cf82613444ecc5f831` |
| HEAD subject | `evidence: gnucobol-rs-corpus 0.1.3 ENTERPRISE.1 release packet at release commit 256bee474 + support packet refreshed` |
| origin/main | `d197c7abf3cdf8616361d3cf82613444ecc5f831` (synced) |
| Codeberg HEAD | `d197c7abf3cdf8616361d3cf82613444ecc5f831` (mirror synced) |
| worktree | clean (0 modified, 0 untracked) |
| unpushed commits | 0 |
| current version | gnucobol-rs 0.8.57, gnucobol-rs-corpus 0.1.3, gnucobol-rs-bench 0.1.1, gnucobol-rs-testsuite 0.1.1 |
| remote tag | `v0.8.57` → `d8a50f851` |
| published crates | gnucobol-rs 0.8.57, gnucobol-rs-corpus 0.1.3, gnucobol-rs-bench 0.1.1, gnucobol-rs-testsuite 0.1.1 |
| courts | 175 (139 GNURUST + 31 KOBOLD + 5 other) |
| receipts | 130 (129 GNURUST + 1 SIZE.ERROR.ATLAS.1) |
| casefiles | 175 |

## 0.2 Admitted source identities

| source | identity |
|---|---|
| stable GnuCOBOL 3.2 tarball | `research/gnucobol-3.2.tar.lz`, sha256 `8ecc77d0a4c9401618b8b99adf2050adef14767916767c54bb42341f0ab504fb` |
| stable testsuite source files (36 `.at` under `tests/testsuite.src/`) | combined sha256 `741b782f1cdf26cb559484e0c14155f5216274b1978f1689f6dbefc60a966bc5` |
| stable `tests/testsuite.at` entry | present (m4_include list + AT_INIT/AT_BANNER) |
| stable generated `tests/testsuite` | present, 8123454 bytes, header `Generated from testsuite.at by GNU Autoconf 2.69` |
| current upstream | `lab/admit/gnucobol-upstream-current` @ `5568b8fc770ff310e5017300d561d8f3deec257c` |

## 0.3 Upstream regeneration mechanism (Phase 0.4)

The admitted stable tree builds its generated Autotest `testsuite` with the upstream mechanism:

```make
# tests/Makefile.am:133-135
$(TESTSUITE): $(testsuite_sources) $(srcdir)/package.m4 $(srcdir)/testsuite.at
	autom4te --language=autotest -I $(srcdir) -I $(srcdir)/testsuite.src -o $(srcdir)/testsuite $(srcdir)/testsuite.at
```

i.e. `make -C tests testsuite` regenerates `tests/testsuite` from `tests/testsuite.at`
+ `tests/testsuite.src/*.at` + `tests/package.m4`. `package.m4` is generated from
`configure.ac` by an existing rule (Makefile.am:121-131). The tree is configured with
`./configure --prefix=/work/oracle/prefix --with-db BDB_CFLAGS=-I/usr/include/db5.3
BDB_LIBS=-ldb-5.3 CFLAGS=-O2 -std=gnu17 -fsigned-char` (identical to the pristine court lane).

The lane will therefore:
1. Fresh-extract the admitted tarball (sha256-verified) into a scratch tree,
2. apply the mechanically generated diagnostic-ignore patch to `tests/testsuite.src/*.at`,
3. run `make -C tests testsuite` (the real upstream build mechanism),
4. run the regenerated suite with the oracle (`make check`) and the candidate
   (`make localcheck` with `COBC=cobc-rs`), exactly like the pristine lane.

## 0.4 Current evidence summaries (baselines this lane must not change)

| ledger | value |
|---|---|
| pristine testsuite oracle | 1242 pass / 9 skip / 31 xfail / 0 fail (1282 reconciled groups) |
| pristine testsuite candidate | 196 observable matches, 648 check rejects, 34 parse rejects, 137 runtime fails, 0 timeouts |
| AT_CHECK-level corpus (testsuite family) | 4732 valid-programs units extracted at step level |
| unified corpus | 6442 units across 6 families |

## 0.5 Gates run at freeze time (2026-08-12)

| gate | result |
|---|---|
| `bash lab/check-docs.sh` | PASS |
| `bash lab/verify-sealed-courts.sh` | 107 green, 0 red |
| `cargo run -p gnucobol-rs-corpus -- gate` | GREEN |
| `cargo test --workspace` | 31 binaries, 0 failures (last verified) |
| corpus court sweep | 14/14 PASS |

## 0.6 Design constraints honored by this lane

- The pristine lane (`lab/gnucobol-testsuite/run-docker.sh`), its raw evidence, its
  reports (`reports/gnucobol-testsuite/*`), the corpus extractor, candidate phase probes,
  historical receipts and classifications are ALL left untouched.
- The diagnostic-unblocked lane is additive: new report root
  `reports/gnucobol-testsuite/diagnostic-unblocked/`, new court
  `GNURUST.GNUCOBOL-TESTSUITE.DIAGNOSTIC-UNBLOCKED.1`.
- The transformer reuses the existing syntax-aware M4/Autotest parser
  (`crates/gnucobol-rs-corpus/src/extract/{m4,at}.rs`) — no regex-only parsing, fail closed
  on uncertain constructs.
