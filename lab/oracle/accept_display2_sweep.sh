#!/usr/bin/env bash
# DISPLAY-numeric byte sweep (GNURUST.ACCEPT.DISPLAY.2). DISPLAY signed/V-scaled numeric fields and check
# display_numeric == the oracle's emitted text (signed -> +/- prefix, V -> '.').
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/accept_display2_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cat > "$TMP/p.cob" <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. AD2.
DATA DIVISION.
WORKING-STORAGE SECTION.
01 SN  PIC S9(3) VALUE -42.
01 SP  PIC S9(3) VALUE 42.
01 SZ  PIC S9(3) VALUE 0.
01 UV  PIC 9(3)V99 VALUE 12.34.
01 SNV PIC S9(3)V99 VALUE -12.34.
01 SPV PIC S9(3)V99 VALUE 12.34.
01 BIG PIC S9(5)V9(3) VALUE -123.456.
01 UZ  PIC 9(2)V9 VALUE 0.
PROCEDURE DIVISION.
    DISPLAY "sn=[" SN "]".
    DISPLAY "sp=[" SP "]".
    DISPLAY "sz=[" SZ "]".
    DISPLAY "uv=[" UV "]".
    DISPLAY "snv=[" SNV "]".
    DISPLAY "spv=[" SPV "]".
    DISPLAY "big=[" BIG "]".
    DISPLAY "uz=[" UZ "]".
    STOP RUN.
COB
cobc -free -x -o "$TMP/p" "$TMP/p.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }
"$TMP/p" | "$ROWS"
