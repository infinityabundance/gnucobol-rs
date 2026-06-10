#!/usr/bin/env bash
# GNURUST.LINEAGE.CORPUS.20M.SMOKE -- verify-sealed adapter for the 200K real-cobc PILOT burn (the
# HISTORICAL discovery run, sealed at the pre-patch commit). Like .20M.1, this asserts a committed
# seal-of-record (reports/lineage20m/smoke-seal.json) and RECOMPUTES the Merkle root-of-roots from the
# committed shard receipts -- a CODE-INDEPENDENT integrity check (witnesses + oracle bytes, not the
# current Rust value_image). It does NOT re-run the live rust classification: this is a point-in-time
# forensic record, and the GNURUST.VALUE.NEGZERO.EDGE.1 patch (0.7.27) DELIBERATELY changed value_image,
# which would (correctly) make a live replay disagree -- that is a forward fix, not corpus tampering.
# (Same-version live re-verification is still available via `run.py check` over the .SMOKE tree.)
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
python3 - "$ROOT" <<'PY'
import json, os, sys, glob
ROOT = sys.argv[1]
sys.path.insert(0, os.path.join(ROOT, "lab"))
from lineage20m import merkle
seal = json.load(open(os.path.join(ROOT, "reports/lineage20m/smoke-seal.json")))
recs = [json.load(open(f)) for f in sorted(glob.glob(os.path.join(ROOT, "reports/lineage20m/shards/*.receipt.json")))]
c = []
c.append(("status-complete", seal.get("status") == "complete"))
c.append(("gate-pass", seal.get("gate_at_seal", {}).get("verdict") == "PASS"))
c.append(("untriaged-zero", seal.get("untriaged") == 0))
c.append(("witnesses", seal.get("witnesses", 0) >= 200_000))
c.append(("injected-faults-4-4", seal.get("gate_at_seal", {}).get("injected_faults", "").startswith("4/4")))
for f in seal.get("confirmed_findings", []):
    c.append((f"finding-{f['id'][:18]}", bool(f.get("count")) and bool(f.get("oracle_hex")) and bool(f.get("candidate_court"))))
# code-INDEPENDENT integrity: recompute root-of-roots from the committed shard receipts
roots = [r["merkle_root"] for r in sorted(recs, key=lambda x: x["shard_id"])]
ror = merkle.root_of_roots(roots)
c.append(("merkle-root-of-roots-matches-seal", ror == seal.get("root_of_roots")))
fails = [n for n, ok in c if not ok]
print(f"PASS={seal.get('witnesses', 0) if not fails else 0} FAIL={len(fails)}")
for n in fails: print("  FAIL", n)
PY
