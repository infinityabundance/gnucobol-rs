#!/usr/bin/env bash
# GNURUST.CALL.LAYOUT.ATLAS.1 -- deepen GNURUST.CALL.EXTENSION.ATLAS.1 from "BY REFERENCE shares / BY
# CONTENT copies" to the BYTE-EXACT parameter layout and the length-mismatch behavior, witnessed against
# real cobc/libcob (call.c). The migration hazard is a copybook whose LINKAGE length disagrees with the
# caller's field:
#   (1) BY REFERENCE is a pure ADDRESS OVERLAY -- a callee item LARGER than the caller's field reads PAST
#       it into ADJACENT caller storage (no truncation, no padding, no bounds), and the callee's write
#       lands back in the caller's bytes,
#   (2) BY CONTENT passes a SIZED COPY -- the caller is untouched and the callee's larger view is space-
#       padded,
#   (3) a numeric LINKAGE item narrower than the caller's field overlays the LEADING bytes positionally.
# OBSERVED court: gnucobol-rs executes NO subprograms (multi-module runtime is behavioral-ladder L8, NOT
# CLAIMED). This MAPS the parameter-passing byte layout; it runs nothing.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8 TERM=dumb
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cat > "$TMP/p.cob" <<'COB'
       IDENTIFICATION DIVISION.
       PROGRAM-ID. MAINP.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 REFREC.
          05 PA PIC X(3) VALUE "ABC".
          05 PB PIC X(3) VALUE "XYZ".
       01 CONTREC.
          05 PC PIC X(3) VALUE "DEF".
          05 PD PIC X(3) VALUE "QRS".
       01 NUM4 PIC 9(4) VALUE 1234.
       PROCEDURE DIVISION.
           CALL "SUBX5" USING BY REFERENCE PA.
           DISPLAY "REF-CALLER=" PA.
           CALL "SUBX3" USING BY CONTENT PC.
           DISPLAY "CONTENT-CALLER=" PC.
           CALL "SUBN2" USING BY REFERENCE NUM4.
           STOP RUN.
       END PROGRAM MAINP.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. SUBX5.
       DATA DIVISION.
       LINKAGE SECTION.
       01 L PIC X(5).
       PROCEDURE DIVISION USING L.
           DISPLAY "SEE5=[" L "]".
           MOVE "Z" TO L(1:1).
           EXIT PROGRAM.
       END PROGRAM SUBX5.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. SUBX3.
       DATA DIVISION.
       LINKAGE SECTION.
       01 L3 PIC X(3).
       PROCEDURE DIVISION USING L3.
           DISPLAY "SEE3=[" L3 "]".
           EXIT PROGRAM.
       END PROGRAM SUBX3.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. SUBN2.
       DATA DIVISION.
       LINKAGE SECTION.
       01 L2 PIC 9(2).
       PROCEDURE DIVISION USING L2.
           DISPLAY "SEEN2=[" L2 "]".
           EXIT PROGRAM.
       END PROGRAM SUBN2.
COB
cobc -free -x -o "$TMP/p" "$TMP/p.cob" 2>/dev/null || { echo "compile failed"; exit 2; }
OUT="$("$TMP/p" 2>/dev/null)"

( cd "$ROOT" && OUT="$OUT" cargo run -q -p xtask -- atlas-call-layout )
