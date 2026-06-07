#!/usr/bin/env python3
"""TRUST.2 — generated receipts. For each campaign, run its sweep LIVE and emit
reports/receipts/<CAMPAIGN>/receipt.json (binding evidence) + receipt.md (GENERATED from the json).

  run.py generate [stamp] [git_commit]   # regenerate all current receipts from live replay
  run.py check                           # regenerate in memory; FAIL (exit 1) if a committed receipt's
                                         #   evidence drifted from live, or receipt.md != render(json),
                                         #   or a manifest campaign lacks a receipt. (the doc-gate hook)

The receipt of record is the .json. The .md is a rendering — never hand-edited. generated_at / git_commit
are informational stamps and are excluded from the drift comparison (only evidence + prose bind)."""
import json, os, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
PREFIX = os.path.join(ROOT, "lab/oracle/prefix")
MAN = json.load(open(os.path.join(ROOT, "lab/receipt/manifest.json")))
RECDIR = os.path.join(ROOT, "reports/receipts")

def crate_version():
    for ln in open(os.path.join(ROOT, "crates/gnucobol-rs/Cargo.toml")):
        if ln.startswith("version"):
            return ln.split('"')[1]
    return "?"

def oracle_version():
    cobc = os.path.join(PREFIX, "bin/cobc")
    if not os.path.exists(cobc):
        return "not-built"
    env = dict(os.environ, LD_LIBRARY_PATH=os.path.join(PREFIX, "lib"))
    try:
        return subprocess.run([cobc, "--version"], capture_output=True, text=True, env=env).stdout.splitlines()[0]
    except Exception:
        return "error"

def run_sweep(script, arg):
    path = os.path.join(ROOT, "lab/oracle", script)
    if not os.path.exists(os.path.join(PREFIX, "bin/cobc")) or not os.path.exists(path):
        return "oracle-not-built"
    cmd = ["bash", path] + ([arg] if arg else [])
    try:
        out = subprocess.run(cmd, capture_output=True, text=True, cwd=ROOT).stdout
        for line in out.splitlines():
            if line.startswith("PASS=") and "FAIL=" in line:
                return line.strip()
    except Exception:
        pass
    return "no-result"

def build(code, stamp, git_commit):
    m = MAN["campaigns"][code]
    result = run_sweep(m["sweep"], m.get("arg"))
    verdict = "pass" if result.endswith("FAIL=0") else ("oracle-not-built" if "not-built" in result else "fail")
    return {
        "schema": "gnurust-replay-receipt-v1",
        "campaign": code,
        "court": m["court"],
        "generated_at": stamp,
        "git_commit": git_commit,
        "crate_version": crate_version(),
        "oracle": {"name": "GnuCOBOL", "version": oracle_version()},
        "command": {"replay": f"bash lab/oracle/{m['sweep']}" + (f" {m['arg']}" if m.get('arg') else "")},
        "byte_domain": m["byte_domain"],
        "results": {"sweep": result},
        "non_claims": m["non_claims"],
        "verdict": verdict,
        "receipt_status": m.get("receipt_status", "current"),
        "superseded_by": m.get("superseded_by", None),
        "current_authority": "STATUS.md",
    }

def evidence(r):
    """The binding subset (drift comparison ignores volatile stamps)."""
    return {k: r[k] for k in ("campaign", "court", "crate_version", "command", "byte_domain", "results", "non_claims", "verdict", "receipt_status", "superseded_by")} | {"oracle_version": r["oracle"]["version"]}

