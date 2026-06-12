#!/usr/bin/env bash
# Binary-float ENCODE differential sweep (GNURUST.FLOAT.1): DISPLAY -> COMP-1/COMP-2 vs libcob cob_move.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PREFIX="$ROOT/lab/oracle/prefix"
export LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
H="$ROOT/lab/oracle/decimal_harness"
[ -x "$H" ] || gcc -O2 -I"$PREFIX/include" "$ROOT/lab/oracle/decimal_harness.c" -o "$H" -L"$PREFIX/lib" -lcob || exit 2
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_float"; ROWS="$ROOT/target/release/examples/float_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
"$GEN" > "$TMP/rows.txt"; TOTAL=$(grep -cvE '^\s*(#|$)' "$TMP/rows.txt")
"$H"    < "$TMP/rows.txt" | sort > "$TMP/c.out"
"$ROWS" < "$TMP/rows.txt" | sort > "$TMP/rs.out"
PASS=0; FAIL=0
while read -r label val; do
  oracle=$(grep -m1 "^$label " "$TMP/c.out" | awk '{print $2}')
  if [ "$oracle" = "$val" ]; then PASS=$((PASS+1)); else
    FAIL=$((FAIL+1)); [ "$FAIL" -le 15 ] && echo "MISMATCH $label oracle=$oracle rust=$val" >&2
  fi
done < "$TMP/rs.out"
echo "total=$TOTAL  PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
