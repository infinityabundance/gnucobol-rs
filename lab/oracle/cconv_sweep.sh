#!/usr/bin/env bash
# cconv.c differential (cob_toupper/tolower/field_to_string/load_collation; static hex/skip_blanks covered
# transitively): the libcob oracle (cconv_harness.c) and the Rust port (cconv_rows) run the SAME fixed
# scenarios + the same .ttbl files; compare line-for-line. PASS=n FAIL=n. ROOT from script path.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PREFIX="$ROOT/lab/oracle/prefix"
export LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
H="$ROOT/lab/oracle/cconv_harness"
# cob_toupper/tolower/field_to_string/init_cconv are internal (hidden) in libcob.so; only the EXACT
# oracle object compiled from cconv.c (extracted from the static libcob.a) exposes them. cconv.o needs
# only cob_runtime_error (stubbed in the harness) + libc, so no db/ncurses/gmp cascade.
if [ ! -x "$H" ]; then
  OBJ="$(mktemp -d)"; ( cd "$OBJ" && ar x "$PREFIX/lib/libcob.a" cconv.o ) || exit 2
  gcc -O2 -I"$PREFIX/include" "$ROOT/lab/oracle/cconv_harness.c" "$OBJ/cconv.o" -o "$H" || exit 2
  rm -rf "$OBJ"
fi
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/cconv_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
# the .ttbl collation tables shipped in the oracle config dir
TTBLS=$(ls "$PREFIX"/share/gnucobol/config/*.ttbl 2>/dev/null)
"$H"    $TTBLS > "$TMP/c.out"
"$ROWS" $TTBLS > "$TMP/rs.out"
TOTAL=$(wc -l < "$TMP/c.out")
PASS=0; FAIL=0
paste -d'\n' /dev/null /dev/null >/dev/null 2>&1
# compare line by line (same ordering on both sides)
mapfile -t C < "$TMP/c.out"
mapfile -t R < "$TMP/rs.out"
n=${#C[@]}
for ((i=0;i<n;i++)); do
  if [ "${C[$i]:-}" = "${R[$i]:-}" ]; then PASS=$((PASS+1)); else
    FAIL=$((FAIL+1)); [ "$FAIL" -le 12 ] && { echo "MISMATCH line $((i+1))" >&2; echo "  oracle: ${C[$i]:0:120}" >&2; echo "  rust:   ${R[$i]:0:120}" >&2; }
  fi
done
echo "total=$TOTAL  PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ] && [ "$n" = "$(wc -l < "$TMP/rs.out")" ]
