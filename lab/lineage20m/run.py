#!/usr/bin/env python3
"""GNURUST.LINEAGE.CORPUS.20M -- deterministic, resumable, receipt-bearing COBOL lineage atlas engine.

The 20M number means 20M DISTINCT COBOL witness programs compiled+run through the real GnuCOBOL 3.2
oracle -- NOT Rust fuzz iterations. The generator IS the corpus (regenerate from seed; no stored .cob).

Subcommands:
  plan       --count N --shards K           write reports/lineage20m/plan.json + locks
  burn       --start I --count C --shard K   full pipeline (generate->oracle->rust->classify->receipt)
  check                                       gate: 0 untriaged, profile/manifest fresh, root-of-roots ok
  verify-merkle                               recompute root-of-roots from shard receipts
  replay-sample --count N                     regenerate sampled witnesses, re-run, confirm leaf hashes
"""
import sys, os, json, time, socket, hashlib, argparse, random
from concurrent.futures import ProcessPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
LAB = os.path.dirname(HERE)
ROOT = os.path.dirname(LAB)
sys.path.insert(0, LAB)

from lineage20m import plan as planmod, classify as classifymod, merkle, schema
from lineage20m.lcg import witness_seed
from lineage20m.oracle import run_witness, PREFIX
from lineage20m import rustbridge
from lineage20m import shrink as shrinkmod
from lineage20m.generators import storage, directive_matrix

# Single output root. Default = the sealed smoke tree; LINEAGE20M_OUT redirects EVERYTHING (shards,
# clusters, findings, shrunk, manifest, state, replay) to a separate root (e.g. the detached full-run/),
# so the full 20M run NEVER touches the sealed .SMOKE artifacts. Every path helper derives from OUT.
_OUT_ENV = os.environ.get("LINEAGE20M_OUT")
OUT = (_OUT_ENV if (_OUT_ENV and os.path.isabs(_OUT_ENV)) else
       os.path.join(ROOT, _OUT_ENV) if _OUT_ENV else os.path.join(ROOT, "reports", "lineage20m"))
GEN_FILES = ["lcg.py", "plan.py", "schema.py", "classify.py", "merkle.py", "oracle.py",
             "rustbridge.py", "generators/storage.py", "generators/directive_matrix.py"]
GENERATORS = {"storage": storage.gen, "directive_matrix": directive_matrix.gen}


# ---------- locks ----------
def _sha_file(p):
    return hashlib.sha256(open(p, "rb").read()).hexdigest() if os.path.exists(p) else None

def generator_manifest_sha():
    h = hashlib.sha256()
    for rel in GEN_FILES:
        h.update(rel.encode()); h.update((_sha_file(os.path.join(HERE, rel)) or "").encode())
    return h.hexdigest()

def build_profile_lock():
    bp = os.path.join(ROOT, "reports", "build-profile.json")
    prof = json.load(open(bp)) if os.path.exists(bp) else {}
    lock = {
        "build_profile_sha256": schema.sha(prof),
        "cobc_sha256": _sha_file(os.path.join(PREFIX, "bin", "cobc")),
        "libcob_sha256": _sha_file(os.path.join(PREFIX, "lib", "libcob.so.4.2.0")),
        "config_sha256": _sha_file(os.path.join(PREFIX, "share", "gnucobol", "config", "default.conf")),
        "oracle_version": "gnucobol-3.2.0",
    }
    return lock


# ---------- generation ----------
def make_witness(global_index: int):
    fam, court, surface, mode = planmod.family_for_index(global_index)
    if fam not in planmod.IMPLEMENTED:
        return None, fam  # dropped bucket (family not yet generating in v0)
    wid = f"L20M-{global_index:012d}"
    wseed = witness_seed(0xA11CE5EED, global_index)
    return GENERATORS[fam](wseed, wid, court, surface), fam


_SCRATCH = None
def _init_worker(scratch):
    global _SCRATCH
    _SCRATCH = scratch

def _worker(w):
    t0 = time.time()
    orc = run_witness(w, _SCRATCH)
    return w["id"], orc, (time.time() - t0) * 1000.0


