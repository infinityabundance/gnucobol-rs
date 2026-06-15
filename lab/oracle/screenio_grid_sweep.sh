#!/usr/bin/env bash
# SCREEN SECTION positioned-DISPLAY grid sweep (GNURUST.SCREENIO.DISPLAY.2). For each (LINE,COLUMN) in a grid,
# compile+run `DISPLAY "Z" LINE l COLUMN c.` then `STOP RUN.`, capture the raw terminal bytes under a pty,
# and check the native pure-Rust emitter (screenio::display_and_stop, exercising the ncurses mvcur cost model
# move_cursor_from_home) is BYTE-IDENTICAL -- proving the cursor-movement reproduction across positions, no
# ncurses linked. Terminal = TERM=xterm + the admitted ncurses 6.6 (the declared non-claim is other
# terminals). Prints PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8 TERM=xterm
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
command -v script >/dev/null 2>&1 || { echo "script(1) unavailable -- skipped"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
EMIT="$ROOT/target/release/examples/screenio_emit"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cd "$TMP" || exit 2

PASS=0; FAIL=0
LINES="1 2 3 5 10"
COLS="1 2 3 4 5 6 7 8 9 10 11 15 20 40"
for L in $LINES; do
  for C in $COLS; do
    cat > p.cob <<COB
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. P.
PROCEDURE DIVISION.
    DISPLAY "Z" LINE $L COLUMN $C.
    STOP RUN.
COB
    cobc -free -x -o p p.cob 2>/dev/null || { echo "($L,$C) compile-fail"; FAIL=$((FAIL+1)); continue; }
    script -qefc './p </dev/null' /dev/null > oracle.raw 2>/dev/null
    "$EMIT" "$L" "$C" Z > mine.raw
    if cmp -s oracle.raw mine.raw; then
      PASS=$((PASS+1))
    else
      FAIL=$((FAIL+1))
      echo "FAIL ($L,$C):"
      echo "  oracle: $(sed 's/\x1b/<E>/g' oracle.raw | cat -v | sed 's/.*\[2J//')"
      echo "  mine  : $(sed 's/\x1b/<E>/g' mine.raw   | cat -v | sed 's/.*\[2J//')"
    fi
  done
done
echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ] || exit 1
