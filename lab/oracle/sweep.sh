#!/usr/bin/env bash
# Differential sweep: feed identical rows to the libcob C oracle and the Rust port; compare full
# byte streams. Prints PASS=n FAIL=n. On FAIL, emits a replayable counterexample (GNURUST.DIFFSHRINK.0).
# ROOT is derived from this script's location (no absolute-path leaks; GNURUST hygiene).
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PREFIX="$ROOT/lab/oracle/prefix"
SEED="${1:-0}"

export LD_LIBRARY_PATH="$PREFIX/lib"
export COB_CONFIG_DIR="$PREFIX/share/gnucobol/config"
export COB_COPY_DIR="$PREFIX/share/gnucobol/copy"
export LC_ALL=C.UTF-8

HARNESS="$ROOT/lab/oracle/decimal_harness"
if [ ! -x "$HARNESS" ]; then
  echo "building C harness..."
  gcc -O2 -I"$PREFIX/include" "$ROOT/lab/oracle/decimal_harness.c" -o "$HARNESS" -L"$PREFIX/lib" -lcob || exit 2
fi

echo "building Rust rows + gen_rows (release)..."
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_rows"
ROWS_BIN="$ROOT/target/release/examples/rows"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

"$GEN" "$SEED" > "$TMP/rows.txt"
TOTAL=$(grep -cvE '^\s*(#|$)' "$TMP/rows.txt")

"$HARNESS" < "$TMP/rows.txt" > "$TMP/c.out"
"$ROWS_BIN" < "$TMP/rows.txt" > "$TMP/rs.out"

if diff -q "$TMP/c.out" "$TMP/rs.out" >/dev/null; then
  echo "PASS=$TOTAL FAIL=0"
  exit 0
fi

# Replayable counterexamples
FAILS=$(diff "$TMP/c.out" "$TMP/rs.out" | grep -cE '^<')
echo "PASS=$((TOTAL - FAILS)) FAIL=$FAILS"
echo "--- first divergences (label: oracle vs rust), with input row ---"
diff "$TMP/c.out" "$TMP/rs.out" | grep -E '^<' | head -10 | while read -r _ label hexc; do
  rust=$(grep -E "^$label " "$TMP/rs.out" | awk '{print $2}')
  row=$(grep -E "^$label " "$TMP/rows.txt")
  echo "$label  oracle=$hexc  rust=$rust"
  echo "   row: $row"
done
exit 1
