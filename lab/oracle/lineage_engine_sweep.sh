#!/usr/bin/env bash
# GNURUST.LINEAGE.CORPUS.20M.0 -- ENGINE self-test (no burn). Verifies the machinery is sound:
# the plan budgets sum to exactly 20M, every family is owned, the LCG is deterministic + matches the
# Rust Lehmer constant, the Merkle tree is well-formed (tamper -> root change), and the schema validates.
# This seals the ENGINE; the 200K burn is GNURUST.LINEAGE.CORPUS.20M.SMOKE (lineage_corpus_sweep.sh).
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
python3 - "$ROOT" <<'PY'
import sys, os
ROOT = sys.argv[1]; sys.path.insert(0, os.path.join(ROOT, "lab"))
from lineage20m import plan, merkle, schema
from lineage20m.lcg import Lcg, witness_seed
checks = []
# 1) budgets sum to exactly 20M, every family owned
checks.append(("plan-sums-20M", plan.TOTAL == 20_000_000))
checks.append(("families-owned", all(f[2] and f[3] and f[4] for f in plan.FAMILIES)))
# 2) LCG determinism + Rust-constant match
a = Lcg(12345); b = Lcg(12345)
checks.append(("lcg-deterministic", [a.step() for _ in range(5)] == [b.step() for _ in range(5)]))
checks.append(("lcg-rust-constant", Lcg.__module__ and (lambda l: (l.step(), l.state)[1])(Lcg(0)) == ((0*6364136223846793005+1442695040888963407) & ((1<<64)-1))))
checks.append(("witness-seed-stable", witness_seed(7, 99) == witness_seed(7, 99) and witness_seed(7, 99) != witness_seed(7, 100)))
# 3) Merkle well-formed: tamper -> root changes; stable otherwise
leaves = [merkle.leaf(f"row{i}".encode()) for i in range(10)]
r1 = merkle.root(leaves); r2 = merkle.root(leaves)
tampered = list(leaves); tampered[3] = merkle.leaf(b"evil")
checks.append(("merkle-stable", r1 == r2))
checks.append(("merkle-tamper-detected", merkle.root(tampered) != r1))
checks.append(("root-of-roots", merkle.root_of_roots(["a"*64, "b"*64]) != merkle.root_of_roots(["b"*64, "a"*64])))
# 4) schema canonical hashing deterministic + taxonomy present
checks.append(("canon-deterministic", schema.sha({"b": 1, "a": 2}) == schema.sha({"a": 2, "b": 1})))
checks.append(("taxonomy-reddening", {"default_mismatch", "untriaged"} <= schema.CLASSES))
fails = [n for n, ok in checks if not ok]
print(f"PASS={len(checks)-len(fails)} FAIL={len(fails)}")
for n in fails: print("  FAIL", n)
PY