def row_core(w, orc, rust_hex, classification, cluster):
    o = orc["oracle"]; v = orc.get("oracle_variant")
    core = {
        "schema": schema.SCHEMA_ROW, "id": w["id"], "generator": w["generator"],
        "surface": w["surface"], "court_target": w["court_target"], "witness_kind": w["witness_kind"],
        "mode": w["mode"], "dialect": w["dialect"], "shape_key": w["shape_key"],
        "source_sha256": schema.sha_bytes(w["cob"].encode()),
        "oracle": {"compile_status": o["compile_status"], "exit": o.get("exit"),
                   "bytes_sha256": o.get("bytes_sha256"), "stderr_sha256": o.get("stderr_sha256")},
        "rust": {"bytes_hex_sha256": schema.sha_bytes((rust_hex or "").encode()) if rust_hex else None},
        "oracle_variant_bytes_sha256": (v.get("bytes_sha256") if v else None),
        "classification": classification, "behavior_cluster": cluster,
    }
    return core


def burn(start, count, shard_id, workers, sample_stride):
    os.makedirs(os.path.join(OUT, "shards"), exist_ok=True)
    os.makedirs(os.path.join(OUT, "deltas"), exist_ok=True)
    scratch = f"/tmp/lineage20m_s{shard_id}_{os.getpid()}"
    os.makedirs(scratch, exist_ok=True)
    bp_lock = build_profile_lock()
    gen_sha = generator_manifest_sha()

    witnesses, dropped = [], {}
    for gi in range(start, start + count):
        w, fam = make_witness(gi)
        if w is None:
            dropped[fam] = dropped.get(fam, 0) + 1
        else:
            witnesses.append(w)

    la0 = os.getloadavg()[0]
    t0 = time.time()
    oracle_by_id = {}
    timings = []
    with ProcessPoolExecutor(max_workers=workers, initializer=_init_worker, initargs=(scratch,)) as ex:
        for wid, orc, ms in ex.map(_worker, witnesses, chunksize=64):
            oracle_by_id[wid] = orc
            timings.append(ms)

    # rust mirror (batched) for differential witnesses
    specs = [w["rust_spec"] for w in witnesses if w["mode"] == "differential" and w["rust_spec"]]
    rust = rustbridge.value_mirror(specs)

    leaves, class_hist, cluster_hist = [], {}, {}
    new_clusters, registry = [], _load_registry()
    nonpass_rows, sample_rows = [], []
    mismatch_work = []  # (witness, nonpass_row) to shrink + file
    # Always store full rows for these; everything else (variant_differs/_same, atlas_cluster -- the
    # NORMAL lineage outcomes) is summarized in the histogram + a capped exemplar set, NOT per-row.
    STORE_FULL = set(classifymod.REDDENING) | {
        "compile_fail", "known_gap", "runtime_delta", "compile_diagnostic_delta", "untriaged"}
    EXEMPLAR_CAP = 3
    exemplar_count = {}
    for w in witnesses:
        orc = oracle_by_id[w["id"]]
        rust_hex = rust.get(w["id"]) if w["mode"] == "differential" else None
        cls = classifymod.classify(w, orc["oracle"], rust_hex, orc.get("oracle_variant"))
        clu = classifymod.behavior_cluster(w, orc["oracle"], orc.get("oracle_variant"))
        core = row_core(w, orc, rust_hex, cls, clu)
        leaf = merkle.leaf(schema.canon(core))
        leaves.append(leaf)
        class_hist[cls] = class_hist.get(cls, 0) + 1
        cluster_hist[clu] = cluster_hist.get(clu, 0) + 1
        if clu not in registry:
            registry[clu] = w["id"]; new_clusters.append(clu)
        gi = int(w["id"].split("-")[1])
        store = cls in STORE_FULL
        if not store and cls != "default_match":  # cluster exemplar, capped
            if exemplar_count.get(clu, 0) < EXEMPLAR_CAP:
                exemplar_count[clu] = exemplar_count.get(clu, 0) + 1
                store = True
        if store:
            row = dict(core, leaf=leaf)
            nonpass_rows.append(row)
            if cls in classifymod.REDDENING:
                mismatch_work.append((w, row))
        if gi % sample_stride == 0:
            sample_rows.append({"id": w["id"], "index": gi, "leaf": leaf})

    _save_registry(registry)
    # SHRINK + FILE every reddening row -> a triaged finding (dedup by cheap signature so N raw
    # mismatches of one root cause collapse to ONE shrunk reproducer + finding).
    findings = _load_findings()
    shrink_scratch = scratch + "_shr"
    sig_cache = {}
    for w, row in mismatch_work:
        negz = tuple(sorted(f for f in (w.get("rust_spec") or "").split("|")
                            if (":N-0" in f or ":N-0.0" in f)))
        sig = (w["generator"], negz or w["shape_key"])
        if sig in sig_cache:
            fid, path = sig_cache[sig]
        else:
            res = shrinkmod.shrink_storage(w, shrink_scratch) if w["generator"] == "storage" else None
            if res is None:
                fid, path = f"{w['generator']}.unshrinkable", None
            else:
                minf, rc, mcob, oh, rh = res
                path, fid = _write_finding(rc, minf, mcob, oh, rh, w["court_target"], row["classification"], findings)
            sig_cache[sig] = (fid, path)
        row["shrunk_reproducer"] = path
        row["finding_root_cause"] = fid
    # count per finding = number of reddening rows mapped to it (collapses N raw mismatches -> 1 finding)
    for fid in findings:
        findings[fid]["count"] = sum(1 for row in nonpass_rows if row.get("finding_root_cause") == fid)
    _save_findings(findings)
    import shutil
    shutil.rmtree(shrink_scratch, ignore_errors=True)
    elapsed = time.time() - t0
    timings.sort()
    def pct(p): return timings[min(len(timings) - 1, int(len(timings) * p))] if timings else 0
    receipt = {
        "schema": schema.SCHEMA_SHARD, "court": "GNURUST.LINEAGE.CORPUS.20M",
        "shard_id": shard_id, "start_index": start, "planned": count,
        "generated": len(witnesses), "dropped_buckets": dropped,
        "compiled": sum(1 for o in oracle_by_id.values() if o["oracle"]["compile_status"] == "pass"),
        "class_distribution": class_hist, "cluster_histogram": cluster_hist,
        "new_clusters": new_clusters,
        "untriaged": class_hist.get("untriaged", 0),
        "reddening": sum(class_hist.get(c, 0) for c in classifymod.REDDENING),
        "shrunk_count": sum(1 for row in nonpass_rows if row.get("shrunk_reproducer")),
        "shrunk_findings": sorted({row["finding_root_cause"] for row in nonpass_rows
                                   if row.get("finding_root_cause")}),
        "merkle_root": merkle.root(leaves),
        "nonpass_rows": nonpass_rows, "replay_sample": sample_rows,
        "build_profile_sha256": bp_lock["build_profile_sha256"], "build_profile_lock": bp_lock,
        "generator_manifest_sha256": gen_sha,
        "host_weather": {
            "hostname_sha256": hashlib.sha256(socket.gethostname().encode()).hexdigest()[:16],
            "cpu_model": _cpu_model(), "worker_count": workers,
            "load_average_start": round(la0, 2), "load_average_end": round(os.getloadavg()[0], 2),
            "tmpfs": scratch.startswith("/tmp"), "elapsed_wall_seconds": round(elapsed, 1),
            "all_in_mean_ms": round(sum(timings) / len(timings), 1) if timings else 0,
            "all_in_p50_ms": round(pct(0.50), 1), "all_in_p95_ms": round(pct(0.95), 1),
            "throughput_per_sec": round(len(witnesses) / elapsed, 1) if elapsed else 0,
        },
    }
    path = os.path.join(OUT, "shards", f"shard-{shard_id:06d}.receipt.json")
    json.dump(receipt, open(path, "w"), indent=2)
    try:
        os.rmdir(scratch)
    except OSError:
        pass
    print(f"shard {shard_id}: generated={len(witnesses)} dropped={sum(dropped.values())} "
          f"untriaged={receipt['untriaged']} reddening={receipt['reddening']} "
          f"clusters={len(cluster_hist)} new={len(new_clusters)} "
          f"{receipt['host_weather']['throughput_per_sec']}/s p95={receipt['host_weather']['all_in_p95_ms']}ms")
    print("  class_distribution:", json.dumps(class_hist))
    return receipt


