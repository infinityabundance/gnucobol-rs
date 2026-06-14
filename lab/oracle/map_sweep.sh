#!/usr/bin/env bash
# Filename-mapping sweep (GNURUST.FILEIO.MAPPING.1). The oracle OPENs files whose ASSIGN names resolve
# through the environment (DD_* and COB_FILE_PATH); check fileio::cob_chk_file_mapping (reading the same
# env) points at the exact paths the oracle created the files at. PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/map_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cd "$TMP" || exit 2
mkdir -p sub base

cat > m.cob <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. M.
ENVIRONMENT DIVISION.
INPUT-OUTPUT SECTION.
FILE-CONTROL.
    SELECT FA ASSIGN "MAPA" ORGANIZATION IS LINE SEQUENTIAL.
    SELECT FB ASSIGN "bmap.dat" ORGANIZATION IS LINE SEQUENTIAL.
DATA DIVISION.
FILE SECTION.
FD FA.
01 RA PIC X(4).
FD FB.
01 RB PIC X(4).
PROCEDURE DIVISION.
    OPEN OUTPUT FA. MOVE "AAAA" TO RA. WRITE RA. CLOSE FA.
    OPEN OUTPUT FB. MOVE "BBBB" TO RB. WRITE RB. CLOSE FB.
    STOP RUN.
COB
cobc -free -x -o m m.cob 2>e || { echo "compile failed"; cat e; exit 2; }
# DD_MAPA -> an absolute resolved path; COB_FILE_PATH -> a prefix for the unmapped bmap.dat
env DD_MAPA="$TMP/sub/a_resolved.dat" COB_FILE_PATH="$TMP/base" ./m >/dev/null 2>&1
env DD_MAPA="$TMP/sub/a_resolved.dat" COB_FILE_PATH="$TMP/base" "$ROWS"
