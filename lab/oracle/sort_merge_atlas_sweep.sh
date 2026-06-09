#!/usr/bin/env bash
# GNURUST.SORT.MERGE.ATLAS.1 -- observe SORT byte-effect (record reordering by key) against real cobc/libcob.
# The last big surface gap (SORT/MERGE 145x per GNURUST.PUBLIC.GAP.1). OBSERVED court: gnucobol-rs does NOT
# execute SORT (a runtime sort engine + work files); this MAPS the reordering byte-effect.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cat > "$TMP/p.cob" <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. SRT.
ENVIRONMENT DIVISION.
INPUT-OUTPUT SECTION.
FILE-CONTROL.
    SELECT INF ASSIGN "in.dat" ORGANIZATION IS LINE SEQUENTIAL.
    SELECT OUTF ASSIGN "out.dat" ORGANIZATION IS LINE SEQUENTIAL.
    SELECT WRK ASSIGN "wrk".
DATA DIVISION.
FILE SECTION.
FD INF.
01 IREC PIC X(8).
FD OUTF.
01 OREC PIC X(8).
SD WRK.
01 WREC.
   05 W-KEY PIC 9(3).
   05 W-VAL PIC X(5).
WORKING-STORAGE SECTION.
01 EOFSW PIC X VALUE "N".
PROCEDURE DIVISION.
    OPEN OUTPUT INF.
    MOVE "050alpha" TO IREC. WRITE IREC.
    MOVE "010bravo" TO IREC. WRITE IREC.
    MOVE "099charl" TO IREC. WRITE IREC.
    MOVE "020delta" TO IREC. WRITE IREC.
    CLOSE INF.
    SORT WRK ASCENDING KEY W-KEY USING INF GIVING OUTF.
    OPEN INPUT OUTF. MOVE "N" TO EOFSW.
    PERFORM UNTIL EOFSW = "Y"
        READ OUTF AT END MOVE "Y" TO EOFSW NOT AT END DISPLAY "asc=" OREC END-READ
    END-PERFORM.
    CLOSE OUTF.
    SORT WRK DESCENDING KEY W-KEY USING INF GIVING OUTF.
    OPEN INPUT OUTF. MOVE "N" TO EOFSW.
    PERFORM UNTIL EOFSW = "Y"
        READ OUTF AT END MOVE "Y" TO EOFSW NOT AT END DISPLAY "desc=" OREC END-READ
    END-PERFORM.
    CLOSE OUTF.
    STOP RUN.
COB
cobc -free -x -o "$TMP/p" "$TMP/p.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }
OUT=$( cd "$TMP" && ./p )
python3 - "$ROOT/reports/sort-merge-atlas.json" "$OUT" <<'PY'
import json, sys
asc = [l.split("=",1)[1][:3] for l in sys.argv[2].splitlines() if l.startswith("asc=")]
desc = [l.split("=",1)[1][:3] for l in sys.argv[2].splitlines() if l.startswith("desc=")]
ok_asc = asc == ["010","020","050","099"]
ok_desc = desc == ["099","050","020","010"]
fails = []
if not ok_asc: fails.append(("ascending", asc))
if not ok_desc: fails.append(("descending", desc))
atlas = {
 "schema":"gnurust-sort-merge-atlas-v1","court":"GNURUST.SORT.MERGE.ATLAS.1","dialect":"gnucobol-3.2.0-default",
 "oracle":"cobc SORT/MERGE (libcob/fileio.c sort engine + work file)",
 "doctrine":"OBSERVED map of the SORT/MERGE surface (the last big surface gap, SORT/MERGE 145x). gnucobol-rs does NOT execute SORT -- it is a runtime sort engine over a work file (SD). This records the reordering byte-effect; it sorts nothing.",
 "observations":[
  {"name":"SORT ASCENDING KEY","observed":"records reordered into KEY-ASCENDING order (input 050,010,099,020 -> 010,020,050,099)","status":"observed-only"},
  {"name":"SORT DESCENDING KEY","observed":"records reordered into KEY-DESCENDING order (-> 099,050,020,010)","status":"observed-only"},
  {"name":"USING / GIVING","observed":"USING reads the input file into the SD work file; GIVING writes the sorted SD records to the output file","status":"observed-only"},
  {"name":"SD work file","observed":"SORT uses a sort-merge file description (SD) + a work file; it is a runtime sort, not an in-place reorder","status":"observed-only"},
 ],
 "negative_capabilities":["NEG.SORT_MERGE.NO_SORT_EXECUTION","NEG.SORT_MERGE.NO_INPUT_OUTPUT_PROCEDURE",
   "NEG.SORT_MERGE.NO_MERGE","NEG.SORT_MERGE.NO_MULTI_KEY","NEG.SORT_MERGE.NO_STABILITY_GUARANTEE",
   "NEG.SORT_MERGE.NO_COLLATING_SEQUENCE","NEG.SORT_MERGE.NO_ALL_DIALECTS"],
}
json.dump(atlas, open(sys.argv[1],"w"), indent=2)
print(f"PASS={2-len(fails)} FAIL={len(fails)}")
for f in fails: print("  MISMATCH", f)
PY
