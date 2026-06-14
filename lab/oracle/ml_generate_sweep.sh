#!/usr/bin/env bash
# XML/JSON GENERATE sweep (GNURUST.MLIO.GENERATE.1). Run cobc XML GENERATE + JSON GENERATE over a fixed
# record and check the native Rust serializer (mlio::cob_xml_generate_new / cob_json_generate_new) produces
# byte-identical output. Proves the dependency-free Rust reimplementation matches GnuCOBOL's libxml2/json-c
# output without those libraries. PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/mlio_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cd "$TMP" || exit 2

cat > mg.cob <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. MG.
DATA DIVISION.
WORKING-STORAGE SECTION.
01 G.
   05 NEG PIC S9(3) VALUE -42.
   05 DEC PIC 9(3)V99 VALUE 12.50.
   05 SPC PIC X(5) VALUE "a<b&c".
   05 GRP.
      10 X PIC X(2) VALUE "hi".
      10 Y PIC 9 VALUE 7.
01 OUT PIC X(400).
01 CNT PIC 9(4).
PROCEDURE DIVISION.
    XML GENERATE OUT FROM G COUNT IN CNT.
    DISPLAY "xml=" OUT(1:CNT).
    DISPLAY "xmlcount=" CNT.
    JSON GENERATE OUT FROM G COUNT IN CNT.
    DISPLAY "json=" OUT(1:CNT).
    DISPLAY "jsoncount=" CNT.
    STOP RUN.
COB
cobc -free -x -o mg mg.cob 2>e || { echo "compile failed"; cat e; exit 2; }
./mg 2>/dev/null | "$ROWS"
