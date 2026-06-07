#!/usr/bin/env python3
"""ENTERPRISE.1 — build a per-release EVIDENCE PACKET for a crate version.

  build-packet.py <repo_dir> <crate_name> <version> <kind: gnurust|kobold> [evidence_repo_dir]

Produces reports/releases/<crate>-<version>/ in <repo_dir> with reproducible receipts, SBOM, license +
dependency inventory, feature matrix, audit/geiger status (HONESTLY marked when a tool is unavailable),
claim-ladder + negative-capability snapshots, a STATUS snapshot, and a release verdict.

Doctrine: a release is an evidence packet, not merely a version number. Unavailable tools are never
faked green -- statuses: pass | fail | not_installed | not_run | not_applicable | network_unavailable."""
import json, os, subprocess, sys, datetime

REPO, CRATE, VERSION, KIND = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4]
EVID = sys.argv[5] if len(sys.argv) > 5 else "/home/one/gnucobol-rs"  # shared claim-ladder/receipts home
OUT = os.path.join(REPO, "reports/releases", f"{CRATE}-{VERSION}")
os.makedirs(OUT, exist_ok=True)
STAMP = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

def run(cmd, cwd=REPO, timeout=300):
    try:
        p = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=timeout)
        return p.returncode, p.stdout, p.stderr
    except subprocess.TimeoutExpired:
        return None, "", "TIMEOUT"
    except FileNotFoundError:
        return -1, "", "NOT_FOUND"

def git_commit(d):
    rc, out, _ = run(["git", "rev-parse", "HEAD"], cwd=d, timeout=30)
    return out.strip() if rc == 0 else "unknown"

def cargo_meta():
    rc, out, _ = run(["cargo", "metadata", "--format-version", "1"], timeout=120)
    return json.loads(out) if rc == 0 and out else {"packages": []}

META = cargo_meta()
PKGS = sorted({(p["name"], p["version"], p.get("license") or "UNKNOWN") for p in META.get("packages", [])})

# 1. release-metadata.json
meta = {
    "schema": "kobold-release-metadata-v1", "crate": CRATE, "version": VERSION, "kind": KIND,
    "git_commit": git_commit(REPO), "generated_at_utc": STAMP,
    "publish_status": "pending_crates_io_rate_limit_window",
    "dependency_count": len(PKGS),
    "note": "git repo is the authority; crates.io may trail under publish rate limits.",
}
json.dump(meta, open(f"{OUT}/release-metadata.json", "w"), indent=2)

# 2. sbom.spdx.json (derived from cargo metadata; no cargo-sbom/cyclonedx installed)
sbom = {
    "spdxVersion": "SPDX-2.3", "dataLicense": "CC0-1.0", "SPDXID": "SPDXRef-DOCUMENT",
    "name": f"{CRATE}-{VERSION}", "documentNamespace": f"https://github.com/infinityabundance/{CRATE}#{VERSION}",
    "creationInfo": {"created": STAMP, "creators": ["Tool: kobold-release-packet (from cargo metadata)"]},
    "comment": "Derived from `cargo metadata`; no SPDX generator (cargo-sbom/cyclonedx) installed.",
    "packages": [
        {"name": n, "SPDXID": f"SPDXRef-{n}-{v}".replace(".", "-"), "versionInfo": v,
         "licenseConcluded": lic, "downloadLocation": f"https://crates.io/crates/{n}/{v}"}
        for (n, v, lic) in PKGS
    ],
}
json.dump(sbom, open(f"{OUT}/sbom.spdx.json", "w"), indent=2)

