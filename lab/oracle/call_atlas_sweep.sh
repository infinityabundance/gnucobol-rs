#!/usr/bin/env bash
# GNURUST.CALL.EXTENSION.ATLAS.1 -- observe CALL / linkage / C$ extension behavior against real cobc/libcob.
# The #1 gap by frequency in the admitted testsuite (CALL 959x). OBSERVED court: gnucobol-rs does NOT execute
# subprogram CALLs (multi-module runtime = behavioral-ladder L8, NOT CLAIMED). This MAPS the surface: the
# byte-effect of parameter passing (BY REFERENCE shares storage, BY CONTENT copies), C$ extensions, ON
# EXCEPTION, CANCEL, RETURN-CODE.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cat > "$TMP/p.cob" <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. MAINP.
DATA DIVISION.
WORKING-STORAGE SECTION.
01 A PIC 9(3) VALUE 100.
01 B PIC 9(3) VALUE 100.
01 TXT PIC X(5) VALUE "abcde".
PROCEDURE DIVISION.
    CALL "SUBP" USING BY REFERENCE A. DISPLAY "ref_A=" A.
    CALL "SUBP" USING BY CONTENT B. DISPLAY "content_B=" B.
    CALL "C$TOUPPER" USING TXT, BY VALUE 5. DISPLAY "toupper=[" TXT "]".
    CALL "NOSUCH" ON EXCEPTION DISPLAY "exception=caught" END-CALL.
    CANCEL "SUBP". DISPLAY "cancel=ok".
    STOP RUN.
END PROGRAM MAINP.
IDENTIFICATION DIVISION.
PROGRAM-ID. SUBP.
DATA DIVISION.
LINKAGE SECTION.
01 P PIC 9(3).
PROCEDURE DIVISION USING P.
    ADD 1 TO P. EXIT PROGRAM.
END PROGRAM SUBP.
COB
cobc -free -x -o "$TMP/p" "$TMP/p.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }
OUT=$("$TMP/p")
( cd "$ROOT" && OUT="$OUT" cargo run -q -p xtask -- atlas-call )
