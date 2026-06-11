#!/usr/bin/env bash
# OCCURS DEPENDING ON sweep (GNURUST.ODO.1): a variable-length REC; per case COMPUTE L = LENGTH OF REC (at N)
# or DISPLAY E(i); compare cobc to odo_used_length / odo_element. PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" \
  COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_odo"
ROWS="$ROOT/target/release/examples/odo_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

"$GEN" > "$TMP/specs.txt"
"$ROWS" < "$TMP/specs.txt" | sort > "$TMP/rust.txt"

{
  echo ">>SOURCE FORMAT FREE"
  echo "IDENTIFICATION DIVISION."
  echo "PROGRAM-ID. ODOPROG."
  echo "DATA DIVISION."
  echo "WORKING-STORAGE SECTION."
  echo "01 REC."
  echo "   05 N PIC 9."
  echo "   05 E OCCURS 1 TO 5 DEPENDING ON N PIC X(3)."
  echo "01 L PIC 9(3)."
  echo "PROCEDURE DIVISION."
  while IFS='|' read -r label ty n i hex; do
    [ -z "$label" ] && continue
    case "$ty" in
      len)  echo "MOVE $n TO N. COMPUTE L = LENGTH OF REC. DISPLAY \"$label[\" L \"]\".";;
      elem) echo "MOVE 5 TO N. MOVE \"AAA\" TO E(1). MOVE \"BBB\" TO E(2). MOVE \"CCC\" TO E(3). MOVE \"DDD\" TO E(4). MOVE \"EEE\" TO E(5). DISPLAY \"$label[\" E($i) \"]\".";;
    esac
  done < "$TMP/specs.txt"
  echo "STOP RUN."
} > "$TMP/odoprog.cob"

if ! cobc -free -x -o "$TMP/odoprog" "$TMP/odoprog.cob" 2>"$TMP/cobc.err"; then
  echo "compile failed:"; cat "$TMP/cobc.err"; exit 2
fi
"$TMP/odoprog" > "$TMP/out.txt" 2>/dev/null

while IFS='|' read -r label ty n i hex; do
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
