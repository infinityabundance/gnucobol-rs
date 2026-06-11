#!/usr/bin/env bash
# Procedure-flow atlas sweep (GNURUST.PROCEDURE.FLOW.ATLAS.1). Probe the control-flow statement classes (IF,
# EVALUATE, PERFORM TIMES/VARYING/UNTIL/paragraph, GO TO) against real cobc/libcob, assert the observed
# behavior is stable, and emit reports/procedure-flow-atlas.json. OBSERVED court: gnucobol-rs does NOT execute
# Procedure Division -- this MAPS the control-flow surface; execution is the loudest non-claim.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cat > "$TMP/p.cob" <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. P.
DATA DIVISION.
WORKING-STORAGE SECTION.
01 C PIC 9(3).
01 I PIC 9(3).
01 X PIC 9.
PROCEDURE DIVISION.
    IF 5 > 3 THEN DISPLAY "if=THEN" ELSE DISPLAY "if=ELSE" END-IF.
    MOVE 2 TO X.
    EVALUATE X WHEN 1 DISPLAY "eval=1" WHEN 2 DISPLAY "eval=2" WHEN OTHER DISPLAY "eval=O" END-EVALUATE.
    MOVE 0 TO C. PERFORM 3 TIMES ADD 1 TO C END-PERFORM. DISPLAY "perform_times=" C.
    MOVE 0 TO C. PERFORM VARYING I FROM 1 BY 1 UNTIL I > 4 ADD 1 TO C END-PERFORM.
    DISPLAY "varying_body=" C. DISPLAY "varying_ends=" I.
    MOVE 0 TO C. PERFORM UNTIL C >= 5 ADD 1 TO C END-PERFORM. DISPLAY "until=" C.
    MOVE 0 TO C. PERFORM SUB-PARA. DISPLAY "perform_para=" C.
    GO TO SKIP-IT.
    MOVE 99 TO C.
    SKIP-IT.
    DISPLAY "goto_skipped=" C.
    STOP RUN.
    SUB-PARA.
    ADD 7 TO C.
COB
cobc -free -x -o "$TMP/p" "$TMP/p.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }
OUT=$("$TMP/p")
( cd "$ROOT" && OUT="$OUT" cargo run -q -p xtask -- atlas-procedure-flow )
