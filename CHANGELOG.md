# Changelog

All notable changes to `gnucobol-rs` are documented here. The project follows the
oracle-first method: each entry names the slice sealed and the parity it proved.

## [0.2.1]

### GNURUST.4 — `layout`: DATA DIVISION record layout
- `gnucobol_rs::layout::lay_out` assigns each record item its byte offset and one-occurrence size
  — level-numbered nested groups, fixed `OCCURS n TIMES`, `REDEFINES` overlay, `FILLER` — proven to
  match the GnuCOBOL compiler's own record layout (differential sweep vs `cobc` `records=6 PASS=32
  FAIL=0`; cross-checked by runtime `LENGTH OF`). `OCCURS DEPENDING ON`, `SYNCHRONIZED`, and a
  `REDEFINES` larger than its target fail closed. Fuzz: 5M runs, 0 crashes.
- Documentation refresh gate strengthened: every sealed campaign (per receipt) must now be
  referenced in the README and all load-bearing docs, or the gate fails (anti-staleness).

## [0.2.0]

### GNURUST.3 — `pic`: PICTURE → field model
- `gnucobol_rs::pic::build_field` parses the sealed PIC subset (`9 X A S V`, repeats,
  `SIGN [LEADING|TRAILING] [SEPARATE]`, `USAGE DISPLAY`/`COMP-3`) into `{type, digits, scale,
  flags, size}`, proven **byte-identical to the GnuCOBOL compiler's own field-attribute
  computation** (differential sweep PASS=192 FAIL=0 vs `cobc`-emitted `cob_field_attr` + size;
  cross-checked by runtime `LENGTH OF`).
- Fails closed (typed `PicError`) on the `P` scaling symbol (deferred), edited pictures, other
  usages, and malformed/oversized pictures. Fuzz: 5M runs, 0 crashes after fixing an OOM on giant
  repeats (now streamed, O(1) memory, resource-bounded — `GNURUST.DOS.0`).

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
