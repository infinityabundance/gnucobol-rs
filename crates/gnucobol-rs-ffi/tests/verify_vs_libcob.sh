#!/usr/bin/env bash
# Proves the gnucobol-rs-ffi C-ABI is a drop-in libcob replacement: compile tests/verify.c against BOTH
# libgnucobol_rs_ffi (native Rust) and the real libcob, and require byte-identical stdout.
set -eu
HERE="$(cd "$(dirname "$0")" && pwd)"; ROOT="$(cd "$HERE/../../.." && pwd)"
PREFIX="$ROOT/lab/oracle/prefix"
cargo build -q -p gnucobol-rs-ffi --manifest-path "$ROOT/Cargo.toml"
gcc "$HERE/verify.c" -L "$ROOT/target/debug" -lgnucobol_rs_ffi -o "$HERE/.verify_ffi"
LD_LIBRARY_PATH="$ROOT/target/debug" "$HERE/.verify_ffi" > "$HERE/.out_ffi"
if [ -f "$PREFIX/lib/libcob.so" ] || ls "$PREFIX/lib"/libcob.* >/dev/null 2>&1; then
  gcc "$HERE/verify.c" -L "$PREFIX/lib" -lcob -o "$HERE/.verify_libcob"
  LD_LIBRARY_PATH="$PREFIX/lib" "$HERE/.verify_libcob" > "$HERE/.out_libcob"
  if diff -u "$HERE/.out_libcob" "$HERE/.out_ffi"; then echo "FFI ABI byte-identical to libcob"; else echo "DIVERGENCE"; exit 1; fi
else
  echo "libcob oracle not built; ffi output:"; cat "$HERE/.out_ffi"
fi
rm -f "$HERE/.verify_ffi" "$HERE/.verify_libcob" "$HERE/.out_ffi" "$HERE/.out_libcob"
