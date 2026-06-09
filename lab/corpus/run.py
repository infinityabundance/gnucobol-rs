#!/usr/bin/env python3
"""GNURUST.PUBLIC.CORPUS.1 — admitted public-COBOL corpus index for GAP DISCOVERY (not parity).

A curated index of public COBOL / GnuCOBOL corpora used as a HOSTILE WITNESS SET: real public terrain to grow
the court map against. This court admits the INDEX ONLY -- it does not fetch, compile, run, or claim parity over
any corpus. Running the tools over the corpus and classifying every failure is the future companion campaign
GNURUST.PUBLIC.GAP.1. Licenses/commits/features are recorded as declared flags at index time, not verified.

  run.py generate | check
"""
import json, os, sys
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# tier: 1 = GnuCOBOL-runnable (locally / minor harness) · 2 = mainframe-realistic (CICS/DB2/VSAM, many won't run
#        -- that is the point) · 3 = research/mining (statistical blind-spot discovery).
# features: declared from each corpus's description (NOT a code scan). status: "indexed" (not fetched/run).
CORPUS = [
 # id, name, url, tier, priority(best-first-10 order or None), features, note
 ("gnucobol-testsuite", "GnuCOBOL upstream testsuite", "gnucobol (sourceforge) tests/testsuite.src", 1, 1,
  ["sequential","line-sequential","file-status","reports","accept-display","intrinsics","linage","edge-cases"],
  "source-of-truth behavior mine; oracle-authoritative witnesses (run_file.at etc.)"),
 ("dsc-gnu-indexed", "dscobol/dsc-gnu-Indexed", "github.com/dscobol/dsc-gnu-Indexed", 1, 2,
  ["indexed","sequential","copybooks","crud"], "indexed-file atlas seed (GNURUST.INDEXED.FILE.ATLAS.1)"),
 ("cobol-unit-test", "neopragma/cobol-unit-test", "github.com/neopragma/cobol-unit-test", 1, 3,
  ["line-sequential","copybooks","string-unstring","search","call-linkage","table-redefines-occurs"],
  "UNSTRING + CALL extensions (C$TOUPPER/C$JUSTIFY) + SEARCH ALL + table REDEFINES/OCCURS witness"),
 ("gc-examples", "dinodev76/GC-Examples", "github.com/dinodev76/GC-Examples", 1, 4,
  ["copybooks","sample-data","indexed","relative","rdw-variable-length"], "GnuCOBOL file formats; relative/indexed/RDW"),
 ("writ3it-examples", "writ3it/cobol-examples", "github.com/writ3it/cobol-examples", 1, 5,
  ["line-sequential","indexed","dynamic-access","record-key"], "simple GnuCOBOL file/indexed workflows"),
 ("gnucobol-samples", "JohnDovey/GNUCobol-Samples", "github.com/JohnDovey/GNUCobol-Samples", 1, 6,
  ["intrinsics","file-status","screen-section","copybooks"], "intrinsics / file-status / screen refusal witnesses"),
 ("dscobol-cobol-code", "dscobol/Cobol-Code", "github.com/dscobol/Cobol-Code", 1, 7,
  ["sql-db2","indexed","internal-sort","copybooks","mixed-dialect"], "GnuCOBOL vs IBM Enterprise comparative corpus"),
 ("aws-carddemo", "AWS CardDemo", "github.com/aws-samples/aws-mainframe-modernization-carddemo", 2, 8,
  ["jcl","sql-db2","mq","ims","sample-data","copybooks","reports","call-linkage"], "flagship realistic modernization corpus"),
 ("ibm-cbsa", "IBM CICS Banking Sample (CBSA)", "github.com/cicsdev/cics-banking-sample-application-cbsa", 2, 9,
  ["cics","bms","sql-db2","copybooks","call-linkage"], "serious CICS/banking application; limits of static data courts"),
 ("ibm-zopeneditor", "IBM/zopeneditor-sample", "github.com/IBM/zopeneditor-sample", 2, 10,
  ["jcl","sample-data","pli","hlasm","rexx","copybooks"], "COBOL/JCL/data + IBM tooling assumptions"),
 # remainder (no best-first priority yet)
 ("zbank", "Nantero1/zBANK", "github.com/Nantero1/zBANK", 2, None, ["cics","bms","vsam-ksds"], "VSAM/CICS refusal + future atlas target"),
 ("dscobol-projects", "dscobol/Cobol-Projects", "github.com/dscobol/Cobol-Projects", 1, None, ["vsam","tk4","mixed-dialect"], "archaeology / feature mining"),
 ("gem-gnu-db2", "dscobol/gem-gnu-DB2", "github.com/dscobol/gem-gnu-DB2", 1, None, ["sql-db2","host-variables"], "DB2 host-variable / SQL precompiler refusal seed"),
 ("michelou-examples", "michelou/cobol-examples", "github.com/michelou/cobol-examples", 1, None, ["build-profile-variance","windows-file-semantics"], "GnuCOBOL 3.2 dialect/build variance"),
 ("shamrice-examples", "shamrice/COBOL-Examples", "github.com/shamrice/COBOL-Examples", 1, None, ["syntax","features"], "lightweight smoke corpus"),
 ("neopragma-samples", "neopragma/cobol-samples", "github.com/neopragma/cobol-samples", 1, None, ["small-tasks"], "isolated court probes"),
 ("accounting-system", "tjsingh85/cobol-accounting-system", "github.com/tjsingh85/cobol-accounting-system", 1, None, ["accounting","control-totals","numeric-role"], "numeric-shape-is-not-money checks"),
 ("core-banking", "fzn0x/core-banking-system", "github.com/fzn0x/core-banking-system", 1, None, ["copybooks","fixed-record","accounts"], "fixed-record banking/account decode"),
 ("payroll-mgmt", "Payroll_Management_System", "github.com/JAGADISHSUNILPEDNEKAR/Payroll_Management_System", 1, None, ["accept-display","reports","numeric-rounding"], "salary calc / rounding / reports"),
 ("legacy-ledger", "COBOL Legacy Ledger", "github.com/albertdobmeyer/cobol-legacy-ledger", 1, None, ["ledger","settlement","integrity-wrapper"], "transaction/ledger refusal-boundary tests (treat cautiously)"),
 ("benchmark-suite", "COBOL Legacy Benchmark Suite (Investment Portfolio)", "github.com/sentientsergio/COBOL-Legacy-Benchmark-Suite", 1, None, ["generated","app-shape-stress"], "broad app-shape stress (lower authority)"),
 ("datastage-mainframe", "IBM DataStage standalone workshop (mainframe example)", "ibm.github.io/datastage-standalone-workshop", 2, None, ["ebcdic","binary","copybooks","mainframe-binary-data"], "EBCDIC/binary/copybook fixed-record decode comparator"),
 ("aws-data-utilities", "AWS mainframe-data-utilities", "github.com/aws-samples/mainframe-data-utilities", 3, None, ["copybook-ebcdic-utility"], "adversarial comparator for KOBOLD decode (not a program corpus)"),
 ("x-cobol", "X-COBOL dataset", "arxiv.org/abs/2306.04892", 3, None, ["84-repos","1255-files","surface-mining"], "statistical surface mining: unsupported verbs / COPY patterns / dialect clues"),
 ("opencbs", "OpenCBS defect benchmark", "arxiv.org/abs/2206.06260", 3, None, ["defects","behavioral-traps"], "behavioral-trap discovery, not just syntax"),
]

