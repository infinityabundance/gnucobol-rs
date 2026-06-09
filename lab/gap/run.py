#!/usr/bin/env python3
"""GNURUST.PUBLIC.GAP.1 (testsuite tier) — surface gap board over the ADMITTED GnuCOBOL upstream testsuite.

The companion to GNURUST.PUBLIC.CORPUS.1, run against corpus #1 (the authoritative GnuCOBOL testsuite, already
admitted in lab/admit -- zero network). It scans the .at run-tests for COBOL verbs/features, counts them, and
classifies each surface against the court map: SEALED (a court exists), OBSERVED (an atlas), REFUSED (a declared
negative capability), or MISSING (no court yet). The output is a brutally honest MISSING-COURT BOARD: what the
authoritative corpus exercises that the project does not yet admit.

This is a SURFACE-FREQUENCY scan (verb presence), NOT compilation / execution / parity. A verb appearing is not
proof a court is needed for full behavior, and the testsuite is ONE corpus. The full multi-corpus run+classify
remains future work (needs fetching the tier-2/3 corpora).
  run.py generate | check
"""
import json, os, re, sys, glob
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
TESTS = os.path.join(ROOT, "lab/admit/gnucobol-3.2/tests/testsuite.src")

# (regex, surface, classification, court-or-reason). classification: sealed/observed/refused/missing.
SURFACES = [
 (r"\bINITIALIZE\b", "INITIALIZE", "sealed", "GNURUST.INITIALIZE.1"),
 (r"\bINSPECT\b", "INSPECT", "sealed", "GNURUST.INSPECT.1"),
 (r"\b(STRING|UNSTRING)\b", "STRING/UNSTRING", "sealed", "GNURUST.STRING.UNSTRING.1"),
 (r"\b(ACCEPT|DISPLAY)\b", "ACCEPT/DISPLAY", "sealed", "GNURUST.ACCEPT.DISPLAY.1/.2"),
 (r"\b(COMPUTE|ADD|SUBTRACT|MULTIPLY)\b", "arithmetic", "sealed", "GNURUST.7/13"),
 (r"\bDIVIDE\b", "DIVIDE", "sealed", "GNURUST.19/REMAINDER.1"),
 (r"\bFUNCTION\b", "intrinsics", "sealed", "GNURUST.INTRINSIC.* (LENGTH/NUMVAL/INTEGER/MOD/REM/DATE/CASE/ORD-CHAR)"),
 (r"\bEVALUATE\b", "EVALUATE", "sealed", "GNURUST.IF.EVALUATE.SLICE.1 (bounded slice)"),
 (r"\bPERFORM\b", "PERFORM", "sealed", "GNURUST.PERFORM.SLICE.1 + TABLE.PERFORM.SLICE.1 (bounded slices)"),
 (r"\bMOVE\b", "MOVE", "sealed", "GNURUST.2 (decimal/display) + slices"),
 (r"\b(WRITE|READ|OPEN|CLOSE)\b", "sequential file I/O", "sealed", "GNURUST.FILE.SEQUENTIAL.1/WRITE.1"),
 (r"ORGANIZATION\s+IS\s+(RECORD|LINE)\s+SEQUENTIAL", "sequential org", "sealed", "GNURUST.FILE.SEQUENTIAL.1"),
 (r"\bGO\s+TO\b", "GO TO / procedure flow", "observed", "GNURUST.PROCEDURE.FLOW.ATLAS.1"),
 (r"FILE\s+STATUS", "file status", "observed", "GNURUST.FILE.STATUS.1"),
 (r"ON\s+SIZE\s+ERROR", "SIZE ERROR", "sealed", "GNURUST.SIZE.ERROR.1"),
 # --- MISSING surfaces (exercised by the authoritative corpus, no court yet) ---
 (r"\bCALL\b", "CALL / linkage", "observed", "GNURUST.CALL.EXTENSION.ATLAS.1"),
 (r"\b(SORT|MERGE)\b", "SORT / MERGE", "missing", "GNURUST.SORT.MERGE.ATLAS.1 (proposed)"),
 (r"\bSEARCH\b", "SEARCH (table lookup)", "missing", "GNURUST.SEARCH.TABLE.1 (proposed)"),
 (r"\bSTART\b", "START (indexed/relative key positioning)", "missing", "GNURUST.INDEXED.FILE.ATLAS.1 (proposed)"),
 (r"\bDELETE\b", "DELETE (indexed/relative)", "missing", "GNURUST.INDEXED.FILE.ATLAS.1 (proposed)"),
 (r"ORGANIZATION\s+IS\s+INDEXED", "indexed file org", "missing", "GNURUST.INDEXED.FILE.ATLAS.1 (proposed)"),
 (r"ORGANIZATION\s+IS\s+RELATIVE", "relative file org", "missing", "GNURUST.RELATIVE.FILE.ATLAS.1 (proposed)"),
 (r"\bSCREEN\s+SECTION\b", "SCREEN SECTION", "refused", "NEG (screen I/O out of the data-evidence lane)"),
 (r"\bRD\b|REPORT\s+SECTION", "REPORT WRITER", "refused", "NEG (report writer out of scope)"),
 (r"EXEC\s+SQL", "embedded SQL / DB2", "refused", "NEG.DB2.* / NEG.SQL.PRECOMPILER"),
 (r"EXEC\s+CICS", "CICS", "refused", "NEG.CICS.*"),
]

