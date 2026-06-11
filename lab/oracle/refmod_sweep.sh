#!/usr/bin/env bash
# Reference-modification sweep (GNURUST.REFMOD.1): build ONE program exercising F(start:length), F(start:),
# and MOVE src TO F(start:length); compare cobc's bytes to ref_mod/ref_mod_to_end/apply_ref_mod. PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" \
  COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_refmod"
ROWS="$ROOT/target/release/examples/refmod_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

"$GEN" > "$TMP/specs.txt"
"$ROWS" < "$TMP/specs.txt" | sort > "$TMP/rust.txt"

{
  echo ">>SOURCE FORMAT FREE"
  echo "IDENTIFICATION DIVISION."
  echo "PROGRAM-ID. RMPROG."
  echo "DATA DIVISION."
  echo "WORKING-STORAGE SECTION."
  while IFS='|' read -r label field op start length src; do
    [ -z "$label" ] && continue
    echo "01 F-$label PIC X(${#field}) VALUE \"$field\"."
  done < "$TMP/specs.txt"
  echo "PROCEDURE DIVISION."
  while IFS='|' read -r label field op start length src; do
    [ -z "$label" ] && continue
    case "$op" in
      src)  echo "DISPLAY \"$label[\" F-$label($start:$length) \"]\".";;
      end)  echo "DISPLAY \"$label[\" F-$label($start:) \"]\".";;
      recv) echo "MOVE \"$src\" TO F-$label($start:$length). DISPLAY \"$label[\" F-$label \"]\".";;
    esac
  done < "$TMP/specs.txt"
  echo "STOP RUN."
} > "$TMP/rmprog.cob"

if ! cobc -free -x -o "$TMP/rmprog" "$TMP/rmprog.cob" 2>"$TMP/cobc.err"; then
  echo "compile failed:"; cat "$TMP/cobc.err"; exit 2
fi
"$TMP/rmprog" > "$TMP/out.txt" 2>/dev/null

while IFS='|' read -r label field op start length src; do
  [ -z "$label" ] && continue
  line=$(grep -m1 "^$label\[" "$TMP/out.txt")
  inner="${line#*[}"; inner="${inner%]*}"
  hex=$(printf '%s' "$inner" | od -An -tx1 | tr -d ' \n')
  echo "$label $hex"
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