# the future companion campaign + accepted-but-separate next courts (recorded as owning campaigns, not done here)
FUTURE = {
 "GNURUST.PUBLIC.GAP.1": "run the tools over the corpus + classify every failure (missing/refused/parser/dialect/build-profile/file-runtime/data-custody gap)",
 "GNURUST.BUILD.PROFILE.1": "bind cobc version/config/binary-byteorder/binary-size/COMP-5/COMP-X/libcob sha/target-triple/endianness as first-class ABI evidence; every ABI-sensitive court cites it",
 "GNURUST.INDEXED.FILE.ATLAS.1": "observed indexed-file (ISAM/BDB/VBISAM) atlas -- later, after the Procedure Division spine",
}

# behavioral-parity ladder (the accepted axis ABOVE the coverage map): where the current work sits.
BEHAVIORAL_LADDER = [
 (0, "static layout / representation", "GNURUST.3/4/9/10"),
 (1, "storage decode / encode", "GNURUST.2/8/14/15/16/17/18"),
 (2, "expression / arithmetic receiver bytes", "GNURUST.7/13/19/REMAINDER.1, INTRINSIC.*"),
 (3, "isolated Procedure Division statement byte effects", "INITIALIZE/INSPECT/STRING-UNSTRING/ACCEPT-DISPLAY/SIZE.ERROR"),
 (4, "file / status / runtime side effects", "FILE.SEQUENTIAL/WRITE/REWRITE/STATUS"),
 (5, "procedure-flow branch atlas", "PROCEDURE.FLOW.ATLAS.1"),
 (6, "bounded trace replay", "IF/EVALUATE + PERFORM + TABLE + FILE.FLOW/FILTER slices (current frontier)"),
 (7, "bounded interpreter slice", "(composing the slices -- next)"),
 (8, "multi-module runtime parity", "(not started; the summit)"),
]

