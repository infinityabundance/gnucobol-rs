#!/usr/bin/env bash
# SIZE.ERROR.ATLAS.1 — observe GnuCOBOL arithmetic size-error behavior (ATLAS, not implementation).
# Each case pre-fills a receiver with a SENTINEL value, runs an overflowing/divide-by-zero arithmetic op
# either plain or with ON SIZE ERROR, and DISPLAYs the receiver's raw bytes (via a REDEFINES X) BEFORE and
# AFTER + a size-error flag. We then record: receiver_written (after != before) and size_error_signaled.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" \
  COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; echo "PASS=0 FAIL=0"; exit 2; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

# case: label  recv_pic  usage  size  init  arith   expect_written_plain  (assertion for the SE variant is
# always preserved+signaled; for plain it is the documented truncation/zero behavior)
# arith uses RCV as the receiver; %SE% is replaced by the ON SIZE ERROR clause (or empty).
emit_case() {  # label recv_pic usage size init arith
  local label="$1" pic="$2" usage="$3" size="$4" init="$5" arith="$6" se="$7"
  local seclause=""; [ "$se" = "1" ] && seclause='ON SIZE ERROR MOVE "Y" TO SEF'
  {
    echo ">>SOURCE FORMAT FREE"
    echo "IDENTIFICATION DIVISION."
    echo "PROGRAM-ID. SE${label}."
    echo "DATA DIVISION."
    echo "WORKING-STORAGE SECTION."
    echo "01 RCV PIC ${pic}${usage}."
    echo "01 RCVX REDEFINES RCV PIC X(${size})."
    echo "01 ZERO-V PIC 9 VALUE 0."
    echo "01 SEF PIC X VALUE \"N\"."
    echo "PROCEDURE DIVISION."
    echo "MOVE ${init} TO RCV."
    echo "DISPLAY \"BEFORE[\" RCVX \"]\"."
    echo "${arith} ${seclause}."
    echo "DISPLAY \"AFTER[\" RCVX \"]\"."
    echo "DISPLAY \"SE[\" SEF \"]\"."
    echo "STOP RUN."
  } > "$TMP/$1.cob"
  ( cd "$TMP" && cobc -x -free -o "$1.bin" "$1.cob" >/dev/null 2>"$TMP/$1.err" ) || { echo "COMPILE-FAIL $1"; return 1; }
  ( cd "$TMP" && ./"$1.bin" > "$TMP/$1.out" 2>/dev/null ); :
}

# scenarios (label op recv usage size init arith) — each run plain (P) and size-error (S)
# ADD/MUL/SUB/DIVIDE × DISPLAY/COMP-3 × signed/ROUNDED/divide-by-zero
gen() { # base pic usage size init arith
  emit_case "${1}P" "$2" "$3" "$4" "$5" "$6" 0
  emit_case "${1}S" "$2" "$3" "$4" "$5" "$6" 1
}
gen ADDDISP  "9(3)" ""             3 999  "ADD 500 TO RCV"
gen ADDC3    "9(3)" " USAGE COMP-3" 2 999  "ADD 500 TO RCV"
gen MULDISP  "9(3)" ""             3 999  "MULTIPLY 5 BY RCV"
gen SUBSIGN  "S9(3)" ""            3 -999 "SUBTRACT 500 FROM RCV"
gen DIV0DISP "9(3)" ""             3 999  "DIVIDE RCV BY ZERO-V GIVING RCV"
gen ROUNDDISP "9(3)" ""            3 999  "ADD 0.6 TO RCV ROUNDED"

( cd "$ROOT" && cargo run -q -p xtask -- atlas-size-error "$TMP" )
