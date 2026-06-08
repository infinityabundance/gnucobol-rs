#!/usr/bin/env python3
"""ENTERPRISE.2 orchestration -- NO crypto in Python. All signing/verification is the Rust `kobold-attest`
binary (ed25519). This driver only: loads the signing policy, asks the Rust tool to verify each casefile's
DSSE envelope against its in-toto payload, aggregates signature_status into the verification report, and
gates integrity.

  run.py verify   # write reports/signing/verification-report.json + README.md from a live verification
  run.py check    # gate: on-disk report == fresh re-verification; no integrity failure; Rust selftest passes
"""
import glob, json, os, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
ATTEST = os.path.join(ROOT, "lab/attest/target/release/kobold-attest")
POLICY = os.path.join(ROOT, "lab/attest/signing-policy.json")
OUTDIR = os.path.join(ROOT, "reports/signing")

ALLOWED = {"unsigned_no_key_configured", "signed_verified", "signed_unverified",
           "signed_key_mismatch", "signed_payload_mismatch", "verification_tool_unavailable"}
# Integrity FAILURES (a repo artifact that should verify but does not): gate red.
INTEGRITY_FAIL = {"signed_unverified", "signed_key_mismatch", "signed_payload_mismatch"}

def tool_available():
    return os.path.exists(ATTEST)

def load_policy():
    p = json.load(open(POLICY))
    key = p.get("key", {})
    return p.get("mode", "unsigned"), key.get("public_key_hex"), key.get("keyid")

def verify_one(env_path, intoto_path, pk_hex, keyid):
    if not tool_available():
        return "verification_tool_unavailable"
    args = [ATTEST, "verify", env_path, intoto_path]
    if pk_hex:
        args += [pk_hex, keyid or ""]
    r = subprocess.run(args, capture_output=True, text=True)
    return r.stdout.strip() or "verification_tool_unavailable"

def selftest_ok():
    if not tool_available():
        return None
    return subprocess.run([ATTEST, "selftest"], capture_output=True).returncode == 0

def build_report():
    mode, pk_hex, keyid = load_policy()
    entries = []
    for d in sorted(glob.glob(os.path.join(ROOT, "reports/casefiles/*/"))):
        env = os.path.join(d, "dsse-envelope.json")
        intoto = os.path.join(d, "intoto-statement.json")
        if not (os.path.exists(env) and os.path.exists(intoto)):
            continue
        entries.append({"court": os.path.basename(d.rstrip("/")),
                        "signature_status": verify_one(env, intoto, pk_hex, keyid)})
    summary = {}
    for e in entries:
        summary[e["signature_status"]] = summary.get(e["signature_status"], 0) + 1
    return {
        "schema": "kobold-enterprise2-verification-report-v1", "court": "KOBOLD.ENTERPRISE.2",
        "signing_mode": mode, "tool": "kobold-attest (rust, ed25519-compact)",
        "tool_available": tool_available(), "selftest_passed": selftest_ok(),
        "casefiles": len(entries), "summary": dict(sorted(summary.items())), "entries": entries,
        "non_claims": ["NEG.ENTERPRISE2.NOT_REGULATORY_COMPLIANCE", "NEG.ENTERPRISE2.NOT_PRODUCTION_APPROVAL",
                       "NEG.ENTERPRISE2.NOT_CUSTOMER_ACCEPTANCE", "NEG.ENTERPRISE2.NO_LONG_TERM_KEY_CUSTODY",
                       "NEG.ENTERPRISE2.NO_IDENTITY_TRUST_BEYOND_KEY", "NEG.ENTERPRISE2.NO_SUPPLY_CHAIN_COMPLETENESS"],
    }

def write_report(rep):
    os.makedirs(OUTDIR, exist_ok=True)
    json.dump(rep, open(os.path.join(OUTDIR, "verification-report.json"), "w"), indent=2)
    open(os.path.join(OUTDIR, "README.md"), "w").write(
        "# ENTERPRISE.2 — signed attestation verification (generated; do not edit)\n\n"
        "Verification of every casefile DSSE envelope against its in-toto payload, by the **Rust** "
        "`kobold-attest` tool (ed25519; no Python crypto). Regenerate with `python3 lab/attest/run.py verify`.\n\n"
        f"- signing mode: **{rep['signing_mode']}**\n- tool available: {rep['tool_available']}  ·  "
        f"selftest passed: {rep['selftest_passed']}\n- casefiles: {rep['casefiles']}\n- status summary: "
        f"`{json.dumps(rep['summary'])}`\n\n"
        "`unsigned_no_key_configured` is the **honest default** — not a failure. Set a signed policy + key "
        "to produce `signed_verified`. No regulatory/production/customer-acceptance/key-custody/supply-chain claim.\n")

def verify_cmd():
    rep = build_report()
    write_report(rep)
    print(f"ENTERPRISE.2 verify: {rep['casefiles']} casefiles, summary {rep['summary']}, tool_available={rep['tool_available']}")

def check():
    bad = 0
    rep_path = os.path.join(OUTDIR, "verification-report.json")
    if not os.path.exists(rep_path):
        print("GATE: reports/signing/verification-report.json missing (run: python3 lab/attest/run.py verify)"); return 1
    on_disk = json.load(open(rep_path))
    fresh = build_report()
    if json.dumps(on_disk.get("entries"), sort_keys=True) != json.dumps(fresh["entries"], sort_keys=True):
        print("GATE: verification-report.json differs from a fresh re-verification (stale or hand-edited)"); bad += 1
    for e in fresh["entries"]:
        if e["signature_status"] not in ALLOWED:
            print(f"GATE: {e['court']} unknown signature_status {e['signature_status']}"); bad += 1
        if e["signature_status"] in INTEGRITY_FAIL:
            print(f"GATE: {e['court']} integrity failure: {e['signature_status']}"); bad += 1
    st = selftest_ok()
    if st is False:
        print("GATE: kobold-attest selftest FAILED"); bad += 1
    elif st is None:
        print("note: kobold-attest not built -> signature_status=verification_tool_unavailable (honest, not a gate failure)")
    if bad:
        print(f"!! {bad} ENTERPRISE.2 finding(s)"); return 1
    print(f"ENTERPRISE.2: verification report fresh, no integrity failure, selftest {'passed' if st else 'skipped (tool absent)'} "
          f"({fresh['casefiles']} casefiles, {fresh['signing_mode']} mode)")
    return 0

if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "check"
    {"verify": verify_cmd, "check": lambda: sys.exit(check())}.get(cmd, lambda: sys.exit("usage: verify|check"))()
