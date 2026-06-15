#!/usr/bin/env bash
# SCREEN SECTION NUMERIC-EDITED DISPLAY sweep (GNURUST.SCREENIO.NUMEDIT.1). A field with an edited PIC
# (zero-suppression Z, floating/fixed sign, CR/DB, comma/decimal insertion) FROM a numeric source
# displays the move/edit engine's edited image -- right-aligned, so it carries leading blanks (and a
# short trailing-blank run for a positive CR/DB or trailing sign). For a grid of (pic, value, position)
# compile+run the DISPLAY under a pty, capture the raw terminal bytes, and check the native composition
# (edited::encode_edited -> screenio::display_edited_and_stop) is BYTE-IDENTICAL, no ncurses linked.
# This proves the screen POSITIONING of an edited field (leading-blank skip, all-blank field, trailing
# space-fill); the numeric editing itself is the move.c court. PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8 TERM=xterm
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
command -v script >/dev/null 2>&1 || { echo "script(1) unavailable -- skipped"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
EMIT="$ROOT/target/release/examples/screenio_numedit"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cd "$TMP" || exit 2

PASS=0; FAIL=0
# (line col pic value) -- the source is PIC S9(5)V99 holding <value>; the screen field re-edits it.
# Picks span: Z-suppression (large/small/tiny), all-blank zero, fixed/floating sign, CR/DB pos+neg,
# a trailing fixed sign, and a far position.
SPECS=(
  "2 3 ZZ,ZZ9.99 1234.56"
  "2 3 ZZ,ZZ9.99 7.00"
  "2 3 ZZZZ.ZZ 0.00"
  "2 3 ZZZZ.ZZ 0.07"
  "2 3 9(4).99CR 12.30"
  "2 3 9(4).99CR -12.30"
  "2 3 9(4).99DB -5.00"
  "2 3 -9(5).99 12.30"
  "2 3 -9(5).99 -12.30"
  "2 3 +9(4).99 12.30"
  "3 6 ZZ,ZZ9.99- -88.10"
  "5 10 ZZ,ZZ9.99 1234.56"
  "4 1 ZZ,ZZ9.99 12.34"
  "6 12 ZZZ9.99 0.05"
)
for spec in "${SPECS[@]}"; do
  set -- $spec; L=$1; C=$2; PIC=$3; V=$4
  cat > p.cob <<COB
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. P.
DATA DIVISION.
WORKING-STORAGE SECTION.
01 WS-N PIC S9(5)V99 VALUE $V.
SCREEN SECTION.
01 SC-REC.
   05 LINE $L COLUMN $C PIC $PIC FROM WS-N.
PROCEDURE DIVISION.
    DISPLAY SC-REC.
    STOP RUN.
COB
  cobc -free -x -o p p.cob 2>/dev/null || { echo "($L,$C,$PIC,$V) compile-fail"; FAIL=$((FAIL+1)); continue; }
  script -qefc './p </dev/null' /dev/null > oracle.raw 2>/dev/null
  "$EMIT" "$L" "$C" "$PIC" "$V" > mine.raw
  if cmp -s oracle.raw mine.raw; then
    PASS=$((PASS+1))
  else
    FAIL=$((FAIL+1))
    echo "FAIL ($L,$C,$PIC,$V):"
    echo "  oracle: $(sed 's/\x1b/<E>/g' oracle.raw | cat -v | sed 's/.*\[2J//;s/end of program.*//')"
    echo "  mine  : $(sed 's/\x1b/<E>/g' mine.raw   | cat -v | sed 's/.*\[2J//;s/end of program.*//')"
  fi
done
echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ] || exit 1