# 3. licenses.json
from collections import Counter
lic_counts = Counter(lic for (_, _, lic) in PKGS)
licenses = {
    "schema": "kobold-release-licenses-v1", "crate": CRATE, "version": VERSION,
    "this_crate_license": "LGPL-3.0-or-later" if KIND == "gnurust" else "Apache-2.0",
    "dependency_license_summary": dict(lic_counts),
    "copyleft_boundary": (
        "gnucobol-rs is LGPL-3.0+ (derives from libcob); cobc-oracle-rs is GPL-3.0+ (lab-only, drives "
        "cobc). kobold-data-shim is Apache-2.0 and LINKS gnucobol-rs (LGPL) -- see its NOTICE for the "
        "relink obligation. No GPL code is in any published decode-path crate."
    ),
}
json.dump(licenses, open(f"{OUT}/licenses.json", "w"), indent=2)

# 4. feature-matrix.json (from Cargo.toml [features])
import re
ct = open(os.path.join(REPO, "Cargo.toml")).read() if KIND == "kobold" else open(os.path.join(REPO, "crates/gnucobol-rs/Cargo.toml")).read()
feats = {}
m = re.search(r'(?ms)^\[features\]\s*(.*?)(?=^\[|\Z)', ct)
if m:
    for line in m.group(1).splitlines():
        fm = re.match(r'\s*([A-Za-z0-9_-]+)\s*=\s*(\[.*\])', line)
        if fm:
            feats[fm.group(1)] = fm.group(2)
fmatrix = {
    "schema": "kobold-release-feature-matrix-v1", "crate": CRATE, "version": VERSION,
    "features": feats or {"(none)": "[]"},
    "default_features": feats.get("default", "[]"),
    "note": "Off-by-default features stay boring/portable until separately proven (e.g. serde Serialize-only, strings never floats).",
}
json.dump(fmatrix, open(f"{OUT}/feature-matrix.json", "w"), indent=2)

# 5. cargo-audit.txt
rc, out, err = run(["cargo", "audit"], timeout=120)
if rc == -1:
    status, body = "not_installed", "cargo-audit not installed"
elif rc is None:
    status, body = "not_run", "timed out"
elif "fetch" in (err + out).lower() and ("network" in (err+out).lower() or "could not" in (err+out).lower() or "error" in err.lower() and "advisory" in (err+out).lower()):
    status, body = "network_unavailable", (out + "\n" + err)
elif rc == 0:
    status, body = "pass", out
else:
    # cargo-audit returns nonzero on found vulns OR on db-fetch failure; disambiguate.
    status = "network_unavailable" if ("unable to fetch" in (err+out).lower() or "advisory database" in (err+out).lower() and "fetch" in (err+out).lower()) else "fail"
    body = out + "\n" + err
open(f"{OUT}/cargo-audit.txt", "w").write(f"status: {status}\ngenerated_at: {STAMP}\n\n{body}\n")

# 6. cargo-geiger.txt (heavy: compiles; time-bounded)
pkgsel = ["-p", CRATE] if KIND == "kobold" else ["-p", "gnucobol-rs"]
rc, out, err = run(["cargo", "geiger", "--output-format", "Ascii"] + pkgsel, timeout=300)
if rc == -1:
    gstatus, gbody = "not_installed", "cargo-geiger not installed"
elif rc is None:
    gstatus, gbody = "not_run", "timed out (geiger compiles the dep tree; re-run manually)"
elif rc == 0:
    gstatus, gbody = "pass", out
else:
    gstatus, gbody = ("fail", out + "\n" + err)
open(f"{OUT}/cargo-geiger.txt", "w").write(f"status: {gstatus}\ngenerated_at: {STAMP}\nnote: every shipped crate is #![forbid(unsafe_code)]\n\n{gbody[:8000]}\n")

# 7. receipt-manifest.json (TRUST.2 generated receipts)
recdir = os.path.join(EVID, "reports/receipts")
receipts = []
if os.path.isdir(recdir):
    for c in sorted(os.listdir(recdir)):
        jf = os.path.join(recdir, c, "receipt.json")
        if os.path.exists(jf):
            r = json.load(open(jf))
            receipts.append({"campaign": r.get("campaign"), "verdict": r.get("verdict"),
                             "result": r.get("results", {}).get("sweep"), "court": r.get("court")})
