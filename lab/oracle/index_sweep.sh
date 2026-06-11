#!/usr/bin/env bash
# USAGE INDEX sweep (GNURUST.INDEX.1): per case SET IXS TO start [UP/DOWN BY k]; dump the 4 native-endian
# index bytes (via FUNCTION ORD per byte) and compare cobc to set_index_to/up_by/down_by. PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" \
  COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_index"
ROWS="$ROOT/target/release/examples/index_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

"$GEN" > "$TMP/specs.txt"
"$ROWS" < "$TMP/specs.txt" | sort > "$TMP/rust.txt"

{
  echo ">>SOURCE FORMAT FREE"
  echo "IDENTIFICATION DIVISION."
  echo "PROGRAM-ID. IDXPROG."
  echo "DATA DIVISION."
  echo "WORKING-STORAGE SECTION."
  echo "01 T4."
  echo "   05 E4  PIC X(4)  OCCURS 10 INDEXED BY I4."
  echo "01 T17."
  echo "   05 E17 PIC X(17) OCCURS 10 INDEXED BY I17."
  echo "01 IXS USAGE INDEX."
  echo "01 RAW REDEFINES IXS."
  echo "   05 BB PIC X OCCURS 4."
  echo "01 O1 PIC 9(3)."
  echo "01 O2 PIC 9(3)."
  echo "01 O3 PIC 9(3)."
  echo "01 O4 PIC 9(3)."
  echo "PROCEDURE DIVISION."
  while IFS='|' read -r label start op k stride; do
    [ -z "$label" ] && continue
    # stride 0 = standalone USAGE INDEX; >0 = INDEXED BY index-name over PIC X(stride), copied into IXS
    # to dump its bytes (index-names cannot be REDEFINES'd; MOVE index-name TO USAGE INDEX is byte-exact).
    case "$stride" in
      4)  IX="I4";;
      17) IX="I17";;
      *)  IX="IXS";;
    esac
    echo "SET $IX TO $start."
    case "$op" in
      up)   echo "SET $IX UP BY $k.";;
      down) echo "SET $IX DOWN BY $k.";;
    esac
    [ "$IX" != "IXS" ] && echo "MOVE $IX TO IXS."
    echo "MOVE FUNCTION ORD(BB(1)) TO O1."
    echo "MOVE FUNCTION ORD(BB(2)) TO O2."
    echo "MOVE FUNCTION ORD(BB(3)) TO O3."
    echo "MOVE FUNCTION ORD(BB(4)) TO O4."
    echo "DISPLAY \"$label[\" O1 \" \" O2 \" \" O3 \" \" O4 \"]\"."
  done < "$TMP/specs.txt"
  echo "STOP RUN."
} > "$TMP/idxprog.cob"

if ! cobc -free -x -o "$TMP/idxprog" "$TMP/idxprog.cob" 2>"$TMP/cobc.err"; then
  echo "compile failed:"; cat "$TMP/cobc.err"; exit 2
fi
"$TMP/idxprog" > "$TMP/out.txt" 2>/dev/null

# FUNCTION ORD is 1-based (byte value + 1); subtract 1 and render the 4 bytes as hex.
while IFS='|' read -r label start op k stride; do
  [ -z "$label" ] && continue
  line=$(grep -m1 "^$label\[" "$TMP/out.txt")
  inner="${line#*[}"; inner="${inner%]*}"
  hx=""
  for ord in $inner; do
    byte=$((10#$ord - 1))
    hx+=$(printf '%02x' "$byte")
  done
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
