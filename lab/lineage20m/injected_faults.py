#!/usr/bin/env python3
"""Injected-fault tests for GNURUST.LINEAGE.CORPUS.20M.SMOKE -- prove the seal gate is NOT ceremonial.

Each fault is applied to a COPY-then-restore of the sealed receipt tree; `run.py check` MUST go red for
every one. Writes reports/lineage20m/injected-faults.json (the structured evidence block the .SMOKE court
cites). A green baseline before and after proves the faults, not the harness, caused each red."""
import json, os, subprocess, shutil, sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "reports", "lineage20m")


def check_red():
    """Return True iff `run.py check` FAILS (red)."""
    return subprocess.run([sys.executable, os.path.join(ROOT, "lab/lineage20m/run.py"), "check"],
                          capture_output=True).returncode != 0


def main():
    bak = "/tmp/lineage20m_ifbak"
    shutil.rmtree(bak, ignore_errors=True); shutil.copytree(OUT, bak)
    sh = sorted(f for f in os.listdir(os.path.join(OUT, "shards")) if f.endswith(".receipt.json"))[0]
    sh = os.path.join(OUT, "shards", sh)
    findings = os.path.join(OUT, "findings.json")

    def restore():
        shutil.rmtree(OUT); shutil.copytree(bak, OUT)

    results = {}
    # baseline must be GREEN
    results["baseline_green"] = not check_red()
    # 1) generator-manifest mutation
    d = json.load(open(sh)); d["generator_manifest_sha256"] = "deadbeef" * 8; json.dump(d, open(sh, "w"))
    results["manifest_mutation_forces_red"] = check_red(); restore()
    # 2) build-profile mutation
    d = json.load(open(sh)); d["build_profile_sha256"] = "cafe" * 16; json.dump(d, open(sh, "w"))
    results["build_profile_mutation_forces_red"] = check_red(); restore()
    # 3) receipt tamper (flip merkle_root -> root-of-roots mismatch)
    d = json.load(open(sh)); d["merkle_root"] = "0" * 64; json.dump(d, open(sh, "w"))
    results["receipt_tamper_forces_red"] = check_red(); restore()
    # 4) finding reproducer removed (confirmed finding -> malformed)
    f = json.load(open(findings)); k = list(f)[0]; f[k]["shrunk_reproducer"] = None; json.dump(f, open(findings, "w"))
    results["finding_or_refusal_removed_forces_red"] = check_red(); restore()
    # restored baseline must be GREEN again
    results["restored_green"] = not check_red()

    fault_keys = [k for k in results if k.endswith("_forces_red")]
    passed = sum(1 for k in fault_keys if results[k])
    block = {
        "schema": "gnurust-lineage20m-injected-faults-v1",
        "court": "GNURUST.LINEAGE.CORPUS.20M.SMOKE",
        "manifest_mutation_forces_red": results["manifest_mutation_forces_red"],
        "build_profile_mutation_forces_red": results["build_profile_mutation_forces_red"],
        "receipt_tamper_forces_red": results["receipt_tamper_forces_red"],
        "finding_or_refusal_removed_forces_red": results["finding_or_refusal_removed_forces_red"],
        "baseline_green": results["baseline_green"], "restored_green": results["restored_green"],
        "passed": passed, "total": len(fault_keys),
    }
    json.dump(block, open(os.path.join(OUT, "injected-faults.json"), "w"), indent=2)
    allok = passed == len(fault_keys) and results["baseline_green"] and results["restored_green"]
    print("INJECTED-FAULT TESTS:")
    for k in fault_keys:
        print(f"  [{'OK' if results[k] else 'XX'}] {k}: {'forced red' if results[k] else 'DID NOT force red'}")
    print(f"  baseline_green={results['baseline_green']} restored_green={results['restored_green']}")
    print(f"INJECTED-FAULTS: {passed}/{len(fault_keys)} forced red -> {'PASS' if allok else 'FAIL'}")
    shutil.rmtree(bak, ignore_errors=True)
    return 0 if allok else 1


if __name__ == "__main__":
    sys.exit(main())
