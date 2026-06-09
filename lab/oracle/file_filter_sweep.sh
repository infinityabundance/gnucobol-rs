#!/usr/bin/env bash
# Filter read-loop sweep (GNURUST.FILE.FILTER.SLICE.1). The oracle runs two conditional read-loops (numeric
# R-AMT>=50 and alphanumeric R-ST="A"); the mirror recomputes via eval_filter_loop over the same file bytes.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/file_filter_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cat > "$TMP/p.cob" <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. FLT.
ENVIRONMENT DIVISION.
INPUT-OUTPUT SECTION.
FILE-CONTROL.
    SELECT F ASSIGN "in.dat" ORGANIZATION IS RECORD SEQUENTIAL.
DATA DIVISION.
FILE SECTION.
FD F.
01 R.
   05 R-ST  PIC X(1).
   05 R-AMT PIC 9(3).
WORKING-STORAGE SECTION.
01 CNT PIC 9(3).
01 SM  PIC 9(5).
01 EOFSW PIC X VALUE "N".
PROCEDURE DIVISION.
    OPEN OUTPUT F.
    MOVE "A" TO R-ST. MOVE 100 TO R-AMT. WRITE R.
    MOVE "B" TO R-ST. MOVE 025 TO R-AMT. WRITE R.
    MOVE "A" TO R-ST. MOVE 050 TO R-AMT. WRITE R.
    MOVE "A" TO R-ST. MOVE 200 TO R-AMT. WRITE R.
    MOVE "B" TO R-ST. MOVE 007 TO R-AMT. WRITE R.
    CLOSE F.
    MOVE 0 TO CNT. MOVE 0 TO SM. MOVE "N" TO EOFSW.
    OPEN INPUT F. READ F AT END MOVE "Y" TO EOFSW END-READ.
    PERFORM UNTIL EOFSW = "Y"
        IF R-AMT >= 50 ADD 1 TO CNT ADD R-AMT TO SM END-IF
        READ F AT END MOVE "Y" TO EOFSW END-READ
    END-PERFORM.
    CLOSE F. DISPLAY "amt_ge50_count=" CNT. DISPLAY "amt_ge50_sum=" SM.
    MOVE 0 TO CNT. MOVE 0 TO SM. MOVE "N" TO EOFSW.
    OPEN INPUT F. READ F AT END MOVE "Y" TO EOFSW END-READ.
    PERFORM UNTIL EOFSW = "Y"
        IF R-ST = "A" ADD 1 TO CNT ADD R-AMT TO SM END-IF
        READ F AT END MOVE "Y" TO EOFSW END-READ
    END-PERFORM.
    CLOSE F. DISPLAY "st_A_count=" CNT. DISPLAY "st_A_sum=" SM.
    STOP RUN.
COB
( cd "$TMP" && cobc -free -x -o p p.cob 2>err && ./p > out.txt ) || { echo "compile/run failed"; cat "$TMP/err"; exit 2; }
{ printf 'file=%s\n' "$(xxd -p "$TMP/in.dat" | tr -d '\n')"; cat "$TMP/out.txt"; } | "$ROWS"