def build():
    return {
        "schema": "gnurust-public-corpus-index-v1", "court": "GNURUST.PUBLIC.CORPUS.1",
        "doctrine": "An admitted index of public COBOL/GnuCOBOL corpora as a hostile witness set for GAP DISCOVERY. "
                    "Admits the INDEX ONLY: no corpus is fetched, compiled, run, or claimed for parity here. "
                    "Running + classifying is the future campaign GNURUST.PUBLIC.GAP.1.",
        "corpus_count": len(CORPUS),
        "tiers": {"1_gnucobol_runnable": sum(1 for c in CORPUS if c[3]==1),
                  "2_mainframe_realistic": sum(1 for c in CORPUS if c[3]==2),
                  "3_research_mining": sum(1 for c in CORPUS if c[3]==3)},
        "best_first_10": [c[0] for c in sorted([c for c in CORPUS if c[4]], key=lambda c: c[4])],
        "corpora": [{"id": c[0], "name": c[1], "url": c[2], "tier": c[3], "priority": c[4],
                     "features": c[5], "note": c[6], "license": "unverified-at-index-time",
                     "commit": "unverified-at-index-time", "status": "indexed"} for c in CORPUS],
        "future_campaigns": FUTURE,
        "behavioral_parity_ladder": [{"level": l, "name": n, "current": cur} for (l, n, cur) in BEHAVIORAL_LADDER],
        "negative_capabilities": ["NEG.PUBLIC_CORPUS.NOT_PARITY", "NEG.PUBLIC_CORPUS.NOT_FETCHED",
            "NEG.PUBLIC_CORPUS.NOT_RUN", "NEG.PUBLIC_CORPUS.LICENSE_NOT_LEGAL_ADVICE",
            "NEG.PUBLIC_CORPUS.FEATURES_DECLARED_NOT_VERIFIED", "NEG.PUBLIC_CORPUS.NOT_EXHAUSTIVE"],
    }

def generate():
    b = build(); json.dump(b, open(os.path.join(ROOT, "reports/public-corpus-index.json"), "w"), indent=2)
    print(f"public corpus index: {b['corpus_count']} corpora ({b['tiers']}); best-first {len(b['best_first_10'])}")

def check():
    b = build(); bad = 0
    seen = set()
    for c in b["corpora"]:
        if c["id"] in seen: print("GATE: duplicate corpus id", c["id"]); bad += 1
        seen.add(c["id"])
        if not c["url"] or not c["features"]: print("GATE: corpus row incomplete", c["id"]); bad += 1
    # best-first-10 must be exactly 10 and a prefix of priorities 1..10
    if len(b["best_first_10"]) != 10: print("GATE: best_first_10 is not 10"); bad += 1
    if bad: print(f"!! {bad} corpus-index finding(s)"); return 1
    print(f"GNURUST.PUBLIC.CORPUS.1: {b['corpus_count']} corpora indexed, {len(b['best_first_10'])} prioritized; index-only (not fetched/run)")
    return 0

if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "check"
    {"generate": generate, "check": lambda: sys.exit(check())}.get(cmd, lambda: sys.exit("usage: generate|check"))()
