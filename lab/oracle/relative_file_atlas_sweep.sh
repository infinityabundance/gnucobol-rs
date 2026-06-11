#!/usr/bin/env bash
# GNURUST.RELATIVE.FILE.ATLAS.1 -- observe RELATIVE-file random access (by relative record number) + status.
# OBSERVED court: gnucobol-rs implements no relative file I/O (the on-disk slotted format is backend-specific).
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cat > "$TMP/p.cob" <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. REL.
ENVIRONMENT DIVISION.
INPUT-OUTPUT SECTION.
FILE-CONTROL.
    SELECT F ASSIGN "rel.dat" ORGANIZATION IS RELATIVE
        ACCESS MODE IS RANDOM RELATIVE KEY IS R-NUM FILE STATUS IS FS.
DATA DIVISION.
FILE SECTION.
FD F.
01 R PIC X(5).
WORKING-STORAGE SECTION.
01 R-NUM PIC 9(2).
01 FS PIC X(2).
PROCEDURE DIVISION.
    OPEN OUTPUT F.
    MOVE 3 TO R-NUM. MOVE "three" TO R. WRITE R.
    MOVE 1 TO R-NUM. MOVE "one  " TO R. WRITE R.
    MOVE 5 TO R-NUM. MOVE "five " TO R. WRITE R.
    CLOSE F.
    OPEN INPUT F.
    MOVE 3 TO R-NUM. READ F. DISPLAY "r3=" FS "/" R.
    MOVE 2 TO R-NUM. READ F. DISPLAY "r2=" FS.
    MOVE 1 TO R-NUM. READ F. DISPLAY "r1=" FS "/" R.
    CLOSE F.
    STOP RUN.
COB
cobc -free -x -o "$TMP/p" "$TMP/p.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }
OUT=$( cd "$TMP" && ./p )
( cd "$ROOT" && OUT="$OUT" cargo run -q -p xtask -- atlas-relative )
