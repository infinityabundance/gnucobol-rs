#!/usr/bin/env bash
# GNURUST.LINEAGE.CORPUS.20M -- verify-sealed-courts adapter. Runs the lineage engine's seal-grade gate
# (classification + Merkle + stratified replay determinism + parallel-vs-serial harness isolation +
# findings completeness) and emits PASS=<witnesses> FAIL=0/1 for the verify table. The gate is the court:
# a 20M-witness real-cobc lineage corpus is only as trustworthy as its replay+isolation determinism.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
if python3 "$ROOT/lab/lineage20m/run.py" check >/tmp/_lineage_gate.out 2>&1; then
  N=$(python3 -c "import json,glob; print(sum(json.load(open(f)).get('generated',0) for f in glob.glob('$ROOT/reports/lineage20m/shards/*.receipt.json')))" 2>/dev/null || echo 0)
  echo "PASS=$N FAIL=0"
else
  echo "PASS=0 FAIL=1"
  cat /tmp/_lineage_gate.out
fi
