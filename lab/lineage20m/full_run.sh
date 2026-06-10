#!/usr/bin/env bash
# GNURUST.LINEAGE.CORPUS.20M.1 -- the DETACHED, RESUMABLE full 20M run. Writes ONLY to
# reports/lineage20m/full-run/ (LINEAGE20M_OUT) so the sealed .SMOKE artifacts stay immutable.
# Resumable: a shard whose receipt already exists is skipped. Run detached; re-run to resume.
#   nohup nice -n 10 bash lab/lineage20m/full_run.sh >reports/lineage20m/full-run/run.log 2>&1 &
#
# v0 engine emits 2 of 15 planned families (storage + directive); the other 13 families' indices are
# logged as dropped buckets (honest -- not silently covered). The 20M index space therefore yields the
# storage (2M) + directive (2M) real-cobc witnesses + ~16M dropped-bucket entries. ~8h at 16 workers.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
export LINEAGE20M_OUT="reports/lineage20m/full-run"
N_SHARDS=1024
TOTAL=20000000
PER=$((TOTAL / N_SHARDS))
mkdir -p "$LINEAGE20M_OUT/shards"
echo "FULL-RUN START $(date -u) shards=$N_SHARDS per=$PER out=$LINEAGE20M_OUT"
for ((s=0; s<N_SHARDS; s++)); do
  rcpt="$LINEAGE20M_OUT/shards/shard-$(printf '%06d' "$s").receipt.json"
  [ -f "$rcpt" ] && continue                      # resumable: skip completed shard
  start=$((s * PER)); cnt=$PER
  [ "$s" -eq $((N_SHARDS - 1)) ] && cnt=$((TOTAL - start))
  python3 lab/lineage20m/run.py burn --start "$start" --count "$cnt" --shard "$s" \
          --workers 16 --sample-stride 500 2>&1 | tail -1
  if (( s % 16 == 0 )); then
    python3 - "$LINEAGE20M_OUT" <<'PY' 2>/dev/null || true
import json, glob, sys, os
out = sys.argv[1]
rs = [json.load(open(f)) for f in glob.glob(os.path.join(out, "shards", "*.receipt.json"))]
summ = {"schema": "gnurust-lineage20m-live-summary-v1", "court": "GNURUST.LINEAGE.CORPUS.20M.1",
        "shards_done": len(rs), "witnesses": sum(r.get("generated", 0) for r in rs),
        "reddening": sum(r.get("reddening", 0) for r in rs),
        "untriaged": sum(r.get("untriaged", 0) for r in rs),
        "dropped_total": sum(sum(r.get("dropped_buckets", {}).values()) for r in rs),
        "status": "running"}
json.dump(summ, open(os.path.join(out, "live-summary.json"), "w"), indent=2)
PY
  fi
done
python3 lab/lineage20m/run.py verify-merkle 2>&1 | tail -1
python3 - "$LINEAGE20M_OUT" <<'PY' 2>/dev/null || true
import json, glob, sys, os
out = sys.argv[1]
rs = [json.load(open(f)) for f in glob.glob(os.path.join(out, "shards", "*.receipt.json"))]
summ = {"schema": "gnurust-lineage20m-live-summary-v1", "court": "GNURUST.LINEAGE.CORPUS.20M.1",
        "shards_done": len(rs), "witnesses": sum(r.get("generated", 0) for r in rs),
        "reddening": sum(r.get("reddening", 0) for r in rs),
        "untriaged": sum(r.get("untriaged", 0) for r in rs), "status": "complete"}
json.dump(summ, open(os.path.join(out, "live-summary.json"), "w"), indent=2)
PY
echo "FULL-RUN DONE $(date -u)"
