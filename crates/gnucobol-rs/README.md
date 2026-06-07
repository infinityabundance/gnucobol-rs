# gnucobol-rs

**A faithful, line-cited Rust port of GnuCOBOL's packed-decimal (COMP-3), zoned, and display
numeric *byte* semantics and the `MOVE` conversions between them — proven byte-identical against
the GnuCOBOL 3.2 `libcob` oracle.**

This is a memory-safe (`#![forbid(unsafe_code)]`), dependency-free, **pure** kernel: every function
is a deterministic function of its `(bytes, attrs)` inputs — no global state, no env/locale/fs
reads, panic-free on hostile input. It is part of the [`gnucobol-rs`](https://github.com/infinityabundance/gnucobol-rs)
compatibility court.

## What it does (sealed claims)

Each is proven byte-identical against the GnuCOBOL 3.2 oracle: **`GNURUST.2`** decimal `MOVE` bytes
(below), **`GNURUST.3`** PIC→field-model (`pic`), and **`GNURUST.4`** record layout (`layout`).

### `GNURUST.2` — decimal `MOVE` bytes

For three elementary `cob_move` type pairs on a little-endian ASCII host under `LC_ALL=C.UTF-8`:

- **DISPLAY → DISPLAY** (zoned store, scale alignment, sign)
- **DISPLAY → PACKED** (COMP-3 encode)
- **PACKED → DISPLAY** (COMP-3 decode)

`gnucobol-rs::cob_move` produces **byte-identical** destination field bytes to `libcob`'s
`cob_move`. Verified by a differential sweep of 13,152 cases/seed across 7 seeds (`FAIL=0`), two
sharp Kani proofs, and 20M fuzz runs.

```rust
use gnucobol_rs::{cob_move, FieldAttr, COB_TYPE_NUMERIC_DISPLAY, COB_TYPE_NUMERIC_PACKED, COB_FLAG_HAVE_SIGN};

// MOVE a signed display S9(3)V99 value -012.34 into a COMP-3 field.
let src = [0x30, 0x31, 0x32, 0x33, 0x74]; // "0123" + overpunched '4' ('t')
let src_attr = FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: 5, scale: 2, flags: COB_FLAG_HAVE_SIGN };
let dst_attr = FieldAttr { field_type: COB_TYPE_NUMERIC_PACKED,  digits: 5, scale: 2, flags: COB_FLAG_HAVE_SIGN };
let mut dst = [0u8; 3];
cob_move(&src, &src_attr, &mut dst, &dst_attr).unwrap();
assert_eq!(dst, [0x01, 0x23, 0x4d]); // COMP-3, negative sign nibble 0x0d
```

## `GNURUST.3` — PIC → field model (`gnucobol_rs::pic`)

Parse a COBOL `PIC` clause + `USAGE` into the same field model, matching the GnuCOBOL compiler's
own `cob_field_attr` + storage-size computation (differential sweep vs `cobc`, `PASS=192 FAIL=0`):

```rust
use gnucobol_rs::{build_field, Usage};

let f = build_field("S9(5)V99", Usage::Comp3, false, false).unwrap();
assert_eq!((f.attr.field_type, f.attr.digits, f.attr.scale, f.size), (0x12, 7, 2, 4)); // COMP-3, 4 bytes
```

Sealed subset: `9 X A S V`, repeats `(n)`, `SIGN [LEADING|TRAILING] [SEPARATE]`,
`USAGE DISPLAY`/`COMP-3`. The `P` scaling symbol, edited pictures, and other usages **fail closed**
with a typed `PicError`.

## `GNURUST.4` — DATA DIVISION layout (`gnucobol_rs::layout`)

`lay_out` assigns each record item its byte **offset** and **size** — nested groups, fixed
`OCCURS n TIMES`, `REDEFINES` overlay, and `FILLER` — matching the GnuCOBOL compiler's own record
layout (differential sweep vs `cobc`, `PASS=32 FAIL=0`). `OCCURS DEPENDING ON`, `SYNCHRONIZED`, and
a `REDEFINES` larger than its target **fail closed** with a typed `LayoutError`.

## What it does NOT do

Not a GnuCOBOL replacement, not a compiler, not `libcob`. **No** decimal arithmetic (deferred), no
edited pictures, no `DISPLAY`-statement output, no comparison/collation, no binary/float, no files.
Every other `cob_move` pair **fails closed** with `UnsupportedConversion`. See the repository's
`reports/negative-claims.md` and `docs/future-risk-register.md`.

## License

**LGPL-3.0-or-later** — this is a faithful derivative port of `libcob/move.c`, `libcob/numeric.c`,
and `libcob/common.c` (GnuCOBOL 3.2, © Free Software Foundation, Inc.; authors Keisuke Nishida,
Roger While, Simon Sobisch, et al.), and inherits their copyleft. See `COPYING.LESSER`.
