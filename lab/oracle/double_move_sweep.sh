#!/usr/bin/env bash
# Float->decimal differential sweep (numeric.c cob_decimal_set_double + the 2048-bit mpf path): MOVE
# COMP-2 doubles to DISPLAY fields through the libcob oracle (cob_move) and the Rust port
# (cob_decimal_setget_fld); compare result bytes. Prints PASS=n FAIL=n. ROOT derived from script path.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PREFIX="$ROOT/lab/oracle/prefix"
export LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8

H="$ROOT/lab/oracle/decimal_harness"
[ -x "$H" ] || gcc -O2 -I"$PREFIX/include" "$ROOT/lab/oracle/decimal_harness.c" -o "$H" -L"$PREFIX/lib" -lcob || exit 2
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_double_move"
ROWS="$ROOT/target/release/examples/double_move_rows"

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
"$GEN" > "$TMP/rows.txt"
TOTAL=$(grep -cvE '^\s*(#|$)' "$TMP/rows.txt")
"$H"    < "$TMP/rows.txt" | sort > "$TMP/c.out"
"$ROWS" < "$TMP/rows.txt" | sort > "$TMP/rs.out"

PASS=0; FAIL=0
while read -r label bytes; do
  [ "$bytes" = "UNSUPPORTED" ] && continue
  oracle=$(grep -m1 "^$label " "$TMP/c.out" | awk '{print $2}')
  if [ "$oracle" = "$bytes" ]; then PASS=$((PASS+1)); else
    FAIL=$((FAIL+1))
    [ "$FAIL" -le 20 ] && { echo "MISMATCH $label oracle=$oracle rust=$bytes" >&2; echo "  row: $(grep -m1 "^$label " "$TMP/rows.txt")" >&2; }
  fi
done < "$TMP/rs.out"

echo "total=$TOTAL  PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