def render_md(r):
    nc = "\n".join(f"- {n}" for n in r["non_claims"])
    sup = f" (superseded_by {r['superseded_by']})" if r.get("superseded_by") else ""
    return f"""<!-- GENERATED from receipt.json by lab/receipt/run.py — DO NOT EDIT BY HAND.
     Regenerate: python3 lab/receipt/run.py generate -->
# {r['campaign']} — {r['court']}

**Verdict: {r['verdict'].upper()}** · replay `{r['results']['sweep']}`

| field | value |
|-------|-------|
| campaign | `{r['campaign']}` |
| court | {r['court']} |
| crate_version | `{r['crate_version']}` |
| oracle | {r['oracle']['version']} |
| byte_domain | {r['byte_domain']} |
| replay command | `{r['command']['replay']}` |
| generated_at | {r['generated_at']} |
| git_commit | `{r['git_commit']}` |
| receipt_status | {r['receipt_status']}{sup} |

## Non-claims
{nc}

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `python3 lab/receipt/run.py generate`.
"""

def generate(stamp, git_commit):
    for code in MAN["campaigns"]:
        r = build(code, stamp, git_commit)
        d = os.path.join(RECDIR, code)
        os.makedirs(d, exist_ok=True)
        json.dump(r, open(os.path.join(d, "receipt.json"), "w"), indent=2)
        open(os.path.join(d, "receipt.md"), "w").write(render_md(r))
    print(f"generated {len(MAN['campaigns'])} receipts in reports/receipts/")

def check():
    bad = 0
    for code in MAN["campaigns"]:
        d = os.path.join(RECDIR, code)
        jf, mf = os.path.join(d, "receipt.json"), os.path.join(d, "receipt.md")
        if not os.path.exists(jf):
            print(f"DRIFT: {code} has no generated receipt.json"); bad += 1; continue
        committed = json.load(open(jf))
        fresh = build(code, committed.get("generated_at", ""), committed.get("git_commit", ""))
        if evidence(committed) != evidence(fresh):
            print(f"DRIFT: {code} receipt evidence != live replay (regenerate)"); bad += 1
        if open(mf).read() != render_md(committed):
            print(f"DRIFT: {code} receipt.md was hand-edited (!= render(receipt.json))"); bad += 1
    # Static authored RECEIPT-*.md may summarize, but their stated sweep numbers must MATCH the
    # generated (live) receipt — so the prose receipts cannot drift either.
    import glob, re
    gen_result = {c: json.load(open(os.path.join(RECDIR, c, "receipt.json")))["results"]["sweep"]
                  for c in MAN["campaigns"] if os.path.exists(os.path.join(RECDIR, c, "receipt.json"))}
    for sf in glob.glob(os.path.join(ROOT, "reports/RECEIPT-GNURUST-*.md")):
        txt = open(sf).read()
        cm = re.search(r"^campaign:\s*(GNURUST\.\d+)", txt, re.M)
        if not cm:
            continue
        code = cm.group(1)
        live = gen_result.get(code)
        if not live or live == "oracle-not-built":
            continue
        # The static receipt must state the current live primary-sweep result (it may also cite related
        # sweeps — so we require presence, not that every PASS= line matches).
        if live not in txt:
            print(f"DRIFT: static {os.path.basename(sf)} does not state live {code} result '{live}'")
            bad += 1
    # claim-ladder must only cite campaigns that have a generated receipt
    cl = json.load(open(os.path.join(ROOT, "reports/claim-ladder.json")))
    for c in cl["courts"]:
        cid = c["id"]
        if cid.startswith("GNURUST.") and cid in MAN["campaigns"] and not os.path.exists(os.path.join(RECDIR, cid, "receipt.json")):
            print(f"DRIFT: claim-ladder cites {cid} with no generated receipt"); bad += 1
    if bad:
        print(f"!! {bad} receipt drift(s) — regenerate with: python3 lab/receipt/run.py generate")
        return 1
    print("receipts: all current, .md == render(.json), claim-ladder covered")
    return 0

if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "check"
    if cmd == "generate":
        generate(sys.argv[2] if len(sys.argv) > 2 else "unstamped", sys.argv[3] if len(sys.argv) > 3 else "unstamped")
    elif cmd == "check":
        sys.exit(check())
    else:
        print("usage: run.py generate|check"); sys.exit(2)
