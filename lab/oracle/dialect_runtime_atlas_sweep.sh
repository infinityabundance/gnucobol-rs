#!/usr/bin/env bash
# GNURUST.DIALECT.RUNTIME.ATLAS.1 -- observe where RUNTIME behavior diverges across GnuCOBOL's own
# -std dialects (default/cobol85/cobol2014/ibm-strict/mvs-strict/mf-strict/bs2000-strict/rm-strict/
# gcos-strict). The project's sealed courts are all witnessed under gnucobol-3.2.0-DEFAULT; this atlas
# MAPS the boundary of that single-dialect witness:
#   (1) STORED zoned-sign bytes are dialect-INVARIANT (the record-decode lane is not dialect-sensitive),
#   (2) the DISPLAY *presentation* of a signed field DIVERGES (leading vs trailing sign camp),
#   (3) COMPILE-acceptance of GnuCOBOL/ISO extensions diverges by dialect.
# OBSERVED court: gnucobol-rs runs ONE witness (default). This records the cross-dialect divergence; it
# does NOT implement any non-default dialect and claims NO vendor (IBM/MF/...) parity -- the -std modes
# are GnuCOBOL's *approximations* of those dialects, never the vendor compilers themselves.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8 TERM=dumb
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

# --- Probe 1: STORED bytes of a signed DISPLAY field (via REDEFINES X) across runtime dialects ---
cat > "$TMP/store.cob" <<'COB'
       IDENTIFICATION DIVISION.
       PROGRAM-ID. STOREP.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 S-DISP PIC S9(4) VALUE -123.
       01 R REDEFINES S-DISP PIC X(4).
       PROCEDURE DIVISION.
           DISPLAY R.
           STOP RUN.
COB
# --- Probe 2: DISPLAY *presentation* of the same signed field ---
cat > "$TMP/disp.cob" <<'COB'
       IDENTIFICATION DIVISION.
       PROGRAM-ID. DISPP.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 S-DISP PIC S9(4) VALUE -123.
       PROCEDURE DIVISION.
           DISPLAY S-DISP.
           STOP RUN.
COB

RUNTIME_DIALECTS="default cobol85 cobol2014 ibm-strict mvs-strict mf-strict bs2000-strict rm-strict gcos-strict"

store_hex() { # dialect -> stored bytes hex (first 4 bytes, no trailing newline byte)
  cobc -free -x -std="$1" -o "$TMP/s_$1" "$TMP/store.cob" 2>/dev/null || { echo "REJECT"; return; }
  "$TMP/s_$1" 2>/dev/null | head -c 4 | xxd -p
}
disp_text() { # dialect -> presentation text (sans newline)
  cobc -free -x -std="$1" -o "$TMP/d_$1" "$TMP/disp.cob" 2>/dev/null || { echo "REJECT"; return; }
  "$TMP/d_$1" 2>/dev/null | tr -d '\n'
}
accepts() { # dialect, ws, proc -> OK|REJ
  cat > "$TMP/c.cob" <<COB
       IDENTIFICATION DIVISION.
       PROGRAM-ID. C.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       $2
       PROCEDURE DIVISION.
           $3
           STOP RUN.
COB
  if cobc -free -x -std="$1" -o /dev/null "$TMP/c.cob" >/dev/null 2>&1; then echo OK; else echo REJ; fi
}

: > "$TMP/store.txt"; : > "$TMP/disp.txt"
for d in $RUNTIME_DIALECTS; do
  echo "$d $(store_hex "$d")" >> "$TMP/store.txt"
  echo "$d $(disp_text "$d")" >> "$TMP/disp.txt"
done

C5_85=$(accepts cobol85 "01 N PIC 9(4) COMP-5." 'DISPLAY N.')
C5_DEF=$(accepts default "01 N PIC 9(4) COMP-5." 'DISPLAY N.')
TRIM_85=$(accepts cobol85 '01 S PIC X(3) VALUE "a  ".' 'DISPLAY FUNCTION TRIM(S).')
BL_IBM=$(accepts ibm-strict "01 N USAGE BINARY-LONG." 'MOVE 1 TO N.')
BL_DEF=$(accepts default "01 N USAGE BINARY-LONG." 'MOVE 1 TO N.')

( cd "$ROOT" && STORE_TXT="$TMP/store.txt" DISP_TXT="$TMP/disp.txt" C5_85="$C5_85" C5_DEF="$C5_DEF" TRIM_85="$TRIM_85" BL_IBM="$BL_IBM" BL_DEF="$BL_DEF" cargo run -q -p xtask -- atlas-dialect-runtime )
