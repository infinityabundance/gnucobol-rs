#!/usr/bin/env bash
# Date-conversion intrinsic sweep (GNURUST.INTRINSIC.DATE.1). INTEGER-OF-DATE / DATE-OF-INTEGER / INTEGER-OF-DAY
# / DAY-OF-INTEGER against the oracle.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_date"; ROWS="$ROOT/target/release/examples/date_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
"$GEN" > "$TMP/cases.tsv"
{
  echo ">>SOURCE FORMAT FREE"; echo "IDENTIFICATION DIVISION."; echo "PROGRAM-ID. DT."
  echo "DATA DIVISION."; echo "WORKING-STORAGE SECTION."; echo "01 N PIC 9(8)."
  echo "PROCEDURE DIVISION."
  while IFS=$'\t' read -r label op arg; do
    case "$op" in
      IOD)  echo "MOVE FUNCTION INTEGER-OF-DATE($arg) TO N. DISPLAY \"$label=\" N." ;;
      DOI)  echo "MOVE FUNCTION DATE-OF-INTEGER($arg) TO N. DISPLAY \"$label=\" N." ;;
      IODY) echo "MOVE FUNCTION INTEGER-OF-DAY($arg) TO N. DISPLAY \"$label=\" N." ;;
      DYOI) echo "MOVE FUNCTION DAY-OF-INTEGER($arg) TO N. DISPLAY \"$label=\" N." ;;
    esac
  done < "$TMP/cases.tsv"
  echo "STOP RUN."
} > "$TMP/p.cob"
cobc -free -x -o "$TMP/p" "$TMP/p.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }
"$TMP/p" > "$TMP/out.txt"
python3 - "$TMP/cases.tsv" "$TMP/out.txt" <<'PY' | "$ROWS"
import sys
vals = {}
for line in open(sys.argv[2]):
    if "=" in line: k,v = line.rstrip("\n").split("=",1); vals[k]=v
for line in open(sys.argv[1]):
    label, op, arg = line.rstrip("\n").split("\t")
    print("\t".join([label, op, arg, vals.get(label,"")]))
PY
