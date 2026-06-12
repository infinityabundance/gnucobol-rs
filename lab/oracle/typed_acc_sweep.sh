#!/usr/bin/env bash
# Typed-accessor differential (move.c cob_put/get_*_compx/comp5/comp3/comp6/pic9): put+get via the libcob
# oracle and the Rust port; compare the stored bytes AND the round-tripped value. PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
H="$ROOT/lab/oracle/typed_acc_harness"
[ -x "$H" ] || gcc -O2 -I"$PREFIX/include" "$ROOT/lab/oracle/typed_acc_harness.c" -o "$H" -L"$PREFIX/lib" -lcob || exit 2
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_typed_acc"; ROWS="$ROOT/target/release/examples/typed_acc_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
"$GEN" > "$TMP/rows.txt"; TOTAL=$(grep -cvE '^\s*(#|$)' "$TMP/rows.txt")
"$H" < "$TMP/rows.txt" | sort > "$TMP/c.out"; "$ROWS" < "$TMP/rows.txt" | sort > "$TMP/rs.out"
PASS=0;FAIL=0
while read -r label rest; do
  oracle=$(grep -m1 "^$label " "$TMP/c.out" | cut -d' ' -f2-)
  if [ "$oracle" = "$rest" ]; then PASS=$((PASS+1)); else FAIL=$((FAIL+1)); [ "$FAIL" -le 20 ] && echo "MISMATCH $label oracle=[$oracle] rust=[$rest] row=$(grep -m1 "^$label " "$TMP/rows.txt")" >&2; fi
done < "$TMP/rs.out"
echo "total=$TOTAL  PASS=$PASS FAIL=$FAIL"; [ "$FAIL" -eq 0 ]
