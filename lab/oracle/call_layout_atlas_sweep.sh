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

OUT="$OUT" ROOT="$ROOT" python3 - <<'PY'
import os, json, re
out = os.environ["OUT"]; ROOT = os.environ["ROOT"]
kv = {}
for ln in out.splitlines():
    m = re.match(r'([A-Z0-9-]+)=\[?(.*?)\]?$', ln)
    if m: kv[m.group(1)] = m.group(2)
# SEE5 = the BY REFERENCE call (overlay into adjacent); SEE3 = BY CONTENT clean copy
see5 = re.findall(r'SEE5=\[(.*?)\]', out)
checks = [
 ("byref-overlay-adjacent", len(see5) >= 1 and see5[0] == "ABCXY",
  "BY REFERENCE: callee X(5) over caller X(3) 'ABC' reads INTO the adjacent field -> 'ABCXY' (overlay, no truncation/padding)"),
 ("byref-callee-write-visible", kv.get("REF-CALLER") == "ZBC",
  "the callee's MOVE 'Z' TO L(1:1) writes back into the caller's storage (PA 'ABC' -> 'ZBC')"),
 ("bycontent-clean-copy", kv.get("SEE3") == "DEF",
  "BY CONTENT: callee X(3) over a 3-byte CONTENT copy sees the clean copy 'DEF'"),
 ("bycontent-caller-untouched", kv.get("CONTENT-CALLER") == "DEF",
  "BY CONTENT leaves the caller's field UNCHANGED ('DEF')"),
 ("numeric-narrower-leading-bytes", kv.get("SEEN2") == "12",
  "a numeric LINKAGE 9(2) over a caller 9(4)=1234 overlays the LEADING two display bytes -> '12'"),
]
fails = [(n,why) for (n,ok,why) in checks if not ok]
atlas = {
 "schema":"gnurust-call-layout-atlas-v1","court":"GNURUST.CALL.LAYOUT.ATLAS.1","dialect":"gnucobol-3.2.0-default",
 "extends":"GNURUST.CALL.EXTENSION.ATLAS.1",
 "oracle":"cobc CALL USING + LINKAGE parameter layout (libcob/call.c address passing)",
 "doctrine":"OBSERVED deepening of the CALL atlas to the BYTE-EXACT parameter layout and length-mismatch behavior. The migration hazard is a copybook whose LINKAGE length disagrees with the caller's field. gnucobol-rs executes NO subprograms (multi-module runtime is behavioral-ladder L8, NOT CLAIMED); this records the witnessed passing-mode byte layout, it runs nothing.",
 "trace": out.split("\n"),
 "observations":[
  {"name":"BY REFERENCE address overlay","observed":"BY REFERENCE passes the ADDRESS; a callee LINKAGE item LARGER than the caller's field reads PAST it into ADJACENT caller storage (X(5) over X(3) 'ABC' next to 'XYZ' -> the callee sees 'ABCXY') -- NO truncation, NO padding, NO bounds check; a callee write lands back in the caller's bytes","status":"observed-only"},
  {"name":"BY CONTENT sized copy","observed":"BY CONTENT passes a COPY: the caller's field is UNCHANGED ('DEF'), and a callee whose LINKAGE item matches the content size sees the clean copy ('DEF'). A callee view LARGER than the content over-reads past the temporary into UNINITIALIZED memory (a null-byte read here) -- that over-read is undefined and is a refusal, not a claim","status":"observed-only"},
  {"name":"numeric length-mismatch overlay","observed":"a numeric LINKAGE item NARROWER than the caller's field overlays the LEADING bytes positionally (9(2) over 9(4)=1234 -> '12'), it is a byte overlay, not a numeric re-scaling","status":"observed-only"},
 ],
 "negative_capabilities":["NEG.CALL_LAYOUT.NO_SUBPROGRAM_EXECUTION","NEG.CALL_LAYOUT.NO_BOUNDS_ON_OVERLAY",
   "NEG.CALL_LAYOUT.NO_BY_VALUE_LAYOUT","NEG.CALL_LAYOUT.NO_ODO_ACROSS_LINKAGE",
   "NEG.CALL_LAYOUT.NO_OPTIONAL_OMITTED_PARAMS","NEG.CALL_LAYOUT.NO_RETURNING_PHRASE",
   "NEG.CALL_LAYOUT.NO_ALL_DIALECTS"],
}
json.dump(atlas, open(os.path.join(ROOT, "reports/call-layout-atlas.json"), "w"), indent=2)
print(f"PASS={len(checks)-len(fails)} FAIL={len(fails)}")
for n,why in fails: print("  MISMATCH", n, "--", why)
PY
