#!/usr/bin/env bash
# GNURUST.DIRECTIVE.VARIANCE.ATLAS.1 -- observe how COMPILER DIRECTIVES change the record bytes. The
# sealed courts (esp. the binary courts GNURUST.14/18) are witnessed under the DEFAULT build config that
# GNURUST.BUILD.PROFILE.1 pins (binary-size 1-2-4-8, binary-byteorder big-endian, binary-truncate yes).
# This atlas records the BYTE-LEVEL DELTA when the same source is compiled under a NON-default directive --
# the "configure the decoder to your exact build environment" surface:
#   (1) -fbinary-size=2-4-8 grows 9(2) COMP from 1 byte to 2 (record layout shifts),
#   (2) -fbinary-byteorder=native stores COMP host-little-endian (34 12) vs default big-endian (12 34),
#   (3) -fno-binary-truncate keeps the raw binary value where the default truncates to PIC digits.
# OBSERVED court: gnucobol-rs decodes under ONE profile (the BUILD.PROFILE.1 default). This MAPS the
# directive-sensitivity boundary; it does NOT auto-detect a build profile or implement non-default configs.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8 TERM=dumb
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

cat > "$TMP/size.cob" <<'COB'
       IDENTIFICATION DIVISION.
       PROGRAM-ID. SZ.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 G.
          05 A PIC 9(2) COMP VALUE 7.
          05 B PIC 9(4) COMP VALUE 7.
          05 C PIC 9(9) COMP VALUE 7.
       PROCEDURE DIVISION.
           DISPLAY LENGTH OF G.
           STOP RUN.
COB
cat > "$TMP/ord.cob" <<'COB'
       IDENTIFICATION DIVISION.
       PROGRAM-ID. OD.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 N PIC 9(4) COMP VALUE 4660.
       01 R REDEFINES N PIC X(2).
       PROCEDURE DIVISION.
           DISPLAY R.
           STOP RUN.
COB
cat > "$TMP/trunc.cob" <<'COB'
       IDENTIFICATION DIVISION.
       PROGRAM-ID. TR.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 N PIC 9(2) COMP.
       PROCEDURE DIVISION.
           MOVE 300 TO N.
           DISPLAY N.
           STOP RUN.
COB

run() { # flags, src -> stdout (trimmed)  | "REJECT"
  cobc -free -x $1 -o "$TMP/p" "$2" 2>/dev/null || { echo REJECT; return; }
  "$TMP/p" 2>/dev/null | tr -d '\n'
}
hex2() { # flags, src -> first 2 bytes hex
  cobc -free -x $1 -o "$TMP/p" "$2" 2>/dev/null || { echo REJECT; return; }
  "$TMP/p" 2>/dev/null | head -c2 | xxd -p
}

SZ_DEF=$(run "-fbinary-size=1-2-4-8" "$TMP/size.cob")
SZ_248=$(run "-fbinary-size=2-4-8"   "$TMP/size.cob")
OD_DEF=$(hex2 "-fbinary-byteorder=big-endian" "$TMP/ord.cob")
OD_NAT=$(hex2 "-fbinary-byteorder=native"     "$TMP/ord.cob")
TR_DEF=$(run "-fbinary-truncate"    "$TMP/trunc.cob")
TR_NO=$(run  "-fno-binary-truncate" "$TMP/trunc.cob")

( cd "$ROOT" && SZ_DEF="$SZ_DEF" SZ_248="$SZ_248" OD_DEF="$OD_DEF" OD_NAT="$OD_NAT" TR_DEF="$TR_DEF" TR_NO="$TR_NO" cargo run -q -p xtask -- atlas-directive-variance )
