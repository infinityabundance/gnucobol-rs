#!/usr/bin/env python3
"""kani+fuzz coverage receipts (GNURUST.VERIFY.KANI-FUZZ.1).

Every court with an executable byte implementation must carry BOTH a Kani proof of its sharp invariant and a
libfuzzer target (panic-freedom). Observed atlases and pure governance/view courts have no byte kernel -> N/A
with a declared reason. This scans the crate for COURT-tagged Kani proofs + fuzz targets, maps every claim-
ladder court to its {kani, fuzz} evidence, writes reports/kani-fuzz-coverage.json, and (check) FAILS if any
impl court lacks either.

Tagging convention (auditable, not inferred):
  - Kani proof:  a line `// KANIFOR: <COURT-ID>[, <COURT-ID>...]` immediately above `#[kani::proof]`.
  - Fuzz target: a line `//! FUZZFOR: <COURT-ID>[, <COURT-ID>...]` in the target file.
  run.py generate | check
"""
import json, os, re, sys, glob
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# Courts with NO byte kernel -> kani/fuzz N/A, with the reason. (Observed atlases + governance/view courts.)
NA_REASON = {
    "GNURUST.COVERAGE.1": "meta: the coverage map itself (a generated receipt, not a byte kernel)",
    "GNURUST.INTRINSIC.ATLAS.1": "observed atlas: records oracle behavior, no Rust implementation to prove/fuzz",
    "GNURUST.PROCEDURE.FLOW.ATLAS.1": "observed atlas: control-flow observation, no execution kernel",
    "SIZE.ERROR.ATLAS.1": "observed atlas: size-error observation, no Rust implementation",
    "GNURUST.FILE.STATUS.1": "observed atlas: file-status observation, no Rust implementation",
    "GNURUST.LINEAGE.CORPUS.20M.0": "meta: differential corpus engine (drives the real cobc oracle); no single Rust byte kernel -- its own seal-grade gate is replay+Merkle+isolation",
    "GNURUST.LINEAGE.CORPUS.20M.SMOKE": "meta: 200K real-cobc witness burn classified by the engine; no single Rust byte kernel",
}

def scan_tags(pattern_files, tag):
    """Return {court_id: [files]} from `// TAG: id, id` / `//! TAG: id` markers."""
    out = {}
    rx = re.compile(re.escape(tag) + r"\s*:?\s*(.+)")
    for f in pattern_files:
        for line in open(f):
            m = rx.search(line)
            if m:
                for cid in re.split(r"[,\s]+", m.group(1).strip()):
                    cid = cid.strip().rstrip(".")
                    if cid.startswith(("GNURUST.", "KOBOLD.", "SIZE.")):
                        out.setdefault(cid, []).append(os.path.basename(f))
    return out

def build():
    cl = json.load(open(os.path.join(ROOT, "reports/claim-ladder.json")))["courts"]
    src = glob.glob(os.path.join(ROOT, "crates/gnucobol-rs/src/*.rs"))
    fuzz = glob.glob(os.path.join(ROOT, "crates/gnucobol-rs/fuzz/fuzz_targets/*.rs"))
    kani = scan_tags(src, "KANIFOR")
    fz = scan_tags(fuzz, "FUZZFOR")
    rows = []
    for c in cl:
        cid = c["id"]
        is_atlas = ("ATLAS" in cid) or cid in ("GNURUST.COVERAGE.1", "GNURUST.FILE.STATUS.1", "GNURUST.PUBLIC.CORPUS.1", "GNURUST.BUILD.PROFILE.1", "GNURUST.PUBLIC.GAP.1", "GNURUST.LINEAGE.CORPUS.20M.0", "GNURUST.LINEAGE.CORPUS.20M.SMOKE")
        # Only GNURUST byte courts carry a gnucobol-rs kernel -> need kani + fuzz here. KOBOLD courts compose
        # the sealed courts (fuzzed in the shim crate); governance/view/atlas courts have no byte kernel.
        if not cid.startswith("GNURUST.") or is_atlas:
            reason = NA_REASON.get(cid) or (
                "observed atlas / meta: no Rust byte kernel" if is_atlas
                else "KOBOLD composition / governance / view court: no gnucobol-rs byte kernel (composes sealed courts)")
            rows.append({"court": cid, "kani": "n/a", "fuzz": "n/a", "reason": reason})
        else:
            rows.append({"court": cid, "kani": kani.get(cid, []), "fuzz": fz.get(cid, [])})
    impl = [r for r in rows if r["kani"] != "n/a"]
    return {
        "schema": "gnurust-kani-fuzz-coverage-v1", "court": "GNURUST.VERIFY.KANI-FUZZ.1",
        "total_courts": len(rows),
        "na_courts": len([r for r in rows if r["kani"] == "n/a"]),
        "impl_courts": len(impl),
        "kani_covered": len([r for r in impl if r["kani"]]),
        "fuzz_covered": len([r for r in impl if r["fuzz"]]),
        "rows": rows,
    }

def generate():
    b = build()
    json.dump(b, open(os.path.join(ROOT, "reports/kani-fuzz-coverage.json"), "w"), indent=2)
    print(f"kani-fuzz coverage: {b['kani_covered']}/{b['impl_courts']} kani, {b['fuzz_covered']}/{b['impl_courts']} fuzz ({b['na_courts']} n/a)")

def check():
    b = build()
    missing_k = [r["court"] for r in b["rows"] if r["kani"] != "n/a" and not r["kani"]]
    missing_f = [r["court"] for r in b["rows"] if r["kani"] != "n/a" and not r["fuzz"]]
    if missing_k:
        print(f"GATE: {len(missing_k)} impl courts with NO Kani proof:")
        for c in missing_k: print("   kani-missing:", c)
    if missing_f:
        print(f"GATE: {len(missing_f)} impl courts with NO fuzz target:")
        for c in missing_f: print("   fuzz-missing:", c)
    if missing_k or missing_f:
        print(f"!! kani-fuzz coverage incomplete: {len(missing_k)} kani + {len(missing_f)} fuzz missing")
        return 1
    print(f"kani-fuzz: all {b['impl_courts']} impl courts have BOTH a Kani proof and a fuzz target; {b['na_courts']} n/a (declared)")
    return 0

if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "check"
    {"generate": generate, "check": lambda: sys.exit(check())}.get(cmd, lambda: sys.exit("usage: generate|check"))()
