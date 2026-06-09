#!/usr/bin/env python3
"""GNURUST.COVERAGE.1 — forensic coverage map of the GnuCOBOL semantic surface.

A HARD RECEIPT, not a passive roadmap. Every declared semantic surface binds to real admitted GnuCOBOL 3.2
source module(s) and to its current court(s) / refusal(s) / future campaign, with a status
(sealed | observed | negative | missing) and a risk. The gate FAILS if (a) an admitted claim-ladder
GNURUST/atlas court is not mapped to any surface, (b) a surface declares a court that is not in the
claim-ladder, (c) a 'sealed' surface has no real court, (d) a surface binds to a source module that does not
exist, or (e) the committed map != a fresh re-derive.

  run.py generate   # write reports/gnurust-coverage.{json,md}
  run.py check      # the gate above
"""
import glob, json, os, sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SRC = os.path.join(ROOT, "lab/admit/gnucobol-3.2")

# Each surface: (id, category, [source modules], status, [courts], future_campaign|None, risk)
SURFACES = [
 ("data-representation", "data representation", ["libcob/move.c","libcob/numeric.c","libcob/common.c"], "sealed",
   ["GNURUST.2","GNURUST.14","GNURUST.18"], None, "wrong decoded bytes silently corrupt every downstream value"),
 ("field-model", "data representation", ["cobc/field.c","cobc/typeck.c"], "sealed",
   ["GNURUST.3","GNURUST.9"], None, "a wrong PIC->{type,digits,scale,size} mislays every field"),
 ("record-layout", "data representation", ["cobc/typeck.c","cobc/tree.c"], "sealed",
   ["GNURUST.4","GNURUST.10"], None, "a wrong offset/ODO size shifts the whole record"),
 ("copybook-expansion", "source", ["cobc/pplex.c","cobc/ppparse.c","cobc/replace.c"], "sealed",
   ["GNURUST.5","GNURUST.6"], None, "a wrong COPY/REPLACING expansion changes the layout before decode"),
 ("move-storage", "MOVE/storage", ["libcob/move.c"], "sealed", ["GNURUST.2"], None,
   "a wrong MOVE store/truncation/sign corrupts the destination bytes"),
 ("arithmetic", "arithmetic", ["libcob/numeric.c"], "sealed",
   ["GNURUST.7","GNURUST.13","GNURUST.19","GNURUST.REMAINDER.1"], None,
   "a wrong receiver scale/rounding/sign misstates money"),
 ("value-initialization", "data representation", ["cobc/field.c"], "sealed", ["GNURUST.8"], None,
   "a wrong VALUE image gives a record the wrong initial bytes"),
 ("conditions-level88", "conditions", ["cobc/typeck.c"], "sealed", ["GNURUST.11","GNURUST.12"], None,
   "a wrong LEVEL-88 truth or SET flips a branch"),
 ("codepage-ebcdic", "data representation", ["libcob/common.c","libcob/move.c"], "sealed",
   ["GNURUST.15","GNURUST.17"], None, "a wrong EBCDIC table mis-decodes text and zoned numerics"),
 ("edited-pictures", "MOVE/storage", ["libcob/move.c"], "sealed", ["GNURUST.16"], None,
   "a wrong edited decode mis-recovers a presentation value"),
 ("dialect-options", "compiler dialect options", ["cobc/config.c"], "sealed", ["DIALECT.PROFILE.1"], None,
   "an unrecorded dialect makes 'GnuCOBOL says' ambiguous"),
 ("size-error", "runtime exceptions", ["libcob/numeric.c"], "sealed", ["SIZE.ERROR.ATLAS.1", "GNURUST.SIZE.ERROR.1"], None,
   "overflow writes truncated / divide-by-zero preserves the receiver, both WITHOUT signal — silent corruption"),
 ("file-io-sequential", "file I/O", ["libcob/fileio.c"], "sealed", ["GNURUST.FILE.SEQUENTIAL.1", "GNURUST.FILE.WRITE.1", "GNURUST.FILE.REWRITE.1"], None,
   "sequential/line READ record bytes + AT END drive most batch COBOL logic"),
  ("file-io-indexed", "file I/O (keyed)", ["libcob/fileio.c"], "observed", ["GNURUST.INDEXED.FILE.ATLAS.1"], None,
   "indexed/keyed file I/O -- the largest remaining gap cluster; backend-specific on-disk ISAM/BDB format observed, not implemented"),
 ("file-io-relative", "file I/O (relative)", ["libcob/fileio.c"], "observed", ["GNURUST.RELATIVE.FILE.ATLAS.1"], None,
   "relative (by record number) file I/O observed; backend slotted format not implemented"),
("file-status", "file I/O", ["libcob/fileio.c"], "observed", ["GNURUST.FILE.STATUS.1"], None,
   "real COBOL branches on file-status codes (00/10/35/...); observed atlas of which status arises from which condition"),
 ("initialize", "MOVE/storage", ["cobc/typeck.c"], "sealed", ["GNURUST.INITIALIZE.1"], None,
   "INITIALIZE group/FILLER/REDEFINES/OCCURS defaults are easy to get wrong"),
 ("inspect", "MOVE/storage", ["libcob/strings.c"], "sealed", ["GNURUST.INSPECT.1"], None,
   "INSPECT TALLYING/REPLACING/CONVERTING is classic data-munging with real byte effects"),
 ("string-unstring", "MOVE/storage", ["libcob/strings.c"], "sealed", ["GNURUST.STRING.UNSTRING.1"], None,
   "STRING/UNSTRING pointer/overflow/delimiter byte effects are high migration value"),
 ("intrinsics", "intrinsics", ["libcob/intrinsic.c"], "observed", ["GNURUST.INTRINSIC.ATLAS.1", "GNURUST.INTRINSIC.LENGTH.1", "GNURUST.INTRINSIC.NUMVAL.1", "GNURUST.INTRINSIC.MOD-REM.1", "GNURUST.INTRINSIC.INTEGER.1", "GNURUST.INTRINSIC.CASE.1", "GNURUST.INTRINSIC.ORD-CHAR.1", "GNURUST.INTRINSIC.NUMVAL-C.1", "GNURUST.INTRINSIC.DATE.1"], None,
   "NUMVAL/LENGTH/MOD/CURRENT-DATE etc.; observed atlas + per-intrinsic implementation courts (LENGTH sealed)"),
 ("accept-display", "ACCEPT/DISPLAY", ["libcob/termio.c"], "sealed", ["GNURUST.ACCEPT.DISPLAY.1", "GNURUST.ACCEPT.DISPLAY.2"], None,
   "emitted DISPLAY text + ACCEPT is runtime evidence too"),
 ("procedure-flow", "control flow", ["cobc/typeck.c","cobc/codegen.c"], "observed", ["GNURUST.PROCEDURE.FLOW.ATLAS.1", "GNURUST.IF.EVALUATE.SLICE.1", "GNURUST.IF.NUMERIC.SLICE.1", "GNURUST.PERFORM.SLICE.1", "GNURUST.TABLE.PERFORM.SLICE.1", "GNURUST.SEARCH.TABLE.1", "GNURUST.FILE.FLOW.SLICE.1", "GNURUST.FILE.FILTER.SLICE.1"], None,
   "IF/EVALUATE/PERFORM/GO TO control flow is the bulk of unported Procedure Division; observed atlas, execution NOT claimed"),
 ("call-linkage", "CALL/linkage", ["libcob/call.c"], "observed", ["GNURUST.CALL.EXTENSION.ATLAS.1"], None,
   "CALL/linkage/USING is a large surface; refused until receipt-backed"),
 ("sort-merge", "SORT/MERGE", ["libcob/fileio.c"], "observed", ["GNURUST.SORT.MERGE.ATLAS.1"], None,
   "SORT/MERGE is its own runtime; refused until receipt-backed"),
 ("screen-section", "screen/report/CICS/SQL unsupported", ["libcob/screenio.c"], "negative", [], None,
   "SCREEN SECTION is terminal UI; out of the data-evidence lane"),
 ("report-writer", "screen/report/CICS/SQL unsupported", ["libcob/reportio.c"], "negative", [], None,
   "REPORT WRITER is a presentation engine; out of lane"),
 ("ml-io-xml-json", "screen/report/CICS/SQL unsupported", ["libcob/mlio.c"], "negative", [], None,
   "XML/JSON GENERATE/PARSE is out of the fixed-record lane"),
 ("diagnostics", "diagnostics", ["cobc/error.c"], "negative", [], None,
   "compiler message parity is not claimed"),
 ("cics-sql-preprocessor", "screen/report/CICS/SQL unsupported", ["cobc/ppparse.c"], "negative", [], None,
   "EXEC CICS / EXEC SQL are precompiler surfaces; refused (NEG.CICS.*, NEG.SQL.PRECOMPILER)"),
]
STATUSES = ("sealed", "observed", "negative", "missing")

