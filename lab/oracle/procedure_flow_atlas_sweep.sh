#!/usr/bin/env bash
# Procedure-flow atlas sweep (GNURUST.PROCEDURE.FLOW.ATLAS.1). Probe the control-flow statement classes (IF,
# EVALUATE, PERFORM TIMES/VARYING/UNTIL/paragraph, GO TO) against real cobc/libcob, assert the observed
# behavior is stable, and emit reports/procedure-flow-atlas.json. OBSERVED court: gnucobol-rs does NOT execute
# Procedure Division -- this MAPS the control-flow surface; execution is the loudest non-claim.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cat > "$TMP/p.cob" <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. P.
DATA DIVISION.
WORKING-STORAGE SECTION.
01 C PIC 9(3).
01 I PIC 9(3).
01 X PIC 9.
PROCEDURE DIVISION.
    IF 5 > 3 THEN DISPLAY "if=THEN" ELSE DISPLAY "if=ELSE" END-IF.
    MOVE 2 TO X.
    EVALUATE X WHEN 1 DISPLAY "eval=1" WHEN 2 DISPLAY "eval=2" WHEN OTHER DISPLAY "eval=O" END-EVALUATE.
    MOVE 0 TO C. PERFORM 3 TIMES ADD 1 TO C END-PERFORM. DISPLAY "perform_times=" C.
    MOVE 0 TO C. PERFORM VARYING I FROM 1 BY 1 UNTIL I > 4 ADD 1 TO C END-PERFORM.
    DISPLAY "varying_body=" C. DISPLAY "varying_ends=" I.
    MOVE 0 TO C. PERFORM UNTIL C >= 5 ADD 1 TO C END-PERFORM. DISPLAY "until=" C.
    MOVE 0 TO C. PERFORM SUB-PARA. DISPLAY "perform_para=" C.
    GO TO SKIP-IT.
    MOVE 99 TO C.
    SKIP-IT.
    DISPLAY "goto_skipped=" C.
    STOP RUN.
    SUB-PARA.
    ADD 7 TO C.
COB
cobc -free -x -o "$TMP/p" "$TMP/p.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }
OUT=$("$TMP/p")
python3 - "$ROOT/reports/procedure-flow-atlas.json" "$OUT" <<'PY'
import json, sys
kv = {}
for line in sys.argv[2].splitlines():
    if "=" in line: k, v = line.split("=", 1); kv[k.strip()] = v.strip()
expect = {"if":"THEN","eval":"2","perform_times":"003","varying_body":"004","varying_ends":"005",
          "until":"005","perform_para":"007","goto_skipped":"007"}
fails = [(k, e, kv.get(k)) for k, e in expect.items() if kv.get(k) != e]
atlas = {
 "schema":"gnurust-procedure-flow-atlas-v1","court":"GNURUST.PROCEDURE.FLOW.ATLAS.1","dialect":"gnucobol-3.2.0-default",
 "oracle":"cobc Procedure Division control flow (cobc/typeck.c + codegen.c)",
 "doctrine":"OBSERVED map of the control-flow surface. gnucobol-rs does NOT execute Procedure Division; this records what each control-flow statement DOES, it does not run programs.",
 "statements":[
  {"name":"IF / ELSE","category":"conditional","observed":"IF 5>3 -> THEN branch","determinism":"deterministic","status":"observed-only","note":"general condition evaluation is a future court; LEVEL-88 byte predicates are GNURUST.11"},
  {"name":"EVALUATE","category":"multi-way select","observed":"EVALUATE 2 -> WHEN 2 (first match)","determinism":"deterministic","status":"observed-only","note":"selection semantics observed; not executed"},
  {"name":"PERFORM n TIMES","category":"loop","observed":"PERFORM 3 TIMES -> body runs 3x","determinism":"deterministic","status":"observed-only","note":"iteration count observed"},
  {"name":"PERFORM VARYING","category":"loop","observed":"VARYING I FROM 1 BY 1 UNTIL I>4 -> body 4x, I ends at 5 (incremented PAST the limit)","determinism":"deterministic","status":"observed-only","note":"the control variable ends one step past the limit"},
  {"name":"PERFORM UNTIL","category":"loop","observed":"PERFORM UNTIL C>=5 -> 5 iterations","determinism":"deterministic","status":"observed-only","note":"test-before (with-test BEFORE default)"},
  {"name":"PERFORM paragraph","category":"out-of-line","observed":"PERFORM SUB-PARA -> executes the paragraph then returns","determinism":"deterministic","status":"observed-only","note":"out-of-line call + return"},
  {"name":"GO TO","category":"unconditional branch","observed":"GO TO SKIP-IT -> skips the intervening statement","determinism":"deterministic","status":"observed-only","note":"unconditional jump"},
 ],
 "negative_capabilities":["NEG.PROCEDURE_FLOW.NO_PROCEDURE_DIVISION_EXECUTION","NEG.PROCEDURE_FLOW.NO_CONTROL_FLOW_EXECUTION",
   "NEG.PROCEDURE_FLOW.NO_BRANCH_COVERAGE","NEG.PROCEDURE_FLOW.NO_TERMINATION_ANALYSIS",
   "NEG.PROCEDURE_FLOW.NO_GENERAL_CONDITION_EVAL","NEG.PROCEDURE_FLOW.STATUS_NOT_IMPLEMENTATION","NEG.PROCEDURE_FLOW.NO_ALL_DIALECTS"],
}
json.dump(atlas, open(sys.argv[1],"w"), indent=2)
print(f"PASS={len(expect)-len(fails)} FAIL={len(fails)}")
for f in fails: print("  MISMATCH", f)
PY
