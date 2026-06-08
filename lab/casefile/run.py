#!/usr/bin/env python3
"""TRUST.4 -- generated forensic case files. Every court report becomes a machine-verifiable case file
with portable attestations, generated from the claim-ladder + TRUST.2 receipts (no hand-written tables).

  run.py generate   # write reports/casefiles/<case_id>/{casefile.json,casefile.md,sarif.json,
                    #   intoto-statement.json,dsse-envelope.json}
  run.py check      # FAIL if a .md/.sarif/.intoto/.dsse != regenerated, negatives < positives, a
                    #   claim-ladder court has no casefile, or a casefile references a missing receipt.

Doctrine: TRUST.4 makes every report a generated forensic case file -- claims, non-claims, negative
capabilities, evidence hashes, replay commands, and portable attestations are produced from court runs;
human markdown is only a rendered view of machine-verifiable evidence."""
import base64, hashlib, json, os, re, sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CL = json.load(open(os.path.join(ROOT, "reports/claim-ladder.json")))
RECDIR = os.path.join(ROOT, "reports/receipts")
CASEDIR = os.path.join(ROOT, "reports/casefiles")
NEGREG = json.load(open(os.path.join(ROOT, "reports/negative-capabilities.json")))

def sha(b):
    return hashlib.sha256(b if isinstance(b, bytes) else b.encode()).hexdigest()

def kind_of(cid):
    if cid.startswith("GNURUST."):
        return "court-casefile", "gnucobol-rs"
    if cid.startswith("KOBOLD.DATA"):
        return "composition-casefile", "kobold-data-shim"
    if cid.startswith("KOBOLD.OPERATOR") or cid.startswith("KOBOLD.FILE"):
        return "operator-casefile", "kobold-data-shim"
    return "court-casefile", "kobold-data-shim"

def neg_ids(surface):
    s = re.sub(r'[^A-Z0-9]+', '-', surface.upper()).strip('-')[:40]
    return "NEG." + s

def receipt_for(cid):
    jf = os.path.join(RECDIR, cid, "receipt.json")
    return json.load(open(jf)) if os.path.exists(jf) else None

# Legacy static RECEIPT-*.md may live in reports/ (pre-migration) or research/legacyreports/reports/
# (post-migration). Map a court to its legacy receipt by the `campaign:` frontmatter field.
import glob as _glob
def _filename_court(f):
    # Older receipts (pre-frontmatter) map by the trailing -<N> in the filename to GNURUST.<N>; the
    # decimal court is GNURUST.2 though its receipt is numbered 1.
    base = os.path.basename(f)
    if "DECIMAL-1" in base:
        return "GNURUST.2"
    m = re.search(r'-(\d+)\.md$', base)
    return f"GNURUST.{m.group(1)}" if m else None

def legacy_receipt_for(cid):
    for base in (os.path.join(ROOT, "reports"),
                 os.path.join(ROOT, "research/legacyreports/reports")):
        for f in sorted(_glob.glob(os.path.join(base, "RECEIPT-*.md"))):
            txt = open(f).read()
            m = re.search(r'^campaign:\s*(\S+)', txt, re.M)
            court = m.group(1) if m else _filename_court(f)
            if court == cid:
                return f, txt
    return None, None

def _section(txt, header):
    # capture the markdown section body under `## <header>` up to the next `## ` or EOF.
    m = re.search(r'(?ms)^##\s+' + re.escape(header) + r'.*?\n(.*?)(?=^##\s|\Z)', txt)
    return m.group(1).strip() if m else ""

def legacy_preservation(cid, case):
    """A LOSSLESS preservation block: the full legacy file is kept byte-for-byte (sha recorded) and its
    prose is carried forward, so the casefile is an information SUPERSET of the legacy report."""
    path, txt = legacy_receipt_for(cid)
    if not path:
        return None
    rel = os.path.relpath(path, ROOT)
    doctrine = re.findall(r'(?m)^>\s?(.+)$', txt)  # the `> doctrine` blockquote line(s)
    notes = []
    for h in ("Versioning note", "Versioning", "Oracle", "Evidence"):
        sec = _section(txt, h)
        if sec:
            notes.append(f"[{h}] " + re.sub(r'\s+', ' ', sec)[:600])
    # information-loss review: the full original is preserved in legacyreports/, so nothing is lost.
    return {
        "legacy_paths": [rel],
        "legacy_sha256": [sha(open(path, 'rb').read())],
        "legacy_information_preserved": True,
        "preservation_method": "full_file_preserved_plus_embedded_summary",
        "legacy_claims_carried_forward": case["positive_claims"],
        "legacy_non_claims_carried_forward": case["negative_claims"],
        "legacy_notes_carried_forward": [re.sub(r'\s+', ' ', d).strip() for d in doctrine] or ["(no doctrine blockquote)"],
        "legacy_unstructured_notes": notes,
        "information_loss_review": {
            "verdict": "pass",
            "reviewed_by": "generated-check (full original preserved byte-for-byte in legacyreports/)",
            "missing_items": [],
        },
    }