def claim_ladder_ids():
    return {c["id"] for c in json.load(open(os.path.join(ROOT, "reports/claim-ladder.json")))["courts"]}

def admitted_gnucobol_courts():
    return {c for c in claim_ladder_ids() if (c.startswith("GNURUST.") or "ATLAS" in c) and c not in ("GNURUST.COVERAGE.1", "GNURUST.PUBLIC.CORPUS.1", "GNURUST.BUILD.PROFILE.1", "GNURUST.PUBLIC.GAP.1")}

def build():
    cl = claim_ladder_ids()
    rows = []
    for (sid, cat, mods, status, courts, fut, risk) in SURFACES:
        mod_ok = [m for m in mods if os.path.exists(os.path.join(SRC, m))]
        rows.append({"surface": sid, "category": cat, "source_modules": mods,
                     "source_modules_present": mod_ok == mods, "status": status,
                     "courts": courts, "future_campaign": fut, "risk_if_unported": risk})
    counts = {s: sum(1 for r in rows if r["status"] == s) for s in STATUSES}
    total = len(rows)
    return {
        "schema": "gnurust-coverage-v1", "court": "GNURUST.COVERAGE.1",
        "doctrine": "Forensic port: every GnuCOBOL semantic surface is bound to admitted source + a court/refusal/future campaign with a status. This is the honest map of the port -- the data-representation spine is sealed; file I/O, intrinsics, and most of Procedure Division are not.",
        "oracle_source": "lab/admit/gnucobol-3.2 (admitted GnuCOBOL 3.2 source)",
        "surface_count": total, "status_counts": counts,
        "sealed_fraction": f"{counts['sealed']}/{total}",
        "honest_note": "sealed surfaces are the data-representation + fixed-record spine; the file I/O, runtime-statement, intrinsic, and control-flow surfaces are mostly missing. This is NOT a near-complete port of GnuCOBOL.",
        "surfaces": rows,
        "non_claims": ["NEG.COVERAGE.NOT_A_COMPLETENESS_CLAIM","NEG.COVERAGE.SURFACE_LIST_NOT_EXHAUSTIVE",
                       "NEG.COVERAGE.STATUS_NOT_QUALITY","NEG.COVERAGE.NO_NEW_TRUTH"],
    }

