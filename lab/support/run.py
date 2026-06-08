#!/usr/bin/env python3
"""SUPPORT-PACKET.1 — generate a reviewer/operator evidence bundle FROM EXISTING generated artifacts.
It creates NO new truth: it gathers the governance artifacts already produced (STATUS, claim-ladder,
negative capabilities, casefiles + their SARIF/in-toto/DSSE, the DSSE verification report, release packets,
the size-error atlas, the truth-boundary doc, crate versions) into one manifest + index, and POINTS at the
runtime/operator artifacts (bench/scale receipts, bank-reconcile/diff reports, redaction policy) without
embedding them.

  run.py generate   # write reports/support-packet/{support-packet.json,support-packet.md}
  run.py check      # gate: the packet matches a fresh re-gather (no stale/hand-edit)
"""
import glob, hashlib, json, os, re, sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "reports/support-packet")

def sha(path):
    p = os.path.join(ROOT, path)
    return hashlib.sha256(open(p, "rb").read()).hexdigest() if os.path.exists(p) else None

def crate_version():
    for ln in open(os.path.join(ROOT, "crates/gnucobol-rs/Cargo.toml")):
        m = re.match(r'^version\s*=\s*"([^"]+)"', ln)
        if m:
            return m.group(1)
    return "?"

def gather():
    # generated-and-committed governance artifacts (embedded by reference + sha)
    committed = [
        ("status", "STATUS.md", "generated_doc"),
        ("changelog", "CHANGELOG.md", "generated_doc"),
        ("claim_ladder", "reports/claim-ladder.json", "machine_spine"),
        ("negative_capabilities", "reports/negative-capabilities.json", "refusal_registry"),
        ("dsse_verification", "reports/signing/verification-report.json", "attestation_report"),
        ("size_error_atlas", "reports/size-error-atlas.json", "observed_atlas"),
        ("truth_boundaries", "docs/truth-boundaries.md", "doctrine_doc"),
        ("future_risk_register", "docs/future-risk-register.md", "doctrine_doc"),
    ]
    artifacts = []
    for aid, path, kind in committed:
        s = sha(path)
        artifacts.append({"id": aid, "path": path, "kind": kind, "present": s is not None, "sha256": s})
    # casefile index (court -> casefile.json sha) incl SARIF/in-toto/DSSE presence
    cases = []
    for d in sorted(glob.glob(os.path.join(ROOT, "reports/casefiles/*/"))):
        court = os.path.basename(d.rstrip("/"))
        cf = f"reports/casefiles/{court}/casefile.json"
        cases.append({"court": court, "casefile_sha256": sha(cf),
                      "sarif": os.path.exists(os.path.join(d, "sarif.json")),
                      "intoto": os.path.exists(os.path.join(d, "intoto-statement.json")),
                      "dsse": os.path.exists(os.path.join(d, "dsse-envelope.json"))})
    artifacts.append({"id": "casefile_index", "kind": "casefile_index", "present": bool(cases), "count": len(cases), "casefiles": cases})
    # release packet index
    rels = sorted(os.path.basename(p.rstrip("/")) for p in glob.glob(os.path.join(ROOT, "reports/releases/*/")))
    artifacts.append({"id": "release_packets", "kind": "release_index", "present": bool(rels), "packets": rels})
    # runtime / sibling-repo / operator-supplied artifacts: POINTED AT, not embedded (present:false honest)
    runtime = [
        ("bench_receipt", "kobold-bench reports/BENCH-2-receipt.json", "kobold-bench2 --features rayon"),
        ("scale_receipt", "kobold-bench reports/SCALE-1-receipt-*.json", "kobold-scale 1g"),
        ("bank_reconcile_report", "kobold-bank-reconcile-report-v1", "bank_reconcile_report() at runtime"),
        ("diff_report", "kobold-diff-report-v1", "diff_artifacts() at runtime"),
        ("redaction_policy", "kobold-redaction-policy-v1", "operator-declared"),
        ("currency_manifest", "kobold-currency-manifest-v1", "currency_validate() at runtime"),
        ("date_manifest", "kobold-date-manifest-v1", "date_validate() at runtime"),
        ("sentinel_manifest", "kobold-sentinel-manifest-v1", "sentinel_scan() at runtime"),
    ]
    for aid, pointer, how in runtime:
        artifacts.append({"id": aid, "kind": "runtime_or_operator", "present": False, "pointer": pointer, "produced_by": how})
    return {
        "schema": "kobold-support-packet-v1", "court": "SUPPORT-PACKET.1",
        "generated_from": "existing generated artifacts only; creates no new truth",
        "crate_versions": {"gnucobol-rs": crate_version() + " (this repo)",
                           "kobold-data-shim": "sibling repo (see its Cargo.toml/crates.io)",
                           "kobold-bench": "sibling repo (bench/scale receipts)",
                           "kobold-attest": "lab/attest (publish=false; Cargo.lock pinned)"},
        "truth_boundary_summary": "bytes < record < transform < custody < reconciliation evidence < privacy-preserved < generated attestation; REFUSED above: posting, ledger, settlement, account-balance, business truth, production readiness, customer-workload representativeness, regulatory compliance.",
        "artifacts": artifacts,
        "non_claims": ["NEG.SUPPORT.NO_NEW_TRUTH", "NEG.SUPPORT.NOT_CERTIFICATION", "NEG.SUPPORT.NOT_COMPLIANCE",
                       "NEG.SUPPORT.NOT_PRODUCTION_APPROVAL", "NEG.SUPPORT.NOT_CUSTOMER_ACCEPTANCE", "NEG.SUPPORT.SNAPSHOT_NOT_LIVE"],
    }

