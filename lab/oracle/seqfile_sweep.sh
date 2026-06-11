#!/usr/bin/env bash
# Sequential-file READ byte+status sweep (GNURUST.FILE.SEQUENTIAL.1). For each case: write the input file,
# run the org-specific cobc program (OPEN INPUT / READ NEXT / AT END), capture each record's raw bytes + file
# status, and check read_sequential == the oracle sequence.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_seqfile"; ROWS="$ROOT/target/release/examples/seqfile_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
"$GEN" > "$TMP/cases.tsv"

# one reader program per organization (the ORGANIZATION clause is compile-time)
prog() {
cat > "$TMP/$1.cob" <<COB
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. RDR.
ENVIRONMENT DIVISION.
INPUT-OUTPUT SECTION.
FILE-CONTROL.
    SELECT F ASSIGN TO "DDIN" ORGANIZATION IS $2 FILE STATUS IS WS.
DATA DIVISION.
FILE SECTION.
FD F.
01 R PIC X(8).
WORKING-STORAGE SECTION.
01 WS PIC XX.
01 EOFF PIC X VALUE "N".
PROCEDURE DIVISION.
    OPEN INPUT F.
    PERFORM UNTIL EOFF = "Y"
        READ F NEXT RECORD
            AT END DISPLAY "END[" WS "]" MOVE "Y" TO EOFF
            NOT AT END DISPLAY "REC[" R "][" WS "]"
        END-READ
    END-PERFORM.
    CLOSE F. STOP RUN.
COB
cobc -free -x -o "$TMP/$1" "$TMP/$1.cob" 2>"$TMP/$1.err" || { echo "compile $1 failed"; cat "$TMP/$1.err"; exit 2; }
}
prog rec "RECORD SEQUENTIAL"
prog lin "LINE SEQUENTIAL"

# per case: write file, run the org program, extract events, append to the row
( cd "$ROOT" && cargo run -q -p xtask -- sweep-seqfile "$TMP/cases.tsv" "$TMP" ) | "$ROWS"
