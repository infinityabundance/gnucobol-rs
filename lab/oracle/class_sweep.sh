#!/usr/bin/env bash
# Class-condition sweep (GNURUST.CLASS.1): build ONE program with a field + IF <class> per case, comparing
# cobc's Y/N branch to the Rust class predicates (is_numeric / is_alphabetic[-upper/-lower]). PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" \
  COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_class"
ROWS="$ROOT/target/release/examples/class_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

"$GEN" > "$TMP/specs.txt"
"$ROWS" < "$TMP/specs.txt" | sort > "$TMP/rust.txt"

cls_of() { case "$1" in num|snum|lsep|tsep|lovp) echo NUMERIC;; alp) echo ALPHABETIC;; upr) echo "ALPHABETIC-UPPER";; lwr) echo "ALPHABETIC-LOWER";; esac; }

{
  echo ">>SOURCE FORMAT FREE"
  echo "IDENTIFICATION DIVISION."
  echo "PROGRAM-ID. CLPROG."
  echo "DATA DIVISION."
  echo "WORKING-STORAGE SECTION."
  while IFS='|' read -r label pic test hex; do
    [ -z "$label" ] && continue
    bl=$(( ${#hex} / 2 ))
    echo "01 F-$label PIC $pic."
    echo "01 FX-$label REDEFINES F-$label PIC X($bl)."
  done < "$TMP/specs.txt"
  echo "PROCEDURE DIVISION."
  while IFS='|' read -r label pic test hex; do
    [ -z "$label" ] && continue
    cls=$(cls_of "$test")
    echo "MOVE X\"$hex\" TO FX-$label."
    echo "IF F-$label IS $cls THEN DISPLAY \"$label[Y]\" ELSE DISPLAY \"$label[N]\" END-IF."
  done < "$TMP/specs.txt"
  echo "STOP RUN."
} > "$TMP/clprog.cob"

if ! cobc -free -x -o "$TMP/clprog" "$TMP/clprog.cob" 2>"$TMP/cobc.err"; then
  echo "compile failed:"; cat "$TMP/cobc.err"; exit 2
fi
"$TMP/clprog" > "$TMP/out.txt" 2>/dev/null

# oracle Y/N per label
while IFS='|' read -r label pic test hex; do
  [ -z "$label" ] && continue
  line=$(grep -m1 "^$label\[" "$TMP/out.txt")
  yn="${line#*[}"; yn="${yn%]*}"
  echo "$label $yn"
done < "$TMP/specs.txt" | sort > "$TMP/oracle.txt"

PASS=0; FAIL=0
while read -r label r; do
  o=$(grep -m1 "^$label " "$TMP/oracle.txt" | awk '{print $2}')
  if [ "$o" = "$r" ]; then PASS=$((PASS+1)); else
    FAIL=$((FAIL+1)); [ "$FAIL" -le 10 ] && echo "MISMATCH $label rust=$r oracle=$o" >&2
  fi
done < "$TMP/rust.txt"
echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
