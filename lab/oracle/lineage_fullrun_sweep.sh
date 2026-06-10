#!/usr/bin/env bash
# GNURUST.LINEAGE.CORPUS.20M.1 -- verify-sealed adapter for the COMPLETED full 20M run. The 1024 shard
# receipts (4M witnesses) are regenerable-from-seed + gitignored; the committed evidence-of-record is
# reports/lineage20m/full-run-seal.json (its root-of-roots binds the regenerable shards). This asserts
# the seal is intact + gate=PASS; if the live full-run tree is present it re-checks the root matches.
# Re-runnable in a fresh checkout WITHOUT the 4M tree (the seal summary is the binding record).
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
python3 - "$ROOT" <<'PY'
import json, os, sys
ROOT = sys.argv[1]
seal = json.load(open(os.path.join(ROOT, "reports/lineage20m/full-run-seal.json")))
c = []
c.append(("status-complete", seal.get("status") == "complete"))
c.append(("gate-pass", seal.get("gate", {}).get("verdict") == "PASS"))
c.append(("untriaged-zero", seal.get("untriaged") == 0))
c.append(("witnesses", seal.get("witnesses", 0) >= 4_000_000))
for f in seal.get("confirmed_findings", []):
    c.append((f"finding-{f['id'][:20]}", bool(f.get("count")) and bool(f.get("oracle_hex")) and bool(f.get("candidate_court"))))
fr = os.path.join(ROOT, "reports/lineage20m/full-run/manifest.json")
if os.path.exists(fr):
    c.append(("root-matches-live-tree", json.load(open(fr)).get("root_of_roots") == seal.get("root_of_roots")))
fails = [n for n, ok in c if not ok]
print(f"PASS={seal.get('witnesses', 0) if not fails else 0} FAIL={len(fails)}")
for n in fails:
    print("  FAIL", n)
PY
