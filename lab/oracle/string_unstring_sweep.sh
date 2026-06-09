#!/usr/bin/env bash
# STRING/UNSTRING byte-effect sweep (GNURUST.STRING.UNSTRING.1). One program runs each case and DISPLAYs a
# result RECORD (the relevant receivers concatenated) as label=<hex bytes>; the Rust mirror recomputes each
# via string_into/unstring and compares the result-record bytes.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/string_unstring_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cat > "$TMP/p.cob" <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. SU.
DATA DIVISION.
WORKING-STORAGE SECTION.
01 T PIC X(6).
01 P PIC 9(2).
01 OVF PIC X VALUE "0".
01 S PIC X(10).
01 RES.
   05 RFD1 PIC X(4).
   05 RFC1 PIC 9(2).
   05 RFL1 PIC X(1).
   05 RFD2 PIC X(4).
   05 RFC2 PIC 9(2).
   05 RFL2 PIC X(1).
   05 RFD3 PIC X(4).
   05 RFC3 PIC 9(2).
   05 RFL3 PIC X(1).
   05 RTLY PIC 9(2).
01 RESX REDEFINES RES PIC X(23).
01 SR.
   05 SF1 PIC X(4).
   05 SC1 PIC 9(2).
01 SRX REDEFINES SR PIC X(6).
PROCEDURE DIVISION.
    MOVE ALL "~" TO T. STRING "AB" "CDE" DELIMITED BY SIZE INTO T.
    DISPLAY "s_size=" FUNCTION HEX-OF(T).
    MOVE ALL "~" TO T. MOVE 2 TO P. STRING "XY" DELIMITED BY SIZE INTO T WITH POINTER P.
    DISPLAY "s_ptr=" FUNCTION HEX-OF(T) FUNCTION HEX-OF(P).
    MOVE ALL "~" TO T. MOVE "0" TO OVF.
    STRING "ABCDEF" "GH" DELIMITED BY SIZE INTO T ON OVERFLOW MOVE "1" TO OVF.
    DISPLAY "s_ovf=" FUNCTION HEX-OF(T) FUNCTION HEX-OF(OVF).
    MOVE ALL "~" TO T. STRING "HELLO,WORLD" DELIMITED BY "," INTO T.
    DISPLAY "s_delim=" FUNCTION HEX-OF(T).
    MOVE "AB,CDE,F  " TO S. MOVE SPACES TO RESX. MOVE 0 TO RTLY.
    UNSTRING S DELIMITED BY "," INTO RFD1 DELIMITER IN RFL1 COUNT IN RFC1
                                  RFD2 DELIMITER IN RFL2 COUNT IN RFC2
                                  RFD3 DELIMITER IN RFL3 COUNT IN RFC3 TALLYING IN RTLY.
    DISPLAY "u_base=" FUNCTION HEX-OF(RESX).
    MOVE "A,,B      " TO S. MOVE SPACES TO RESX.
    UNSTRING S DELIMITED BY "," INTO RFD1 COUNT IN RFC1 RFD2 COUNT IN RFC2 RFD3 COUNT IN RFC3.
    DISPLAY "u_empty=" FUNCTION HEX-OF(RESX).
    MOVE "ABCDEFGH  " TO S. MOVE 3 TO P. MOVE SPACES TO SF1.
    UNSTRING S INTO SF1 WITH POINTER P.
    DISPLAY "u_ptr=" FUNCTION HEX-OF(SF1) FUNCTION HEX-OF(P).
    STOP RUN.
COB
cobc -free -x -o "$TMP/p" "$TMP/p.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }
"$TMP/p" | "$ROWS"
