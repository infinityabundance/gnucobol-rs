#!/usr/bin/env bash
# EBCDIC code-page sweep (GNURUST.15): compare gnucobol-rs's embedded cp500 table to the admitted
# oracle's table (libcob cob_load_collation over the shipped ebcdic500_ascii8bit.ttbl). 256/256 bytes.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PREFIX="$ROOT/lab/oracle/prefix"
export LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
H="$ROOT/lab/oracle/ebcdic_harness"
[ -x "$H" ] || gcc -O2 -I"$PREFIX/include" "$ROOT/lab/oracle/ebcdic_harness.c" -o "$H" -L"$PREFIX/lib" -lcob || exit 2
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ORACLE=$("$H" ebcdic500_ascii8bit)
RUST=$("$ROOT/target/release/examples/ebcdic_rows")
if [ "$ORACLE" = "$RUST" ] && [ -n "$ORACLE" ]; then
  echo "PASS=256 FAIL=0"
else
  # count differing bytes
  diff=0; for i in $(seq 0 255); do o=${ORACLE:$((i*2)):2}; r=${RUST:$((i*2)):2}; [ "$o" != "$r" ] && diff=$((diff+1)); done
  echo "PASS=$((256-diff)) FAIL=$diff"
  exit 1
fi