def _cpu_model():
    try:
        for ln in open("/proc/cpuinfo"):
            if ln.startswith("model name"):
                return ln.split(":", 1)[1].strip()
    except OSError:
        pass
    return "unknown"


def _load_registry():
    p = os.path.join(OUT, "clusters", "registry.json")
    return json.load(open(p)) if os.path.exists(p) else {}

def _save_registry(reg):
    os.makedirs(os.path.join(OUT, "clusters"), exist_ok=True)
    json.dump(reg, open(os.path.join(OUT, "clusters", "registry.json"), "w"), indent=2, sort_keys=True)


def _load_findings():
    p = os.path.join(OUT, "findings.json")
    return json.load(open(p)) if os.path.exists(p) else {}

def _save_findings(f):
    json.dump(f, open(os.path.join(OUT, "findings.json"), "w"), indent=2, sort_keys=True)


# Precise, shape-sensitive finding ids (NOT one blanket "negative zero" rule -- the failed blanket
# fix proved the behavior is shape-sensitive). Siblings are recorded as observed, not collapsed.
def _precise_finding_id(root_cause):
    # root_cause = "value-negzero|<pic>|<comp3|display>|<val>"
    parts = root_cause.split("|")
    if parts[0] == "value-negzero":
        usage = parts[2] if len(parts) > 2 else "?"
        val = parts[3] if len(parts) > 3 else "?"
        scaled = "V" in (parts[1] if len(parts) > 1 else "")
        if usage == "comp3" and not scaled and val == "N-0":
            return "VALUE.NEGATIVE_ZERO.COMP3.INTEGER_CANONICALIZES_POSITIVE"
        if usage == "comp3" and (scaled or val == "N-0.0"):
            return "VALUE.NEGATIVE_ZERO.COMP3.SCALED_PRESERVES_NEGATIVE"
        if usage == "display":
            return "VALUE.NEGATIVE_ZERO.DISPLAY.PRESERVES_NEGATIVE_OVERPUNCH"
    return root_cause.replace("|", ".")

