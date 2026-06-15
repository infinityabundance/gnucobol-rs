#!/usr/bin/env bash
# Not-numeric runtime diagnostic sweep (GNURUST.COMMON.NUMCHECK.1). Compiles a `-debug` program that moves a
# non-numeric value into a NUMERIC DISPLAY field and uses it in arithmetic, runs it against the oracle,
# extracts the `error:` line, and checks the native common.c reproduction (cob_check_numeric +
# explain_field_type) is byte-identical. DIFFERENTIAL: required to match BOTH oracles (3.1.2 + 3.2) when the
# second is built -- confirming the diagnostic is version-stable. PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PRIMARY="$ROOT/lab/oracle/prefix"
SECOND="$ROOT/lab/oracle/prefix-312"
[ -x "$PRIMARY/bin/cobc" ] || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/bounds_rows"

SRC='>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. P.
DATA DIVISION.
WORKING-STORAGE SECTION.
01 N PIC 9(3) VALUE ZERO.
01 R PIC 9(5).
PROCEDURE DIVISION.
    MOVE "12X" TO N
    COMPUTE R = N + 1
    DISPLAY R
    STOP RUN.'

extract() {  # prefix
  local pfx="$1" d; d="$(mktemp -d)"
  ( cd "$d" && printf '%s' "$SRC" > p.cob \
    && PATH="$pfx/bin:$PATH" LD_LIBRARY_PATH="$pfx/lib" COB_CONFIG_DIR="$pfx/share/gnucobol/config" \
       COB_COPY_DIR="$pfx/share/gnucobol/copy" LC_ALL=C.UTF-8 "$pfx/bin/cobc" -free -debug -x -o p p.cob 2>/dev/null \
    && PATH="$pfx/bin:$PATH" LD_LIBRARY_PATH="$pfx/lib" COB_CONFIG_DIR="$pfx/share/gnucobol/config" ./p 2>&1 ) \
    | sed -nE "s/^libcob:[^:]*:[0-9]+: error: (.*)$/error=\\1/p"
  rm -rf "$d"
}

PASS=0; FAIL=0
{ echo "case=numeric"; extract "$PRIMARY"; } | "$ROWS" >/dev/null && PASS=$((PASS+1)) || { FAIL=$((FAIL+1)); echo "PRIMARY mismatch"; }
if [ -x "$SECOND/bin/cobc" ]; then
  { echo "case=numeric"; extract "$SECOND"; } | "$ROWS" >/dev/null && PASS=$((PASS+1)) || { FAIL=$((FAIL+1)); echo "SECOND(3.1.2) mismatch"; }
fi
echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ] || exit 1
