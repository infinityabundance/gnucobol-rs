#!/usr/bin/env bash
# Numeric IF/EVALUATE execution-slice sweep (GNURUST.IF.NUMERIC.SLICE.1).
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/if_numeric_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cat > "$TMP/p.cob" <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. IFN.
DATA DIVISION.
WORKING-STORAGE SECTION.
01 N PIC 9(3).
01 F PIC 9(2).
PROCEDURE DIVISION.
    MOVE 50 TO N. MOVE 0 TO F. IF N > 100 MOVE 1 TO F ELSE MOVE 9 TO F END-IF. DISPLAY "gt100=" F.
    MOVE 50 TO N. MOVE 0 TO F. IF N < 100 MOVE 1 TO F ELSE MOVE 9 TO F END-IF. DISPLAY "lt100=" F.
    MOVE 50 TO N. MOVE 0 TO F. IF N = 50 MOVE 7 TO F ELSE MOVE 0 TO F END-IF. DISPLAY "eq50=" F.
    MOVE 50 TO N. MOVE 0 TO F. IF N >= 50 MOVE 1 TO F ELSE MOVE 9 TO F END-IF. DISPLAY "ge50=" F.
    MOVE 50 TO N. MOVE 0 TO F. EVALUATE N WHEN 10 MOVE 1 TO F WHEN 50 MOVE 5 TO F WHEN OTHER MOVE 8 TO F END-EVALUATE. DISPLAY "ev50=" F.
    MOVE 99 TO N. MOVE 0 TO F. EVALUATE N WHEN 10 MOVE 1 TO F WHEN 50 MOVE 5 TO F WHEN OTHER MOVE 8 TO F END-EVALUATE. DISPLAY "ev99=" F.
    MOVE 5 TO N. MOVE 0 TO F. IF N > 9 MOVE 1 TO F ELSE MOVE 9 TO F END-IF. DISPLAY "num5gt9=" F.
    STOP RUN.
COB
cobc -free -x -o "$TMP/p" "$TMP/p.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }
"$TMP/p" | "$ROWS"
