#!/usr/bin/env bash
# VALUE differential sweep (GNURUST.8): for each gen_value spec, build a COBOL program, compile with
# the built cobc, run it (DISPLAY of the group emits the raw initial record bytes), and compare those
# bytes to the Rust value_image mirror. Prints PASS=n FAIL=n. ROOT derived from script path.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" \
  COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" COB_COPY_DIR="$PREFIX/share/gnucobol/copy" LC_ALL=C.UTF-8

( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_value"
ROWS="$ROOT/target/release/examples/value_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

"$GEN" > "$TMP/specs.txt"
"$ROWS" < "$TMP/specs.txt" | sort > "$TMP/rust.txt"

# Build + run each program; emit "label hex" of the displayed record bytes.
build_and_run() {
  local label="$1"; shift
  local spec="$1"
  local prog="$TMP/$label.cob"
  {
    echo "       IDENTIFICATION DIVISION."
    echo "       PROGRAM-ID. P$label."
    echo "       DATA DIVISION."
    echo "       WORKING-STORAGE SECTION."
    local IFS='|'
    for f in $spec; do
      local level name pic usage value
      level="${f%%:*}"; rest="${f#*:}"
      name="${rest%%:*}"; rest="${rest#*:}"
      pic="${rest%%:*}"; rest="${rest#*:}"
      usage="${rest%%:*}"; value="${rest#*:}"
      if [ "$usage" = "G" ]; then
        printf '       %s %s.\n' "$level" "$name"
      else
        line="       $level $name PIC $pic"
        [ "$usage" = "C" ] && line="$line USAGE COMP-3"
        case "$value" in
          "") ;;
          N*) line="$line VALUE ${value#N}";;
          A*) line="$line VALUE \"${value#A}\"";;
          Z)  line="$line VALUE ZERO";;
          S)  line="$line VALUE SPACE";;
        esac
        printf '%s.\n' "$line"
      fi
    done
    echo "       PROCEDURE DIVISION."
    echo "           DISPLAY REC WITH NO ADVANCING."
    echo "           STOP RUN."
  } > "$prog"
  if cobc -x "$prog" -o "$TMP/$label.bin" 2>>"$TMP/cobc.err"; then
    bytes=$("$TMP/$label.bin" | od -An -tx1 | tr -d ' \n')
    echo "$label $bytes"
  else
    echo "$label COMPILE_FAIL"
  fi
}

while IFS='|' read -r label spec_rest; do
  [ -z "$label" ] && continue
  build_and_run "$label" "$spec_rest"
done < "$TMP/specs.txt" | sort > "$TMP/oracle.txt"

TOTAL=$(grep -cvE '^\s*$' "$TMP/specs.txt")
PASS=0; FAIL=0; CLASSIFIED=0
while read -r label bytes; do
  [ "$bytes" = "UNSUPPORTED" ] && { CLASSIFIED=$((CLASSIFIED+1)); continue; }
  oracle=$(grep -m1 "^$label " "$TMP/oracle.txt" | awk '{print $2}')
  if [ "$oracle" = "COMPILE_FAIL" ] || [ -z "$oracle" ]; then
    CLASSIFIED=$((CLASSIFIED+1)); continue
  fi
  if [ "$oracle" = "$bytes" ]; then PASS=$((PASS+1)); else
    FAIL=$((FAIL+1))
    [ "$FAIL" -le 15 ] && { echo "MISMATCH $label oracle=$oracle rust=$bytes" >&2; echo "  spec: $(grep -m1 "^$label|" "$TMP/specs.txt")" >&2; }
  fi
done < "$TMP/rust.txt"

echo "total=$TOTAL classified_out=$CLASSIFIED  PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
