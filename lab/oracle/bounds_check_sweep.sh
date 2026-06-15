#!/usr/bin/env bash
# Runtime bounds-check message sweep (GNURUST.COMMON.BOUNDCHECK.1). Compiles `-debug` COBOL programs that
# go out of bounds on a table subscript, a reference modification, and an OCCURS DEPENDING ON length, runs
# them against the admitted oracle, extracts the `error:`/`note:` runtime lines, and checks the native
# common.c bounds-check reproduction (cob_check_subscript / cob_check_ref_mod / cob_check_odo) is
# byte-identical. DIFFERENTIAL: if the 3.1.2 second oracle is built (lab/oracle/prefix-312) the same lines
# are required to match it too, confirming the messages are version-stable, not a 3.2 quirk. PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PRIMARY="$ROOT/lab/oracle/prefix"
SECOND="$ROOT/lab/oracle/prefix-312"
[ -x "$PRIMARY/bin/cobc" ] || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/bounds_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

# Extract `error=`/`note=` lines from a -debug run's stderr/stdout.
extract() {  # prefix  src  -> prints case=.. error=.. note=..
  local pfx="$1" src="$2" id="$3"
  local d; d="$(mktemp -d)"
  ( cd "$d" && printf '%s' "$src" > p.cob \
    && PATH="$pfx/bin:$PATH" LD_LIBRARY_PATH="$pfx/lib" COB_CONFIG_DIR="$pfx/share/gnucobol/config" \
       COB_COPY_DIR="$pfx/share/gnucobol/copy" LC_ALL=C.UTF-8 "$pfx/bin/cobc" -free -debug -x -o p p.cob 2>/dev/null \
    && PATH="$pfx/bin:$PATH" LD_LIBRARY_PATH="$pfx/lib" COB_CONFIG_DIR="$pfx/share/gnucobol/config" ./p 2>&1 ) \
    | sed -nE "s/^libcob:[^:]*:[0-9]+: error: (.*)$/error=\\1/p; s/^note: (.*)$/note=\\1/p" > "$d/out"
  { echo "case=$id"; cat "$d/out"; }
  rm -rf "$d"
}

SUB='>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. P.
DATA DIVISION.
WORKING-STORAGE SECTION.
01 T. 05 E PIC X OCCURS 3.
01 I PIC 9 VALUE 5.
PROCEDURE DIVISION.
    DISPLAY E(I).
    STOP RUN.'
REF='>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. P.
DATA DIVISION.
WORKING-STORAGE SECTION.
01 F PIC X(5) VALUE "ABCDE".
01 O PIC 9 VALUE 2.
01 L PIC 9 VALUE 9.
PROCEDURE DIVISION.
    DISPLAY F(O:L).
    STOP RUN.'
ODO='>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. P.
DATA DIVISION.
WORKING-STORAGE SECTION.
01 N PIC 9 VALUE 7.
01 T. 05 E PIC X OCCURS 1 TO 5 DEPENDING ON N.
PROCEDURE DIVISION.
    DISPLAY T.
    STOP RUN.'

PASS=0; FAIL=0
check_case() {  # id  src
  local id="$1" src="$2"
  local pout; pout="$(extract "$PRIMARY" "$src" "$id")"
  echo "$pout" | "$ROWS" >/dev/null && PASS=$((PASS+1)) || { FAIL=$((FAIL+1)); echo "PRIMARY mismatch ($id)"; echo "$pout"; }
  if [ -x "$SECOND/bin/cobc" ]; then
    local sout; sout="$(extract "$SECOND" "$src" "$id")"
    # The native reproduction must match the SECOND oracle too (differential / version-stable).
    echo "$sout" | "$ROWS" >/dev/null && PASS=$((PASS+1)) || { FAIL=$((FAIL+1)); echo "SECOND(3.1.2) mismatch ($id)"; echo "$sout"; }
  fi
}
check_case subscript "$SUB"
check_case refmod "$REF"
check_case odo "$ODO"
echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ] || exit 1