def render_md(pk):
    present = sum(1 for a in pk["artifacts"] if a.get("present"))
    lines = ["<!-- generated by lab/support/run.py — do not edit by hand -->", "",
             "# Support packet (SUPPORT-PACKET.1)", "",
             "> [!IMPORTANT]", "> A reviewer/operator evidence bundle gathered from **existing generated artifacts**. "
             "It creates **no** new truth, certification, compliance, production approval, or customer acceptance.", "",
             f"- crate (this repo): `gnucobol-rs {crate_version()}`",
             f"- artifacts gathered: **{present}** committed + pointers to runtime/operator artifacts",
             f"- casefiles: **{pk['artifacts'][8]['count'] if len(pk['artifacts'])>8 else '?'}**", "",
             "## Truth boundary", "", "> " + pk["truth_boundary_summary"], "",
             "## Committed governance artifacts", "", "| id | path | sha256 |", "|---|---|---|"]
    for a in pk["artifacts"]:
        if a.get("present") and a.get("sha256"):
            lines.append(f"| `{a['id']}` | [`{a['path']}`]({os.path.relpath(os.path.join(ROOT,a['path']),OUT)}) | `{a['sha256'][:16]}…` |")
    lines += ["", "## Runtime / operator artifacts (pointers — not embedded)", "", "| id | produced by |", "|---|---|"]
    for a in pk["artifacts"]:
        if a["kind"] == "runtime_or_operator":
            lines.append(f"| `{a['id']}` | {a['produced_by']} |")
    lines += ["", "## Non-claims", ""] + [f"- `{n}`" for n in pk["non_claims"]] + [""]
    return "\n".join(lines) + "\n"

def generate():
    os.makedirs(OUT, exist_ok=True)
    pk = gather()
    json.dump(pk, open(os.path.join(OUT, "support-packet.json"), "w"), indent=2)
    open(os.path.join(OUT, "support-packet.md"), "w").write(render_md(pk))
    print(f"support packet: {sum(1 for a in pk['artifacts'] if a.get('present'))} committed artifacts + runtime pointers")

def check():
    jp = os.path.join(OUT, "support-packet.json")
    if not os.path.exists(jp):
        print("GATE: reports/support-packet/support-packet.json missing (run: python3 lab/support/run.py generate)"); return 1
    fresh = gather()
    on_disk = json.load(open(jp))
    if json.dumps(on_disk, sort_keys=True) != json.dumps(fresh, sort_keys=True):
        print("GATE: support-packet.json != a fresh re-gather (stale or hand-edited; run lab/support/run.py generate)"); return 1
    md_fresh = render_md(fresh)
    if open(os.path.join(OUT, "support-packet.md")).read() != md_fresh:
        print("GATE: support-packet.md != regenerated"); return 1
    print(f"SUPPORT-PACKET.1: bundle fresh ({sum(1 for a in fresh['artifacts'] if a.get('present'))} committed artifacts)")
    return 0

if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "check"
    {"generate": generate, "check": lambda: sys.exit(check())}.get(cmd, lambda: sys.exit("usage: generate|check"))()
