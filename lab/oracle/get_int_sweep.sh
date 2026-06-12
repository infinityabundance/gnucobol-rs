#!/usr/bin/env bash
# cob_get_int / cob_get_llint accessor differential (move.c): feed identical fields to the libcob oracle
# and the Rust port; compare both returned integers. PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
H="$ROOT/lab/oracle/get_int_harness"
[ -x "$H" ] || gcc -O2 -I"$PREFIX/include" "$ROOT/lab/oracle/get_int_harness.c" -o "$H" -L"$PREFIX/lib" -lcob || exit 2
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_get_int"; ROWS="$ROOT/target/release/examples/get_int_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
"$GEN" > "$TMP/rows.txt"; TOTAL=$(grep -cvE '^\s*(#|$)' "$TMP/rows.txt")
"$H" < "$TMP/rows.txt" | sort > "$TMP/c.out"; "$ROWS" < "$TMP/rows.txt" | sort > "$TMP/rs.out"
PASS=0;FAIL=0
while read -r label rest; do
  oracle=$(grep -m1 "^$label " "$TMP/c.out" | cut -d' ' -f2-)
  if [ "$oracle" = "$rest" ]; then PASS=$((PASS+1)); else FAIL=$((FAIL+1)); [ "$FAIL" -le 20 ] && echo "MISMATCH $label oracle=[$oracle] rust=[$rest]" >&2; fi
done < "$TMP/rs.out"
echo "total=$TOTAL  PASS=$PASS FAIL=$FAIL"; [ "$FAIL" -eq 0 ]
