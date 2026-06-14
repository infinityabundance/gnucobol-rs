#!/usr/bin/env bash
# CBL_* system-routine sweep (GNURUST.FILEIO.SYS.1). CALL CBL_CREATE_DIR/DELETE_DIR/DELETE_FILE/CHANGE_DIR/
# GET_CURRENT_DIR in a fixed sequence, capture RETURN-CODE after each, and check fileio::cob_sys_* produce
# the same status sequence. PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/cob_sys_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cd "$TMP" || exit 2

cat > sys.cob <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. SYS.
DATA DIVISION.
WORKING-STORAGE SECTION.
01 DIRNAME   PIC X(64) VALUE "gnucobol_rs_sys_sweep_oracle".
01 MISSING   PIC X(64) VALUE "gnucobol_rs_sys_sweep_oracle/nofile".
01 BADDIR    PIC X(32) VALUE "/gnucobol_rs_no_such_dir_zz".
01 FLAGS     PIC 9(9) COMP-5 VALUE 0.
01 DLEN      PIC 9(9) COMP-5 VALUE 4096.
01 CWD       PIC X(4096).
01 OUT       PIC X(60) VALUE SPACES.
01 P         PIC 99 VALUE 1.
01 RC2       PIC ZZ9.
PROCEDURE DIVISION.
    CALL "CBL_CREATE_DIR"      USING DIRNAME              END-CALL PERFORM PUTRC.
    CALL "CBL_CREATE_DIR"      USING DIRNAME              END-CALL PERFORM PUTRC.
    CALL "CBL_DELETE_FILE"     USING MISSING              END-CALL PERFORM PUTRC.
    CALL "CBL_DELETE_DIR"      USING DIRNAME              END-CALL PERFORM PUTRC.
    CALL "CBL_DELETE_DIR"      USING DIRNAME              END-CALL PERFORM PUTRC.
    CALL "CBL_CHANGE_DIR"      USING BADDIR               END-CALL PERFORM PUTRC.
    CALL "CBL_GET_CURRENT_DIR" USING BY VALUE FLAGS DLEN BY REFERENCE CWD END-CALL PERFORM PUTRC.
    DISPLAY "statuses=" FUNCTION TRIM(OUT).
    STOP RUN.
PUTRC.
    MOVE RETURN-CODE TO RC2.
    IF P > 1
        STRING "," DELIMITED BY SIZE INTO OUT WITH POINTER P
    END-IF.
    STRING FUNCTION TRIM(RC2) DELIMITED BY SIZE INTO OUT WITH POINTER P.
COB
cobc -free -x -o sys sys.cob 2>e || { echo "compile failed"; cat e; exit 2; }
./sys 2>/dev/null | "$ROWS"
