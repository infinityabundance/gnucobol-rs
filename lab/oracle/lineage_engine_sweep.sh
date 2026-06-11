#!/usr/bin/env bash
# GNURUST.LINEAGE.CORPUS.20M.0 -- ENGINE self-test (no burn). Verifies the machinery is sound:
# the plan budgets sum to exactly 20M, every family is owned, the LCG is deterministic + matches the
# Rust Lehmer constant, the Merkle tree is well-formed (tamper -> root change), and the schema validates.
# This seals the ENGINE; the 200K burn is GNURUST.LINEAGE.CORPUS.20M.SMOKE (lineage_corpus_sweep.sh).
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
( cd "$ROOT" && cargo run -q -p xtask -- lineage engine )
