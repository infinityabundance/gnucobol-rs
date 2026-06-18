# gnucobol-rs-ffi

A **C-ABI shim** that exposes the native-Rust `libcob` algorithms of [`gnucobol-rs`](../gnucobol-rs) with
`extern "C"` linkage, so a **C program can link gnucobol-rs as a drop-in `libcob` replacement**. The
`cob_field` / `cob_field_attr` struct layouts match GnuCOBOL 3.2's `libcob` (`common.h`), so existing
field buffers work unchanged.

```c
#include "gnucobol-rs-ffi.h"
cob_field_attr da = { 0x12 /*PACKED*/, 4, 0, 0, 0 };
cob_field dst = { 3, buf, &da };
cob_move(&src, &dst);            /* native-Rust MOVE, byte-identical to libcob */
int n = cob_get_int(&dst);
```

Exposes `cob_move`, `cob_get_int`, `cob_get_llint`, `cob_set_int`, and `cobrs_version`. Build a `cdylib`
(`libgnucobol_rs_ffi.so`) or `staticlib` and link it where you would link `libcob`.

**Verified byte-identical to libcob:** `tests/verify_vs_libcob.sh` compiles the same C program against
both `libgnucobol_rs_ffi` and the real `libcob` and requires identical output.

This is the **only** crate that uses `unsafe` (the C boundary). The `gnucobol-rs` core stays
`#![forbid(unsafe_code)]`; every function here just reads the C struct, calls the safe-Rust algorithm,
and writes the result back.

LGPL-3.0-or-later. This is a clean-room native-Rust reimplementation; **not** GnuCOBOL, and not affiliated
with the GNU project.
