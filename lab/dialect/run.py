#!/usr/bin/env python3
"""DIALECT.PROFILE.1 — record the declared GnuCOBOL WITNESS profile that produced a court's evidence.
*Dialect profile is evidence, not metadata.* It captures: GnuCOBOL version, dialect/-std, source format,
options, oracle identity (cobc/libcob sha256), and a stable profile_sha256 — so every oracle-grounded
casefile can say "GnuCOBOL 3.2.0 under profile X produced this," not "COBOL says."

  run.py generate   # derive reports/dialect-profile/default.json from the admitted oracle
  run.py check      # gate: profile well-formed, profile_sha256 self-consistent, -std changes the hash,
                    #       and (when the oracle is present) the witness sha/version still match.
It claims NOTHING about general COBOL, other dialects, vendor parity, runtime portability, or NIST.
"""
import hashlib, json, os, re, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
PREFIX = os.path.join(ROOT, "lab/oracle/prefix")
OUT = os.path.join(ROOT, "reports/dialect-profile/default.json")

def sha_file(p):
    return hashlib.sha256(open(p, "rb").read()).hexdigest() if os.path.exists(p) else None

def oracle_present():
    return os.path.exists(os.path.join(PREFIX, "bin/cobc"))

def cobc_version():
    if not oracle_present():
        return None
    env = dict(os.environ, LD_LIBRARY_PATH=os.path.join(PREFIX, "lib"))
    out = subprocess.run([os.path.join(PREFIX, "bin/cobc"), "--version"], capture_output=True, text=True, env=env).stdout
    m = re.search(r"\(GnuCOBOL\)\s*([0-9.]+)", out)
    return m.group(1) if m else None

def libcob_path():
    d = os.path.join(PREFIX, "lib")
    if not os.path.isdir(d):
        return None
    cands = sorted([f for f in os.listdir(d) if f.startswith("libcob.so.") and f.count(".") >= 3], reverse=True)
    return os.path.join(d, cands[0]) if cands else None

def build_profile(std="default", version=None, cobc_sha=None, libcob_sha=None):
    """Return the profile dict with a stable profile_sha256 over its canonical content."""
    content = {
        "schema": "kobold-dialect-profile-v1",
        "profile_id": f"gnucobol-{version or 'unknown'}-{std}",
        "witness": {"compiler": "GnuCOBOL", "version": version or "unknown",
                    "cobc_sha256": cobc_sha, "libcob_sha256": libcob_sha},
        "dialect": {"std": std, "source_format": "free", "options": []},
        "oracle_status": "admitted_witness",
        "non_claims": ["NEG.DIALECT.IMPLICIT", "NEG.DIALECT.GENERAL_COBOL", "NEG.DIALECT.VENDOR_PARITY",
                       "NEG.DIALECT.RUNTIME_PORTABILITY", "NEG.COBOL.NIST_CONFORMANCE", "NEG.PLATFORM.RUNTIME_NOT_CLAIMED"],
    }
    psha = hashlib.sha256(json.dumps(content, sort_keys=True).encode()).hexdigest()
    return content | {"profile_sha256": psha}

def derive_default():
    """Derive from the live oracle if present, else from the committed profile (witness sha pinned there)."""
    if oracle_present():
        return build_profile("default", cobc_version(),
                             sha_file(os.path.join(PREFIX, "bin/cobc")), sha_file(libcob_path()))
    if os.path.exists(OUT):
        c = json.load(open(OUT))
        return build_profile("default", c["witness"]["version"], c["witness"]["cobc_sha256"], c["witness"]["libcob_sha256"])
    return None

def generate():
    p = derive_default()
    if p is None:
        print("DIALECT.PROFILE.1: oracle absent and no committed profile -> cannot generate"); sys.exit(2)
    json.dump(p, open(OUT, "w"), indent=2)
    print(f"dialect profile: {p['profile_id']} (cobc {p['witness']['cobc_sha256'][:12] if p['witness']['cobc_sha256'] else 'n/a'}…) profile_sha256 {p['profile_sha256'][:12]}…")

def check():
    bad = 0
    if not os.path.exists(OUT):
        print("GATE: reports/dialect-profile/default.json missing (run: python3 lab/dialect/run.py generate)"); return 1
    c = json.load(open(OUT))
    # 1. profile_sha256 self-consistency (no hand-edit)
    body = {k: v for k, v in c.items() if k != "profile_sha256"}
    if hashlib.sha256(json.dumps(body, sort_keys=True).encode()).hexdigest() != c.get("profile_sha256"):
        print("GATE: dialect profile_sha256 != recomputed (hand-edited or stale)"); bad += 1
    # 2. default dialect is EXPLICIT, not implicit
    if c.get("dialect", {}).get("std") != "default":
        print("GATE: default profile std is not explicitly 'default'"); bad += 1
    # 3. changing -std changes the profile hash (dialect is part of the hashed content)
    v, cs, ls = c["witness"]["version"], c["witness"]["cobc_sha256"], c["witness"]["libcob_sha256"]
    if build_profile("default", v, cs, ls)["profile_sha256"] == build_profile("ibm-strict", v, cs, ls)["profile_sha256"]:
        print("GATE: changing -std did NOT change profile_sha256 (dialect not bound into the hash)"); bad += 1
    # 4. when the oracle is present, the committed witness must still match the live binaries
    if oracle_present():
        if c["witness"]["version"] != cobc_version():
            print("GATE: committed dialect version != live cobc --version"); bad += 1
        if c["witness"]["cobc_sha256"] != sha_file(os.path.join(PREFIX, "bin/cobc")):
            print("GATE: committed cobc_sha256 != live cobc binary"); bad += 1
    if bad:
        print(f"!! {bad} DIALECT.PROFILE.1 finding(s)"); return 1
    print(f"DIALECT.PROFILE.1: profile {c['profile_id']} self-consistent; -std binds the hash; witness {'matches live oracle' if oracle_present() else 'pinned (oracle absent)'}")
    return 0

if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "check"
    {"generate": generate, "check": lambda: sys.exit(check())}.get(cmd, lambda: sys.exit("usage: generate|check"))()
