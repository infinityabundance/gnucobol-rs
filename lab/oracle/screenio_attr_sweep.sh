#!/usr/bin/env bash
# SCREEN SECTION monochrome-attribute DISPLAY sweep (GNURUST.SCREENIO.ATTR.1). For each display attribute
# (HIGHLIGHT/LOWLIGHT/UNDERLINE/BLINK/REVERSE-VIDEO) at a couple of positions, compile+run the attributed
# DISPLAY under a pty, capture the raw terminal bytes, and check the native emitter (screenio::display_and_stop
# with ScreenAttr -- the SGR on/off wrapping) is BYTE-IDENTICAL, no ncurses linked. The terminfo-dependent
# colour-attribute repaint path is NOT covered here (see ATTR.1 non-claims). PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8 TERM=xterm
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
command -v script >/dev/null 2>&1 || { echo "script(1) unavailable -- skipped"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
EMIT="$ROOT/target/release/examples/screenio_attr"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cd "$TMP" || exit 2

# attr-name -> COBOL clause
declare -A CLAUSE=( [highlight]=HIGHLIGHT [lowlight]=LOWLIGHT [underline]=UNDERLINE [blink]=BLINK [reverse]=REVERSE-VIDEO )
PASS=0; FAIL=0
for pos in "2 3" "5 10"; do
  set -- $pos; L=$1; C=$2
  for a in highlight lowlight underline blink reverse; do
    cat > p.cob <<COB
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. P.
PROCEDURE DIVISION.
    DISPLAY "X" LINE $L COLUMN $C ${CLAUSE[$a]}.
    STOP RUN.
COB
    cobc -free -x -o p p.cob 2>/dev/null || { echo "$a ($L,$C) compile-fail"; FAIL=$((FAIL+1)); continue; }
    script -qefc './p </dev/null' /dev/null > oracle.raw 2>/dev/null
    "$EMIT" "$L" "$C" X "$a" > mine.raw
    if cmp -s oracle.raw mine.raw; then
      PASS=$((PASS+1))
    else
      FAIL=$((FAIL+1))
      echo "FAIL $a ($L,$C):"
      echo "  oracle: $(sed 's/\x1b/<E>/g' oracle.raw | cat -v | sed 's/.*\[2J//;s/end of program.*//')"
      echo "  mine  : $(sed 's/\x1b/<E>/g' mine.raw   | cat -v | sed 's/.*\[2J//;s/end of program.*//')"
    fi
  done
done
echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ] || exit 1
