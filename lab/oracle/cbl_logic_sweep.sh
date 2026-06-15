#!/usr/bin/env bash
# CBL_ logic/bit builtins sweep (GNURUST.COMMON.CBL.1). A COBOL program CALLs each CBL_* routine
# (CBL_AND/OR/XOR/NOR/IMP/NIMP/EQ/NOT/TOUPPER/TOLOWER) on fixed inputs and DISPLAYs the 1-byte result after
# an "op:" marker; the native common.c reproduction (cob_sys_*) must produce the same bytes. DIFFERENTIAL:
# required to match BOTH oracles 3.1.2 + 3.2 when the second is built. PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PRIMARY="$ROOT/lab/oracle/prefix"; SECOND="$ROOT/lab/oracle/prefix-312"
[ -x "$PRIMARY/bin/cobc" ] || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/cbl_logic_rows"

SRC='>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. P.
DATA DIVISION.
WORKING-STORAGE SECTION.
01 SRC PIC X VALUE X"CC".
01 BAND PIC X VALUE X"AA".
01 BOR  PIC X VALUE X"AA".
01 BXOR PIC X VALUE X"AA".
01 BNOR PIC X VALUE X"AA".
01 BIMP PIC X VALUE X"AA".
01 BNIM PIC X VALUE X"AA".
01 BEQ  PIC X VALUE X"AA".
01 BNOT PIC X VALUE X"AA".
01 LUP  PIC X VALUE "a".
01 LLO  PIC X VALUE "Z".
PROCEDURE DIVISION.
    CALL "CBL_AND"     USING SRC BAND BY VALUE 1 END-CALL
    DISPLAY "and:" BAND
    CALL "CBL_OR"      USING SRC BOR  BY VALUE 1 END-CALL
    DISPLAY "or:" BOR
    CALL "CBL_XOR"     USING SRC BXOR BY VALUE 1 END-CALL
    DISPLAY "xor:" BXOR
    CALL "CBL_NOR"     USING SRC BNOR BY VALUE 1 END-CALL
    DISPLAY "nor:" BNOR
    CALL "CBL_IMP"     USING SRC BIMP BY VALUE 1 END-CALL
    DISPLAY "imp:" BIMP
    CALL "CBL_NIMP"    USING SRC BNIM BY VALUE 1 END-CALL
    DISPLAY "nimp:" BNIM
    CALL "CBL_EQ"      USING SRC BEQ  BY VALUE 1 END-CALL
    DISPLAY "eq:" BEQ
    CALL "CBL_NOT"     USING BNOT BY VALUE 1 END-CALL
    DISPLAY "not:" BNOT
    CALL "CBL_TOUPPER" USING LUP BY VALUE 1 END-CALL
    DISPLAY "upper:" LUP
    CALL "CBL_TOLOWER" USING LLO BY VALUE 1 END-CALL
    DISPLAY "lower:" LLO
    STOP RUN.'

run_one() {  # prefix
  local pfx="$1" d; d="$(mktemp -d)"
  ( cd "$d" && printf '%s' "$SRC" > p.cob \
    && PATH="$pfx/bin:$PATH" LD_LIBRARY_PATH="$pfx/lib" COB_CONFIG_DIR="$pfx/share/gnucobol/config" \
       COB_COPY_DIR="$pfx/share/gnucobol/copy" LC_ALL=C.UTF-8 "$pfx/bin/cobc" -free -x -o p p.cob 2>ce ) || { echo "COMPILE-FAIL"; cat "$d/ce"; rm -rf "$d"; return 1; }
  ( cd "$d" && PATH="$pfx/bin:$PATH" LD_LIBRARY_PATH="$pfx/lib" COB_CONFIG_DIR="$pfx/share/gnucobol/config" ./p 2>/dev/null )
  rm -rf "$d"
}

PASS=0; FAIL=0
out="$(run_one "$PRIMARY")" || { echo "$out"; echo "PASS=0 FAIL=1"; exit 1; }
echo "$out" | "$ROWS" >/dev/null && PASS=$((PASS+1)) || { FAIL=$((FAIL+1)); echo "PRIMARY mismatch"; echo "$out" | "$ROWS"; }
if [ -x "$SECOND/bin/cobc" ]; then
  out2="$(run_one "$SECOND")"
  echo "$out2" | "$ROWS" >/dev/null && PASS=$((PASS+1)) || { FAIL=$((FAIL+1)); echo "SECOND(3.1.2) mismatch"; }
fi
echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ] || exit 1