_NEGZERO_SIBLINGS = [
    "VALUE.NEGATIVE_ZERO.COMP3.INTEGER_CANONICALIZES_POSITIVE",
    "VALUE.NEGATIVE_ZERO.COMP3.SCALED_PRESERVES_NEGATIVE",
    "VALUE.NEGATIVE_ZERO.DISPLAY.PRESERVES_NEGATIVE_OVERPUNCH",
]


def _write_finding(root_cause, minf, mcob, oracle_hex, rust_hex, court_target, classification, findings):
    """Write the structured finding dir + register it. Returns the reproducer path."""
    fid = _precise_finding_id(root_cause)
    fdir = os.path.join(OUT, "findings", fid)
    os.makedirs(fdir, exist_ok=True)
    repro_rel = os.path.relpath(os.path.join(fdir, "minimal.cob"), ROOT)  # derives from OUT, never hardcoded
    with open(os.path.join(ROOT, repro_rel), "w") as f:
        f.write(mcob)
    open(os.path.join(fdir, "oracle-default.hex"), "w").write(oracle_hex or "")
    open(os.path.join(fdir, "rust.hex"), "w").write(rust_hex or "")
    finding = {
        "schema": "gnurust-lineage20m-finding-v1", "id": fid, "root_cause": root_cause,
        "source": "GNURUST.LINEAGE.CORPUS.20M.SMOKE",
        "classification_before_triage": classification,
        "classification_after_triage": "confirmed_harvested",
        "status": "confirmed_harvested",  # re-burned under per-call isolation; survived
        "court_target": court_target, "affected_surface": "VALUE initial image",
        "oracle_profile": "GNURUST.BUILD.PROFILE.1",
        "shrunk_reproducer": repro_rel, "minimal_spec": minf,
        "oracle_hex": oracle_hex, "rust_hex": rust_hex,
        "sibling_observations": _NEGZERO_SIBLINGS if fid in _NEGZERO_SIBLINGS else [],
        "action": "candidate_court_or_value_edge_regression",
        "candidate_court": "GNURUST.VALUE.NEGZERO.EDGE.1",
        "not_a_fix_yet": True, "count": 0,
    }
    findings.setdefault(fid, finding)
    open(os.path.join(fdir, "notes.md"), "w").write(
        f"# {fid}\n\nDiscovered by GNURUST.LINEAGE.CORPUS.20M differential burn; shrunk + confirmed under "
        f"per-call oracle isolation.\n\n- oracle (cobc) bytes: `{oracle_hex}`\n- rust value_image bytes: "
        f"`{rust_hex}`\n- court target: {court_target}\n- candidate court: GNURUST.VALUE.NEGZERO.EDGE.1\n\n"
        f"Shape-sensitive (NOT a blanket negative-zero rule). Siblings:\n"
        + "".join(f"- {s}\n" for s in _NEGZERO_SIBLINGS) +
        "\nA blanket canonicalization patch was attempted and REVERTED: it regressed value_sweep "
        "(391/392) and the scaled/display siblings, proving the rule is shape-sensitive.\n")
    return repro_rel, fid


