#!/usr/bin/env bash
# SEARCH / SEARCH ALL sweep (GNURUST.SEARCH.TABLE.1). Run serial + binary table searches and check the landing
# index == search_serial / search_all.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/search_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cat > "$TMP/p.cob" <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. SCH.
DATA DIVISION.
WORKING-STORAGE SECTION.
01 T.
   05 E OCCURS 5 ASCENDING KEY IS E-KEY INDEXED BY IX.
      10 E-KEY PIC 9(3).
PROCEDURE DIVISION.
    MOVE 010 TO E-KEY(1). MOVE 020 TO E-KEY(2). MOVE 050 TO E-KEY(3).
    MOVE 080 TO E-KEY(4). MOVE 099 TO E-KEY(5).
    SET IX TO 1. SEARCH E AT END DISPLAY "serial_50=notfound" WHEN E-KEY(IX) = 50 DISPLAY "serial_50=" IX END-SEARCH.
    SET IX TO 1. SEARCH E AT END DISPLAY "serial_77=notfound" WHEN E-KEY(IX) = 77 DISPLAY "serial_77=" IX END-SEARCH.
    SET IX TO 3. SEARCH E AT END DISPLAY "serial_from3_10=notfound" WHEN E-KEY(IX) = 10 DISPLAY "serial_from3_10=" IX END-SEARCH.
    SEARCH ALL E AT END DISPLAY "binary_80=notfound" WHEN E-KEY(IX) = 80 DISPLAY "binary_80=" IX END-SEARCH.
    SEARCH ALL E AT END DISPLAY "binary_55=notfound" WHEN E-KEY(IX) = 55 DISPLAY "binary_55=" IX END-SEARCH.
    SEARCH ALL E AT END DISPLAY "binary_10=notfound" WHEN E-KEY(IX) = 10 DISPLAY "binary_10=" IX END-SEARCH.
    STOP RUN.
COB
cobc -free -x -o "$TMP/p" "$TMP/p.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }
"$TMP/p" | "$ROWS"
