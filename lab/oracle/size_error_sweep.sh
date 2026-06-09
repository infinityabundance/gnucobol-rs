#!/usr/bin/env bash
# SIZE ERROR sweep (GNURUST.SIZE.ERROR.1). For each arithmetic store: no-ON-SIZE-ERROR truncated receiver +
# the ON-SIZE-ERROR condition flag, checked against arith_size_error.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/size_error_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cat > "$TMP/p.cob" <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. SE.
DATA DIVISION.
WORKING-STORAGE SECTION.
01 RN1 PIC 9(3).
01 RN2 PIC 9(3)V99.
01 RN3 PIC 9(3).
01 RN4 PIC 9(3).
01 RN5 PIC 9(1)V9.
01 RN6 PIC 9(1)V9.
01 F  PIC 9.
PROCEDURE DIVISION.
    MOVE 0 TO RN1.
    COMPUTE RN1 = 999 + 999.
    DISPLAY "a_t=[" RN1 "]".
    MOVE 0 TO F.
    COMPUTE RN1 = 999 + 999 ON SIZE ERROR MOVE 1 TO F END-COMPUTE.
    DISPLAY "a_e=" F.
    MOVE 0 TO RN2.
    COMPUTE RN2 = 1234.567.
    DISPLAY "b_t=[" RN2 "]".
    MOVE 0 TO F.
    COMPUTE RN2 = 1234.567 ON SIZE ERROR MOVE 1 TO F END-COMPUTE.
    DISPLAY "b_e=" F.
    MOVE 0 TO RN3.
    COMPUTE RN3 = 12 + 34.
    DISPLAY "c_t=[" RN3 "]".
    MOVE 0 TO F.
    COMPUTE RN3 = 12 + 34 ON SIZE ERROR MOVE 1 TO F END-COMPUTE.
    DISPLAY "c_e=" F.
    MOVE 0 TO RN4.
    COMPUTE RN4 = 50000.
    DISPLAY "d_t=[" RN4 "]".
    MOVE 0 TO F.
    COMPUTE RN4 = 50000 ON SIZE ERROR MOVE 1 TO F END-COMPUTE.
    DISPLAY "d_e=" F.
    MOVE 0 TO RN5.
    COMPUTE RN5 = 12.5.
    DISPLAY "e_t=[" RN5 "]".
    MOVE 0 TO F.
    COMPUTE RN5 = 12.5 ON SIZE ERROR MOVE 1 TO F END-COMPUTE.
    DISPLAY "e_e=" F.
    MOVE 0 TO RN6.
    COMPUTE RN6 = 7.89.
    DISPLAY "f_t=[" RN6 "]".
    MOVE 0 TO F.
    COMPUTE RN6 = 7.89 ON SIZE ERROR MOVE 1 TO F END-COMPUTE.
    DISPLAY "f_e=" F.
    STOP RUN.
COB
cobc -free -x -o "$TMP/p" "$TMP/p.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }
"$TMP/p" | "$ROWS"