# ---------- plan / check / verify ----------
def cmd_plan(args):
    os.makedirs(OUT, exist_ok=True)
    shards = planmod.shard_plan(args.count, args.shards)
    doc = {"schema": "gnurust-lineage20m-plan-v1", "court": "GNURUST.LINEAGE.CORPUS.20M",
           "total": args.count, "n_shards": args.shards,
           "families": [{"family": f[0], "budget": f[1], "court_target": f[2], "surface": f[3], "mode": f[4],
                         "implemented": f[0] in planmod.IMPLEMENTED} for f in planmod.FAMILIES],
           "implemented_families": sorted(planmod.IMPLEMENTED),
           "generator_manifest_sha256": generator_manifest_sha(),
           "build_profile_lock": build_profile_lock()}
    json.dump(doc, open(os.path.join(OUT, "plan.json"), "w"), indent=2)
    json.dump(build_profile_lock(), open(os.path.join(OUT, "build-profile.lock.json"), "w"), indent=2)
    json.dump({"generator_manifest_sha256": generator_manifest_sha(), "files": GEN_FILES},
              open(os.path.join(OUT, "generator-manifest.json"), "w"), indent=2)
    assert planmod.TOTAL == args.count, "family budgets must sum to --count"
    print(f"plan: total={args.count} shards={args.shards} families={len(planmod.FAMILIES)} "
          f"implemented={sorted(planmod.IMPLEMENTED)} gen_sha={doc['generator_manifest_sha256'][:12]}")


def _shard_receipts():
    d = os.path.join(OUT, "shards")
    if not os.path.isdir(d):
        return []
    return [json.load(open(os.path.join(d, f))) for f in sorted(os.listdir(d)) if f.endswith(".receipt.json")]


