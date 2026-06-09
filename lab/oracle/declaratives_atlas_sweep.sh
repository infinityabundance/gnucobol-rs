#!/usr/bin/env bash
# GNURUST.DECLARATIVES.ATLAS.1 -- observe the runtime behavior of a USE AFTER STANDARD ERROR PROCEDURE
# declarative on file I/O. DECLARATIVES are one of COBOL's most under-specified corners; this maps the
# witnessed runtime semantics under gnucobol-3.2.0-default:
#   (1) a FAILING file op invokes that file's USE declarative (OPEN missing -> status 35; CLOSE-not-open
#       -> status 42), a SUCCESSFUL op invokes nothing,
#   (2) the binding is PER-FILE (file F's errors run F's section, never G's),
#   (3) FILE STATUS is VISIBLE inside the declarative (the per-op code),
#   (4) after the declarative, execution RESUMES at the statement after the failed I/O (rc=0, normal end).
# OBSERVED court: gnucobol-rs executes NO Procedure Division and runs NO declaratives (that is the L8
# multi-statement-execution summit, NOT CLAIMED). This MAPS the surface from real cobc/libcob output.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8 TERM=dumb
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cd "$TMP"
rm -f missing.dat exists.dat
cat > decl.cob <<'COB'
       IDENTIFICATION DIVISION.
       PROGRAM-ID. DECL.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT F ASSIGN TO "missing.dat"
               ORGANIZATION IS LINE SEQUENTIAL FILE STATUS IS FSF.
           SELECT G ASSIGN TO "exists.dat"
               ORGANIZATION IS LINE SEQUENTIAL FILE STATUS IS FSG.
       DATA DIVISION.
       FILE SECTION.
       FD F.
       01 RECF PIC X(10).
       FD G.
       01 RECG PIC X(10).
       WORKING-STORAGE SECTION.
       01 FSF PIC XX VALUE "00".
       01 FSG PIC XX VALUE "00".
       PROCEDURE DIVISION.
       DECLARATIVES.
       ERR-F SECTION.
           USE AFTER STANDARD ERROR PROCEDURE ON F.
           DISPLAY "DECL-F fs=" FSF.
       ERR-G SECTION.
           USE AFTER STANDARD ERROR PROCEDURE ON G.
           DISPLAY "DECL-G fs=" FSG.
       END DECLARATIVES.
       MAIN SECTION.
           OPEN INPUT F.
           OPEN OUTPUT G.
           MOVE "HELLO" TO RECG.
           WRITE RECG.
           CLOSE F.
           CLOSE G.
           DISPLAY "REACHED-END".
           STOP RUN.
COB
cobc -free -x -o decl decl.cob 2>/dev/null || { echo "compile failed"; exit 2; }
OUT="$(./decl 2>/dev/null)"; RC=$?

OUT="$OUT" RC="$RC" ROOT="$ROOT" python3 - <<'PY'
import os
out = os.environ["OUT"]; rc = os.environ["RC"]; ROOT = os.environ["ROOT"]
checks = [
 ("open-failure-fires-decl", "DECL-F fs=35" in out, "OPEN INPUT of a missing file (status 35) invokes file F's USE declarative"),
 ("close-failure-fires-decl", "DECL-F fs=42" in out, "CLOSE of a not-open file (status 42) re-invokes the declarative"),
 ("success-fires-nothing", "DECL-G" not in out, "the successful ops on file G (OPEN OUTPUT/WRITE/CLOSE) invoke NO declarative"),
 ("status-visible-inside", "fs=35" in out and "fs=42" in out, "FILE STATUS holds the per-op code inside the declarative"),
 ("execution-resumes", "REACHED-END" in out and rc == "0", "after each declarative, execution RESUMES at the next statement; program ends rc=0"),
]
fails = [(n,why) for (n,ok,why) in checks if not ok]
import json
atlas = {
 "schema":"gnurust-declaratives-atlas-v1","court":"GNURUST.DECLARATIVES.ATLAS.1","dialect":"gnucobol-3.2.0-default",
 "oracle":"cobc DECLARATIVES / USE AFTER STANDARD ERROR PROCEDURE (cobc/typeck.c + libcob/fileio.c error path)",
 "doctrine":"OBSERVED map of the runtime behavior of a USE AFTER STANDARD ERROR PROCEDURE declarative on file I/O -- one of COBOL's most under-specified corners. gnucobol-rs executes NO Procedure Division and runs NO declaratives (the L8 multi-statement-execution summit, NOT CLAIMED); this records the witnessed semantics: which op fires it, per-file binding, FILE STATUS visibility, and resume-after.",
 "trace": out.split("\n"),
 "return_code": rc,
 "observations":[
  {"name":"failing I/O invokes the file's USE declarative","observed":"a failing file op runs that file's USE AFTER STANDARD ERROR section (OPEN of a missing file -> status 35 fires it; CLOSE of a not-open file -> status 42 re-fires it); a SUCCESSFUL op fires nothing","status":"observed-only"},
  {"name":"per-file binding","observed":"the declarative is bound PER FILE -- file F's errors run F's ERR-F section, never G's ERR-G; the successful file G never triggers its own section","status":"observed-only"},
  {"name":"FILE STATUS visible inside","observed":"inside the declarative the file's FILE STATUS data item holds the current per-op code (35 then 42), so the handler can branch on it","status":"observed-only"},
  {"name":"resume after declarative","observed":"after the declarative returns, execution RESUMES at the statement following the failed I/O (it does NOT abort); the program reaches its end and terminates rc=0","status":"observed-only"},
 ],
 "negative_capabilities":["NEG.DECLARATIVES.NO_DECLARATIVE_EXECUTION","NEG.DECLARATIVES.NO_USE_FOR_DEBUGGING",
   "NEG.DECLARATIVES.NO_NONFILE_EXCEPTIONS","NEG.DECLARATIVES.NO_GLOBAL_DECLARATIVES",
   "NEG.DECLARATIVES.NO_MULTI_DECLARATIVE_PRECEDENCE","NEG.DECLARATIVES.NO_RESUME_FOR_NONFILE",
   "NEG.DECLARATIVES.NO_ALL_DIALECTS"],
}
json.dump(atlas, open(os.path.join(ROOT, "reports/declaratives-atlas.json"), "w"), indent=2)
print(f"PASS={len(checks)-len(fails)} FAIL={len(fails)}")
for n,why in fails: print("  MISMATCH", n, "--", why)
PY
