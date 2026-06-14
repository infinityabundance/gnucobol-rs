#!/usr/bin/env bash
# SCREEN SECTION DISPLAY framing sweep (GNURUST.SCREENIO.INIT.1). GnuCOBOL drives ncurses for SCREEN SECTION
# I/O; the terminal byte stream it writes is deterministic for a fixed terminal (TERM=xterm, the admitted
# ncurses 6.6). This sweep compiles+runs the canonical screen program under a pty (script), captures the raw
# bytes, and checks the native pure-Rust reproduction (screenio::display_and_stop) is BYTE-IDENTICAL -- no
# ncurses linked. The terminal-dependence (xterm + this ncurses) is the declared non-claim. PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8 TERM=xterm
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
command -v script >/dev/null 2>&1 || { echo "util-linux script(1) not available -- screenio pty capture skipped"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/screenio_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cd "$TMP" || exit 2

cat > sc.cob <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. SC.
PROCEDURE DIVISION.
    DISPLAY "X" LINE 2 COLUMN 3.
    STOP RUN.
COB
cobc -free -x -o sc sc.cob 2>e || { echo "compile failed"; cat e; exit 2; }
# Determinism guard: the capture must be stable across runs.
script -qefc './sc </dev/null' /dev/null > cap1.raw 2>/dev/null
script -qefc './sc </dev/null' /dev/null > cap2.raw 2>/dev/null
if ! cmp -s cap1.raw cap2.raw; then echo "non-deterministic capture"; echo "PASS=0 FAIL=1"; exit 1; fi
"$ROWS" < cap1.raw
