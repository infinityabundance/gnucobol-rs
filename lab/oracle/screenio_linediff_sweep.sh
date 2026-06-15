#!/usr/bin/env bash
# SCREEN multi-DISPLAY same-row LINE-DIFF sweep (GNURUST.SCREENIO.LINEDIFF.1). Two DISPLAY statements
# to the same row: the second is an ncurses doupdate/TransformLine refresh of the first -- a virtual
# screen diff that repositions, writes the changed run, and erases the now-blank tail (clr_eol). For a
# grid of (first col/len, second col across the range, second text) compile+run under a pty and check
# the native emitter (screenio::two_display_line_and_stop) is BYTE-IDENTICAL, no ncurses linked.
# DIFFERENTIAL: every case checked against BOTH GnuCOBOL 3.2 AND the 3.1.2 second oracle (when built).
# Envelope: exactly TWO same-row DISPLAYs (3+ displays, distant gaps, and multi-row are follow-ons).
# PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
P32="$ROOT/lab/oracle/prefix"; P312="$ROOT/lab/oracle/prefix-312"
command -v script >/dev/null 2>&1 || { echo "script(1) unavailable -- skipped"; exit 2; }
[ -x "$P32/bin/cobc" ] || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
EMIT="$ROOT/target/release/examples/screenio_linediff"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cd "$TMP" || exit 2

run_oracle() {
  local prefix="$1" r="$2" c1="$3" d1="$4" c2="$5" d2="$6"
  cat > p.cob <<COB
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. P.
PROCEDURE DIVISION.
    DISPLAY "$d1" LINE $r COLUMN $c1.
    DISPLAY "$d2" LINE $r COLUMN $c2.
    STOP RUN.
COB
  PATH="$prefix/bin:$PATH" LD_LIBRARY_PATH="$prefix/lib" COB_CONFIG_DIR="$prefix/share/gnucobol/config" \
    cobc -x -free -o p p.cob 2>/dev/null || { echo "COMPILE-FAIL"; return 1; }
  LD_LIBRARY_PATH="$prefix/lib" TERM=xterm script -qefc './p </dev/null' /dev/null > cap.raw 2>/dev/null
  python3 - <<PY
d=open('cap.raw','rb').read()
j=d.find(b'\x1b[2J')+4
k=d.find(b'end of program', j)
import sys; sys.stdout.buffer.write(d[j:k] if k>=0 else d[j:])
PY
}
mine_body() { "$EMIT" "$@" | python3 -c "import sys;d=sys.stdin.buffer.read();j=d.find(b'\x1b[2J')+4;k=d.find(b'end of program',j);sys.stdout.buffer.write(d[j:k])"; }

PASS=0; FAIL=0; DIFF=0
for spec in \
  "2 3 ABCDEFGH 3 XY" "2 3 ABCDEFGH 4 WXYZ" "2 3 ABCDEFGH 5 XY" "2 3 ABCDEFGH 6 XY" \
  "2 3 ABCDEFGH 9 XY" "2 3 ABCDEFGH 11 XY" "2 3 ABCDEFGH 13 XY" "2 3 ABCDEFGH 16 YY" \
  "2 10 ABCDEFGH 10 YY" "2 10 ABCDEFGH 11 YY" "2 10 ABCDEFGH 13 YY" "2 20 ABCDEFGH 20 YY" \
  "2 5 MNOPQR 5 YY" "2 5 MNOPQR 7 YY" "2 3 ABC 3 YY" "2 3 ABCDEFGH 3 XYZWVUTSRQ" \
  "5 4 ABCDEFGHI 4 YY" "3 7 ABCDE 7 P" "2 2 ABCDEFGHIJ 4 WXYZ"; do
  set -- $spec; R=$1; C1=$2; D1=$3; C2=$4; D2=$5
  o32="$(run_oracle "$P32" "$R" "$C1" "$D1" "$C2" "$D2")"
  mine="$(mine_body "$R" "$C1" "$D1" "$C2" "$D2")"
  if [ "$o32" = "$mine" ]; then
    PASS=$((PASS+1))
    if [ -x "$P312/bin/cobc" ]; then
      o312="$(run_oracle "$P312" "$R" "$C1" "$D1" "$C2" "$D2")"
      [ "$o312" = "$mine" ] && DIFF=$((DIFF+1)) || { echo "DIFF-FAIL 3.1.2 ($spec)"; FAIL=$((FAIL+1)); }
    fi
  else
    FAIL=$((FAIL+1))
    echo "FAIL ($spec):"
    echo "  3.2 : $(printf '%s' "$o32" | sed 's/\x1b/<E>/g' | cat -v)"
    echo "  mine: $(printf '%s' "$mine" | sed 's/\x1b/<E>/g' | cat -v)"
  fi
done
echo "PASS=$PASS FAIL=$FAIL (3.1.2 differential-matched=$DIFF)"
[ "$FAIL" -eq 0 ] || exit 1
