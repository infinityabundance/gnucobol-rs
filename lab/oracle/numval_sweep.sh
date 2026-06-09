#!/usr/bin/env bash
# FUNCTION NUMVAL sweep (GNURUST.INTRINSIC.NUMVAL.1). MOVE FUNCTION NUMVAL("input") TO S9(8)V9(4), DISPLAY the
# receiver, and check numval_display(intrinsic_numval(input)) == the oracle.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_numval"; ROWS="$ROOT/target/release/examples/numval_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
"$GEN" > "$TMP/cases.tsv"
{
  echo ">>SOURCE FORMAT FREE"
  echo "IDENTIFICATION DIVISION."
  echo "PROGRAM-ID. NV."
  echo "DATA DIVISION."
  echo "WORKING-STORAGE SECTION."
  echo "01 R PIC S9(8)V9(4)."
  echo "PROCEDURE DIVISION."
  while IFS=$'\t' read -r label input; do
    echo "MOVE FUNCTION NUMVAL(\"$input\") TO R. DISPLAY \"$label=\" R."
  done < "$TMP/cases.tsv"
  echo "STOP RUN."
} > "$TMP/p.cob"
cobc -free -x -o "$TMP/p" "$TMP/p.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }
"$TMP/p" > "$TMP/out.txt"
python3 - "$TMP/cases.tsv" "$TMP/out.txt" <<'PY' | "$ROWS"
import sys
vals = {}
for line in open(sys.argv[2]):
    if "=" in line: k, v = line.rstrip("\n").split("=", 1); vals[k] = v
for line in open(sys.argv[1]):
    label, inp = line.rstrip("\n").split("\t")
    print("\t".join([label, inp, vals.get(label, "")]))
PY
