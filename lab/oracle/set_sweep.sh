#!/usr/bin/env bash
# SET condition-name TO TRUE differential sweep (GNURUST.12). For each gen_set case, build a program
# that SETs the condition TRUE and dumps the parent's raw bytes (via a REDEFINES X(size)); compare to
# the Rust set_88_true (which also self-checks eval_88 on its own output). Prints PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" \
  COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" COB_COPY_DIR="$PREFIX/share/gnucobol/copy" LC_ALL=C.UTF-8

( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_set"
ROWS="$ROOT/target/release/examples/set_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

"$GEN" > "$TMP/specs.txt"
"$ROWS" < "$TMP/specs.txt" | sort > "$TMP/rust.txt"

render_def() {
  local def="$1" out="" IFS=';'
  for entry in $def; do
    local p; IFS=':' read -ra p <<< "$entry"
    case "${p[0]}" in
      la) out="$out \"${p[1]}\"" ;;
      ln) out="$out ${p[1]}" ;;
      ra) out="$out \"${p[1]}\" THRU \"${p[2]}\"" ;;
      rn) out="$out ${p[1]} THRU ${p[2]}" ;;
    esac
  done
  echo "$out"
}

while IFS='|' read -r label decl size def; do
  [ -z "$label" ] && continue
  case "$label" in \#*) continue ;; esac
  vals=$(render_def "$def")
  prog="$TMP/$label.cob"
  {
    echo "       IDENTIFICATION DIVISION."
    echo "       PROGRAM-ID. P$label."
    echo "       DATA DIVISION."
    echo "       WORKING-STORAGE SECTION."
    echo "       01 P PIC $decl."
    echo "          88 C VALUE$vals."
    echo "       01 PR REDEFINES P PIC X($size)."
    echo "       PROCEDURE DIVISION."
    echo "           SET C TO TRUE."
    echo "           DISPLAY PR WITH NO ADVANCING."
    echo "           STOP RUN."
  } > "$prog"
  if cobc -free -x "$prog" -o "$TMP/$label.bin" 2>>"$TMP/cobc.err"; then
    hex=$("$TMP/$label.bin" | head -c "$size" | od -An -tx1 | tr -d ' \n')
    echo "$label $hex"
  else
    echo "$label COMPILE_FAIL"
  fi
done < "$TMP/specs.txt" | sort > "$TMP/oracle.txt"

TOTAL=$(grep -cvE '^\s*(#|$)' "$TMP/specs.txt")
PASS=0; FAIL=0; CLASSIFIED=0
while read -r label hex; do
  case "$hex" in UNSUPPORTED|SELFCHECK_FAIL) [ "$hex" = "SELFCHECK_FAIL" ] && { echo "SELFCHECK_FAIL $label" >&2; FAIL=$((FAIL+1)); } || CLASSIFIED=$((CLASSIFIED+1)); continue ;; esac
  oracle=$(grep -m1 "^$label " "$TMP/oracle.txt" | awk '{print $2}')
  if [ "$oracle" = "COMPILE_FAIL" ] || [ -z "$oracle" ]; then CLASSIFIED=$((CLASSIFIED+1)); continue; fi
  if [ "$oracle" = "$hex" ]; then PASS=$((PASS+1)); else
    FAIL=$((FAIL+1))
    [ "$FAIL" -le 15 ] && echo "MISMATCH $label oracle=$oracle rust=$hex  spec: $(grep -m1 "^$label|" "$TMP/specs.txt")" >&2
  fi
done < "$TMP/rust.txt"

echo "total=$TOTAL classified_out=$CLASSIFIED  PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
