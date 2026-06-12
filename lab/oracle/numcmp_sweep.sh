#!/usr/bin/env bash
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
H="$ROOT/lab/oracle/cmp_harness"
if [ ! -x "$H" ] || [ "$ROOT/lab/oracle/cmp_harness.c" -nt "$H" ]; then
  gcc -O2 -I"$PREFIX/include" "$ROOT/lab/oracle/cmp_harness.c" -o "$H" -L"$PREFIX/lib" -lcob || exit 2; fi
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_numcmp"; ROWS="$ROOT/target/release/examples/numcmp_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
"$GEN" > "$TMP/r.txt"; TOTAL=$(grep -cvE '^\s*(#|$)' "$TMP/r.txt")
"$H" < "$TMP/r.txt" | sort > "$TMP/c.out"; "$ROWS" < "$TMP/r.txt" | sort > "$TMP/rs.out"
PASS=0; FAIL=0
while read -r label val; do
  o=$(grep -m1 "^$label " "$TMP/c.out" | awk '{print $2}')
  if [ "$o" = "$val" ]; then PASS=$((PASS+1)); else FAIL=$((FAIL+1)); [ "$FAIL" -le 15 ] && echo "MISMATCH $label oracle=$o rust=$val" >&2; fi
done < "$TMP/rs.out"
echo "total=$TOTAL  PASS=$PASS FAIL=$FAIL"; [ "$FAIL" -eq 0 ]
