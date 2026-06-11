#!/usr/bin/env bash
# Intrinsic-function atlas sweep (GNURUST.INTRINSIC.ATLAS.1). Probe high-use intrinsics with declared inputs
# against real cobc/libcob, assert the DETERMINISTIC results are stable, and emit reports/intrinsic-atlas.json
# classifying each (deterministic candidate-court vs environment-sensitive shape-only). OBSERVED court: maps
# the intrinsic surface before any are implemented.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cat > "$TMP/p.cob" <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. P.
DATA DIVISION.
WORKING-STORAGE SECTION.
01 A5 PIC X(5) VALUE "HELLO".
01 N9 PIC 9(8)V99.
01 NS PIC S9(8)V99.
01 RT PIC X(8).
PROCEDURE DIVISION.
    MOVE FUNCTION LENGTH(A5) TO N9.        DISPLAY "LENGTH=" N9.
    MOVE FUNCTION BYTE-LENGTH(A5) TO N9.   DISPLAY "BYTE_LENGTH=" N9.
    MOVE FUNCTION NUMVAL("123.45") TO N9.  DISPLAY "NUMVAL=" N9.
    MOVE FUNCTION NUMVAL-C("$1,234.56") TO N9. DISPLAY "NUMVAL_C=" N9.
    MOVE FUNCTION INTEGER(3.7) TO NS.      DISPLAY "INTEGER_P=" NS.
    MOVE FUNCTION INTEGER(-3.7) TO NS.     DISPLAY "INTEGER_N=" NS.
    MOVE FUNCTION INTEGER-PART(3.7) TO NS. DISPLAY "INTPART_P=" NS.
    MOVE FUNCTION INTEGER-PART(-3.7) TO NS. DISPLAY "INTPART_N=" NS.
    MOVE FUNCTION MOD(17,5) TO NS.         DISPLAY "MOD_P=" NS.
    MOVE FUNCTION MOD(-17,5) TO NS.        DISPLAY "MOD_N=" NS.
    MOVE FUNCTION REM(17,5) TO NS.         DISPLAY "REM_P=" NS.
    MOVE FUNCTION REM(-17,5) TO NS.        DISPLAY "REM_N=" NS.
    MOVE FUNCTION UPPER-CASE("abc") TO RT. DISPLAY "UPPER=[" RT "]".
    MOVE FUNCTION LOWER-CASE("ABC") TO RT. DISPLAY "LOWER=[" RT "]".
    MOVE FUNCTION REVERSE("abcd") TO RT.   DISPLAY "REVERSE=[" RT "]".
    MOVE FUNCTION ORD("A") TO N9.          DISPLAY "ORD=" N9.
    MOVE FUNCTION CHAR(66) TO RT.          DISPLAY "CHAR=[" RT "]".
    DISPLAY "CURRENT_DATE_LEN=" FUNCTION LENGTH(FUNCTION CURRENT-DATE).
    DISPLAY "WHEN_COMPILED_LEN=" FUNCTION LENGTH(FUNCTION WHEN-COMPILED).
    STOP RUN.
COB
cobc -free -x -o "$TMP/p" "$TMP/p.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }
OUT=$("$TMP/p")
( cd "$ROOT" && OUT="$OUT" cargo run -q -p xtask -- atlas-intrinsic )
