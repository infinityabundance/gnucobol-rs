#!/usr/bin/env bash
# GNURUST.INDEXED.FILE.ATLAS.1 -- observe INDEXED-file keyed access + status against real cobc/libcob. The
# largest remaining gap cluster (START 238x + DELETE 118x + indexed-org per GNURUST.PUBLIC.GAP.1). OBSERVED
# court: gnucobol-rs does NOT implement indexed files -- the on-disk ISAM/BDB/VBISAM format is backend-specific
# and out of the fixed-record evidence lane. This MAPS the surface: keyed random access, key-order retrieval,
# duplicate-key/not-found status, START positioning, DELETE.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cat > "$TMP/p.cob" <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. IDX.
ENVIRONMENT DIVISION.
INPUT-OUTPUT SECTION.
FILE-CONTROL.
    SELECT F ASSIGN "idx.dat" ORGANIZATION IS INDEXED
        ACCESS MODE IS DYNAMIC RECORD KEY IS R-KEY FILE STATUS IS FS.
DATA DIVISION.
FILE SECTION.
FD F.
01 R.
   05 R-KEY PIC X(3).
   05 R-VAL PIC X(5).
WORKING-STORAGE SECTION.
01 FS PIC X(2).
PROCEDURE DIVISION.
    OPEN OUTPUT F.
    MOVE "AAA" TO R-KEY. MOVE "alpha" TO R-VAL. WRITE R.
    MOVE "CCC" TO R-KEY. MOVE "gamma" TO R-VAL. WRITE R.
    MOVE "BBB" TO R-KEY. MOVE "beta " TO R-VAL. WRITE R.
    MOVE "AAA" TO R-KEY. MOVE "dup  " TO R-VAL. WRITE R. DISPLAY "dup=" FS.
    CLOSE F.
    OPEN I-O F.
    MOVE "BBB" TO R-KEY. READ F. DISPLAY "read_hit=" FS "/" R-VAL.
    MOVE "ZZZ" TO R-KEY. READ F. DISPLAY "read_miss=" FS.
    MOVE "AAA" TO R-KEY. START F KEY >= R-KEY. DISPLAY "start=" FS.
    READ F NEXT. DISPLAY "n1=" R-KEY. READ F NEXT. DISPLAY "n2=" R-KEY. READ F NEXT. DISPLAY "n3=" R-KEY.
    MOVE "BBB" TO R-KEY. DELETE F. DISPLAY "del=" FS.
    MOVE "BBB" TO R-KEY. READ F. DISPLAY "read_del=" FS.
    CLOSE F.
    STOP RUN.
COB
cobc -free -x -o "$TMP/p" "$TMP/p.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }
OUT=$( cd "$TMP" && ./p )
( cd "$ROOT" && OUT="$OUT" cargo run -q -p xtask -- atlas-indexed )
