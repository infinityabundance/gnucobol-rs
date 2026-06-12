#!/usr/bin/env bash
# PACKED in-place arithmetic differential sweep (numeric.c cob_add_bcd / cob_addsub_optimized): feed the
# PACKED-receiver ADD/SUBTRACT rows of gen_arith to the libcob oracle (cob_add/cob_sub -> the real
# cob_add_bcd) and to the Rust port routed through packed::cob_addsub_optimized; compare result bytes.
# Prints PASS=n FAIL=n. On FAIL emits replayable rows. ROOT derived from script path.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PREFIX="$ROOT/lab/oracle/prefix"
export LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8

HARNESS="$ROOT/lab/oracle/arith_harness"
if [ ! -x "$HARNESS" ]; then
  gcc -O2 -I"$PREFIX/include" "$ROOT/lab/oracle/arith_harness.c" -o "$HARNESS" -L"$PREFIX/lib" -lcob || exit 2
fi
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_arith"
ROWS="$ROOT/target/release/examples/packed_arith_rows"

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
# Keep only PACKED-receiver (a_type==18) ADD/SUBTRACT (op 1/2) rows -- the cob_add_bcd fast path.
"$GEN" | awk 'NF>=15 && $2 ~ /^[12]$/ && $3==18' > "$TMP/rows.txt"
TOTAL=$(grep -cvE '^\s*(#|$)' "$TMP/rows.txt")

"$HARNESS" < "$TMP/rows.txt" | sort > "$TMP/c.out"
"$ROWS"    < "$TMP/rows.txt" | sort > "$TMP/rs.out"

PASS=0; FAIL=0
while read -r label bytes; do
  [ "$bytes" = "UNSUPPORTED" ] && continue
  oracle=$(grep -m1 "^$label " "$TMP/c.out" | awk '{print $2}')
  if [ "$oracle" = "$bytes" ]; then PASS=$((PASS+1)); else
    FAIL=$((FAIL+1))
    [ "$FAIL" -le 15 ] && { echo "MISMATCH $label oracle=$oracle rust=$bytes" >&2; echo "  row: $(grep -m1 "^$label " "$TMP/rows.txt")" >&2; }
  fi
done < "$TMP/rs.out"

echo "total=$TOTAL  PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