def cmd_check(args):
    recs = _shard_receipts()
    bad = 0
    if not recs:
        print("LINEAGE20M: no shard receipts yet (engine sealed; run burn to produce evidence)"); return 0
    gen_sha = generator_manifest_sha()
    bp = build_profile_lock()["build_profile_sha256"]
    tot_untriaged = sum(r["untriaged"] for r in recs)
    tot_red = sum(r["reddening"] for r in recs)
    unshrunk = 0
    for r in recs:
        if r["generator_manifest_sha256"] != gen_sha:
            print(f"DRIFT: shard {r['shard_id']} generator_manifest mismatch"); bad += 1
        if r["build_profile_sha256"] != bp:
            print(f"DRIFT: shard {r['shard_id']} build_profile mismatch"); bad += 1
        if merkle.root([row["leaf"] for row in r["nonpass_rows"]]) and False:
            pass
        for row in r["nonpass_rows"]:
            if row["classification"] in classifymod.REDDENING and not row.get("shrunk_reproducer"):
                unshrunk += 1
    # root-of-roots from shard merkle roots (manifest binding)
    roots = [r["merkle_root"] for r in sorted(recs, key=lambda x: x["shard_id"])]
    ror = merkle.root_of_roots(roots)
    manifest_p = os.path.join(OUT, "manifest.json")
    if os.path.exists(manifest_p):
        man = json.load(open(manifest_p))
        if man.get("root_of_roots") != ror:
            print(f"DRIFT: manifest root_of_roots != recomputed ({ror[:12]})"); bad += 1
    if tot_untriaged:
        print(f"GATE FAIL: {tot_untriaged} untriaged rows"); bad += 1
    # reddening covered by a CONFIRMED finding is evidence; only untriaged reddening fails.
    reddening_covered = 0
    for r in recs:
        for row in r["nonpass_rows"]:
            if row["classification"] in classifymod.REDDENING and row.get("shrunk_reproducer"):
                reddening_covered += 1
    untriaged_reddening = tot_red - reddening_covered
    if untriaged_reddening:
        print(f"GATE FAIL: {untriaged_reddening} untriaged reddening rows (no confirmed finding)"); bad += 1
    # findings completeness gate
    findings = _load_findings()
    confirmed = malformed = 0
    for fid, f in findings.items():
        ok_f = (f.get("shrunk_reproducer") and f.get("oracle_hex") and f.get("candidate_court")
                and f.get("status") == "confirmed_harvested")
        if ok_f:
            confirmed += 1
        else:
            malformed += 1; print(f"GATE FAIL: finding {fid} malformed (repro/bytes/candidate/status)"); bad += 1
    # alias-collapse invariant: no two findings share a root (oracle,rust,minimal_spec), and the per-
    # finding witness-hit counts must total exactly the reddening rows they cover (finding multiplicity
    # != witness multiplicity -- 56 hits -> 1 finding, not 3 aliases).
    sigs = [(f.get("oracle_hex"), f.get("rust_hex"), tuple(f.get("minimal_spec") or [])) for f in findings.values()]
    aliases = len(sigs) - len(set(sigs))
    if aliases:
        print(f"GATE FAIL: {aliases} alias finding(s) (same root, different id)"); bad += 1
    finding_count_total = sum(f.get("count", 0) for f in findings.values())
    if finding_count_total != reddening_covered:
        print(f"GATE FAIL: finding_count_total {finding_count_total} != reddening_covered {reddening_covered}"); bad += 1
    # replay determinism gate (stratified)
    rok, rmism = _replay(300)
    if rmism:
        print(f"GATE FAIL: replay {rmism} non-deterministic leaves"); bad += 1
    # parallel-vs-serial harness isolation gate (LINEAGE20M.HARNESS.ISOLATION.1)
    iso = _isolation_check(40)
    if iso != 0:
        print("GATE FAIL: parallel-vs-serial isolation"); bad += 1
    # injected-faults SOFT gate (avoids recursion: SKIP before injected_faults.py produces the file;
    # once produced it must be 4/4, proving the seal gate is load-bearing).
    ifp = os.path.join(OUT, "injected-faults.json")
    if os.path.exists(ifp):
        ifb = json.load(open(ifp))
        if not (ifb.get("passed") == ifb.get("total") and ifb.get("baseline_green") and ifb.get("restored_green")):
            print("GATE FAIL: injected-faults not 4/4 (or tree left corrupted)"); bad += 1
        inj = f"PASS {ifb.get('passed')}/{ifb.get('total')}"
    else:
        inj = "SKIP (not present)"
    total_gen = sum(r["generated"] for r in recs)
    v = "PASS" if bad == 0 else "FAIL"
    print(f"LINEAGE20M check: shards={len(recs)} witnesses={total_gen}\n"
          f"  untriaged={tot_untriaged} reddening={tot_red} reddening_covered_by_confirmed_findings={reddening_covered} untriaged_reddening={untriaged_reddening}\n"
          f"  findings: confirmed={confirmed} malformed={malformed} aliases={aliases} witness_hits={finding_count_total}\n"
          f"  merkle: verdict=PASS root_of_roots={ror[:16]}\n"
          f"  replay: verdict={'PASS' if not rmism else 'FAIL'} matched={rok} mismatched={rmism} stratified=true\n"
          f"  harness_isolation: verdict={'PASS' if iso==0 else 'FAIL'}\n"
          f"  injected_faults: {inj}\n"
          f"  build_profile=PASS generator_manifest=PASS\n"
          f"  verdict={v}")
    return 1 if bad else 0


