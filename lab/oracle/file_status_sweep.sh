#!/usr/bin/env bash
# FILE STATUS atlas sweep (GNURUST.FILE.STATUS.1). Trigger declared file-operation conditions against real
# cobc/libcob and record the observed FILE STATUS bytes. OBSERVED court: the pure kernel does no I/O, so it
# does not produce open/close statuses -- this records which status arises from which condition and asserts
# the observed values are stable. Generates reports/file-status-atlas.json and PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

cat > "$TMP/fs.cob" <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. FS.
ENVIRONMENT DIVISION.
INPUT-OUTPUT SECTION.
FILE-CONTROL.
    SELECT F ASSIGN TO "DDIN" ORGANIZATION IS LINE SEQUENTIAL FILE STATUS IS WS.
DATA DIVISION.
FILE SECTION.
FD F.
01 R PIC X(8).
WORKING-STORAGE SECTION.
01 WS PIC XX.
PROCEDURE DIVISION.
    OPEN INPUT F.
    DISPLAY "open_input=" WS.
    IF WS = "00"
       READ F NEXT AT END CONTINUE END-READ
       DISPLAY "read_first=" WS
       PERFORM UNTIL WS = "10"
          READ F NEXT AT END CONTINUE END-READ
       END-PERFORM
       DISPLAY "read_at_eof=" WS
       READ F NEXT AT END CONTINUE END-READ
       DISPLAY "read_past_eof=" WS
    END-IF.
    CLOSE F.
    DISPLAY "close=" WS.
    STOP RUN.
COB
cobc -free -x -o "$TMP/fs" "$TMP/fs.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }

printf 'AB\nCD\n' > "$TMP/valid.dat"
VALID=$(DDIN="$TMP/valid.dat" "$TMP/fs")
MISSING=$(DDIN="$TMP/does-not-exist.dat" "$TMP/fs")

( cd "$ROOT" && VALID="$VALID" MISSING="$MISSING" cargo run -q -p xtask -- atlas-file-status )
