#!/usr/bin/env bash
# RELATIVE organization sweep (GNURUST.FILEIO.RELATIVE.1). Keyed WRITE/DELETE/REWRITE/READ + READ NEXT
# over an ORGANIZATION IS RELATIVE file, checked against fileio::relative_* (file bytes + FILE STATUS). PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/relative_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cd "$TMP" || exit 2

cat > rel.cob <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. REL.
ENVIRONMENT DIVISION.
INPUT-OUTPUT SECTION.
FILE-CONTROL.
    SELECT FR ASSIGN "r.dat" ORGANIZATION IS RELATIVE ACCESS MODE IS RANDOM
       RELATIVE KEY IS RK FILE STATUS IS ST.
    SELECT FS ASSIGN "r.dat" ORGANIZATION IS RELATIVE ACCESS MODE IS SEQUENTIAL
       RELATIVE KEY IS SK FILE STATUS IS SS.
DATA DIVISION.
FILE SECTION.
FD FR.
01 RR PIC X(4).
FD FS.
01 RS PIC X(4).
WORKING-STORAGE SECTION.
01 ST PIC XX.
01 SS PIC XX.
01 RK PIC 9(4) COMP.
01 SK PIC 9(4) COMP.
01 OPS PIC X(20) VALUE SPACES.
01 RDS PIC X(20) VALUE SPACES.
01 SCAN PIC X(40) VALUE SPACES.
01 P  PIC 99 VALUE 1.
01 Q  PIC 99 VALUE 1.
01 DONE PIC X VALUE "N".
PROCEDURE DIVISION.
    OPEN OUTPUT FR.
    MOVE 1 TO RK. MOVE "AAAA" TO RR. WRITE RR. MOVE ST TO OPS(P:2). ADD 2 TO P.
    MOVE 3 TO RK. MOVE "CCCC" TO RR. WRITE RR. MOVE ST TO OPS(P:2). ADD 2 TO P.
    CLOSE FR.
    OPEN I-O FR.
    MOVE 1 TO RK. DELETE FR. MOVE ST TO OPS(P:2). ADD 2 TO P.
    MOVE 5 TO RK. MOVE "EEEE" TO RR. WRITE RR. MOVE ST TO OPS(P:2). ADD 2 TO P.
    MOVE 3 TO RK. MOVE "ZZZZ" TO RR. REWRITE RR. MOVE ST TO OPS(P:2). ADD 2 TO P.
    CLOSE FR.
    OPEN INPUT FR.
    MOVE 1 TO RK. READ FR. MOVE ST TO RDS(1:2).
    MOVE 2 TO RK. READ FR. MOVE ST TO RDS(3:2).
    MOVE 3 TO RK. READ FR. MOVE ST TO RDS(5:2).
    CLOSE FR.
    OPEN INPUT FS.
    PERFORM UNTIL DONE = "Y"
        READ FS NEXT RECORD AT END MOVE "Y" TO DONE
          NOT AT END MOVE RS TO SCAN(Q:4) ADD 4 TO Q
        END-READ
        IF SS = "10" MOVE "Y" TO DONE END-IF
    END-PERFORM.
    MOVE SS TO SCAN(Q:2).
    CLOSE FS.
    DISPLAY "opstatus=" FUNCTION TRIM(OPS).
    DISPLAY "readsts=" FUNCTION TRIM(RDS).
    DISPLAY "scan=" FUNCTION TRIM(SCAN).
    STOP RUN.
COB
cobc -free -x -o rel rel.cob 2>e || { echo "compile failed"; cat e; exit 2; }
rm -f r.dat
OUT=$(./rel 2>/dev/null)
RELFILE=$(xxd -p r.dat | tr -d '\n')
{
  printf 'relfile=%s\n' "$RELFILE"
  printf '%s\n' "$OUT"
} | "$ROWS"
