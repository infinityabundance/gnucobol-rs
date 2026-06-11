#!/usr/bin/env bash
# FUNCTION UPPER-CASE/LOWER-CASE/REVERSE sweep (GNURUST.INTRINSIC.CASE.1). MOVE FUNCTION op("input") TO X(8),
# DISPLAY raw bytes, check intrinsic_* (padded to 8) == the oracle.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_case"; ROWS="$ROOT/target/release/examples/case_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
"$GEN" > "$TMP/cases.tsv"
{
  echo ">>SOURCE FORMAT FREE"; echo "IDENTIFICATION DIVISION."; echo "PROGRAM-ID. CS."
  echo "DATA DIVISION."; echo "WORKING-STORAGE SECTION."; echo "01 R PIC X(8)."
  echo "PROCEDURE DIVISION."
  while IFS=$'\t' read -r label op input; do
    echo "MOVE FUNCTION $op(\"$input\") TO R. DISPLAY \"$label[\" R \"]\"."
  done < "$TMP/cases.tsv"
  echo "STOP RUN."
} > "$TMP/p.cob"
cobc -free -x -o "$TMP/p" "$TMP/p.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }
"$TMP/p" > "$TMP/out.txt"
( cd "$ROOT" && cargo run -q -p xtask -- sweep-extract "$TMP/cases.tsv" "$TMP/out.txt" 8 ) | "$ROWS"
