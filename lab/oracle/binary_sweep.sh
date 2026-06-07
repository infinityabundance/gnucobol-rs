#!/usr/bin/env bash
# Binary MOVE differential sweep (GNURUST.14): feed identical DISPLAY<->binary (COMP/COMP-5/COMP-X)
# cob_move rows to the libcob oracle (decimal_harness) and the Rust port (rows); compare destination
# bytes. Prints PASS=n FAIL=n. ROOT derived from script path.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PREFIX="$ROOT/lab/oracle/prefix"
export LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8

HARNESS="$ROOT/lab/oracle/decimal_harness"
[ -x "$HARNESS" ] || { echo "decimal_harness not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_binary"
ROWS="$ROOT/target/release/examples/rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

"$GEN" > "$TMP/rows.txt"
TOTAL=$(grep -cvE '^\s*$' "$TMP/rows.txt")
"$HARNESS" < "$TMP/rows.txt" | sort > "$TMP/oracle.txt"
"$ROWS"    < "$TMP/rows.txt" | sort > "$TMP/rust.txt"

if diff -q "$TMP/oracle.txt" "$TMP/rust.txt" >/dev/null; then
  echo "PASS=$TOTAL FAIL=0"; exit 0
fi
FAILS=$(diff "$TMP/oracle.txt" "$TMP/rust.txt" | grep -cE '^<')
echo "PASS=$((TOTAL - FAILS)) FAIL=$FAILS"
diff "$TMP/oracle.txt" "$TMP/rust.txt" | grep -E '^[<>]' | head -16 | while read -r dir label rest; do
  echo "$dir $label $rest    [row: $(grep -m1 "^$label " "$TMP/rows.txt")]"
done
exit 1
