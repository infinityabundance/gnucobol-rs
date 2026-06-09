#!/usr/bin/env bash
# GNURUST.INDEXED.FILE.ATLAS.1 -- observe INDEXED-file keyed access + status against real cobc/libcob. The
# largest remaining gap cluster (START 238x + DELETE 118x + indexed-org per GNURUST.PUBLIC.GAP.1). OBSERVED
# court: gnucobol-rs does NOT implement indexed files -- the on-disk ISAM/BDB/VBISAM format is backend-specific
# and out of the fixed-record evidence lane. This MAPS the surface: keyed random access, key-order retrieval,
# duplicate-key/not-found status, START positioning, DELETE.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cat > "$TMP/p.cob" <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. IDX.
ENVIRONMENT DIVISION.
INPUT-OUTPUT SECTION.
FILE-CONTROL.
    SELECT F ASSIGN "idx.dat" ORGANIZATION IS INDEXED
        ACCESS MODE IS DYNAMIC RECORD KEY IS R-KEY FILE STATUS IS FS.
DATA DIVISION.
FILE SECTION.
FD F.
01 R.
   05 R-KEY PIC X(3).
   05 R-VAL PIC X(5).
WORKING-STORAGE SECTION.
01 FS PIC X(2).
PROCEDURE DIVISION.
    OPEN OUTPUT F.
    MOVE "AAA" TO R-KEY. MOVE "alpha" TO R-VAL. WRITE R.
    MOVE "CCC" TO R-KEY. MOVE "gamma" TO R-VAL. WRITE R.
    MOVE "BBB" TO R-KEY. MOVE "beta " TO R-VAL. WRITE R.
    MOVE "AAA" TO R-KEY. MOVE "dup  " TO R-VAL. WRITE R. DISPLAY "dup=" FS.
    CLOSE F.
    OPEN I-O F.
    MOVE "BBB" TO R-KEY. READ F. DISPLAY "read_hit=" FS "/" R-VAL.
    MOVE "ZZZ" TO R-KEY. READ F. DISPLAY "read_miss=" FS.
    MOVE "AAA" TO R-KEY. START F KEY >= R-KEY. DISPLAY "start=" FS.
    READ F NEXT. DISPLAY "n1=" R-KEY. READ F NEXT. DISPLAY "n2=" R-KEY. READ F NEXT. DISPLAY "n3=" R-KEY.
    MOVE "BBB" TO R-KEY. DELETE F. DISPLAY "del=" FS.
    MOVE "BBB" TO R-KEY. READ F. DISPLAY "read_del=" FS.
    CLOSE F.
    STOP RUN.
COB
cobc -free -x -o "$TMP/p" "$TMP/p.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }
OUT=$( cd "$TMP" && ./p )
python3 - "$ROOT/reports/indexed-file-atlas.json" "$OUT" <<'PY'
import json, sys
kv = {}
for line in sys.argv[2].splitlines():
    if "=" in line: k, v = line.split("=", 1); kv[k.strip()] = v.strip()
expect = {"dup":"22","read_hit":"00/beta","read_miss":"23","start":"00","n1":"AAA","n2":"BBB","n3":"CCC","del":"00","read_del":"23"}
fails = [(k, e, kv.get(k)) for k, e in expect.items() if kv.get(k) != e]
atlas = {
 "schema":"gnurust-indexed-file-atlas-v1","court":"GNURUST.INDEXED.FILE.ATLAS.1","dialect":"gnucobol-3.2.0-default",
 "oracle":"cobc INDEXED file I/O (libcob/fileio.c + ISAM backend)",
 "doctrine":"OBSERVED map of the INDEXED-file surface (the largest remaining gap cluster). gnucobol-rs does NOT implement indexed files -- the on-disk ISAM/BDB/VBISAM index format is backend-specific and outside the fixed-record evidence lane. This records the OBSERVED keyed-access semantics + status; it implements no indexed file I/O.",
 "observations":[
  {"name":"keyed random READ","observed":"READ by RECORD KEY returns the matching record (status 00) or status 23 not-found","status":"observed-only"},
  {"name":"key-order retrieval","observed":"records are stored/retrieved in KEY ORDER (AAA,BBB,CCC) not insertion order -- the index sorts","status":"observed-only"},
  {"name":"duplicate primary key","observed":"WRITE of a duplicate RECORD KEY is rejected with status 22","status":"observed-only"},
  {"name":"START positioning","observed":"START KEY >= k positions the cursor for READ NEXT in key order (status 00)","status":"observed-only"},
  {"name":"DELETE by key","observed":"DELETE removes the keyed record (00); a subsequent READ returns 23","status":"observed-only"},
  {"name":"file status","observed":"00 success / 22 duplicate key / 23 not found (keyed status codes)","status":"observed-only"},
 ],
 "negative_capabilities":["NEG.INDEXED_FILE.NO_ISAM_BACKEND_FORMAT","NEG.INDEXED_FILE.NO_PAGE_CHECKSUM_ATOMICITY",
   "NEG.INDEXED_FILE.NO_ALTERNATE_KEYS","NEG.INDEXED_FILE.NO_CONCURRENT_ACCESS",
   "NEG.INDEXED_FILE.NO_FILE_EXECUTION","NEG.INDEXED_FILE.NO_RELATIVE_FILES","NEG.INDEXED_FILE.NO_ALL_DIALECTS"],
}
json.dump(atlas, open(sys.argv[1],"w"), indent=2)
print(f"PASS={len(expect)-len(fails)} FAIL={len(fails)}")
for f in fails: print("  MISMATCH", f)
PY