def render_md(c):
    sc = c["status_counts"]
    badge = {"sealed":"✅","observed":"🟡","negative":"⛔","missing":"❌"}
    lines = ["<!-- generated by lab/coverage/run.py — do not edit by hand -->","",
             "# GNURUST.COVERAGE.1 — forensic coverage map of GnuCOBOL","",
             "> [!IMPORTANT]","> "+c["honest_note"],"",
             f"- surfaces: **{c['surface_count']}**  ·  sealed ✅ **{sc['sealed']}**  ·  observed 🟡 {sc['observed']}  ·  "
             f"refused ⛔ {sc['negative']}  ·  **missing ❌ {sc['missing']}**","",
             f"- sealed fraction (data-representation spine): **{c['sealed_fraction']}**","",
             "| surface | category | source | status | courts / future |","|---|---|---|:---:|---|"]
    for r in c["surfaces"]:
        cf = ", ".join(r["courts"]) if r["courts"] else (r["future_campaign"] or "—")
        mods = ", ".join(os.path.basename(m) for m in r["source_modules"])
        lines.append(f"| `{r['surface']}` | {r['category']} | `{mods}` | {badge[r['status']]} {r['status']} | {cf} |")
    lines += ["","## Risk of the unported surfaces (missing ❌)",""]
    for r in c["surfaces"]:
        if r["status"] == "missing":
            lines.append(f"- **`{r['surface']}`** → `{r['future_campaign']}`: {r['risk_if_unported']}")
    lines += ["","## Non-claims",""]+[f"- `{n}`" for n in c["non_claims"]]+[""]
    return "\n".join(lines)+"\n"

