#!/usr/bin/env bash
# SCREEN SECTION COLOURED-field DISPLAY sweep (GNURUST.SCREENIO.COLOR.1). A FOREGROUND-COLOR/
# BACKGROUND-COLOR clause makes ncurses allocate a non-default colour pair and repaint the whole
# touched region, so the observable byte stream is the curses wclear + top-down TransformLine
# sequence -- not a simple positioned write. For a grid of positions (line >= 2) and COBOL colour
# pairs, compile+run the coloured DISPLAY under a pty, capture the raw terminal bytes, and check
# the native emitter (screenio::color_display_and_stop) is BYTE-IDENTICAL, no ncurses linked.
# The default pair (fg=7,bg=0) is pair 0 -> no repaint (plain path) and is included as a control.
# The line==1 single-row-screen case is the declared non-claim and is NOT swept. PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8 TERM=xterm
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
command -v script >/dev/null 2>&1 || { echo "script(1) unavailable -- skipped"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
EMIT="$ROOT/target/release/examples/screenio_color"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cd "$TMP" || exit 2

PASS=0; FAIL=0
# (line col fg bg text) tuples -- line>=2; spans the space-fill (col<=5) and CUP+clr (col>=6)
# field-row positioning branches, the default-pair control, multi-char data, and several colours.
for spec in \
  "2 3 2 1 X" "3 1 2 1 X" "3 6 2 1 X" "5 10 2 1 X" \
  "4 10 4 6 HELLO" "6 2 1 5 QRS" "2 3 7 0 X" "7 12 3 2 AB" \
  "2 5 0 7 X" "4 1 5 3 X" "3 8 6 4 Z"; do
  set -- $spec; L=$1; C=$2; FG=$3; BG=$4; T=$5; N=${#T}
  if [ "$N" -gt 1 ]; then PIC="X($N)"; else PIC="X"; fi
  cat > p.cob <<COB
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. P.
DATA DIVISION.
SCREEN SECTION.
01 SC-REC.
   05 LINE $L COLUMN $C PIC $PIC VALUE "$T"
      FOREGROUND-COLOR $FG BACKGROUND-COLOR $BG.
PROCEDURE DIVISION.
    DISPLAY SC-REC.
    STOP RUN.
COB
  cobc -free -x -o p p.cob 2>/dev/null || { echo "($L,$C,$FG,$BG) compile-fail"; FAIL=$((FAIL+1)); continue; }
  script -qefc './p </dev/null' /dev/null > oracle.raw 2>/dev/null
  "$EMIT" "$L" "$C" "$T" "$FG" "$BG" > mine.raw
  if cmp -s oracle.raw mine.raw; then
    PASS=$((PASS+1))
  else
    FAIL=$((FAIL+1))
    echo "FAIL ($L,$C,$FG,$BG,$T):"
    echo "  oracle: $(sed 's/\x1b/<E>/g' oracle.raw | cat -v | sed 's/.*\[2J//;s/end of program.*//')"
    echo "  mine  : $(sed 's/\x1b/<E>/g' mine.raw   | cat -v | sed 's/.*\[2J//;s/end of program.*//')"
  fi
done
echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ] || exit 1
