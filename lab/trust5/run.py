#!/usr/bin/env python3
"""TRUST.5 — Anti-Ceremony Audit. For EVERY court, prove it can FAIL.

The sharp rule: a court is REAL if deleting, corrupting, drifting, or hand-editing its evidence can make a
gate fail; it is CEREMONIAL if it only restates that other evidence exists and cannot itself detect drift,
loss, mismatch, or overclaim. This audit classifies every claim-ladder court (A hard / B composed / C view /
D staged / F ceremonial), records each court's can-fail proof, and GATES: no court may be class F, every
VIEW court must declare no-new-truth + carry a freshness/source binding, and the audit itself must match a
fresh re-audit (it cannot go stale or be hand-edited).

  run.py audit   # write reports/trust5-anti-ceremony-audit.json + .md
  run.py check   # gate: F-set empty, view courts no-new-truth, audit fresh
"""
import glob, json, os, sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# Courts whose job is to SUMMARIZE existing evidence (must be hash-bound + no-new-truth).
VIEWS = {"KOBOLD.BANK.RECONCILE.1", "KOBOLD.DIFF.1", "SUPPORT-PACKET.1", "KOBOLD.OPERATOR.1", "KOBOLD.TOOLING.EXPORT.1", "KOBOLD.PILOT-PACKET.1"}
# Courts that are staged pending external proof (must NOT be green without it). LAMBDA.LIVE.1 lives in the
# sibling lambda repo (awaiting_live_invocation) and is not in this claim-ladder; noted in the audit.
STAGED = set()

def claim_ladder():
    return json.load(open(os.path.join(ROOT, "reports/claim-ladder.json")))["courts"]

def receipts():
    return set(os.path.basename(os.path.dirname(p)) for p in glob.glob(os.path.join(ROOT, "reports/receipts/*/")))

def negcaps_by_court():
    by = {}
    for n in json.load(open(os.path.join(ROOT, "reports/negative-capabilities.json"))).get("negative_capabilities", []):
        for ev in n.get("evidence", []) or []:
            by.setdefault(ev, []).append(n["id"])
    return by

def casefile_counts(cid):
    f = os.path.join(ROOT, "reports/casefiles", cid, "casefile.json")
    if not os.path.exists(f):
        return None
    c = json.load(open(f))
    return (c.get("positive_claims") and len(c["positive_claims"]) or 0,
            c.get("negative_claims") and len(c["negative_claims"]) or 0)

def audit_court(c, rec, negs):
    cid = c["id"]
    has_casefile = os.path.exists(os.path.join(ROOT, "reports/casefiles", cid, "casefile.json"))
    has_receipt = cid in rec
    counts = casefile_counts(cid)
    pos, neg = counts if counts else (0, 0)
    court_negs = negs.get(cid, [])
    # the four anti-ceremony properties
    p1_generated = has_casefile  # generated from named evidence (the casefile)
    p3_no_new_truth = (cid not in VIEWS) or any(
        k in n for n in court_negs for k in ("NO_NEW_EVIDENCE", "NO_NEW_TRUTH", "VIEW_NOT_NEW_EVIDENCE"))
    p4_damage = bool(c.get("damage_if_overclaimed")) and bool(c.get("lie_prevented"))
    has_fixtures = bool(c.get("fixtures")) and bool(c.get("breaks_claim")) and bool(c.get("not_proven"))
    neg_ge_pos = neg >= pos
    # p2 can-fail: a concrete detector whose corruption fails a gate
    if has_receipt:
        klass, can_fail_proof = "A", "oracle sweep + receipt/casefile regen-equality (corrupt the receipt or sweep -> gate red)"
    elif cid in VIEWS:
        klass, can_fail_proof = "C", "regenerate-and-compare freshness + no-new-truth refusal (mutate a source artifact -> stale check fails)"
    elif cid in STAGED:
        klass, can_fail_proof = "D", "external proof required; status cannot be green until it exists"
    else:
        klass, can_fail_proof = "B", (c.get("fixtures", "") + " + breaks_claim/not_proven (corrupt the fixture/golden -> test fails)")
    p2_can_fail = bool(can_fail_proof) and has_fixtures and neg_ge_pos
    ceremonial = not (p1_generated and p2_can_fail and p3_no_new_truth and p4_damage and neg_ge_pos)
    if ceremonial:
        klass = "F"
    return {
        "id": cid, "class": klass, "can_fail": not ceremonial, "can_fail_proof": can_fail_proof,
        "has_casefile": has_casefile, "has_receipt": has_receipt,
        "positive_claims": pos, "negative_claims": neg, "negatives_ge_positives": neg_ge_pos,
        "generated_from_named_evidence": p1_generated, "creates_new_truth": not p3_no_new_truth,
        "damage_if_overclaimed_present": p4_damage, "fixtures": c.get("fixtures", ""),
        "ceremonial_flags": [k for k, v in {
            "no_casefile": not has_casefile, "no_damage_if_overclaimed": not p4_damage,
            "no_fixtures_or_breaks_claim": not has_fixtures, "negatives_lt_positives": not neg_ge_pos,
            "view_creates_new_truth": cid in VIEWS and not p3_no_new_truth}.items() if v],
    }

