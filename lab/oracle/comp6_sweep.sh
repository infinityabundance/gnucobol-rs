#!/usr/bin/env bash
# COMP-6 MOVE differential sweep (GNURUST.18): DISPLAY<->COMP-6 via libcob (decimal_harness) vs the
# Rust port (rows). COMP-6 = PACKED + NO_SIGN_NIBBLE. PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
H="$ROOT/lab/oracle/decimal_harness"; [ -x "$H" ] || { echo "decimal_harness not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_comp6"; ROWS="$ROOT/target/release/examples/rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
"$GEN" > "$TMP/rows.txt"
TOTAL=$(grep -cvE '^\s*$' "$TMP/rows.txt")
"$H" < "$TMP/rows.txt" | sort > "$TMP/oracle.txt"
"$ROWS" < "$TMP/rows.txt" | sort > "$TMP/rust.txt"
if diff -q "$TMP/oracle.txt" "$TMP/rust.txt" >/dev/null; then echo "PASS=$TOTAL FAIL=0"; exit 0; fi
F=$(diff "$TMP/oracle.txt" "$TMP/rust.txt" | grep -cE '^<'); echo "PASS=$((TOTAL-F)) FAIL=$F"
diff "$TMP/oracle.txt" "$TMP/rust.txt" | grep -E '^[<>]' | head -12
exit 1
