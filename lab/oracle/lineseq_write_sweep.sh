#!/usr/bin/env bash
# Line-sequential WRITE config-matrix sweep (GNURUST.FILEIO.LINESEQ.1). OPEN OUTPUT + WRITE records to a
# LINE SEQUENTIAL file under the COB_LS_* runtime config matrix, hexdump the file, and check
# fileio::write_line_sequential == the oracle file bytes (and the validate-71 status). PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/lineseq_write_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cd "$TMP" || exit 2

# Program writing the VALID record set (no control bytes) to a LINE SEQUENTIAL file.
cat > valid.cob <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. V.
ENVIRONMENT DIVISION.
INPUT-OUTPUT SECTION.
FILE-CONTROL.
    SELECT FL ASSIGN "o.dat" ORGANIZATION IS LINE SEQUENTIAL FILE STATUS IS ST.
DATA DIVISION.
FILE SECTION.
FD FL.
01 RL PIC X(8).
WORKING-STORAGE SECTION.
01 ST PIC XX.
PROCEDURE DIVISION.
    OPEN OUTPUT FL.
    MOVE "AB" TO RL. WRITE RL.
    MOVE "HELLO123" TO RL. WRITE RL.
    MOVE SPACES TO RL. WRITE RL.
    MOVE "XY" TO RL. WRITE RL.
    MOVE "12345678" TO RL. WRITE RL.
    CLOSE FL.
    STOP RUN.
COB

# Program writing the CONTROL-BYTE record set (A,TAB,B in the middle record).
cat > ctrl.cob <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. C.
ENVIRONMENT DIVISION.
INPUT-OUTPUT SECTION.
FILE-CONTROL.
    SELECT FL ASSIGN "o.dat" ORGANIZATION IS LINE SEQUENTIAL FILE STATUS IS ST.
DATA DIVISION.
FILE SECTION.
FD FL.
01 RL PIC X(8).
WORKING-STORAGE SECTION.
01 ST PIC XX.
01 CT PIC X(8) VALUE X"4109422020202020".
PROCEDURE DIVISION.
    OPEN OUTPUT FL.
    MOVE "AB" TO RL. WRITE RL.
    MOVE CT TO RL. WRITE RL.
    MOVE "XY" TO RL. WRITE RL.
    CLOSE FL.
    STOP RUN.
COB

# Program writing a SINGLE bad-char record (isolates the validate-71 single-write semantics).
cat > one.cob <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. O.
ENVIRONMENT DIVISION.
INPUT-OUTPUT SECTION.
FILE-CONTROL.
    SELECT FL ASSIGN "o.dat" ORGANIZATION IS LINE SEQUENTIAL FILE STATUS IS ST.
DATA DIVISION.
FILE SECTION.
FD FL.
01 RL PIC X(8).
WORKING-STORAGE SECTION.
01 ST PIC XX.
01 CT PIC X(8) VALUE X"4109422020202020".
PROCEDURE DIVISION.
    OPEN OUTPUT FL.
    MOVE CT TO RL. WRITE RL.
    DISPLAY "ST=" ST.
    CLOSE FL.
    STOP RUN.
COB

cobc -free -x -o valid valid.cob 2>e1 || { echo "compile valid failed"; cat e1; exit 2; }
cobc -free -x -o ctrl  ctrl.cob  2>e2 || { echo "compile ctrl failed";  cat e2; exit 2; }
cobc -free -x -o one   one.cob   2>e3 || { echo "compile one failed";   cat e3; exit 2; }

filehex() { xxd -p o.dat 2>/dev/null | tr -d '\n'; }

{
  # valid set under default / plain / fixed
  rm -f o.dat; ./valid >/dev/null 2>&1;                          printf 'valid_default=%s\n' "$(filehex)"
  rm -f o.dat; env COB_LS_VALIDATE=0 ./valid >/dev/null 2>&1;    printf 'valid_plain=%s\n'   "$(filehex)"
  rm -f o.dat; env COB_LS_FIXED=1 ./valid >/dev/null 2>&1;       printf 'valid_fixed=%s\n'   "$(filehex)"
  # control-byte set under validate-off variants
  rm -f o.dat; env COB_LS_VALIDATE=0 ./ctrl >/dev/null 2>&1;                          printf 'ctrl_plain=%s\n'      "$(filehex)"
  rm -f o.dat; env COB_LS_VALIDATE=0 COB_LS_NULLS=1 ./ctrl >/dev/null 2>&1;           printf 'ctrl_nulls=%s\n'      "$(filehex)"
  rm -f o.dat; env COB_LS_VALIDATE=0 COB_LS_NULLS=1 COB_LS_FIXED=1 ./ctrl >/dev/null 2>&1; printf 'ctrl_fixednulls=%s\n' "$(filehex)"
  # single bad-char record under default validate: empty file + status 71
  rm -f o.dat; ST=$(./one 2>/dev/null | sed -n 's/^ST=//p'); printf 'ctrl_default_bytes=%s\n' "$(filehex)"; printf 'ctrl_default_status=%s\n' "$ST"
} | "$ROWS"