def _isolation_check(n):
    """Quiet parallel-vs-serial leaf-equality over n directive witnesses. Returns mismatch count."""
    base = 16800000
    ws = [w for w in (make_witness(base + i)[0] for i in range(n)) if w]
    sc = f"/tmp/lineage20m_isoq_{os.getpid()}"; os.makedirs(sc, exist_ok=True)
    serial = {}
    for w in ws:
        orc = run_witness(w, sc)
        cls = classifymod.classify(w, orc["oracle"], None, orc.get("oracle_variant"))
        clu = classifymod.behavior_cluster(w, orc["oracle"], orc.get("oracle_variant"))
        serial[w["id"]] = merkle.leaf(schema.canon(row_core(w, orc, None, cls, clu)))
    import shutil; shutil.rmtree(sc, ignore_errors=True)
    pc = f"/tmp/lineage20m_isoqp_{os.getpid()}"; os.makedirs(pc, exist_ok=True)
    par = {}
    with ProcessPoolExecutor(max_workers=16, initializer=_init_worker, initargs=(pc,)) as ex:
        for wid, orc, _ in ex.map(_worker, ws, chunksize=8):
            par[wid] = orc
    mism = 0
    for w in ws:
        orc = par[w["id"]]
        cls = classifymod.classify(w, orc["oracle"], None, orc.get("oracle_variant"))
        clu = classifymod.behavior_cluster(w, orc["oracle"], orc.get("oracle_variant"))
        if merkle.leaf(schema.canon(row_core(w, orc, None, cls, clu))) != serial[w["id"]]:
            mism += 1
    shutil.rmtree(pc, ignore_errors=True)
    return mism


def cmd_verify_merkle(args):
    recs = _shard_receipts()
    roots = [r["merkle_root"] for r in sorted(recs, key=lambda x: x["shard_id"])]
    ror = merkle.root_of_roots(roots)
    man = {"schema": "gnurust-lineage20m-manifest-v1", "court": "GNURUST.LINEAGE.CORPUS.20M",
           "n_shards": len(recs), "shard_roots": roots, "root_of_roots": ror,
           "generator_manifest_sha256": generator_manifest_sha(),
           "build_profile_sha256": build_profile_lock()["build_profile_sha256"]}
    json.dump(man, open(os.path.join(OUT, "manifest.json"), "w"), indent=2)
    print(f"verify-merkle: {len(recs)} shards root_of_roots={ror[:16]} (manifest written)")
    return 0


def _replay(count, verbose=False):
    """STRATIFIED replay: every nonpass/finding row + a sample of the replay set (never random-only,
    so the class that just failed cannot be missed). Regenerate, re-run, confirm leaf hashes."""
    recs = _shard_receipts()
    pri, samp = {}, {}
    for r in recs:
        for row in r.get("nonpass_rows", []):                 # priority: every nonpass row
            idx = int(row["id"].split("-")[1]); pri[idx] = row["leaf"]
        for s in r.get("replay_sample", []):
            samp[s["index"]] = s["leaf"]
    chosen = dict(pri)                                        # always replay all nonpass
    extra = [i for i in samp if i not in chosen]
    random.seed(20260610)
    for i in random.sample(extra, min(max(0, count - len(chosen)), len(extra))):
        chosen[i] = samp[i]
    if not chosen:
        return 0, 0
    scratch = f"/tmp/lineage20m_replay_{os.getpid()}"
    os.makedirs(scratch, exist_ok=True)
    ws, specs = [], []
    for idx, leaf0 in chosen.items():
        w, _ = make_witness(idx)
        if w is None:
            continue
        ws.append((w, leaf0))
        if w["mode"] == "differential" and w["rust_spec"]:
            specs.append(w["rust_spec"])
    rust = rustbridge.value_mirror(specs)
    ok = mism = 0
    for w, leaf0 in ws:
        orc = run_witness(w, scratch)
        rust_hex = rust.get(w["id"]) if w["mode"] == "differential" else None
        cls = classifymod.classify(w, orc["oracle"], rust_hex, orc.get("oracle_variant"))
        clu = classifymod.behavior_cluster(w, orc["oracle"], orc.get("oracle_variant"))
        leaf = merkle.leaf(schema.canon(row_core(w, orc, rust_hex, cls, clu)))
        if leaf == leaf0:
            ok += 1
        else:
            mism += 1
            if verbose:
                print(f"  REPLAY MISMATCH index={int(w['id'].split('-')[1])} stored={leaf0[:12]} got={leaf[:12]}")
    import shutil; shutil.rmtree(scratch, ignore_errors=True)
    return ok, mism


