#!/usr/bin/env bash
# FILE STATUS atlas sweep (GNURUST.FILE.STATUS.1). Trigger declared file-operation conditions against real
# cobc/libcob and record the observed FILE STATUS bytes. OBSERVED court: the pure kernel does no I/O, so it
# does not produce open/close statuses -- this records which status arises from which condition and asserts
# the observed values are stable. Generates reports/file-status-atlas.json and PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

cat > "$TMP/fs.cob" <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. FS.
ENVIRONMENT DIVISION.
INPUT-OUTPUT SECTION.
FILE-CONTROL.
    SELECT F ASSIGN TO "DDIN" ORGANIZATION IS LINE SEQUENTIAL FILE STATUS IS WS.
DATA DIVISION.
FILE SECTION.
FD F.
01 R PIC X(8).
WORKING-STORAGE SECTION.
01 WS PIC XX.
PROCEDURE DIVISION.
    OPEN INPUT F.
    DISPLAY "open_input=" WS.
    IF WS = "00"
       READ F NEXT AT END CONTINUE END-READ
       DISPLAY "read_first=" WS
       PERFORM UNTIL WS = "10"
          READ F NEXT AT END CONTINUE END-READ
       END-PERFORM
       DISPLAY "read_at_eof=" WS
       READ F NEXT AT END CONTINUE END-READ
       DISPLAY "read_past_eof=" WS
    END-IF.
    CLOSE F.
    DISPLAY "close=" WS.
    STOP RUN.
COB
cobc -free -x -o "$TMP/fs" "$TMP/fs.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }

printf 'AB\nCD\n' > "$TMP/valid.dat"
VALID=$(DDIN="$TMP/valid.dat" "$TMP/fs")
MISSING=$(DDIN="$TMP/does-not-exist.dat" "$TMP/fs")

python3 - "$ROOT/reports/file-status-atlas.json" <<PY
import json, sys, os
def kv(text):
    d = {}
    for line in text.splitlines():
        if "=" in line:
            k, v = line.split("=", 1); d[k.strip()] = v.strip()
    return d
valid = kv('''$VALID''')
missing = kv('''$MISSING''')
# expected observed statuses for each declared condition
expect = {
 ("valid","open_input"): "00", ("valid","read_first"): "00", ("valid","read_at_eof"): "10",
 ("valid","read_past_eof"): "46", ("valid","close"): "00",
 ("missing","open_input"): "35", ("missing","close"): "42",
}
obs = {("valid",k): v for k,v in valid.items()} | {("missing",k): v for k,v in missing.items()}
fails = [(f, e, obs.get(f)) for f,e in expect.items() if obs.get(f) != e]

atlas = {
 "schema":"gnurust-file-status-atlas-v1","court":"GNURUST.FILE.STATUS.1",
 "oracle":"cobc OPEN INPUT/READ NEXT/CLOSE (libcob/fileio.c)","dialect":"gnucobol-3.2.0-default",
 "admitted_statuses":[
   {"code":"00","meaning":"success","condition":"OPEN INPUT of an existing file; a successful READ NEXT","observed_in":"valid"},
   {"code":"06","meaning":"LINE SEQUENTIAL record truncated (line longer than the record); delivered in chunks","condition":"READ NEXT of a line longer than the record length","observed_in":"GNURUST.FILE.SEQUENTIAL.1"},
   {"code":"10","meaning":"end of file","condition":"READ NEXT at end of file","observed_in":"valid"},
   {"code":"35","meaning":"file not found","condition":"OPEN INPUT of a non-existent file","observed_in":"missing"},
   {"code":"42","meaning":"CLOSE attempted on a file not open","condition":"CLOSE after a failed OPEN","observed_in":"missing"},
   {"code":"46","meaning":"READ NEXT with no valid next record","condition":"READ NEXT after AT END (past EOF)","observed_in":"valid"},
 ],
 "not_admitted_statuses":[
   {"code":"30","reason":"permanent host I/O error is environment-weather (filesystem-dependent); not deterministically reproducible in the sealed flat-file sequential subset"},
   {"code":"39","reason":"OPEN attribute conflict does not arise on flat Unix sequential files (no stored record/organization metadata); not reproducible here"},
 ],
 "negative_capabilities":["NEG.FILE_STATUS.NOT_FULL_FILE_IO","NEG.FILE_STATUS.NO_INDEXED_RELATIVE_VSAM",
   "NEG.FILE_STATUS.NO_LOCKING_SHARING","NEG.FILE_STATUS.NO_HOST_ERROR_GENERALIZATION",
   "NEG.FILE_STATUS.NO_PROCEDURE_FLOW","NEG.FILE_STATUS.NO_BUSINESS_COMPLETENESS"],
}
json.dump(atlas, open(sys.argv[1],"w"), indent=2)
print(f"PASS={len(expect)-len(fails)} FAIL={len(fails)}")
for f in fails: print("  MISMATCH", f)
PY
