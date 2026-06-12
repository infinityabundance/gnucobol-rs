# GNURUST.MOVE.CLOSE.1 — move.c file closure receipt

**File:** `libcob/move.c` (2 of 13 libcob source files) · **Oracle:** GnuCOBOL 3.2.0 (FSF)
**Machine receipt:** [`move-c-closure-receipt.json`](move-c-closure-receipt.json)

## What this seals

`move.c` is the **second** GnuCOBOL libcob source file ported function-for-function into `gnucobol-rs` —
all **57/57** functions have a named Rust counterpart, **confirmed by doxygen's preprocessed C parse**
(zero functions missed by the line-based inventory, the discipline that caught numeric.c's `_c` functions).

Not clean-room: an **openly-licensed, provenance-documented, oracle-verified Rust port/reconstruction** of
the admitted GnuCOBOL 3.2 `MOVE` conversions + the `cob_get/put_*` field-accessor API.

## Coverage (57/57)

- **MOVE dispatch + leaves** (`move_ops.rs`): `cob_move`, `store_common_region`, the display/packed/
  alphanumeric/binary/fp leaves, `cob_move_all`/`indirect_move`/`cob_move_ibm`/`cob_init_table`, and the
  binary `mget/mset` helpers (incl. a new `f64`↔x87-80-bit codec).
- **Accessor API** (`accessors.rs`): `cob_get/put_int/llint`, the typed `compx`/`comp5`/`comp3`/`comp6`/
  `pic9` (u64+s64), `comp1`/`comp2`, `picx`, `pointer`.
- **Edited leaves** (`edited.rs`): `display↔edited` (sealed encode/decode) + the alphanumeric-edited walk.

## Differential verification (all vs the admitted oracle, FAIL=0)

| sweep | result | sweep | result |
|---|---|---|---|
| decimal MOVE | 13152/0 | typed accessors | 599/0 |
| alphanumeric MOVE | 368/0 | binary MOVE | 546/0 |
| cob_get_int/llint | 158/0 | edited decode / encode | 92/0 / 141/0 |
| COMP-6 MOVE | 98/0 | float fields | 1476/0 |

## Non-claims

- Edited MOVE leaves take the PICTURE string explicitly (the `FieldAttr` model carries digits/scale, not
  the pic); the `cob_move(src,dst)` dispatch fails closed on edited pairs without a pic.
- `long double` (x87 80-bit) is host-architecture-specific; the codec targets x86-64.
- Business/accounting correctness; National/DBCS; screen/report I/O; Procedure-Division execution — other
  files/courts.

## LICENSE-BOUNDARY / PROVENANCE

Faithful **derivative port** of GPL GnuCOBOL 3.2 `move.c`, **not clean-room**, licensed
**LGPL-3.0-or-later**; the Apache-2.0 KOBOLD layer never mixes (enforced by
[`gpl-license-guard-receipt.json`](../license/gpl-license-guard-receipt.json)).