def generate():
    c = build()
    json.dump(c, open(os.path.join(ROOT,"reports/gnurust-coverage.json"),"w"), indent=2)
    open(os.path.join(ROOT,"reports/gnurust-coverage.md"),"w").write(render_md(c))
    sc = c["status_counts"]
    print(f"coverage: {c['surface_count']} surfaces — sealed {sc['sealed']}, observed {sc['observed']}, refused {sc['negative']}, MISSING {sc['missing']}")

def check():
    bad = 0
    jp = os.path.join(ROOT,"reports/gnurust-coverage.json")
    if not os.path.exists(jp):
        print("GATE: reports/gnurust-coverage.json missing (run: python3 lab/coverage/run.py generate)"); return 1
    fresh = build()
    if json.dumps(json.load(open(jp)), sort_keys=True) != json.dumps(fresh, sort_keys=True):
        print("GATE: coverage map != a fresh re-derive (stale or hand-edited)"); bad += 1
    if open(os.path.join(ROOT,"reports/gnurust-coverage.md")).read() != render_md(fresh):
        print("GATE: coverage .md != regenerated"); bad += 1
    cl = claim_ladder_ids()
    mapped = {court for r in fresh["surfaces"] for court in r["courts"]}
    # (a) every admitted GnuCOBOL court is mapped to a surface
    for court in sorted(admitted_gnucobol_courts()):
        if court not in mapped:
            print(f"GATE: admitted court {court} is NOT mapped to any coverage surface"); bad += 1
    # (b) every declared court actually exists; (c) sealed surface has a real court; (d) modules exist; (e) status valid
    for r in fresh["surfaces"]:
        for court in r["courts"]:
            if court not in cl:
                print(f"GATE: surface {r['surface']} declares court {court} not in the claim-ladder"); bad += 1
        if r["status"] == "sealed" and not r["courts"]:
            print(f"GATE: surface {r['surface']} is 'sealed' but has no court"); bad += 1
        if r["status"] not in STATUSES:
            print(f"GATE: surface {r['surface']} has invalid status {r['status']}"); bad += 1
        if not r["source_modules_present"]:
            print(f"GATE: surface {r['surface']} binds to a missing source module ({r['source_modules']})"); bad += 1
    if bad:
        print(f"!! {bad} GNURUST.COVERAGE.1 finding(s)"); return 1
    sc = fresh["status_counts"]
    print(f"GNURUST.COVERAGE.1: {fresh['surface_count']} surfaces mapped to admitted source; every admitted court mapped; sealed {sc['sealed']}, MISSING {sc['missing']}; fresh")
    return 0

if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "check"
    {"generate": generate, "check": lambda: sys.exit(check())}.get(cmd, lambda: sys.exit("usage: generate|check"))()
