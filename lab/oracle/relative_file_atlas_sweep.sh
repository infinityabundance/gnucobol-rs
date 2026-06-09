#!/usr/bin/env bash
# GNURUST.RELATIVE.FILE.ATLAS.1 -- observe RELATIVE-file random access (by relative record number) + status.
# OBSERVED court: gnucobol-rs implements no relative file I/O (the on-disk slotted format is backend-specific).
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cat > "$TMP/p.cob" <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. REL.
ENVIRONMENT DIVISION.
INPUT-OUTPUT SECTION.
FILE-CONTROL.
    SELECT F ASSIGN "rel.dat" ORGANIZATION IS RELATIVE
        ACCESS MODE IS RANDOM RELATIVE KEY IS R-NUM FILE STATUS IS FS.
DATA DIVISION.
FILE SECTION.
FD F.
01 R PIC X(5).
WORKING-STORAGE SECTION.
01 R-NUM PIC 9(2).
01 FS PIC X(2).
PROCEDURE DIVISION.
    OPEN OUTPUT F.
    MOVE 3 TO R-NUM. MOVE "three" TO R. WRITE R.
    MOVE 1 TO R-NUM. MOVE "one  " TO R. WRITE R.
    MOVE 5 TO R-NUM. MOVE "five " TO R. WRITE R.
    CLOSE F.
    OPEN INPUT F.
    MOVE 3 TO R-NUM. READ F. DISPLAY "r3=" FS "/" R.
    MOVE 2 TO R-NUM. READ F. DISPLAY "r2=" FS.
    MOVE 1 TO R-NUM. READ F. DISPLAY "r1=" FS "/" R.
    CLOSE F.
    STOP RUN.
COB
cobc -free -x -o "$TMP/p" "$TMP/p.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }
OUT=$( cd "$TMP" && ./p )
python3 - "$ROOT/reports/relative-file-atlas.json" "$OUT" <<'PY'
import json, sys
kv = {}
for line in sys.argv[2].splitlines():
    if "=" in line: k, v = line.split("=",1); kv[k.strip()] = v.strip()
expect = {"r3":"00/three","r2":"23","r1":"00/one"}
fails = [(k,e,kv.get(k)) for k,e in expect.items() if kv.get(k) != e]
atlas = {
 "schema":"gnurust-relative-file-atlas-v1","court":"GNURUST.RELATIVE.FILE.ATLAS.1","dialect":"gnucobol-3.2.0-default",
 "oracle":"cobc RELATIVE file I/O (libcob/fileio.c)",
 "doctrine":"OBSERVED map of the RELATIVE-file surface. gnucobol-rs implements no relative file I/O -- the on-disk slotted format is backend-specific. This records the random-access-by-record-number semantics + status.",
 "observations":[
  {"name":"RELATIVE KEY random access","observed":"READ/WRITE address a record by its 1-based RELATIVE record number (slot); R3 reads the record written at slot 3","status":"observed-only"},
  {"name":"empty slot","observed":"reading an unwritten slot returns status 23 (record not found)","status":"observed-only"},
  {"name":"position by number","observed":"records sit at fixed positions by relative number (NOT key-sorted like indexed)","status":"observed-only"},
 ],
 "negative_capabilities":["NEG.RELATIVE_FILE.NO_FILE_EXECUTION","NEG.RELATIVE_FILE.NO_ON_DISK_FORMAT",
   "NEG.RELATIVE_FILE.NO_SEQUENTIAL_DYNAMIC_MODES","NEG.RELATIVE_FILE.NO_REWRITE_DELETE_START",
   "NEG.RELATIVE_FILE.NO_INDEXED_FILES","NEG.RELATIVE_FILE.NO_ALL_DIALECTS"],
}
json.dump(atlas, open(sys.argv[1],"w"), indent=2)
print(f"PASS={len(expect)-len(fails)} FAIL={len(fails)}")
for f in fails: print("  MISMATCH", f)
PY
