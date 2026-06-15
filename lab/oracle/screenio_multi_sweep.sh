#!/usr/bin/env bash
# SCREEN SECTION MULTI-FIELD DISPLAY sweep (GNURUST.SCREENIO.DISPLAY.3). For each pair of field positions,
# compile+run `DISPLAY "AA" LINE l1 COLUMN c1 "BB" LINE l2 COLUMN c2.` then `STOP RUN.`, capture the raw
# terminal bytes under a pty, and check the native emitter (screenio::display_and_stop, exercising the
# general ncurses mvcur cost model `mvcur` for the inter-field move whose origin is NOT home) is
# BYTE-IDENTICAL -- no ncurses linked. Terminal = TERM=xterm + the admitted ncurses 6.6. PASS=n FAIL=n.
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
# A grid of (field1 pos, field2 pos) pairs exercising the inter-field mvcur move whose origin is not home:
# same-row forward, row-change up/down, near/far. Pairs where the second field is positioned LEFT of the
# first on the SAME row are excluded -- they leave the first field's cells stale and trigger ncurses's
# refresh line-diff (clr_eol erase), a separate mechanism sealed in a follow-on court (see DISPLAY.3
# non-claims).
PAIRS="
1,1,1,5  1,1,2,5  2,3,4,10  3,5,3,20  1,1,1,2  5,5,2,2  2,10,2,15  1,1,3,1
2,5,2,9  2,5,2,12  2,5,4,9  2,5,1,8  3,3,3,3  2,2,5,7
1,3,1,9  3,10,3,11  5,1,5,40  1,40,2,1  2,7,2,7  3,4,5,9  1,1,10,40
"
for q in $PAIRS; do
  IFS=, read L1 C1 L2 C2 <<< "$q"
  cat > p.cob <<COB
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. P.
PROCEDURE DIVISION.
    DISPLAY "AA" LINE $L1 COLUMN $C1 "BB" LINE $L2 COLUMN $C2.
    STOP RUN.
COB
  cobc -free -x -o p p.cob 2>/dev/null || { echo "($L1,$C1)->($L2,$C2) compile-fail"; FAIL=$((FAIL+1)); continue; }
  script -qefc './p </dev/null' /dev/null > oracle.raw 2>/dev/null
  "$EMIT" "$L1" "$C1" AA "$L2" "$C2" BB > mine.raw
  if cmp -s oracle.raw mine.raw; then
    PASS=$((PASS+1))
  else
    FAIL=$((FAIL+1))
    echo "FAIL ($L1,$C1)->($L2,$C2):"
    echo "  oracle: $(sed 's/\x1b/<E>/g' oracle.raw | cat -v | sed 's/.*\[2J//;s/end of program.*//')"
    echo "  mine  : $(sed 's/\x1b/<E>/g' mine.raw   | cat -v | sed 's/.*\[2J//;s/end of program.*//')"
  fi
done
echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ] || exit 1
