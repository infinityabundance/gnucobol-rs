#!/usr/bin/env bash
# ODO physical-max layout differential sweep (GNURUST.10). For each gen_odo case (a record whose
# last item is OCCURS ... DEPENDING ON), build a program, run `cobc -C`, and read the record's
# **physical** storage allocation `b_REC[size]` (the max-occurrence layout — NOT runtime LENGTH OF,
# which is the unclaimed logical length). Compare to the Rust lay_out total (via layout_rows) and to
# the pre-ODO field offsets. Prints PASS=n FAIL=n. ROOT derived from script path.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" \
  COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" COB_COPY_DIR="$PREFIX/share/gnucobol/copy" LC_ALL=C.UTF-8

( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_odo"
ROWS="$ROOT/target/release/examples/layout_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

"$GEN" > "$TMP/cases.txt"

PASS=0; FAIL=0; TOTAL=0
label=""; decls=()
process() {
  [ -z "$label" ] && return
  TOTAL=$((TOTAL+1))
  # Rust physical total = REC size from layout_rows.
  rust_total=$(printf '%s\n' "${decls[@]}" | "$ROWS" | awk '$1=="REC"{print $3}')
  # Build the program and read b_REC[size] from generated C.
  local prog="$TMP/$label.cob"
  {
    echo "       IDENTIFICATION DIVISION."
    echo "       PROGRAM-ID. P$label."
    echo "       DATA DIVISION."
    echo "       WORKING-STORAGE SECTION."
    for d in "${decls[@]}"; do
      local decl="${d#*$'\t'}"
      printf '       %s.\n' "$decl"
    done
    echo "       PROCEDURE DIVISION."
    echo "           DISPLAY LENGTH OF REC."
    echo "           STOP RUN."
  } > "$prog"
  # -free: no 72-column margin (long ODO declarations would otherwise truncate). Source format does
  # not affect data layout, so the physical b_REC[size] is unchanged.
  if ( cd "$TMP" && cobc -free -C "$label.cob" 2>>"$TMP/cobc.err" ); then
    # physical allocation of the record: the `static ... b_N[SIZE] ...;  /* REC */` storage line.
    oracle_total=$(grep -h '/\* REC \*/' "$TMP/$label".c.l.h "$TMP/$label".c.h 2>/dev/null \
      | grep -oE 'b_[0-9]+\[[0-9]+\]' | head -1 | grep -oE '[0-9]+\]' | tr -d ']')
    if [ -n "$oracle_total" ] && [ "$oracle_total" = "$rust_total" ]; then
      PASS=$((PASS+1))
    else
      FAIL=$((FAIL+1))
      [ "$FAIL" -le 15 ] && echo "MISMATCH $label oracle_phys=$oracle_total rust=$rust_total" >&2
    fi
  else
    FAIL=$((FAIL+1))
    [ "$FAIL" -le 15 ] && echo "COMPILE_FAIL $label" >&2
  fi
}

while IFS= read -r line; do
  case "$line" in
    "#CASE "*) process; label="${line#\#CASE }"; decls=() ;;
    "") : ;;  # keep accumulating until next #CASE
    *) decls+=("$line") ;;
  esac
done < "$TMP/cases.txt"
process

echo "records=$TOTAL  PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