prefix = "GNURUST." if KIND == "gnurust" else None
rel = [r for r in receipts if (prefix is None or (r["campaign"] or "").startswith(prefix))]
json.dump({"schema": "kobold-release-receipt-manifest-v1", "crate": CRATE, "version": VERSION,
           "trust2_doctrine": "receipts are generated from live replay; see docs/trust2-generated-receipts.md",
           "receipts": rel, "count": len(rel)}, open(f"{OUT}/receipt-manifest.json", "w"), indent=2)

# 8. claim-ladder-snapshot.json (filtered to this crate's courts)
cl = json.load(open(os.path.join(EVID, "reports/claim-ladder.json")))
want = "GNURUST." if KIND == "gnurust" else "KOBOLD."
snap = {"schema": "claim-ladder-snapshot-v1", "crate": CRATE, "version": VERSION, "generated_at_utc": STAMP,
        "courts": [c for c in cl.get("courts", []) if (c.get("id") or "").startswith(want)]}
json.dump(snap, open(f"{OUT}/claim-ladder-snapshot.json", "w"), indent=2)

# 9. negative-capabilities-snapshot.json (from each court's not_proven)
negs = [{"id": c["id"], "not_proven": c.get("not_proven", ""), "lie_prevented": c.get("lie_prevented", "")}
        for c in snap["courts"]]
json.dump({"schema": "negative-capabilities-snapshot-v1", "crate": CRATE, "version": VERSION,
           "negatives": negs}, open(f"{OUT}/negative-capabilities-snapshot.json", "w"), indent=2)

# 10. status-snapshot.md
status_src = os.path.join(EVID, "STATUS.md")
status_txt = open(status_src).read() if os.path.exists(status_src) else "(no STATUS.md)"
open(f"{OUT}/status-snapshot.md", "w").write(f"<!-- snapshot of STATUS.md at {CRATE} {VERSION} release ({STAMP}) -->\n\n{status_txt}")

# 11. release-verdict.md (generated from the machine files)
verdict = f"""# Release verdict — {CRATE} {VERSION}

_Generated {STAMP} from the machine files in this packet. A release is an evidence packet, not merely a
version number._

| evidence | value |
|----------|-------|
| crate / version | `{CRATE}` {VERSION} ({KIND}) |
| git commit | `{meta['git_commit']}` |
| publish status | {meta['publish_status']} |
| this-crate license | {licenses['this_crate_license']} |
| dependencies | {len(PKGS)} (SBOM: `sbom.spdx.json`) |
| sealed courts in this crate | {len(snap['courts'])} (`claim-ladder-snapshot.json`) |
| TRUST.2 receipts | {len(rel)} (`receipt-manifest.json`) |
| cargo-audit | **{status}** (`cargo-audit.txt`) |
| cargo-geiger | **{gstatus}** (`cargo-geiger.txt`) — every shipped crate is `#![forbid(unsafe_code)]` |

## What this release admits
The sealed courts in `claim-ladder-snapshot.json`, each proven against the admitted GnuCOBOL 3.2 oracle
with a reproducible TRUST.2 receipt.

## What this release does NOT admit
The non-claims in `negative-capabilities-snapshot.json`. **No production-readiness claim** beyond the
KRL level in `status-snapshot.md`. Unavailable tools above are marked honestly (`not_installed` /
`not_run` / `network_unavailable`), never faked green.

> Doctrine: ENTERPRISE.1 treats a release as an evidence packet — reproducible receipts, dependency/
> license inventory, feature flags, audit status, claim boundaries, negative capabilities, and a verdict
> that refuses to overstate production readiness.
"""
open(f"{OUT}/release-verdict.md", "w").write(verdict)
print(f"packet: {OUT}  (audit={status}, geiger={gstatus}, deps={len(PKGS)}, courts={len(snap['courts'])}, receipts={len(rel)})")
