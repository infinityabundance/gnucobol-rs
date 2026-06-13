# GNURUST.CCVS85.1 — CCVS85 corpus ingest receipt

**GENERATED** by `cargo run -p gnucobol-rs-port-index -- ccvs85 ingest` — do not edit by hand.

`GNURUST.CCVS85.1` admits the historical **CCVS85** COBOL-85 validation corpus as an external
regression gauntlet. It proves only **corpus custody**: the compressed spine's hash, a reproducible
decompression, the decompressed hash, and stable split/index metadata.

**Conformance claim:** NONE — corpus custody/index only; no COBOL-85 conformance, suite-pass, compiler-replacement, or libcob behaviour-parity claim.

## Custody

| fact | value |
|---|---|
| source | `newcob.val.Z` |
| compressed sha256 | `1e9a92ddbd5d730cbeb764281f7810c22b18e0163985b09675393ab22bbd61f9` |
| compressed bytes | 4417395 |
| decompressor | gzip 1.14-modified |
| decompressed sha256 | `744a04982095a3abea29a9df5faf63d226083edecc7b5bf34bc412eae0d53274` |
| decompressed bytes | 28210031 |
| decompressed lines | 348272 |
| version banner | `CCVS85  VERSION 4.0   01 OCT 1992 0032` |

## Index (no conformance claim)

- dialect: COBOL-85 validation (NIST CCVS85, VERSION 4.0)
- split units (`*HEADER`): **512**
- `*HEADER` records: 512
  - `CLBRY`: 51
  - `COBOL`: 459
  - `DATA*`: 2
- `PROGRAM-ID` lines: 524
- `*END-OF` records: 513

The per-unit index (kind, name, line range) is in `reports/ccvs85/corpus-index.json`.

## Boundary

This milestone makes **no** COBOL-85 conformance or suite-pass claim. CCVS85 is broad and old; it
can expose missing compiler/runtime behaviour (work discovery), but per-function byte parity stays
with the oracle sweeps. Compile/run baselines are deferred to later tiered gates
(`GNURUST.CCVS85.2`/`.3`/`.4`).
