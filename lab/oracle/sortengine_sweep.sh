#!/usr/bin/env bash
# SORT-engine sweep (GNURUST.FILEIO.SORTENGINE.1). SORT a larger, duplicate-laden record set ON ASCENDING
# KEY K1 ON DESCENDING KEY K2, dump the GIVING file bytes, and check the in-memory 4-queue natural-merge
# engine fileio::CobSort (submit all + giving) produces the same order. PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/sortengine_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cd "$TMP" || exit 2

cat > srt.cob <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. SRT.
ENVIRONMENT DIVISION.
INPUT-OUTPUT SECTION.
FILE-CONTROL.
    SELECT WRK ASSIGN "w.tmp".
    SELECT INF ASSIGN "in.dat" ORGANIZATION IS RECORD SEQUENTIAL.
    SELECT OUTF ASSIGN "out.dat" ORGANIZATION IS RECORD SEQUENTIAL.
DATA DIVISION.
FILE SECTION.
SD WRK.
01 W-REC.
   05 K1 PIC X(3).
   05 K2 PIC X(2).
   05 W-REST PIC X(3).
FD INF.
01 I-REC PIC X(8).
FD OUTF.
01 O-REC PIC X(8).
PROCEDURE DIVISION.
    OPEN OUTPUT INF.
    MOVE "BBB10p01" TO I-REC. WRITE I-REC.
    MOVE "AAA20p02" TO I-REC. WRITE I-REC.
    MOVE "BBB05p03" TO I-REC. WRITE I-REC.
    MOVE "AAA20p04" TO I-REC. WRITE I-REC.
    MOVE "CCC00p05" TO I-REC. WRITE I-REC.
    MOVE "BBB10p06" TO I-REC. WRITE I-REC.
    MOVE "AAA20p07" TO I-REC. WRITE I-REC.
    MOVE "DDD99p08" TO I-REC. WRITE I-REC.
    MOVE "BBB05p09" TO I-REC. WRITE I-REC.
    MOVE "CCC00p10" TO I-REC. WRITE I-REC.
    MOVE "AAA10p11" TO I-REC. WRITE I-REC.
    MOVE "BBB10p12" TO I-REC. WRITE I-REC.
    MOVE "AAA20p13" TO I-REC. WRITE I-REC.
    MOVE "CCC50p14" TO I-REC. WRITE I-REC.
    MOVE "DDD99p15" TO I-REC. WRITE I-REC.
    MOVE "AAA10p16" TO I-REC. WRITE I-REC.
    CLOSE INF.
    SORT WRK ON ASCENDING KEY K1 ON DESCENDING KEY K2
        USING INF GIVING OUTF.
    STOP RUN.
COB
cobc -free -x -o srt srt.cob 2>e || { echo "compile failed"; cat e; exit 2; }
./srt >/dev/null 2>&1
printf 'sorted=%s\n' "$(xxd -p out.dat 2>/dev/null | tr -d '\n')" | "$ROWS"
