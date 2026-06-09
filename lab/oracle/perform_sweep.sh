#!/usr/bin/env bash
# PERFORM execution-slice sweep (GNURUST.PERFORM.SLICE.1). Run PERFORM TIMES/UNTIL/VARYING fragments and check
# the resulting counter storage == eval_perform.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/perform_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cat > "$TMP/p.cob" <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. PFM.
DATA DIVISION.
WORKING-STORAGE SECTION.
01 C PIC 9(3).
01 I PIC 9(3).
PROCEDURE DIVISION.
    MOVE 0 TO C. PERFORM 3 TIMES ADD 1 TO C END-PERFORM. DISPLAY "times3=" C.
    MOVE 0 TO C. PERFORM 0 TIMES ADD 1 TO C END-PERFORM. DISPLAY "times0=" C.
    MOVE 0 TO C. PERFORM UNTIL C >= 5 ADD 1 TO C END-PERFORM. DISPLAY "until5=" C.
    MOVE 7 TO C. PERFORM UNTIL C >= 5 ADD 1 TO C END-PERFORM. DISPLAY "until_already=" C.
    MOVE 0 TO C. PERFORM VARYING I FROM 1 BY 1 UNTIL I > 4 ADD 1 TO C END-PERFORM. DISPLAY "vary_body_c=" C. DISPLAY "vary_body_i=" I.
    MOVE 0 TO C. PERFORM VARYING I FROM 2 BY 3 UNTIL I > 10 ADD 1 TO C END-PERFORM. DISPLAY "vary_by3_c=" C. DISPLAY "vary_by3_i=" I.
    MOVE 0 TO C. PERFORM VARYING I FROM 5 BY 1 UNTIL I > 2 ADD 1 TO C END-PERFORM. DISPLAY "vary_none_c=" C. DISPLAY "vary_none_i=" I.
    STOP RUN.
COB
cobc -free -x -o "$TMP/p" "$TMP/p.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }
"$TMP/p" | "$ROWS"
