#!/usr/bin/env bash
# RECORD SEQUENTIAL sweep (GNURUST.FILEIO.SEQ.1). Variable-length WRITE across COB_VARSEQ_FORMAT 0-3 +
# fixed WRITE + variable READ (status+size), checked against fileio::sequential_write/read. PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/seqrec_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cd "$TMP" || exit 2

# variable-length WRITE program
cat > vw.cob <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. VW.
ENVIRONMENT DIVISION.
INPUT-OUTPUT SECTION.
FILE-CONTROL.
    SELECT FV ASSIGN "v.dat" ORGANIZATION IS RECORD SEQUENTIAL.
DATA DIVISION.
FILE SECTION.
FD FV RECORD IS VARYING IN SIZE FROM 1 TO 8 CHARACTERS DEPENDING ON RLEN.
01 RV PIC X(8).
WORKING-STORAGE SECTION.
01 RLEN PIC 9(4) COMP.
PROCEDURE DIVISION.
    OPEN OUTPUT FV.
    MOVE "AB" TO RV. MOVE 2 TO RLEN. WRITE RV.
    MOVE "HELLO" TO RV. MOVE 5 TO RLEN. WRITE RV.
    MOVE "XYZ12678" TO RV. MOVE 8 TO RLEN. WRITE RV.
    CLOSE FV. STOP RUN.
COB

# fixed WRITE program
cat > fw.cob <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. FW.
ENVIRONMENT DIVISION.
INPUT-OUTPUT SECTION.
FILE-CONTROL.
    SELECT FF ASSIGN "f.dat" ORGANIZATION IS RECORD SEQUENTIAL.
DATA DIVISION.
FILE SECTION.
FD FF.
01 RECF PIC X(8).
PROCEDURE DIVISION.
    OPEN OUTPUT FF.
    MOVE "AB" TO RECF. WRITE RECF.
    MOVE "HELLO123" TO RECF. WRITE RECF.
    MOVE SPACES TO RECF. WRITE RECF.
    CLOSE FF. STOP RUN.
COB

# variable READ program (logs status[2] + len[PIC 9(2)] per record to a text log)
cat > vr.cob <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. VR.
ENVIRONMENT DIVISION.
INPUT-OUTPUT SECTION.
FILE-CONTROL.
    SELECT FV ASSIGN "v.dat" ORGANIZATION IS RECORD SEQUENTIAL FILE STATUS IS ST.
DATA DIVISION.
FILE SECTION.
FD FV RECORD IS VARYING IN SIZE FROM 1 TO 8 CHARACTERS DEPENDING ON RLEN.
01 RV PIC X(8).
WORKING-STORAGE SECTION.
01 ST PIC XX.
01 RLEN PIC 9(4) COMP.
01 OUT PIC X(200) VALUE SPACES.
01 P PIC 9(4) COMP VALUE 1.
01 L2 PIC 9(2).
01 DONE PIC X VALUE "N".
PROCEDURE DIVISION.
    OPEN INPUT FV.
    PERFORM UNTIL DONE = "Y"
        READ FV NEXT RECORD AT END MOVE "Y" TO DONE
          NOT AT END
            MOVE RLEN TO L2
            MOVE ST TO OUT(P:2) MOVE L2 TO OUT(P + 2:2) ADD 4 TO P
        END-READ
        IF ST = "10" MOVE "Y" TO DONE END-IF
    END-PERFORM.
    CLOSE FV.
    DISPLAY FUNCTION TRIM(OUT).
    STOP RUN.
COB

cobc -free -x -o vw vw.cob 2>e1 || { echo "vw compile failed"; cat e1; exit 2; }
cobc -free -x -o fw fw.cob 2>e2 || { echo "fw compile failed"; cat e2; exit 2; }
cobc -free -x -o vr vr.cob 2>e3 || { echo "vr compile failed"; cat e3; exit 2; }

{
  for ty in 0 1 2 3; do
    rm -f v.dat; env COB_VARSEQ_FORMAT=$ty ./vw >/dev/null 2>&1
    printf 'vw%s=%s\n' "$ty" "$(xxd -p v.dat 2>/dev/null | tr -d '\n')"
    # read back under the same format
    row=$(env COB_VARSEQ_FORMAT=$ty ./vr 2>/dev/null)
    printf 'vr%s=%s\n' "$ty" "$row"
  done
  rm -f f.dat; ./fw >/dev/null 2>&1
  printf 'fw=%s\n' "$(xxd -p f.dat 2>/dev/null | tr -d '\n')"
} | "$ROWS"
