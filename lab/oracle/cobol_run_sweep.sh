#!/usr/bin/env bash
# END-TO-END EXECUTION sweep: run small COBOL programs on the native gnucobol-rs RUNTIME (no cobc, no
# libcob linked -- the `cobol_run` example hand-wires each statement into the ported runtime
# primitives) and diff the program stdout against the SAME source compiled+run by the admitted cobc.
# A byte-identical result is direct evidence that the 13/13-file libcob runtime port is
# execution-complete for these programs -- the substrate a future COBOL front-end (lexer/parser/codegen)
# would target. The front-end itself is NOT part of this (the wiring here is by hand). PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --example cobol_run >/dev/null 2>&1 ) || exit 2
EMIT="$ROOT/target/release/examples/cobol_run"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT; cd "$TMP" || exit 2

# program-name -> COBOL source (the cobol_run example reproduces each by name on the runtime).
prog_src() {
  case "$1" in
    add) cat <<'COB'
       IDENTIFICATION DIVISION.
       PROGRAM-ID. DEMO.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 WS-A   PIC 9(5) VALUE 100.
       01 WS-B   PIC 9(5) VALUE 250.
       01 WS-RES PIC ZZ,ZZ9.
       PROCEDURE DIVISION.
           ADD WS-A TO WS-B.
           MOVE WS-B TO WS-RES.
           DISPLAY "TOTAL=" WS-RES.
           STOP RUN.
COB
    ;;
    mul) cat <<'COB'
       IDENTIFICATION DIVISION.
       PROGRAM-ID. DEMO.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 WS-P  PIC 9(5) VALUE 12.
       01 WS-Q  PIC 9(3) VALUE 4.
       01 WS-R  PIC 9(5).
       01 WS-RE PIC ZZ,ZZ9.
       PROCEDURE DIVISION.
           MULTIPLY WS-P BY WS-Q GIVING WS-R.
           MOVE WS-R TO WS-RE.
           DISPLAY "PRODUCT=" WS-RE.
           STOP RUN.
COB
    ;;
    div) cat <<'COB'
       IDENTIFICATION DIVISION.
       PROGRAM-ID. DEMO.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 WS-N  PIC 9(6)V99 VALUE 1000.00.
       01 WS-D  PIC 9 VALUE 8.
       01 WS-Q  PIC 9(4)V99.
       01 WS-QE PIC Z,ZZ9.99.
       PROCEDURE DIVISION.
           DIVIDE WS-N BY WS-D GIVING WS-Q.
           MOVE WS-Q TO WS-QE.
           DISPLAY "QUOTIENT=" WS-QE.
           STOP RUN.
COB
    ;;
  esac
}

PASS=0; FAIL=0
for name in add mul div; do
  prog_src "$name" > p.cob
  if ! cobc -x -free -o p p.cob 2>cobc.err; then
    echo "$name: cobc compile FAIL"; head -2 cobc.err; FAIL=$((FAIL+1)); continue
  fi
  ./p > oracle.out 2>/dev/null
  "$EMIT" "$name" > rust.out 2>/dev/null
  if cmp -s oracle.out rust.out; then
    PASS=$((PASS+1)); echo "$name: IDENTICAL [$(cat oracle.out)]"
  else
    FAIL=$((FAIL+1))
    echo "$name: DIFFER"
    echo "  cobc: $(cat -A oracle.out)"
    echo "  rust: $(cat -A rust.out)"
  fi
done
echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ] || exit 1
