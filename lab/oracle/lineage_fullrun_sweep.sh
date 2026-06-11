#!/usr/bin/env bash
# GNURUST.LINEAGE.CORPUS.20M.1 -- verify-sealed adapter for the COMPLETED full 20M run. The 1024 shard
# receipts (4M witnesses) are regenerable-from-seed + gitignored; the committed evidence-of-record is
# reports/lineage20m/full-run-seal.json (its root-of-roots binds the regenerable shards). This asserts
# the seal is intact + gate=PASS; if the live full-run tree is present it re-checks the root matches.
# Re-runnable in a fresh checkout WITHOUT the 4M tree (the seal summary is the binding record).
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
( cd "$ROOT" && cargo run -q -p xtask -- lineage fullrun )
