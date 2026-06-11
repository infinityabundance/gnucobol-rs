#!/usr/bin/env bash
# FUNCTION MOD/REM sweep (GNURUST.INTRINSIC.MOD-REM.1). MOVE FUNCTION MOD/REM(a,b) TO S9(4), DISPLAY, and check
# intrinsic_mod/intrinsic_rem == the oracle (MOD takes divisor sign, REM takes dividend sign).
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_modrem"; ROWS="$ROOT/target/release/examples/modrem_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
"$GEN" > "$TMP/cases.tsv"
{
  echo ">>SOURCE FORMAT FREE"
  echo "IDENTIFICATION DIVISION."; echo "PROGRAM-ID. MR."
  echo "DATA DIVISION."; echo "WORKING-STORAGE SECTION."; echo "01 R PIC S9(5)."
  echo "PROCEDURE DIVISION."
  while IFS=$'\t' read -r label op a b; do
    echo "MOVE FUNCTION $op($a,$b) TO R. DISPLAY \"$label=\" R."
  done < "$TMP/cases.tsv"
  echo "STOP RUN."
} > "$TMP/p.cob"
cobc -free -x -o "$TMP/p" "$TMP/p.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }
"$TMP/p" > "$TMP/out.txt"
( cd "$ROOT" && cargo run -q -p xtask -- sweep-join "$TMP/cases.tsv" "$TMP/out.txt" ) | "$ROWS"
