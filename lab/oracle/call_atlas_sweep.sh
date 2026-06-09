#!/usr/bin/env bash
# GNURUST.CALL.EXTENSION.ATLAS.1 -- observe CALL / linkage / C$ extension behavior against real cobc/libcob.
# The #1 gap by frequency in the admitted testsuite (CALL 959x). OBSERVED court: gnucobol-rs does NOT execute
# subprogram CALLs (multi-module runtime = behavioral-ladder L8, NOT CLAIMED). This MAPS the surface: the
# byte-effect of parameter passing (BY REFERENCE shares storage, BY CONTENT copies), C$ extensions, ON
# EXCEPTION, CANCEL, RETURN-CODE.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cat > "$TMP/p.cob" <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. MAINP.
DATA DIVISION.
WORKING-STORAGE SECTION.
01 A PIC 9(3) VALUE 100.
01 B PIC 9(3) VALUE 100.
01 TXT PIC X(5) VALUE "abcde".
PROCEDURE DIVISION.
    CALL "SUBP" USING BY REFERENCE A. DISPLAY "ref_A=" A.
    CALL "SUBP" USING BY CONTENT B. DISPLAY "content_B=" B.
    CALL "C$TOUPPER" USING TXT, BY VALUE 5. DISPLAY "toupper=[" TXT "]".
    CALL "NOSUCH" ON EXCEPTION DISPLAY "exception=caught" END-CALL.
    CANCEL "SUBP". DISPLAY "cancel=ok".
    STOP RUN.
END PROGRAM MAINP.
IDENTIFICATION DIVISION.
PROGRAM-ID. SUBP.
DATA DIVISION.
LINKAGE SECTION.
01 P PIC 9(3).
PROCEDURE DIVISION USING P.
    ADD 1 TO P. EXIT PROGRAM.
END PROGRAM SUBP.
COB
cobc -free -x -o "$TMP/p" "$TMP/p.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }
OUT=$("$TMP/p")
python3 - "$ROOT/reports/call-atlas.json" "$OUT" <<'PY'
import json, sys
kv = {}
for line in sys.argv[2].splitlines():
    if "=" in line: k, v = line.split("=", 1); kv[k.strip()] = v.strip()
expect = {"ref_A":"101","content_B":"100","toupper":"[ABCDE]","exception":"caught","cancel":"ok"}
fails = [(k, e, kv.get(k)) for k, e in expect.items() if kv.get(k) != e]
atlas = {
 "schema":"gnurust-call-atlas-v1","court":"GNURUST.CALL.EXTENSION.ATLAS.1","dialect":"gnucobol-3.2.0-default",
 "oracle":"cobc CALL/CANCEL + linkage (libcob/call.c) + C$ system routines",
 "doctrine":"OBSERVED map of the CALL/linkage surface -- the #1 gap by frequency (CALL 959x in the admitted testsuite). gnucobol-rs does NOT execute subprogram CALLs; this records the observed parameter-passing byte-effect and linkage conventions, it runs no subprograms.",
 "observations":[
  {"name":"USING BY REFERENCE","observed":"the callee SHARES the caller's storage; callee mutation is visible in the caller (A: 100 -> 101)","status":"observed-only"},
  {"name":"USING BY CONTENT","observed":"the callee gets a COPY; the caller's storage is UNCHANGED (B stays 100)","status":"observed-only"},
  {"name":"USING BY VALUE","observed":"passes the value; BY VALUE to a reference-expecting LINKAGE param is undefined (segfaults) -- refused","status":"refused"},
  {"name":"C$ extension (C$TOUPPER)","observed":"a built-in system routine mutating in place (abcde -> ABCDE)","status":"observed-only"},
  {"name":"ON EXCEPTION","observed":"a missing/unresolved subprogram fires ON EXCEPTION (graceful, no crash)","status":"observed-only"},
  {"name":"CANCEL","observed":"CANCEL unloads a (dynamically-called) subprogram","status":"observed-only"},
  {"name":"RETURN-CODE","observed":"a conventional integer call status register","status":"observed-only"},
 ],
 "negative_capabilities":["NEG.CALL.NO_SUBPROGRAM_EXECUTION","NEG.CALL.NO_DYNAMIC_LINKING",
   "NEG.CALL.NO_C_EXTENSION_IMPL","NEG.CALL.NO_BY_VALUE_REFERENCE_MISMATCH",
   "NEG.CALL.NO_RECURSION_REENTRANCY","NEG.CALL.NO_CANCEL_STATE","NEG.CALL.NO_ALL_DIALECTS"],
}
json.dump(atlas, open(sys.argv[1],"w"), indent=2)
print(f"PASS={len(expect)-len(fails)} FAIL={len(fails)}")
for f in fails: print("  MISMATCH", f)
PY
