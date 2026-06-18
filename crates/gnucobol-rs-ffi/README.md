[![crates.io](https://img.shields.io/crates/v/gnucobol-rs-ffi.svg)](https://crates.io/crates/gnucobol-rs-ffi)

**A `cob_field` C-ABI shim that exposes the native-Rust `libcob` algorithms of [`gnucobol-rs`](https://crates.io/crates/gnucobol-rs) with `extern "C"` linkage, so a C program can link `gnucobol-rs` as a drop-in `libcob` replacement.** The `cob_field` / `cob_field_attr` struct layouts match GnuCOBOL 3.2's `libcob` (`common.h`), so existing field buffers work unchanged.

This is the **only** crate in the workspace that uses `unsafe` — confined to the raw-pointer C boundary. The `gnucobol-rs` core stays `#![forbid(unsafe_code)]`; every function here just reads the C struct, calls the safe-Rust algorithm, and writes the result back.

---

## Use it

```c
#include "gnucobol-rs-ffi.h"

cob_field_attr da  = { 0x12 /*PACKED*/, 4, 0, 0, 0 };
cob_field      dst = { 3, buf, &da };

cob_move(&src, &dst);            /* native-Rust MOVE, byte-identical to libcob */
int n = cob_get_int(&dst);
```

Build a `cdylib` (`libgnucobol_rs_ffi.so`) or `staticlib` and **link it where you would link `libcob`**. Exports: `cob_move`, `cob_get_int`, `cob_get_llint`, `cob_set_int`, and `cobrs_version()`. The header `include/gnucobol-rs-ffi.h` mirrors libcob `common.h` (`cob_field_attr` @1097, `cob_field` @1107).

---

## Verified byte-identical to libcob

`tests/verify_vs_libcob.sh` compiles the same C program against both `libgnucobol_rs_ffi` and the real `libcob` and requires identical output. Correctness is judged against the admitted GnuCOBOL 3.2 oracle, not self-asserted.

---

## Crate types & license

- Crate types: `cdylib` / `staticlib` / `rlib`.
- License: **LGPL-3.0-or-later** — this shim exposes the LGPL `gnucobol-rs` runtime, which is a **faithful copyleft derivative of `libcob`, not clean-room**. A distributed binary that statically links it is a Combined Work under LGPL-3.0 §4.

This is an independent effort that reproduces GnuCOBOL 3.2; it is not GnuCOBOL and not affiliated with or endorsed by the GNU project. See the `docs/derivation-and-license.md` and `docs/license-boundaries.md` in the [repository](https://crates.io/crates/gnucobol-rs) for the full derivation boundary.