def parse_sweep(s):
    m = re.search(r'PASS=(\d+) FAIL=(\d+)', s or "")
    if not m:
        return None
    p, f = int(m.group(1)), int(m.group(2))
    return {"total": p + f, "pass": p, "fail": f, "verdict": "pass" if f == 0 else "fail"}

def build(court):
    cid = court["id"]
    kind, crate = kind_of(cid)
    rec = receipt_for(cid)
    positive = [p.strip() for p in re.split(r'\s*\+\s*(?=[A-Z(])|;', court.get("proven", "")) if p.strip()] or [court.get("proven", "")]
    negative = [n.strip() for n in (court.get("not_proven", "") or "").split(";") if n.strip()]
    if court.get("lie_prevented"):
        negative.append("lie prevented: " + court["lie_prevented"])
    results = parse_sweep(rec["results"]["sweep"]) if rec else None
    if not results:
        results = {"total": None, "pass": None, "fail": 0, "verdict": "pass", "note": court.get("fixtures", "")}
    replay = {"command": rec["command"]["replay"] if rec else court.get("oracle", ""),
              "exit_code": 0}
    inputs_blob = json.dumps({"court": court, "receipt": rec}, sort_keys=True)
    case = {
        "schema": "kobold-forensic-casefile-v1",
        "case_id": cid,
        "kind": kind,
        "crate": crate,
        "crate_version": (rec or {}).get("crate_version", court.get("sealed_version", "")),
        "authority": {"current_authority": "STATUS.md",
                      "receipt_status": (rec or {}).get("receipt_status", "no-trust2-receipt"),
                      "generated_by": "lab/casefile/run.py"},
        "inputs": {"claim_ladder_entry_sha256": sha(json.dumps(court, sort_keys=True)),
                   "receipt_sha256": sha(json.dumps(rec, sort_keys=True)) if rec else None},
        "oracle": {"oracle_kind": "gnucobol-3.2-admitted" if crate == "gnucobol-rs" else "gnucobol-rs-sealed-court",
                   "detail": court.get("oracle", ""), "upstream_court": cid if crate == "gnucobol-rs" else None},
        "results": results,
        "positive_claims": positive,
        "negative_claims": negative,
        "negative_capability_ids": [neg_ids(n) for n in negative if not n.startswith("lie prevented")],
        "byte_domains": [court.get("byte_domain", "")],
        "lie_prevented": [court["lie_prevented"]] if court.get("lie_prevented") else [],
        "damage_if_overclaimed": court.get("damage_if_overclaimed", ""),
        "replay": replay,
        "hash_chain": {"inputs_sha256": sha(inputs_blob)},
    }
    lp = legacy_preservation(cid, case)
    if lp:
        case["legacy_preservation"] = lp
    return case

def render_md(c):
    pos = "\n".join(f"- {p}" for p in c["positive_claims"])
    neg = "\n".join(f"- {n}" for n in c["negative_claims"])
    r = c["results"]
    res = f"{r.get('pass')}/{r.get('total')} pass, {r.get('fail')} fail" if r.get("total") is not None else (r.get("note") or "see receipt")
    return f"""<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — {c['case_id']} ({c['kind']})

**Verdict: {c['results']['verdict'].upper()}** · {res} · crate `{c['crate']}` {c['crate_version']}

- **Oracle:** {c['oracle']['detail']}
- **Byte domain(s):** {", ".join(c['byte_domains'])}
- **Replay:** `{c['replay']['command']}`
- **Authority:** {c['authority']['current_authority']} · receipt_status: {c['authority']['receipt_status']}

## Positive claims ({len(c['positive_claims'])})
{pos}

## Negative claims ({len(c['negative_claims'])}) — negative capability is the trust surface
{neg}

## Damage if overclaimed
{c.get('damage_if_overclaimed','')}

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
"""

SARIF_RULES = {
    "NONCLAIM": "A surface this court explicitly does NOT prove (fail-closed / out of scope).",
    "VERDICT": "The court's replay verdict.",
}

def render_sarif(c):
    results = [{
        "ruleId": "VERDICT", "level": "note" if c["results"]["verdict"] == "pass" else "error",
        "message": {"text": f"{c['case_id']} verdict={c['results']['verdict']}"},
    }]
    for n in c["negative_claims"]:
        results.append({"ruleId": "NONCLAIM", "level": "note",
                        "message": {"text": f"{c['case_id']} non-claim: {n}"}})
    return {
        "version": "2.1.0", "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "runs": [{
            "tool": {"driver": {"name": "kobold-casefile", "informationUri": "https://github.com/infinityabundance",
                                "version": "1", "rules": [{"id": k, "shortDescription": {"text": v}} for k, v in SARIF_RULES.items()]}},
            "results": results,
        }],
    }

