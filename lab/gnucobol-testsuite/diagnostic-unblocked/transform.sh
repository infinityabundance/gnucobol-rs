#!/usr/bin/env bash
# transform.sh — HOST-side reproducible generation of the diagnostic-unblocked patch + manifests,
# and the independent Phase-4 policy gate. The docker lane (run-diagnostic-unblocked-docker.sh)
# re-runs this inside the court container so the evidence is regenerated in the pinned environment.
#
#   bash lab/gnucobol-testsuite/diagnostic-unblocked/transform.sh [--suite-src DIR] [--revision REV]
#
# Produces (in reports/gnucobol-testsuite/diagnostic-unblocked/):
#   diagnostic-ignore.patch  transformations.json  transformations.csv  transformations.md
#   tree-manifest.json  preflight.md
# and prints the gate verdict. Deterministic: running twice produces byte-identical patch + manifests.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
REP="$ROOT/reports/gnucobol-testsuite/diagnostic-unblocked"
SUITE_SRC="${1:-$ROOT/lab/admit/gnucobol-3.2/tests/testsuite.src}"
REVISION="${2:-stable-3.2}"
SCRATCH="$REP/scratch"

mkdir -p "$SCRATCH"
rm -rf "$SCRATCH"/*
rm -rf "$SCRATCH"/pristine "$SCRATCH"/transformed 2>/dev/null || true
cargo run -q --manifest-path "$ROOT/Cargo.toml" -p gnucobol-rs-corpus -- \
  diag-unblocked transform "$SUITE_SRC" "$SCRATCH" --revision="$REVISION"
cp "$SCRATCH/diagnostic-ignore.patch" "$SCRATCH/transformations.json" \
   "$SCRATCH/transformations.csv" "$SCRATCH/transformations.md" \
   "$SCRATCH/tree-manifest.json" "$REP/"

echo "=== diagnostic-unblocked gate (independent patch-policy verification) ==="
cargo run -q --manifest-path "$ROOT/Cargo.toml" -p gnucobol-rs-corpus -- \
  diag-unblocked gate \
  "$SCRATCH/diagnostic-ignore.patch" \
  "$SCRATCH/pristine" "$SCRATCH/transformed" "$SCRATCH/transformations.json"
echo "transform + gate: DONE (evidence under $REP)"
