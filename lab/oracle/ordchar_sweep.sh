#!/usr/bin/env bash
# FUNCTION ORD/CHAR sweep (GNURUST.INTRINSIC.ORD-CHAR.1). ORD("c") -> 1-based int; CHAR(n) -> byte. Check
# intrinsic_ord/intrinsic_char == the oracle.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_ordchar"; ROWS="$ROOT/target/release/examples/ordchar_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
"$GEN" > "$TMP/cases.tsv"
{
  echo ">>SOURCE FORMAT FREE"; echo "IDENTIFICATION DIVISION."; echo "PROGRAM-ID. OC."
  echo "DATA DIVISION."; echo "WORKING-STORAGE SECTION."; echo "01 N PIC 9(4)."; echo "01 RC PIC X(1)."
  echo "PROCEDURE DIVISION."
  while IFS=$'\t' read -r label op arg; do
    if [ "$op" = "ORD" ]; then
      echo "MOVE FUNCTION ORD(\"$arg\") TO N. DISPLAY \"$label=\" N."
    else
      echo "MOVE FUNCTION CHAR($arg) TO RC. DISPLAY \"$label[\" RC \"]\"."
    fi
  done < "$TMP/cases.tsv"
  echo "STOP RUN."
} > "$TMP/p.cob"
cobc -free -x -o "$TMP/p" "$TMP/p.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }
"$TMP/p" > "$TMP/out.txt"
python3 - "$TMP/cases.tsv" "$TMP/out.txt" <<'PY' | "$ROWS"
import sys
out = open(sys.argv[2], "rb").read()
cur = 0
rows = []
for line in open(sys.argv[1]):
    label, op, arg = line.rstrip("\n").split("\t")
    if op == "ORD":
        marker = (label + "=").encode()
        m = out.find(marker, cur); val = ""
        if m >= 0:
            s = m + len(marker); val = out[s:s+4].decode("latin1"); cur = s + 4
        rows.append("\t".join([label, op, arg, val]))
    else:
        marker = (label + "[").encode()
        m = out.find(marker, cur); val = ""
        if m >= 0:
            s = m + len(marker); val = out[s:s+1].hex(); cur = s + 1
        rows.append("\t".join([label, op, arg, val]))
print("\n".join(rows))
PY