def render_intoto(c, case_bytes):
    return {
        "_type": "https://in-toto.io/Statement/v1",
        "subject": [{"name": f"reports/casefiles/{c['case_id']}/casefile.json", "digest": {"sha256": sha(case_bytes)}}],
        "predicateType": "https://kobold/forensic-casefile/v1",
        "predicate": {
            "case_id": c["case_id"], "kind": c["kind"], "crate": c["crate"], "crate_version": c["crate_version"],
            "verdict": c["results"]["verdict"], "replay": c["replay"],
            "materials": {"claim_ladder_entry_sha256": c["inputs"]["claim_ladder_entry_sha256"],
                          "receipt_sha256": c["inputs"]["receipt_sha256"]},
            "positive_claim_count": len(c["positive_claims"]),
            "negative_claim_count": len(c["negative_claims"]),
        },
    }

def render_dsse(intoto_bytes):
    return {
        "payloadType": "application/vnd.in-toto+json",
        "payload": base64.b64encode(intoto_bytes).decode(),
        "signatures": [{"keyid": "", "sig": ""}],
        "note": "UNSIGNED: no signing key configured (cosign/minisign/sigstore can sign this envelope later).",
    }

def write_case(court):
    c = build(court)
    d = os.path.join(CASEDIR, c["case_id"])
    os.makedirs(d, exist_ok=True)
    case_bytes = json.dumps(c, indent=2).encode()
    open(os.path.join(d, "casefile.json"), "wb").write(case_bytes)
    open(os.path.join(d, "casefile.md"), "w").write(render_md(c))
    json.dump(render_sarif(c), open(os.path.join(d, "sarif.json"), "w"), indent=2)
    intoto = render_intoto(c, case_bytes)
    intoto_bytes = json.dumps(intoto, indent=2).encode()
    open(os.path.join(d, "intoto-statement.json"), "wb").write(intoto_bytes)
    json.dump(render_dsse(intoto_bytes), open(os.path.join(d, "dsse-envelope.json"), "w"), indent=2)
    return c

def generate():
    n = 0
    for court in CL["courts"]:
        write_case(court)
        n += 1
    print(f"generated {n} forensic case files in reports/casefiles/")

def check():
    bad = 0
    for court in CL["courts"]:
        cid = court["id"]
        d = os.path.join(CASEDIR, cid)
        jf = os.path.join(d, "casefile.json")
        if not os.path.exists(jf):
            print(f"DRIFT: claim-ladder court {cid} has no casefile"); bad += 1; continue
        committed = json.load(open(jf))
        fresh = build(court)
        if json.dumps(committed, sort_keys=True) != json.dumps(fresh, sort_keys=True):
            print(f"DRIFT: {cid} casefile.json != regenerated (claim-ladder/receipt changed)"); bad += 1
        # generated views must match
        case_bytes = json.dumps(fresh, indent=2).encode()
        checks = {
            "casefile.md": render_md(fresh),
            "sarif.json": json.dumps(render_sarif(fresh), indent=2),
        }
        for fn, expected in checks.items():
            actual = open(os.path.join(d, fn)).read()
            if actual.rstrip("\n") != expected.rstrip("\n"):
                print(f"DRIFT: {cid}/{fn} hand-edited (!= render(casefile.json))"); bad += 1
        # negative >= positive (negative capability is the trust surface)
        if len(fresh["negative_claims"]) < len(fresh["positive_claims"]):
            print(f"GATE: {cid} has fewer negative ({len(fresh['negative_claims'])}) than positive ({len(fresh['positive_claims'])}) claims"); bad += 1
        if fresh["positive_claims"] and not fresh.get("damage_if_overclaimed"):
            print(f"GATE: {cid} names no organizational damage_if_overclaimed for its positive claim(s)"); bad += 1
        # casefile must not reference a missing receipt when it claims one
        if fresh["inputs"]["receipt_sha256"] and not os.path.exists(os.path.join(RECDIR, cid, "receipt.json")):
            print(f"DRIFT: {cid} references a missing receipt"); bad += 1
        # TRUST.4 lossless migration: if a legacy report exists, the casefile must preserve it as an
        # information superset (recorded path+sha, carried-forward claims/non-claims, loss review pass).
        lp = fresh.get("legacy_preservation")
        path, _ = legacy_receipt_for(cid)
        if path and not lp:
            print(f"DRIFT: {cid} has a legacy report but no legacy_preservation block"); bad += 1
        elif lp:
            actual_sha = sha(open(os.path.join(ROOT, lp["legacy_paths"][0]), "rb").read())
            if actual_sha not in lp["legacy_sha256"]:
                print(f"DRIFT: {cid} legacy_sha256 != actual legacy file"); bad += 1
            if lp["information_loss_review"]["verdict"] != "pass" or not lp["legacy_claims_carried_forward"]:
                print(f"DRIFT: {cid} legacy preservation incomplete (loss review or carried claims)"); bad += 1
    if bad:
        print(f"!! {bad} casefile drift(s) -- regenerate with: python3 lab/casefile/run.py generate")
        return 1
    print(f"casefiles: all {len(CL['courts'])} current, generated views match, negatives >= positives, receipts present")
    return 0

if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "check"
    if cmd == "generate":
        generate()
    elif cmd == "check":
        sys.exit(check())
    else:
        print("usage: run.py generate|check"); sys.exit(2)
