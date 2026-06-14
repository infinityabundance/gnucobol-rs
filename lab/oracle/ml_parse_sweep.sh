#!/usr/bin/env bash
# XML PARSE sweep (GNURUST.MLIO.PARSE.1). GnuCOBOL 3.2's XML PARSE is unimplemented even WITH libxml2: it
# hands the document back and emits START-OF-DOCUMENT -> END-OF-INPUT -> END-OF-DOCUMENT without parsing.
# This sweep runs cobc XML PARSE over a fixed document, captures the PROCESSING-PROCEDURE event trace +
# final XML-CODE, and checks the native Rust state machine (mlio::cob_xml_parse) reproduces it exactly --
# proving the dependency-free reimplementation matches the authority's observable behaviour. PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/mlio_parse_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cd "$TMP" || exit 2

cat > xp.cob <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. XP.
DATA DIVISION.
WORKING-STORAGE SECTION.
01 DOC PIC X(60) VALUE '<a x="1">hi<b>z</b></a>'.
PROCEDURE DIVISION.
    XML PARSE DOC PROCESSING PROCEDURE P
    DISPLAY "final=" XML-CODE
    STOP RUN.
    P SECTION.
    P1.
    DISPLAY "ev=" FUNCTION TRIM(XML-EVENT).
COB
cobc -free -x -o xp xp.cob 2>e || { echo "compile failed"; cat e; exit 2; }
./xp 2>/dev/null | "$ROWS"
