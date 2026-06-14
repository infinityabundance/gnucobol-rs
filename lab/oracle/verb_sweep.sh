#!/usr/bin/env bash
# Verb-precondition sweep (GNURUST.FILEIO.VERB.1). Attempt WRITE/READ/REWRITE/DELETE/START in the wrong
# OPEN/ACCESS mode and capture the FILE STATUS, checked against fileio::cob_* verb preconditions. PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/verb_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cd "$TMP" || exit 2

cat > v.cob <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. V.
ENVIRONMENT DIVISION.
INPUT-OUTPUT SECTION.
FILE-CONTROL.
    SELECT FS ASSIGN "s.dat" ORGANIZATION IS RECORD SEQUENTIAL FILE STATUS IS SS.
    SELECT FR ASSIGN "r.dat" ORGANIZATION IS RELATIVE ACCESS MODE IS RANDOM
       RELATIVE KEY IS RK FILE STATUS IS RS.
DATA DIVISION.
FILE SECTION.
FD FS.
01 RECS PIC X(4).
FD FR.
01 RECR PIC X(4).
WORKING-STORAGE SECTION.
01 SS PIC XX.
01 RS PIC XX.
01 RK PIC 9(4) COMP.
PROCEDURE DIVISION.
*> create the files
    OPEN OUTPUT FS. MOVE "AAAA" TO RECS. WRITE RECS. CLOSE FS.
    OPEN OUTPUT FR. MOVE 1 TO RK. MOVE "AAAA" TO RECR. WRITE RECR. CLOSE FR.
*> WRITE on INPUT (sequential) -> 48
    OPEN INPUT FS. WRITE RECS. DISPLAY "w_input_seq=" SS. CLOSE FS.
*> READ on OUTPUT -> 47
    OPEN OUTPUT FS. READ FS. DISPLAY "r_output=" SS. CLOSE FS.
    OPEN OUTPUT FS. MOVE "AAAA" TO RECS. WRITE RECS. CLOSE FS.
*> REWRITE on INPUT -> 49
    OPEN INPUT FS. REWRITE RECS. DISPLAY "rw_input=" SS. CLOSE FS.
*> REWRITE on I-O without prior READ -> 43
    OPEN I-O FS. REWRITE RECS. DISPLAY "rw_io_noread=" SS. CLOSE FS.
*> READ NEXT on OUTPUT -> 47
    OPEN OUTPUT FS. READ FS NEXT RECORD. DISPLAY "rn_output=" SS. CLOSE FS.
    OPEN OUTPUT FS. MOVE "AAAA" TO RECS. WRITE RECS. CLOSE FS.
*> DELETE on INPUT (relative) -> 49
    OPEN INPUT FR. MOVE 1 TO RK. DELETE FR. DISPLAY "del_input=" RS. CLOSE FR.
*> WRITE on INPUT (relative) -> 48
    OPEN INPUT FR. MOVE 2 TO RK. WRITE RECR. DISPLAY "w_input_rel=" RS. CLOSE FR.
    STOP RUN.
COB
cobc -free -x -o v v.cob 2>e || { echo "compile failed"; cat e; exit 2; }
./v 2>/dev/null | "$ROWS"
