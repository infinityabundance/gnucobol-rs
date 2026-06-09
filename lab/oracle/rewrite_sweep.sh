#!/usr/bin/env bash
# Sequential REWRITE sweep (GNURUST.FILE.REWRITE.1). Create a RECORD SEQUENTIAL file, OPEN I-O, REWRITE records
# 0 and 2 in place, hexdump the file, and check rewrite_records == the oracle bytes.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/rewrite_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cat > "$TMP/p.cob" <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. RWS.
ENVIRONMENT DIVISION.
INPUT-OUTPUT SECTION.
FILE-CONTROL.
    SELECT F ASSIGN "io.dat" ORGANIZATION IS RECORD SEQUENTIAL.
DATA DIVISION.
FILE SECTION.
FD F.
01 R PIC X(4).
WORKING-STORAGE SECTION.
PROCEDURE DIVISION.
    OPEN OUTPUT F.
    MOVE "AAAA" TO R. WRITE R.
    MOVE "BBBB" TO R. WRITE R.
    MOVE "CCCC" TO R. WRITE R.
    CLOSE F.
    OPEN I-O F.
    READ F.
    MOVE "X1X1" TO R. REWRITE R.
    READ F.
    READ F.
    MOVE "Z3Z3" TO R. REWRITE R.
    CLOSE F.
    STOP RUN.
COB
( cd "$TMP" && cobc -free -x -o p p.cob 2>err && ./p ) || { echo "compile/run failed"; cat "$TMP/err"; exit 2; }
printf 'rw=%s\n' "$(xxd -p "$TMP/io.dat" | tr -d '\n')" | "$ROWS"
