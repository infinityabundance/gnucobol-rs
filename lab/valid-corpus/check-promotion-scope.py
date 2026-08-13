#!/usr/bin/env python3
"""GNURUST.VALID-PROGRAMS promotion-scope check (atomic-promotion hardening).

Usage: check-promotion-scope.py <staged-dir> <committed-dir>

The valid-corpus docker lane runs each pass against a PRIVATE staged copy of the
repository and promotes the staged evidence into `reports/` only after every gate
passes. This script enforces the mechanical boundary of that promotion: the staged
tree may differ from the committed one ONLY in this lane's regeneration set.

Regeneration set:
  * the six extractor family directories (gnucobol-testsuite, ccvs85,
    gnucobol-manual, extras, omp, xcobol);
  * the unify outputs at the top level (summary.json, licences.json,
    dependencies.json, deduplication.json, dialect-matrix.json,
    first-failure-buckets.json, accuracy.json, performance.json, determinism.json,
    no-delegation.json, programs.csv, summary.md);
  * the sweep file (corpus-court-sweep.txt).

Everything else (raw/, performance/, held-out evidence, preflight/before-state,
baseline, generalization, any unknown file) is owned by another lane or by the
admission phase and must be byte-identical: a change, addition or deletion there
blocks promotion, so the containers can never clobber other lanes' evidence.

Exit: 0 = every changed path is within the regeneration set; 1 = blocked (the
blocked paths are printed).
"""
import os
import sys


ALLOWED_DIRS = {"gnucobol-testsuite", "ccvs85", "gnucobol-manual", "extras", "omp", "xcobol"}
ALLOWED_FILES = {
    "summary.json", "licences.json", "dependencies.json", "deduplication.json",
    "dialect-matrix.json", "first-failure-buckets.json", "accuracy.json",
    "performance.json", "determinism.json", "no-delegation.json",
    "programs.csv", "summary.md", "corpus-court-sweep.txt",
}


def rels(root):
    out = {}
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames.sort()
        for fn in sorted(filenames):
            p = os.path.relpath(os.path.join(dirpath, fn), root)
            out[p] = os.path.join(dirpath, fn)
    return out


def same(rel, a, b):
    if rel not in a or rel not in b:
        return False  # added or deleted
    with open(a[rel], "rb") as fa, open(b[rel], "rb") as fb:
        return fa.read() == fb.read()


def main(argv):
    if len(argv) != 2:
        print(__doc__)
        return 2
    staged, committed = argv
    if not os.path.isdir(staged) or not os.path.isdir(committed):
        print(f"check-promotion-scope: staged={staged!r} committed={committed!r} (need two dirs)")
        return 2
    a, b = rels(staged), rels(committed)
    diff_files = [r for r in sorted(set(a) | set(b)) if not same(r, a, b)]
    protected = [
        r for r in diff_files
        if r not in ALLOWED_FILES and r.split(os.sep, 1)[0] not in ALLOWED_DIRS
    ]
    if protected:
        print("PROMOTION BLOCKED: staged tree modified protected evidence:")
        for p in protected:
            print("  ", p)
        return 1
    print(f"promotion scope: {len(diff_files)} changed file(s), all within the lane's regeneration set")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
