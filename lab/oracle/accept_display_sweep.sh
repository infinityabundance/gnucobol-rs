#!/usr/bin/env bash
# DISPLAY/ACCEPT byte-effect sweep (GNURUST.ACCEPT.DISPLAY.1). DISPLAY literal/alnum/unsigned-numeric operands
# (emitted text) and ACCEPT a piped line into a field (pad/truncate), and check display_line/accept_field ==
# the oracle's labeled output.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/accept_display_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cat > "$TMP/p.cob" <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. AD.
DATA DIVISION.
WORKING-STORAGE SECTION.
01 AN PIC X(5) VALUE "HEL".
01 NU PIC 9(3) VALUE 42.
01 AC PIC X(6).
PROCEDURE DIVISION.
    DISPLAY "d_lit=" "ABC".
    DISPLAY "d_alnum=" AN.
    DISPLAY "d_unum=" NU.
    DISPLAY "d_multi=" "X" "Y" "Z".
    DISPLAY "d_cat=" AN NU.
    ACCEPT AC FROM CONSOLE. DISPLAY "a_short=" AC.
    ACCEPT AC FROM CONSOLE. DISPLAY "a_long=" AC.
    STOP RUN.
COB
cobc -free -x -o "$TMP/p" "$TMP/p.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }
printf 'HI\nABCDEFGH\n' | "$TMP/p" | "$ROWS"
