#!/usr/bin/env bash
# File-runtime OPEN/CLOSE sweep (GNURUST.FILEIO.OPEN.1). OPEN/WRITE/READ/CLOSE a LINE SEQUENTIAL file and
# exercise the open/close status matrix (41 already-open, 42 not-open, 38 closed-with-lock, 35 missing),
# checked against fileio::CobFile + cob_open/cob_close (file image bytes + FILE STATUS sequence). PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/open_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cd "$TMP" || exit 2

cat > op.cob <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. OP.
ENVIRONMENT DIVISION.
INPUT-OUTPUT SECTION.
FILE-CONTROL.
    SELECT FL ASSIGN "ls.dat" ORGANIZATION IS LINE SEQUENTIAL FILE STATUS IS ST.
    SELECT FM ASSIGN "none.dat" ORGANIZATION IS LINE SEQUENTIAL FILE STATUS IS SM.
DATA DIVISION.
FILE SECTION.
FD FL.
01 RL PIC X(8).
FD FM.
01 RM PIC X(8).
WORKING-STORAGE SECTION.
01 ST PIC XX.
01 SM PIC XX.
01 OUT PIC X(60) VALUE SPACES.
01 P PIC 99 VALUE 1.
PROCEDURE DIVISION.
    OPEN OUTPUT FL. PERFORM PUT.
    OPEN OUTPUT FL. PERFORM PUT.
    MOVE "AB" TO RL. WRITE RL.
    MOVE "XY" TO RL. WRITE RL.
    CLOSE FL. PERFORM PUT.
    CLOSE FL. PERFORM PUT.
    OPEN INPUT FL. PERFORM PUT.
    READ FL NEXT RECORD AT END CONTINUE END-READ.
    READ FL NEXT RECORD AT END CONTINUE END-READ.
    CLOSE FL WITH LOCK.
    OPEN INPUT FL. PERFORM PUT.
    CLOSE FL.
    OPEN INPUT FM. MOVE SM TO ST. PERFORM PUT.
    DISPLAY "statuses=" FUNCTION TRIM(OUT).
    STOP RUN.
PUT.
    IF P > 1 STRING "," DELIMITED BY SIZE INTO OUT WITH POINTER P END-IF.
    STRING FUNCTION TRIM(ST) DELIMITED BY SIZE INTO OUT WITH POINTER P.
COB
cobc -free -x -o op op.cob 2>e || { echo "compile failed"; cat e; exit 2; }
rm -f ls.dat
ST_OUT=$(./op 2>/dev/null)
{
  printf '%s\n' "$ST_OUT"
  printf 'image=%s\n' "$(xxd -p ls.dat 2>/dev/null | tr -d '\n')"
} | "$ROWS"
