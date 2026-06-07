#!/usr/bin/env bash
# LEVEL-88 condition-name differential sweep (GNURUST.11). For each gen_cond case, build a program
# that MOVEs the value into the parent and prints whether the 88 is true; compare to the Rust
# eval_88 (which encodes the same value via the sealed value_image). Prints PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" \
  COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" COB_COPY_DIR="$PREFIX/share/gnucobol/copy" LC_ALL=C.UTF-8

( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_cond"
ROWS="$ROOT/target/release/examples/cond_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

"$GEN" > "$TMP/specs.txt"
"$ROWS" < "$TMP/specs.txt" | sort > "$TMP/rust.txt"

# Render a 88def into a COBOL VALUE clause body.
render_def() {
  local def="$1" out=""
  local IFS=';'
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

while IFS='|' read -r label pic mvspec def; do
  [ -z "$label" ] && continue
  case "$label" in \#*) continue ;; esac
  mvkind="${mvspec%%:*}"; mv="${mvspec#*:}"
  if [ "$mvkind" = "A" ]; then movelit="\"$mv\""; else movelit="$mv"; fi
  vals=$(render_def "$def")
  prog="$TMP/$label.cob"
  {
    echo "       IDENTIFICATION DIVISION."
    echo "       PROGRAM-ID. P$label."
    echo "       DATA DIVISION."
    echo "       WORKING-STORAGE SECTION."
    echo "       01 P PIC $pic."
    echo "          88 C VALUE$vals."
    echo "       PROCEDURE DIVISION."
    echo "           MOVE $movelit TO P."
    echo "           IF C DISPLAY \"T\" ELSE DISPLAY \"F\" END-IF."
    echo "           STOP RUN."
  } > "$prog"
  if cobc -free -x "$prog" -o "$TMP/$label.bin" 2>>"$TMP/cobc.err"; then
    tf=$("$TMP/$label.bin" | head -1 | tr -d '[:space:]')
    echo "$label $tf"
  else
    echo "$label COMPILE_FAIL"
  fi
done < "$TMP/specs.txt" | sort > "$TMP/oracle.txt"

TOTAL=$(grep -cvE '^\s*(#|$)' "$TMP/specs.txt")
PASS=0; FAIL=0; CLASSIFIED=0
while read -r label tf; do
  [ "$tf" = "UNSUPPORTED" ] && { CLASSIFIED=$((CLASSIFIED+1)); continue; }
  oracle=$(grep -m1 "^$label " "$TMP/oracle.txt" | awk '{print $2}')
  if [ "$oracle" = "COMPILE_FAIL" ] || [ -z "$oracle" ]; then CLASSIFIED=$((CLASSIFIED+1)); continue; fi
  if [ "$oracle" = "$tf" ]; then PASS=$((PASS+1)); else
    FAIL=$((FAIL+1))
    [ "$FAIL" -le 15 ] && echo "MISMATCH $label oracle=$oracle rust=$tf  spec: $(grep -m1 "^$label|" "$TMP/specs.txt")" >&2
  fi
done < "$TMP/rust.txt"

echo "total=$TOTAL classified_out=$CLASSIFIED  PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
