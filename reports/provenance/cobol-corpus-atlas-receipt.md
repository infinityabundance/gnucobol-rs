# GNURUST.COBOL-CORPUS-ATLAS.1 — COBOL validation corpus atlas

**GENERATED** by `cargo run -p gnucobol-rs-port-index -- corpus-atlas generate` — do not edit by hand.

gnucobol-rs tracks COBOL validation across THREE evidence classes — historical conformance suites,
upstream compiler regression suites, and independent real-world / defect corpora. Each corpus is
admitted FIRST by custody + index (immutable source identity + content hash + counts) before any
compile/run or behaviour claim.

**Conformance claim:** NONE — corpus custody/index across three evidence classes; compile/run/behaviour baselines are deferred to per-corpus tiered gates.

Raw corpora are re-downloadable from their permanent sources (gitignored); this manifest +
`reports/cobol-corpus-atlas/atlas.json` are the committed custody spine. `corpus-atlas check`
re-derives custody for any locally-present corpus and diffs it against the committed receipt.

## Corpora

### CCVS85 — NIST CCVS85 COBOL-85 Validation Suite (VERSION 4.0, 1992)

- gate: `GNURUST.CCVS85.1`  ·  class: historical conformance suite  ·  priority: required  ·  status: **LOCAL**
- source: NIST CCVS85 newcob.val.Z (committed spine; mirrors e.g. github.com/.../nistcobol85)
- license: US Government work / public domain (NIST)
- custody: `1e9a92ddbd5d730cbeb764281f7810c22b18e0163985b09675393ab22bbd61f9` (compressed-sha256 (see GNURUST.CCVS85.1))
- counts: 
- claim: custody/index only; no conformance, suite-pass, or behaviour-parity claim

### GNUCOBOL-TESTS — GnuCOBOL 3.2 upstream test suite (tests/, incl. cobol85 NIST-derived)

- gate: `GNURUST.GNUCOBOL-TESTS.1`  ·  class: upstream compiler regression suite  ·  priority: high  ·  status: **LOCAL**
- source: GnuCOBOL 3.2 admitted source tarball (research/gnucobol-3.2.tar.lz)
- license: GPL-3.0 / LGPL-3.0 (GnuCOBOL project)
- custody: `8ecc77d0a4c9401618b8b99adf2050adef14767916767c54bb42341f0ab504fb` (admitted-tarball-sha256 (custody root))
- counts: autotest=38, cobol=2, files=79, text=15
- claim: custody/index only; no conformance, suite-pass, or behaviour-parity claim

### GCOBOL — GCC gcobol testsuite (cobol.dg, COBOL-2023 front end)

- gate: `GNURUST.GCOBOL.1`  ·  class: upstream compiler regression suite  ·  priority: medium-high  ·  status: **LOCAL**
- source: git https://gcc.gnu.org/git/gcc.git (gcc/testsuite/cobol.dg, gcc/cobol, libgcobol)
- license: GPL-3.0-or-later with GCC Runtime Library Exception
- custody: `f62f68e7c4bde0385fbd2dba3e926586dd2f1281` (git-commit-sha)
- counts: cobol=533, copybook=20, files=2110, text=1
- claim: custody/index only; no conformance, suite-pass, or behaviour-parity claim

### OPEN-CBS — OpenCBS — Open-Source COBOL Defects Benchmark Suite (43 programs)

- gate: `GNURUST.OPEN-CBS.1`  ·  class: independent defect corpus  ·  priority: medium-high  ·  status: **LOCAL**
- source: git https://github.com/PhaseChangeSoftware/cobol-defects-suite
- license: see repo LICENSE (Phase Change Software et al.)
- custody: `a7a10bb0330c021c973792d1fd05275475bbcce1` (git-commit-sha)
- counts: cobol=53, copybook=3, files=798, jcl=49, text=1
- claim: custody/index only; no conformance, suite-pass, or behaviour-parity claim

### X-COBOL — X-COBOL — Dataset of Open-Source COBOL Repositories (84 repos, 1255 files)

- gate: `GNURUST.XCOBOL.1`  ·  class: independent real-world corpus  ·  priority: medium  ·  status: **LOCAL**
- source: Zenodo doi:10.5281/zenodo.14269462 (full record archive)
- license: per-repository upstream licenses (mined open-source)
- custody: `14462d4443e06d159e3eb6af8d8be03f7733a5d851d0fa657d3fa94e023d97b3` (archive-sha256)
- counts: archive_bytes=68427907, autotest=38, cobol=3, copybook=5, files=569, text=16
- claim: custody/index only; no conformance, suite-pass, or behaviour-parity claim

