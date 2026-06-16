#!/usr/bin/env bash
# COBOL FRONT-END sweep (the "Not a COBOL compiler" negative -> positive): for a corpus of small COBOL
# programs, run each through the CLEAN-ROOM gnucobol-rs front-end `cobrun` (parse + execute on the
# ported runtime -- no cobc, no libcob linked) AND through the admitted cobc, and diff stdout. A
# byte-identical result for every program is the proof: gnucobol-rs parses and runs these COBOL
# programs to cobc-identical output. (Sealed subset: WORKING-STORAGE 01 elementary items with
# PIC/VALUE; MOVE / ADD / SUBTRACT / MULTIPLY / DIVIDE / DISPLAY / STOP RUN. Anything else fails
# closed.) PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --example cobrun >/dev/null 2>&1 ) || exit 2
COBRUN="$ROOT/target/release/examples/cobrun"
CORPUS="$ROOT/lab/corpus/frontend"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

PASS=0; FAIL=0
shopt -s nullglob
for cob in "$CORPUS"/*.cob; do
  name="$(basename "$cob" .cob)"
  if ! cobc -x -free -o "$TMP/p" "$cob" 2>"$TMP/cobc.err"; then
    echo "$name: cobc compile FAIL"; head -2 "$TMP/cobc.err"; FAIL=$((FAIL+1)); continue
  fi
  "$TMP/p" </dev/null > "$TMP/oracle.out" 2>/dev/null
  if ! "$COBRUN" "$cob" > "$TMP/rust.out" 2>"$TMP/rust.err"; then
    echo "$name: cobrun FAIL: $(cat "$TMP/rust.err")"; FAIL=$((FAIL+1)); continue
  fi
  if cmp -s "$TMP/oracle.out" "$TMP/rust.out"; then
    PASS=$((PASS+1)); echo "$name: IDENTICAL"
  else
    FAIL=$((FAIL+1))
    echo "$name: DIFFER"
    echo "  cobc:   $(cat -A "$TMP/oracle.out")"
    echo "  cobrun: $(cat -A "$TMP/rust.out")"
  fi
done
echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ] || exit 1
