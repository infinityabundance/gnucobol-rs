#!/usr/bin/env bash
# Sequential WRITE sweep (GNURUST.FILE.WRITE.1). OPEN OUTPUT + WRITE the same records to a RECORD SEQUENTIAL and
# a LINE SEQUENTIAL file, hexdump both, and check write_sequential == the oracle file bytes.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/write_seq_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cat > "$TMP/p.cob" <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. WSQ.
ENVIRONMENT DIVISION.
INPUT-OUTPUT SECTION.
FILE-CONTROL.
    SELECT FR ASSIGN "out_rs.dat" ORGANIZATION IS RECORD SEQUENTIAL.
    SELECT FL ASSIGN "out_ls.dat" ORGANIZATION IS LINE SEQUENTIAL.
DATA DIVISION.
FILE SECTION.
FD FR.
01 RR PIC X(8).
FD FL.
01 RL PIC X(8).
WORKING-STORAGE SECTION.
PROCEDURE DIVISION.
    OPEN OUTPUT FR.
    MOVE "AB" TO RR. WRITE RR.
    MOVE "HELLO123" TO RR. WRITE RR.
    MOVE SPACES TO RR. WRITE RR.
    MOVE "XY" TO RR. WRITE RR.
    MOVE "12345678" TO RR. WRITE RR.
    CLOSE FR.
    OPEN OUTPUT FL.
    MOVE "AB" TO RL. WRITE RL.
    MOVE "HELLO123" TO RL. WRITE RL.
    MOVE SPACES TO RL. WRITE RL.
    MOVE "XY" TO RL. WRITE RL.
    MOVE "12345678" TO RL. WRITE RL.
    CLOSE FL.
    STOP RUN.
COB
( cd "$TMP" && cobc -free -x -o p p.cob 2>err && ./p ) || { echo "compile/run failed"; cat "$TMP/err"; exit 2; }
RS_HEX=$(xxd -p "$TMP/out_rs.dat" | tr -d '\n')
LS_HEX=$(xxd -p "$TMP/out_ls.dat" | tr -d '\n')
printf 'rs=%s\nls=%s\n' "$RS_HEX" "$LS_HEX" | "$ROWS"
