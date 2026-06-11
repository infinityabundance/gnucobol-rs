#!/usr/bin/env bash
# Table-subscript sweep (GNURUST.SUBSCRIPT.1): a program with a 1-D and a 2-D OCCURS table; per case MOVE the
# bytes + DISPLAY E(i) or C(i,j); compare cobc's element to element_1d/element_2d. PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" \
  COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_subscript"
ROWS="$ROOT/target/release/examples/subscript_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

"$GEN" > "$TMP/specs.txt"
"$ROWS" < "$TMP/specs.txt" | sort > "$TMP/rust.txt"

{
  echo ">>SOURCE FORMAT FREE"
  echo "IDENTIFICATION DIVISION."
  echo "PROGRAM-ID. SUBPROG."
  echo "DATA DIVISION."
  echo "WORKING-STORAGE SECTION."
  echo "01 ONE."
  echo "   05 E OCCURS 5 PIC X(3)."
  echo "01 ONER REDEFINES ONE PIC X(15)."
  echo "01 G."
  echo "   05 R OCCURS 3."
  echo "      10 C OCCURS 4 PIC X(2)."
  echo "01 GR REDEFINES G PIC X(24)."
  echo "PROCEDURE DIVISION."
  while IFS='|' read -r label shape hex i j; do
    [ -z "$label" ] && continue
    case "$shape" in
      1d) echo "MOVE X\"$hex\" TO ONER. DISPLAY \"$label[\" E($i) \"]\".";;
      2d) echo "MOVE X\"$hex\" TO GR. DISPLAY \"$label[\" C($i,$j) \"]\".";;
    esac
  done < "$TMP/specs.txt"
  echo "STOP RUN."
} > "$TMP/subprog.cob"

if ! cobc -free -x -o "$TMP/subprog" "$TMP/subprog.cob" 2>"$TMP/cobc.err"; then
  echo "compile failed:"; cat "$TMP/cobc.err"; exit 2
fi
"$TMP/subprog" > "$TMP/out.txt" 2>/dev/null

while IFS='|' read -r label shape hex i j; do
  [ -z "$label" ] && continue
  line=$(grep -m1 "^$label\[" "$TMP/out.txt")
  inner="${line#*[}"; inner="${inner%]*}"
  hx=$(printf '%s' "$inner" | od -An -tx1 | tr -d ' \n')
  echo "$label $hx"
done < "$TMP/specs.txt" | sort > "$TMP/oracle.txt"

PASS=0; FAIL=0
while read -r label r; do
  o=$(grep -m1 "^$label " "$TMP/oracle.txt" | awk '{print $2}')
  if [ "$r" = "$o" ]; then PASS=$((PASS+1)); else
    FAIL=$((FAIL+1)); [ "$FAIL" -le 10 ] && echo "MISMATCH $label rust=$r oracle=$o" >&2
  fi
done < "$TMP/rust.txt"
echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
