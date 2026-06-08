#!/usr/bin/env python3
"""Generate reports/negative-capabilities.json from the claim-ladder not_proven items + the fixed
NEG.DOC.* documentation-staleness capabilities (TRUST.4.DOCS makes staleness a negative capability)."""
import json, os, re
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
cl = json.load(open(os.path.join(ROOT, "reports/claim-ladder.json")))
def slug(s): return "NEG." + re.sub(r'[^A-Z0-9]+', '-', s.upper()).strip('-')[:40]
seen = {}
for c in cl["courts"]:
    for neg in [n.strip() for n in (c.get("not_proven", "") or "").split(";") if n.strip()]:
        k = slug(neg)
        seen.setdefault(k, {"id": k, "surface": neg, "status": "not_admitted_fail_closed",
                            "guard": "fails closed / outside the sealed subset",
                            "risk_if_guessed": "silent mis-decode of a surface this court does not prove",
                            "owning_future_campaign": None, "evidence": []})
        seen[k]["evidence"].append(c["id"])
DOC = [
 ("NEG.DOC.STALE_VERSION", "an authoritative doc naming a version other than Cargo.toml", "lab/docs/generate.py check", "a reader trusts a stale version/claim"),
 ("NEG.DOC.STALE_CLAIM", "an authoritative doc claim contradicting the current casefiles", "lab/docs/generate.py check", "README says future but STATUS/casefile says composed"),
 ("NEG.DOC.STALE_RECEIPT_LINK", "a doc linking a receipt/casefile that no longer exists", "doc-gate link check", "a dangling evidence pointer"),
 ("NEG.DOC.STALE_NONCLAIM", "a fail-closed list contradicting an admitted casefile", "lab/docs/generate.py check", "a non-claim that is actually now admitted"),
 ("NEG.DOC.MANUAL_EDIT", "a generated authoritative doc edited by hand", "lab/docs/generate.py check (== regenerated)", "hand-written authority masquerading as generated"),
 ("NEG.DOC.LEGACY_NOT_REPLACED", "a legacy doc moved without a generated replacement", "lab/trust4/migrate_reports.py check", "lost authority with no successor"),
 ("NEG.DOC.INFORMATION_LOSS", "a generated doc that is not an information superset of its legacy", "legacy_preservation review", "dropped caveat/claim/context"),
 ("NEG.DOC.FRONTDOOR_CONFLICT", "a front-door doc linking a demoted legacy report as authority", "lab/trust4/migrate_reports.py check", "a reader follows a non-authoritative exhibit"),
]
for (i, s, g, r) in DOC:
    seen[i] = {"id": i, "surface": s, "status": "machine_detected_negative_finding", "guard": g,
               "risk_if_guessed": r, "owning_future_campaign": None, "evidence": ["TRUST.4.DOCS"]}
out = {"schema": "kobold-negative-capability-registry-v1",
       "doctrine": "Negative capability is the trust surface. Every entry is a surface this stack refuses to guess (court non-claims) or a documentation-staleness finding it makes machine-detectable (NEG.DOC.*). Casefiles reference these IDs.",
       "count": len(seen), "negative_capabilities": sorted(seen.values(), key=lambda x: x["id"])}
json.dump(out, open(os.path.join(ROOT, "reports/negative-capabilities.json"), "w"), indent=2)
print(f"negative-capabilities.json: {len(seen)} surfaces ({len(DOC)} NEG.DOC.*)")
