#!/usr/bin/env bash
# SCREEN SECTION ACCEPT OVERFLOW sweep (GNURUST.SCREENIO.ACCEPT.2). Typing PAST the field width: with
# the cursor parked on the last cell, each excess key rings the bell `\a` and -- if it differs from the
# character currently shown in that cell -- overwrites it then backspaces (overwriting a cell with its
# own value is a no-op, bell only). For a grid of (width<=6, position, overflow input) compile+run the
# ACCEPT under a pty with the input fed then EOF, capture the raw terminal bytes, and check the native
# emitter (screenio::accept_field_and_stop) is BYTE-IDENTICAL, no ncurses linked. DIFFERENTIAL: every
# case is checked against BOTH GnuCOBOL 3.2 AND the 3.1.2 second oracle (when built). PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
P32="$ROOT/lab/oracle/prefix"; P312="$ROOT/lab/oracle/prefix-312"
command -v script >/dev/null 2>&1 || { echo "script(1) unavailable -- skipped"; exit 2; }
[ -x "$P32/bin/cobc" ] || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
EMIT="$ROOT/target/release/examples/screenio_accept"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cd "$TMP" || exit 2

run_oracle() {
  local prefix="$1" width="$2" col="$3" line="$4" typed="$5"
  cat > p.cob <<COB
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. P.
DATA DIVISION.
WORKING-STORAGE SECTION.
01 WS-N PIC X($width).
SCREEN SECTION.
01 SC-REC.
   05 LINE $line COLUMN $col PIC X($width) USING WS-N.
PROCEDURE DIVISION.
    ACCEPT SC-REC.
    STOP RUN.
COB
  PATH="$prefix/bin:$PATH" LD_LIBRARY_PATH="$prefix/lib" COB_CONFIG_DIR="$prefix/share/gnucobol/config" \
    cobc -x -free -o p p.cob 2>/dev/null || { echo "COMPILE-FAIL"; return 1; }
  printf '%s\r' "$typed" | LD_LIBRARY_PATH="$prefix/lib" TERM=xterm script -qefc './p' /dev/null > cap.raw 2>/dev/null
  python3 - <<PY
d=open('cap.raw','rb').read()
j=d.find(b'\x1b[2J')+4
t=d.find(b'\x1b[?1006;1000l', j)
import sys; sys.stdout.buffer.write(d[j:t] if t>=0 else d[j:])
PY
}

PASS=0; FAIL=0; DIFF=0
# (width col line input) -- input length > width (overflow); distinct + repeated + mixed chars
for spec in \
  "1 3 2 AB" "1 3 2 ABC" "1 3 2 ZZZ" "3 3 2 ABCD" "3 3 2 ABCDEF" "2 5 2 WXYZ" \
  "5 3 2 ABCDEFG" "4 10 4 ABCDEFGH" "6 20 2 ABCDEFGHI" "2 3 2 ZZZZ" "3 8 5 MNOPQR"; do
  set -- $spec; W=$1; C=$2; L=$3; T=$4
  o32="$(run_oracle "$P32" "$W" "$C" "$L" "$T")"
  mine="$("$EMIT" "$L" "$C" "$W" "$T" | python3 -c "import sys;d=sys.stdin.buffer.read();j=d.find(b'\x1b[2J')+4;t=d.find(b'\x1b[?1006;1000l',j);sys.stdout.buffer.write(d[j:t])")"
  if [ "$o32" = "$mine" ]; then
    PASS=$((PASS+1))
    if [ -x "$P312/bin/cobc" ]; then
      o312="$(run_oracle "$P312" "$W" "$C" "$L" "$T")"
      [ "$o312" = "$mine" ] && DIFF=$((DIFF+1)) || { echo "DIFF-FAIL 3.1.2 (W$W C$C L$L '$T')"; FAIL=$((FAIL+1)); }
    fi
  else
    FAIL=$((FAIL+1))
    echo "FAIL (W$W C$C L$L '$T'):"
    echo "  3.2 : $(printf '%s' "$o32" | sed 's/\x1b/<E>/g' | cat -v)"
    echo "  mine: $(printf '%s' "$mine" | sed 's/\x1b/<E>/g' | cat -v)"
  fi
done
echo "PASS=$PASS FAIL=$FAIL (3.1.2 differential-matched=$DIFF)"
[ "$FAIL" -eq 0 ] || exit 1
