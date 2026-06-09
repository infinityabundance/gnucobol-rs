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

SZ_DEF="$SZ_DEF" SZ_248="$SZ_248" OD_DEF="$OD_DEF" OD_NAT="$OD_NAT" TR_DEF="$TR_DEF" TR_NO="$TR_NO" ROOT="$ROOT" \
python3 - <<'PY'
import json, os
g = lambda k: os.environ[k]
sz_def, sz_248, od_def, od_nat, tr_def, tr_no = (g(k) for k in ("SZ_DEF","SZ_248","OD_DEF","OD_NAT","TR_DEF","TR_NO"))
ROOT = g("ROOT")
checks = [
 ("binary-size-default-len7", sz_def == "7", "default binary-size 1-2-4-8: group{9(2),9(4),9(9)}COMP LENGTH = 7"),
 ("binary-size-248-grows-to-8", sz_248 == "8", "-fbinary-size=2-4-8: 9(2)COMP grows 1->2 bytes, group LENGTH = 8"),
 ("byteorder-default-big", od_def == "1234", "default/big-endian: 9(4)COMP=4660 stored 12 34"),
 ("byteorder-native-host-little", od_nat == "3412", "-fbinary-byteorder=native: stored 34 12 (host little-endian)"),
 ("truncate-default-ansi", tr_def == "00", "-fbinary-truncate (default): MOVE 300 TO 9(2)COMP truncates to 00"),
 ("no-truncate-keeps-binary", tr_no == "44", "-fno-binary-truncate: keeps the raw binary value, displays 44"),
]
fails = [(n,why) for (n,ok,why) in checks if not ok]
atlas = {
 "schema":"gnurust-directive-variance-atlas-v1","court":"GNURUST.DIRECTIVE.VARIANCE.ATLAS.1",
 "witness_default_profile":"GNURUST.BUILD.PROFILE.1 (binary-size 1-2-4-8, binary-byteorder big-endian, binary-truncate yes)",
 "oracle":"cobc -f<directive> compile + run (cobc/config.c directive engine + libcob/move.c binary codec)",
 "doctrine":"OBSERVED map of how COMPILER DIRECTIVES change the record bytes. The sealed binary courts (GNURUST.14/18) are witnessed under the DEFAULT build profile (BUILD.PROFILE.1). This records the BYTE-LEVEL DELTA when a NON-default directive is used -- the 'configure the decoder to your exact build environment' surface. gnucobol-rs decodes under ONE profile and does NOT auto-detect or implement non-default directives; the atlas shows what would shift.",
 "directive_variance":{
   "binary-size":{"1-2-4-8(default)":"group LENGTH "+sz_def,"2-4-8":"group LENGTH "+sz_248,"delta":"9(2)COMP 1 byte -> 2 bytes"},
   "binary-byteorder":{"big-endian(default)":od_def,"native":od_nat,"delta":"COMP byte order flips (big 12 34 vs host-little 34 12)"},
   "binary-truncate":{"yes(default,ANSI)":tr_def,"no":tr_no,"delta":"MOVE 300 TO 9(2)COMP: 00 (truncated) vs 44 (raw binary)"},
 },
 "observations":[
  {"name":"binary-size record layout","observed":"-fbinary-size=2-4-8 allocates 9(2) COMP as 2 bytes (vs 1 under the default 1-2-4-8), so a record's COMP-field offsets and total length SHIFT (group LENGTH 7 -> 8) -- a wrong binary-size silently mis-offsets every field after a small COMP","status":"observed-only"},
  {"name":"binary-byteorder","observed":"-fbinary-byteorder=native stores a COMP field in the HOST byte order (34 12 little-endian on this x86_64 host) vs the default config's big-endian (12 34) -- decoding under the wrong byteorder reverses every multi-byte binary value","status":"observed-only"},
  {"name":"binary-truncate","observed":"the default -fbinary-truncate (ANSI) truncates a COMP MOVE to the PIC digit count (300 -> 00) while -fno-binary-truncate keeps the raw binary value (displays 44) -- the same MOVE yields different stored/observed values","status":"observed-only"},
 ],
 "negative_capabilities":["NEG.DIRECTIVE_VARIANCE.NO_NONDEFAULT_IMPLEMENTATION","NEG.DIRECTIVE_VARIANCE.NO_PROFILE_AUTODETECT",
   "NEG.DIRECTIVE_VARIANCE.NO_FULL_DIRECTIVE_ENUMERATION","NEG.DIRECTIVE_VARIANCE.NO_DIALECT_FLAGS",
   "NEG.DIRECTIVE_VARIANCE.NO_CODEGEN_DIRECTIVES","NEG.DIRECTIVE_VARIANCE.NO_RUNTIME_ENV_VARS",
   "NEG.DIRECTIVE_VARIANCE.NO_VENDOR_DIRECTIVES"],
}
json.dump(atlas, open(os.path.join(ROOT, "reports/directive-variance-atlas.json"), "w"), indent=2)
print(f"PASS={len(checks)-len(fails)} FAIL={len(fails)}")
for n,why in fails: print("  MISMATCH", n, "--", why)
PY
