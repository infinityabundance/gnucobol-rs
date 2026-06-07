# cobol-decimal-rs

**A faithful, line-cited Rust port of GnuCOBOL's packed-decimal (COMP-3), zoned, and display
numeric *byte* semantics and the `MOVE` conversions between them — proven byte-identical against
the GnuCOBOL 3.2 `libcob` oracle.**

This is a memory-safe (`#![forbid(unsafe_code)]`), dependency-free, **pure** kernel: every function
is a deterministic function of its `(bytes, attrs)` inputs — no global state, no env/locale/fs
reads, panic-free on hostile input. It is part of the [`gnucobol-rs`](https://github.com/infinityabundance/gnucobol-rs)
compatibility court.

## What it does (sealed claim)

For three elementary `cob_move` type pairs on a little-endian ASCII host under `LC_ALL=C.UTF-8`:

- **DISPLAY → DISPLAY** (zoned store, scale alignment, sign)
- **DISPLAY → PACKED** (COMP-3 encode)
- **PACKED → DISPLAY** (COMP-3 decode)

`cobol-decimal-rs::cob_move` produces **byte-identical** destination field bytes to `libcob`'s
`cob_move`. Verified by a differential sweep of 13,152 cases/seed across 7 seeds (`FAIL=0`), two
sharp Kani proofs, and 20M fuzz runs.

```rust
use cobol_decimal_rs::{cob_move, FieldAttr, COB_TYPE_NUMERIC_DISPLAY, COB_TYPE_NUMERIC_PACKED, COB_FLAG_HAVE_SIGN};

// MOVE a signed display S9(3)V99 value -012.34 into a COMP-3 field.
let src = [0x30, 0x31, 0x32, 0x33, 0x74]; // "0123" + overpunched '4' ('t')
let src_attr = FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: 5, scale: 2, flags: COB_FLAG_HAVE_SIGN };
let dst_attr = FieldAttr { field_type: COB_TYPE_NUMERIC_PACKED,  digits: 5, scale: 2, flags: COB_FLAG_HAVE_SIGN };
let mut dst = [0u8; 3];
cob_move(&src, &src_attr, &mut dst, &dst_attr).unwrap();
assert_eq!(dst, [0x01, 0x23, 0x4d]); // COMP-3, negative sign nibble 0x0d
```

## What it does NOT do

Not a GnuCOBOL replacement, not a compiler, not `libcob`. **No** decimal arithmetic (deferred), no
edited pictures, no `DISPLAY`-statement output, no comparison/collation, no binary/float, no files.
Every other `cob_move` pair **fails closed** with `UnsupportedConversion`. See the repository's
`reports/negative-claims.md` and `docs/future-risk-register.md`.

## License

**LGPL-3.0-or-later** — this is a faithful derivative port of `libcob/move.c`, `libcob/numeric.c`,
and `libcob/common.c` (GnuCOBOL 3.2, © Free Software Foundation, Inc.; authors Keisuke Nishida,
Roger While, Simon Sobisch, et al.), and inherits their copyleft. See `COPYING.LESSER`.