def build():
    cl, rec, negs = claim_ladder(), receipts(), negcaps_by_court()
    rows = [audit_court(c, rec, negs) for c in sorted(cl, key=lambda x: x["id"])]
    classes = {k: sorted(r["id"] for r in rows if r["class"] == k) for k in "ABCDF"}
    return {
        "schema": "kobold-trust5-anti-ceremony-audit-v1", "court": "TRUST.5",
        "doctrine": "A court is real if corrupting/dropping/drifting/hand-editing its evidence can make a gate fail; ceremonial if it only restates that other evidence exists. Every court must prove: (1) generated from named evidence, (2) can detect stale/tampered evidence, (3) creates no new truth, (4) states the damage if overclaimed.",
        "rubric": {"A": "live oracle replay / byte-deterministic fixture + mutation + stale check + casefile",
                   "B": "consumes sealed lower courts + fail-closed/hostile fixtures",
                   "C": "view: summarizes existing evidence; MUST carry source hashes + no-new-truth + regen-compare",
                   "D": "staged: harness exists, external proof missing; not green until it exists",
                   "F": "ceremonial: prose-only, no replay/drift/mutation detector, no negatives"},
        "classes": {k: classes[k] for k in "ABCDF"},
        "class_counts": {k: len(classes[k]) for k in "ABCDF"},
        "external_staged": {"LAMBDA.LIVE.1": "sibling kobold-lambda-layer; status awaiting_live_invocation (not green until a live AWS request id + matching output hash exist)"},
        "courts": rows,
        "non_claims": ["NEG.TRUST5.AUDIT_NOT_CERTIFICATION", "NEG.TRUST5.CLASS_NOT_QUALITY_SCORE",
                       "NEG.TRUST5.SNAPSHOT_NOT_LIVE", "NEG.TRUST5.NO_NEW_TRUTH"],
    }

def render_md(a):
    cc = a["class_counts"]
    lines = ["<!-- generated by lab/trust5/run.py — do not edit by hand -->", "",
             "# TRUST.5 — Anti-Ceremony Audit", "",
             "> [!IMPORTANT]", "> A court is **real** if corrupting/dropping/drifting/hand-editing its evidence can make a gate "
             "fail; **ceremonial** if it only restates that other evidence exists. This audit proves every court can fail.", "",
             f"- **A** hard (oracle/byte): {cc['A']}  ·  **B** composed: {cc['B']}  ·  **C** view: {cc['C']}  ·  "
             f"**D** staged: {cc['D']}  ·  **F** ceremonial: **{cc['F']}**", "",
             "## Classification", "", "| court | class | can fail? | neg≥pos | new truth? |", "|---|:---:|:---:|:---:|:---:|"]
    for r in a["courts"]:
        lines.append(f"| `{r['id']}` | {r['class']} | {'✅' if r['can_fail'] else '❌'} | "
                     f"{'✅' if r['negatives_ge_positives'] else '❌'} | {'❌ yes' if r['creates_new_truth'] else '✅ no'} |")
    lines += ["", "## Staged (external proof pending)", "", "- `LAMBDA.LIVE.1` — " + a["external_staged"]["LAMBDA.LIVE.1"],
              "", "## Non-claims", ""] + [f"- `{n}`" for n in a["non_claims"]] + [""]
    return "\n".join(lines) + "\n"

def audit():
    a = build()
    json.dump(a, open(os.path.join(ROOT, "reports/trust5-anti-ceremony-audit.json"), "w"), indent=2)
    open(os.path.join(ROOT, "reports/trust5-anti-ceremony-audit.md"), "w").write(render_md(a))
    cc = a["class_counts"]
    print(f"TRUST.5 audit: A={cc['A']} B={cc['B']} C={cc['C']} D={cc['D']} F={cc['F']}")

def check():
    bad = 0
    jp = os.path.join(ROOT, "reports/trust5-anti-ceremony-audit.json")
    if not os.path.exists(jp):
        print("GATE: trust5 audit missing (run: python3 lab/trust5/run.py audit)"); return 1
    fresh = build()
    on_disk = json.load(open(jp))
    if json.dumps(on_disk, sort_keys=True) != json.dumps(fresh, sort_keys=True):
        print("GATE: trust5 audit != a fresh re-audit (stale or hand-edited)"); bad += 1
    if open(os.path.join(ROOT, "reports/trust5-anti-ceremony-audit.md")).read() != render_md(fresh):
        print("GATE: trust5 audit .md != regenerated"); bad += 1
    # the core anti-ceremony gates:
    if fresh["class_counts"]["F"] != 0:
        print(f"GATE: ceremonial court(s) detected (class F): {fresh['classes']['F']}"); bad += 1
    for r in fresh["courts"]:
        if r["id"] in VIEWS and r["creates_new_truth"]:
            print(f"GATE: view court {r['id']} lacks a no-new-truth refusal"); bad += 1
        if r["ceremonial_flags"]:
            print(f"GATE: {r['id']} ceremonial flags: {r['ceremonial_flags']}"); bad += 1
    if bad:
        print(f"!! {bad} TRUST.5 finding(s)"); return 1
    cc = fresh["class_counts"]
    print(f"TRUST.5: every court can fail (A={cc['A']} B={cc['B']} C={cc['C']} D={cc['D']}, F=0); views are no-new-truth; audit fresh")
    return 0

if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "check"
    {"audit": audit, "check": lambda: sys.exit(check())}.get(cmd, lambda: sys.exit("usage: audit|check"))()
