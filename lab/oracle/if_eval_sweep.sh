#!/usr/bin/env bash
# IF/EVALUATE execution-slice sweep (GNURUST.IF.EVALUATE.SLICE.1). Run IF/EVALUATE fragments and check the
# resulting storage (T) == eval_if/eval_evaluate.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/if_eval_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cat > "$TMP/p.cob" <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. IFE.
DATA DIVISION.
WORKING-STORAGE SECTION.
01 A PIC X(3).
01 T PIC X(4).
PROCEDURE DIVISION.
    MOVE "BBB" TO A. MOVE "----" TO T. IF A = "BBB" MOVE "YES" TO T ELSE MOVE "NO" TO T END-IF. DISPLAY "if_eq=[" T "]".
    MOVE "BBB" TO A. MOVE "----" TO T. IF A > "AAA" MOVE "GT" TO T ELSE MOVE "LE" TO T END-IF. DISPLAY "if_gt=[" T "]".
    MOVE "BBB" TO A. MOVE "----" TO T. IF A < "AAA" MOVE "Y" TO T ELSE MOVE A TO T END-IF. DISPLAY "if_lt_else=[" T "]".
    MOVE "BBB" TO A. MOVE "----" TO T. IF A NOT = "BBB" MOVE "NE" TO T ELSE MOVE "EQ" TO T END-IF. DISPLAY "if_ne=[" T "]".
    MOVE "BBB" TO A. MOVE "----" TO T. IF A >= "BBB" MOVE "GE" TO T ELSE MOVE "X" TO T END-IF. DISPLAY "if_ge=[" T "]".
    MOVE "BBB" TO A. MOVE "----" TO T. IF A <= "AAA" MOVE "LE" TO T ELSE MOVE "GT" TO T END-IF. DISPLAY "if_le=[" T "]".
    MOVE "B" TO A. MOVE "----" TO T. EVALUATE A WHEN "A" MOVE "AAA" TO T WHEN "B" MOVE "BEE" TO T WHEN OTHER MOVE "OTH" TO T END-EVALUATE. DISPLAY "eval_B=[" T "]".
    MOVE "Z" TO A. MOVE "----" TO T. EVALUATE A WHEN "A" MOVE "AAA" TO T WHEN "B" MOVE "BEE" TO T WHEN OTHER MOVE "OTH" TO T END-EVALUATE. DISPLAY "eval_Z=[" T "]".
    MOVE "A" TO A. MOVE "----" TO T. EVALUATE A WHEN "A" MOVE "AAA" TO T WHEN "B" MOVE "BEE" TO T WHEN OTHER MOVE "OTH" TO T END-EVALUATE. DISPLAY "eval_A=[" T "]".
    STOP RUN.
COB
cobc -free -x -o "$TMP/p" "$TMP/p.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }
"$TMP/p" | "$ROWS"
