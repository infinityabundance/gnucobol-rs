#!/usr/bin/env bash
# ROUNDED-MODE differential sweep (GNURUST.ROUND.1): feed identical (0 + value, mode) rows to the
# libcob oracle (cob_add with each COB_STORE_<mode> opt) and the Rust port (cob_arith), compare the
# receiver bytes. Prints PASS=n FAIL=n. Reuses the arith harness + arith_rows mirror (same row
# format; opt carries the rounding mode). ROOT derived from script path.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PREFIX="$ROOT/lab/oracle/prefix"
export LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8

HARNESS="$ROOT/lab/oracle/arith_harness"
if [ ! -x "$HARNESS" ]; then
  gcc -O2 -I"$PREFIX/include" "$ROOT/lab/oracle/arith_harness.c" -o "$HARNESS" -L"$PREFIX/lib" -lcob || exit 2
fi
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_round"
ROWS="$ROOT/target/release/examples/arith_rows"

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
"$GEN" > "$TMP/rows.txt"
TOTAL=$(grep -cvE '^\s*(#|$)' "$TMP/rows.txt")

"$HARNESS" < "$TMP/rows.txt" | sort > "$TMP/c.out"
"$ROWS"    < "$TMP/rows.txt" | sort > "$TMP/rs.out"

CLASSIFIED=$(grep -c ' UNSUPPORTED$' "$TMP/rs.out" || true)
PASS=0; FAIL=0
while read -r label bytes; do
  [ "$bytes" = "UNSUPPORTED" ] && continue
  oracle=$(grep -m1 "^$label " "$TMP/c.out" | awk '{print $2}')
  if [ "$oracle" = "$bytes" ]; then PASS=$((PASS+1)); else
    FAIL=$((FAIL+1))
    [ "$FAIL" -le 15 ] && { echo "MISMATCH $label oracle=$oracle rust=$bytes" >&2; echo "  row: $(grep -m1 "^$label " "$TMP/rows.txt")" >&2; }
  fi
done < "$TMP/rs.out"

echo "total=$TOTAL classified_out=$CLASSIFIED  PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
