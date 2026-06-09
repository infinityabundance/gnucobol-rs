#!/usr/bin/env bash
# Read-loop execution-slice sweep (GNURUST.FILE.FLOW.SLICE.1). The oracle builds an input file and runs the
# canonical OPEN/PERFORM-UNTIL-EOF/READ/accumulate loop; the mirror recomputes COUNT/SUM over the same bytes.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/file_flow_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cat > "$TMP/p.cob" <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. RL.
ENVIRONMENT DIVISION.
INPUT-OUTPUT SECTION.
FILE-CONTROL.
    SELECT F ASSIGN "in.dat" ORGANIZATION IS RECORD SEQUENTIAL.
DATA DIVISION.
FILE SECTION.
FD F.
01 R.
   05 R-ID  PIC X(2).
   05 R-AMT PIC 9(3).
WORKING-STORAGE SECTION.
01 CNT PIC 9(3).
01 SM  PIC 9(5).
01 EOFSW PIC X VALUE "N".
PROCEDURE DIVISION.
    OPEN OUTPUT F.
    MOVE "AA" TO R-ID. MOVE 100 TO R-AMT. WRITE R.
    MOVE "BB" TO R-ID. MOVE 25 TO R-AMT. WRITE R.
    MOVE "CC" TO R-ID. MOVE 50 TO R-AMT. WRITE R.
    MOVE "DD" TO R-ID. MOVE 200 TO R-AMT. WRITE R.
    MOVE "EE" TO R-ID. MOVE 0 TO R-AMT. WRITE R.
    MOVE "FF" TO R-ID. MOVE 7 TO R-AMT. WRITE R.
    CLOSE F.
    MOVE 0 TO CNT. MOVE 0 TO SM. MOVE "N" TO EOFSW.
    OPEN INPUT F.
    READ F AT END MOVE "Y" TO EOFSW END-READ.
    PERFORM UNTIL EOFSW = "Y"
        ADD 1 TO CNT
        ADD R-AMT TO SM
        READ F AT END MOVE "Y" TO EOFSW END-READ
    END-PERFORM.
    CLOSE F.
    DISPLAY "count=" CNT.
    DISPLAY "sum=" SM.
    STOP RUN.
COB
( cd "$TMP" && cobc -free -x -o p p.cob 2>err && ./p > out.txt ) || { echo "compile/run failed"; cat "$TMP/err"; exit 2; }
{ printf 'file=%s\n' "$(xxd -p "$TMP/in.dat" | tr -d '\n')"; cat "$TMP/out.txt"; } | "$ROWS"
