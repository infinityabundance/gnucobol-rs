#!/usr/bin/env bash
# INDEXED-organization sweep (GNURUST.FILEIO.INDEXED.1). Run a fixed INDEXED script (WRITE incl duplicate
# key, random READ hit/miss, READ NEXT in key order, START =/>/>=/<, REWRITE, DELETE) against the admitted
# libcob and check fileio::IndexedStore produces the same FILE STATUS + record bytes + key order. The
# on-disk index format is the declared OS boundary; the COBOL-observable behaviour is the court. PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/indexed_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cd "$TMP" || exit 2

cat > ix.cob <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. IX.
ENVIRONMENT DIVISION.
INPUT-OUTPUT SECTION.
FILE-CONTROL.
    SELECT IXF ASSIGN "ix.dat" ORGANIZATION INDEXED ACCESS DYNAMIC
        RECORD KEY IS K FILE STATUS IS ST.
DATA DIVISION.
FILE SECTION.
FD IXF.
01 IX-REC.
   05 K PIC X(3).
   05 V PIC X(5).
WORKING-STORAGE SECTION.
01 ST PIC XX.
PROCEDURE DIVISION.
    OPEN OUTPUT IXF.
    MOVE "BBB" TO K. MOVE "bbbbb" TO V. WRITE IX-REC. DISPLAY "w1 " ST.
    MOVE "DDD" TO K. MOVE "ddddd" TO V. WRITE IX-REC. DISPLAY "w2 " ST.
    MOVE "FFF" TO K. MOVE "fffff" TO V. WRITE IX-REC. DISPLAY "w3 " ST.
    MOVE "BBB" TO K. MOVE "dupbb" TO V. WRITE IX-REC. DISPLAY "wdup " ST.
    CLOSE IXF.
    OPEN I-O IXF.
    MOVE "DDD" TO K. READ IXF. DISPLAY "rDDD " ST " " V.
    MOVE "CCC" TO K. READ IXF. DISPLAY "rCCC " ST.
    MOVE LOW-VALUES TO K. START IXF KEY >= K.
    READ IXF NEXT. DISPLAY "n1 " ST " " K " " V.
    READ IXF NEXT. DISPLAY "n2 " ST " " K " " V.
    READ IXF NEXT. DISPLAY "n3 " ST " " K " " V.
    MOVE "CCC" TO K. START IXF KEY >= K. DISPLAY "ge " ST.
    READ IXF NEXT. DISPLAY "gen " ST " " K.
    MOVE "FFF" TO K. START IXF KEY > K. DISPLAY "gtf " ST.
    MOVE "DDD" TO K. MOVE "DD222" TO V. REWRITE IX-REC. DISPLAY "rwD " ST.
    MOVE "DDD" TO K. READ IXF. DISPLAY "rrD " ST " " V.
    MOVE "GGG" TO K. MOVE "ggggg" TO V. REWRITE IX-REC. DISPLAY "rwG " ST.
    MOVE "BBB" TO K. DELETE IXF. DISPLAY "delB " ST.
    MOVE "BBB" TO K. READ IXF. DISPLAY "rB " ST.
    MOVE "BBB" TO K. DELETE IXF. DISPLAY "delB2 " ST.
    CLOSE IXF.
    STOP RUN.
COB
cobc -free -x -o ix ix.cob 2>e || { echo "compile failed"; cat e; exit 2; }
./ix 2>/dev/null | "$ROWS"
