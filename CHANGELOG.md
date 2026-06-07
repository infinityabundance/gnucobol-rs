# Changelog

All notable changes to `gnucobol-rs` are documented here. The project follows the
oracle-first method: each entry names the slice sealed and the parity it proved.

## [0.3.1]

### GNURUST.11 — `cond`: LEVEL-88 condition-name predicate
- `gnucobol_rs::eval_88(attr, bytes, condition)` evaluates whether a LEVEL-88 condition name is true
  for a parent field's current bytes, matching `cobc`: alphanumeric parents compare against the
  literal **space-padded to the parent length** (incl. `THRU` ranges, byte-wise); numeric DISPLAY/
  COMP-3 parents compare by **numeric value** (scale/sign-aware, `THRU` inclusive); single/multiple
  values and ranges. Sweep total=103 PASS=103 FAIL=0; 6M fuzz runs clean. **Predicate only** —
  `SET condition-name`, the `FALSE` clause, condition expressions, Procedure Division execution, and
  collating-sequence-sensitive ranges are non-claims (fail closed). New `cond` module + `Condition`/
  `CondLit`/`CondValue`/`ConditionError` types (purely additive).

## [0.3.0]

### GNURUST.10 — `layout`: OCCURS DEPENDING ON physical-max
- `lay_out` now admits a single, trailing `OCCURS min TO max TIMES DEPENDING ON <ctrl>` (elementary
  or group) as a **physical maximum-layout fact**: the item contributes `max` occurrences, and the
  record total is proven byte-identical to GnuCOBOL's physical allocation `b_REC[size]` (sweep
  `records=30 PASS=30 FAIL=0`). The **active/logical occurrence count is a non-claim**; runtime
  validation, sliding, VALUE-under-ODO, REDEFINES+ODO, multiple/nested ODO, ODO-not-last, and
  `max <= min` fail closed. Layout fuzz with ODO rules: 0 crashes.
- **Breaking (semver-minor, pre-1.0):** `layout::Item` gains a `pub odo: Option<Odo>` field, and a
  new `layout::Odo { min, max, depending_on }` type is exported. Construct `Item` with `odo: None`
  for non-ODO items. (Published companion crates pin `^0.2`, so they are unaffected.)

## [0.2.6]

### GNURUST.9 — `pic`: PIC P-scaling
- `build_field` now admits the `P` scaling symbol, producing the same `(type, digits, scale, size)`
  as `cobc`: **trailing P** (`999PPP`) -> `digits = 9s+P, scale = -P`; **leading P** (`PPP999`) ->
  `digits = 9s, scale = 9s+P`; storage `size` is always the stored `9`s (COMP-3 `n/2+1`, even though
  `digits` carries the P). Sweep PASS=288 FAIL=0. `V`+`P`, P at both ends, and P-only fail closed;
  VALUE/MOVE on a P field is deferred (`GNURUST.VALUE-P.0`) and fails closed (no panic). 6M PIC +
  3M VALUE fuzz runs clean.

## [0.2.5]

### GNURUST.8 — `init`: initial record image from VALUE clauses
- `gnucobol_rs::value_image(items)` computes the WORKING-STORAGE bytes a flat `01` record holds at
  program start, proven byte-identical to `cobc`-initialized storage: alphanumeric `VALUE` (left-
  justified, space-padded), numeric DISPLAY `VALUE` (zoned + overpunch sign), COMP-3 `VALUE` (packed
  via the sealed `cob_move`), and the type-correct defaults — unvalued DISPLAY numeric → `'0'`,
  unvalued alnum → spaces, **unvalued COMP-3 → canonical packed zero** (sign nibble `0x0C`/`0x0F`,
  not raw `0x00`). Sweep PASS=392 FAIL=0; 3M fuzz runs clean. OCCURS/REDEFINES+VALUE, edited/`P`
  PICs, non-fitting literals, and no-VALUE records fail closed.

## [0.2.4]

### GNURUST.7 — `arith`: decimal arithmetic (ADD/SUBTRACT/MULTIPLY)
- `gnucobol_rs::cob_arith` computes `a := a (op) b` in **pure-Rust integer decimal** (i128, zero
  deps, no float), proven byte-identical to libcob `cob_add`/`cob_sub`/`cob_mul`: ADD/SUBTRACT with
  a DISPLAY receiving field + MULTIPLY (DISPLAY/COMP-3), truncation and ROUNDED (nearest-away),
  cross-scale, all sign combos, negative-zero-on-overflow. Sweep PASS=1800 FAIL=0; 8M fuzz runs
  clean. ADD/SUBTRACT into a PACKED field (libcob's separate `cob_add_bcd` path), DIVIDE, the other
  rounding modes, ON SIZE ERROR, and >38-digit (bignum) inputs fail closed (deferred).

## [0.2.3]

### GNURUST.6 — `copybook`: COPY ... REPLACING (whole-text-word)
- `copybook::expand` now applies `COPY name REPLACING ==old== BY ==new== ….` at GnuCOBOL's
  **text-word** granularity (not string substitution): `==AA==` does not touch `AA-X`/`KEEP-AA`,
  the `:tag:` idiom works, multiple pairs apply per word, and a nested copy's brought-in text is
  penetrated by the outer REPLACING (after the nested copy's own) without altering nested operands.
  Proven against `cobc -P` (sweep `programs=7 PASS=7 FAIL=0`). Non-pseudo-text forms
  (`LEADING`/`TRAILING`, identifier operands, unterminated `==`, the `REPLACE` directive) fail
  closed. Fuzz: 4M runs, 0 crashes.

## [0.2.2]

### GNURUST.5 — `copybook`: COPY copybook expansion
- `gnucobol_rs::copybook::expand` splices `COPY <name>.` copybooks into the source — recursively,
  with cycle detection, depth/size limits, and a per-line **provenance map** — proven to match the
  GnuCOBOL preprocessor (`cobc -P`) at text-word granularity (sweep `programs=3 PASS=3 FAIL=0`).
  `COPY ... REPLACING` (whole-text-word replacement) is rejected as a deferred court (`GNURUST.6`);
  recursive/missing/over-deep/over-large copies fail closed. Fuzz: 3M runs, 0 crashes.

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