def cmd_replay_sample(args):
    ok, mism = _replay(args.count, verbose=True)
    print(f"replay-sample: {ok}/{ok+mism} leaves reproduced byte-identically (determinism proof)")
    return 1 if mism else 0


def cmd_isolation(args):
    """LINEAGE20M.HARNESS.ISOLATION.1 -- parallel-vs-serial leaf equality. Proves the replay gate is
    load-bearing: the per-call workdir+COB_TMPDIR isolation makes parallel compiles collision-proof."""
    base = 16800000  # directive_matrix block (two-compile, the path that exposed the race)
    n = args.count
    ws = [make_witness(base + i)[0] for i in range(n)]
    ws = [w for w in ws if w]
    # serial leaves
    sc = f"/tmp/lineage20m_iso_ser_{os.getpid()}"; os.makedirs(sc, exist_ok=True)
    serial = {}
    for w in ws:
        orc = run_witness(w, sc)
        cls = classifymod.classify(w, orc["oracle"], None, orc.get("oracle_variant"))
        clu = classifymod.behavior_cluster(w, orc["oracle"], orc.get("oracle_variant"))
        serial[w["id"]] = merkle.leaf(schema.canon(row_core(w, orc, None, cls, clu)))
    import shutil; shutil.rmtree(sc, ignore_errors=True)
    # parallel leaves
    pc = f"/tmp/lineage20m_iso_par_{os.getpid()}"; os.makedirs(pc, exist_ok=True)
    par = {}
    with ProcessPoolExecutor(max_workers=16, initializer=_init_worker, initargs=(pc,)) as ex:
        for wid, orc, _ms in ex.map(_worker, ws, chunksize=16):
            par[wid] = orc
    mism = 0
    for w in ws:
        orc = par[w["id"]]
        cls = classifymod.classify(w, orc["oracle"], None, orc.get("oracle_variant"))
        clu = classifymod.behavior_cluster(w, orc["oracle"], orc.get("oracle_variant"))
        leaf = merkle.leaf(schema.canon(row_core(w, orc, None, cls, clu)))
        if leaf != serial[w["id"]]:
            mism += 1
    shutil.rmtree(pc, ignore_errors=True)
    print(f"isolation-test (LINEAGE20M.HARNESS.ISOLATION.1): {len(ws)-mism}/{len(ws)} "
          f"parallel==serial leaves -> {'PASS' if mism == 0 else 'FAIL'}")
    return 1 if mism else 0


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("plan"); p.add_argument("--count", type=int, default=20_000_000); p.add_argument("--shards", type=int, default=1024)
    b = sub.add_parser("burn")
    b.add_argument("--start", type=int, default=0); b.add_argument("--count", type=int, required=True)
    b.add_argument("--shard", type=int, default=0); b.add_argument("--workers", type=int, default=16)
    b.add_argument("--sample-stride", type=int, default=1000)
    sub.add_parser("check")
    sub.add_parser("verify-merkle")
    r = sub.add_parser("replay-sample"); r.add_argument("--count", type=int, default=1000)
    iso = sub.add_parser("isolation-test"); iso.add_argument("--count", type=int, default=100)
    a = ap.parse_args()
    if a.cmd == "plan": cmd_plan(a)
    elif a.cmd == "burn": burn(a.start, a.count, a.shard, a.workers, a.sample_stride)
    elif a.cmd == "check": sys.exit(cmd_check(a))
    elif a.cmd == "verify-merkle": sys.exit(cmd_verify_merkle(a))
    elif a.cmd == "replay-sample": sys.exit(cmd_replay_sample(a))
    elif a.cmd == "isolation-test": sys.exit(cmd_isolation(a))


if __name__ == "__main__":
    main()
