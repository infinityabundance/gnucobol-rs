# Changelog

All notable changes to `gnucobol-rs` are documented here. The project follows the
oracle-first method: each entry names the slice sealed and the parity it proved.

## [0.1.1]
- Flagship crate published under the project name **`gnucobol-rs`** (lib `gnucobol_rs`); the
  internal decimal-court receipt id is `RECEIPT-GNURUST-DECIMAL-1`. No semantic change.

## [0.1.0]

### GNURUST.1 — admitted oracle + receipt harness
- Admitted GnuCOBOL 3.2 from pinned source (`research/gnucobol-3.2.tar.lz`), built `cobc` +
  `libcob` 3.2.0 (with Berkeley DB 5.3 INDEXED I/O) into a gitignored lab prefix.
- `cobc-oracle-rs`: build/run `cobc -x` fixtures and capture deterministic JSON receipts
  (source/stdout/stderr/exit + sha256, oracle + platform identity).
- Project doctrine: claim boundary, porting method, derivation/license boundary.

### GNURUST.2 — `gnucobol-rs` byte court
- Faithful LGPL-3.0+ port of GnuCOBOL packed-decimal (COMP-3), zoned, and display numeric
  byte semantics and the `MOVE` conversions between them, proven byte-identical against the
  built `libcob` runtime-library oracle. Kani reduced-surface proof + detached fuzz harness.
