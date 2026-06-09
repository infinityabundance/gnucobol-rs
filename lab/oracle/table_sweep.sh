#!/usr/bin/env bash
# Table (OCCURS) PERFORM VARYING slice sweep (GNURUST.TABLE.PERFORM.SLICE.1).
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/table_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cat > "$TMP/p.cob" <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. TBL.
DATA DIVISION.
WORKING-STORAGE SECTION.
01 T-REC.
   05 T-ELEM PIC 9(3) OCCURS 5.
01 SM  PIC 9(5).
01 CNT PIC 9(2).
01 I   PIC 9(2).
PROCEDURE DIVISION.
    MOVE 100 TO T-ELEM(1). MOVE 025 TO T-ELEM(2). MOVE 050 TO T-ELEM(3).
    MOVE 200 TO T-ELEM(4). MOVE 007 TO T-ELEM(5).
    DISPLAY "table=[" T-REC "]".
    MOVE 0 TO SM. PERFORM VARYING I FROM 1 BY 1 UNTIL I > 5 ADD T-ELEM(I) TO SM END-PERFORM. DISPLAY "sum=" SM.
    MOVE 0 TO CNT. PERFORM VARYING I FROM 1 BY 1 UNTIL I > 5 IF T-ELEM(I) >= 50 ADD 1 TO CNT END-IF END-PERFORM. DISPLAY "ge50_count=" CNT.
    MOVE 0 TO SM. PERFORM VARYING I FROM 1 BY 2 UNTIL I > 5 ADD T-ELEM(I) TO SM END-PERFORM. DISPLAY "sum_by2=" SM.
    STOP RUN.
COB
cobc -free -x -o "$TMP/p" "$TMP/p.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }
"$TMP/p" | "$ROWS"
