#!/usr/bin/env bash
# FUNCTION LENGTH sweep (GNURUST.INTRINSIC.LENGTH.1). Declare each field, DISPLAY FUNCTION LENGTH(field), and
# check intrinsic_length == the oracle storage byte length.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_length"; ROWS="$ROOT/target/release/examples/length_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
"$GEN" > "$TMP/cases.tsv"
{
  echo ">>SOURCE FORMAT FREE"
  echo "IDENTIFICATION DIVISION."
  echo "PROGRAM-ID. LEN."
  echo "DATA DIVISION."
  echo "WORKING-STORAGE SECTION."
  echo "01 N PIC 9(6)."
  i=0
  while IFS=$'\t' read -r label pic usage; do
    u=""; [ "$usage" != "DISPLAY" ] && u=" USAGE $usage"
    echo "01 F-$i PIC $pic$u."
    i=$((i+1))
  done < "$TMP/cases.tsv"
  echo "PROCEDURE DIVISION."
  i=0
  while IFS=$'\t' read -r label pic usage; do
    echo "MOVE FUNCTION LENGTH(F-$i) TO N. DISPLAY \"$label=\" N."
    i=$((i+1))
  done < "$TMP/cases.tsv"
  echo "STOP RUN."
} > "$TMP/p.cob"
cobc -free -x -o "$TMP/p" "$TMP/p.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }
"$TMP/p" > "$TMP/out.txt"
( cd "$ROOT" && cargo run -q -p xtask -- sweep-join "$TMP/cases.tsv" "$TMP/out.txt" ) | "$ROWS"
