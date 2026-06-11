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
( cd "$ROOT" && cargo run -q -p xtask -- lineage smoke )