def scan():
    files = glob.glob(os.path.join(TESTS, "run_*.at")) + glob.glob(os.path.join(TESTS, "syn_*.at"))
    blob = ""
    for f in files:
        try: blob += open(f, errors="ignore").read().upper()
        except OSError: pass
    rows = []
    for rx, surface, cls, court in SURFACES:
        n = len(re.findall(rx, blob, re.IGNORECASE))
        rows.append({"surface": surface, "occurrences": n, "status": cls, "court": court})
    return files, rows

def build():
    files, rows = scan()
    missing = [r for r in rows if r["status"] == "missing" and r["occurrences"] > 0]
    return {
        "schema": "gnurust-public-gap-board-v1", "court": "GNURUST.PUBLIC.GAP.1",
        "corpus": "GnuCOBOL 3.2.0 upstream testsuite (admitted, lab/admit)",
        "scope": "surface-frequency scan of .at run/syn tests -- NOT compilation/execution/parity",
        "files_scanned": len(files),
        "counts": {"sealed": sum(1 for r in rows if r["status"]=="sealed"),
                   "observed": sum(1 for r in rows if r["status"]=="observed"),
                   "refused": sum(1 for r in rows if r["status"]=="refused"),
                   "missing": sum(1 for r in rows if r["status"]=="missing")},
        "missing_court_board": sorted([{"surface": r["surface"], "occurrences": r["occurrences"], "proposed_court": r["court"]}
                                       for r in missing], key=lambda x: -x["occurrences"]),
        "surfaces": sorted(rows, key=lambda r: -r["occurrences"]),
        "negative_capabilities": ["NEG.PUBLIC_GAP.NOT_EXECUTION", "NEG.PUBLIC_GAP.NOT_PARITY",
            "NEG.PUBLIC_GAP.SINGLE_CORPUS", "NEG.PUBLIC_GAP.VERB_PRESENCE_NOT_COURT_NEED",
            "NEG.PUBLIC_GAP.MISSING_IS_CANDIDATE_NOT_COMMITMENT"],
    }

def generate():
    b = build(); json.dump(b, open(os.path.join(ROOT, "reports/public-gap-board.json"), "w"), indent=2)
    print(f"gap board: {b['files_scanned']} files; surfaces sealed {b['counts']['sealed']} observed {b['counts']['observed']} refused {b['counts']['refused']} missing {b['counts']['missing']}")
    print("MISSING-COURT BOARD (authoritative corpus exercises these, no court yet):")
    for m in b["missing_court_board"]: print(f"   {m['occurrences']:>5}x  {m['surface']:<42} -> {m['proposed_court']}")

def check():
    b = build()
    if b["files_scanned"] == 0:
        print("GATE: admitted GnuCOBOL testsuite not found"); return 1
    if not b["missing_court_board"]:
        print("GATE: gap board empty -- the scan found no missing surfaces (suspicious; check the corpus path)"); return 1
    print(f"GNURUST.PUBLIC.GAP.1: {b['files_scanned']} testsuite files scanned; {len(b['missing_court_board'])} missing surfaces on the board; surface-scan only")
    return 0

if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "check"
    {"generate": generate, "check": lambda: sys.exit(check())}.get(cmd, lambda: sys.exit("usage: generate|check"))()
