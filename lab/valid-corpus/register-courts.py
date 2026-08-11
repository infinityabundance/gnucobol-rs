#!/usr/bin/env python3
"""Register the Phase-12 valid-corpus courts in the claim-ladder (reports/claim-ladder.json).
Run after `cargo run -p xtask -- receipt generate` (the receipts are the replayable evidence).
Every court carries positive claims, negative claims (not_proven), a damage-if-overclaimed
doctrine, a freshness gate (the sweep), and a replay command (the sweep script)."""
import json, os, sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
LADDER = os.path.join(ROOT, "reports/claim-ladder.json")

with open(LADDER) as f:
    ladder = json.load(f)
existing = {c["id"] for c in ladder["courts"]}

def add(court):
    if court["id"] in existing:
        print("already present:", court["id"])
        return
    ladder["courts"].append(court)
    print("added:", court["id"])

def mk(cid, name, proven, byte_domain, fixtures, not_proven, breaks_claim, damage):
    return {
        "id": cid,
        "name": name,
        "proven": proven,
        "byte_domain": byte_domain,
        "oracle": "GnuCOBOL 3.2.0 (admitted lab/oracle build) + the committed corpus evidence under reports/valid-corpus/",
        "fixtures": fixtures,
        "sealed_version": "0.8.56",
        "not_proven": not_proven,
        "breaks_claim": breaks_claim,
        "readiness": 1,
        "lie_prevented": "every number in the report is aggregated from the committed per-family evidence; this court re-verifies the evidence tree, it never re-measures or invents values",
        "damage_if_overclaimed": damage,
    }

add(mk(
    "GNURUST.CORPUS.CUSTODY.1",
    "valid-COBOL corpus custody — frozen pre-change state + complete evidence tree",
    "the pre-change repository state is frozen (preflight-repository-state.json + before-state.json + integration-design.md) and every family report directory exists under reports/valid-corpus/",
    "committed corpus evidence files (presence + freeze)",
    "bash lab/valid-corpus/corpus_court_sweep.sh custody",
    "no validity claim: custody proves the evidence tree exists and was frozen, not that any program is valid",
    "a missing family directory or a missing preflight/before-state freeze",
    "presenting custody as validity would certify programs this court never checked",
))
add(mk(
    "GNURUST.CORPUS.LICENCE.1",
    "valid-COBOL corpus licence decisions",
    "licences.json records a licence decision for every admitted family; unknown-licence X-COBOL repositories are quarantined (licence-quarantine.json, REFERENCE_ONLY) and never published",
    "licences.json + xcobol/licence-quarantine.json",
    "bash lab/valid-corpus/corpus_court_sweep.sh licence",
    "no redistribution claim for quarantined source; licence review is recorded, not legal advice",
    "a family without a recorded licence decision, or quarantined source marked redistributable",
    "claiming redistribution rights for quarantined source would misrepresent third-party licensing",
))
add(mk(
    "GNURUST.CORPUS.DEDUP.1",
    "valid-COBOL corpus deduplication",
    "deduplication.json records exact + near-duplicate evidence; grouping is repository-level so the development/validation/held-out partitions never split a repository",
    "deduplication.json + xcobol/dedup.json",
    "bash lab/valid-corpus/corpus_court_sweep.sh dedup",
    "no independent-program count that includes duplicates; near-duplicate thresholds are recorded, not universal",
    "a dedup report that is absent, or partition splits within a repository",
    "counting duplicate programs as independent evidence would inflate generalization claims",
))
add(mk(
    "GNURUST.VALID-PROGRAMS.GNUCOBOL-TESTSUITE.1",
    "valid-program corpus — GnuCOBOL Autotest suite step-level classification",
    "the GnuCOBOL Autotest suite is classified at AT_CHECK-step level (valid-programs.json, discovered-steps.json, invalid-programs.json, mixed-groups.json, dependency-graph.json, stable-current-drift.json, summary.md)",
    "reports/valid-corpus/gnucobol-testsuite/*",
    "bash lab/valid-corpus/corpus_court_sweep.sh valid-testsuite",
    "no real-world generalization claim from upstream tests alone; screen/curses steps are skipped under the no-terminal oracle profile",
    "a missing testsuite report, or a step left unclassified",
    "claiming the candidate generalizes because the upstream suite passes would overstate evidence from the candidate's own development source",
))
add(mk(
    "GNURUST.VALID-PROGRAMS.CCVS85.1",
    "valid-program corpus — CCVS85 classification + packages",
    "every CCVS85 unit is classified and the 512 units reconcile (programs.json); valid executable units have complete packages (source, COPY libraries, inputs, expected report output)",
    "reports/valid-corpus/ccvs85/programs.json (512 units) + the single GNURUST.CCVS85 evidence system",
    "bash lab/valid-corpus/corpus_court_sweep.sh valid-ccvs85",
    "no NIST certification; no COBOL-85 conformance claim; accuracy dimensions are recorded, not verdicts",
    "a unit count different from 512, or a missing classification",
    "presenting CCVS85 replay as certification would certify a suite designed for conformance testing, not certification",
))
add(mk(
    "GNURUST.VALID-PROGRAMS.MANUAL.1",
    "valid-program corpus — GnuCOBOL manual examples classification",
    "every manual code block is classified in both lanes (stable-3.2 + current examples.json + snippets.json); complete examples are replay-verified",
    "reports/valid-corpus/gnucobol-manual/{stable-3.2,current}/*",
    "bash lab/valid-corpus/corpus_court_sweep.sh valid-manual",
    "partial snippets, pseudocode and command examples are not executable programs; incomplete/obsolete commands are recorded, never silently repaired",
    "a missing manual lane report",
    "counting snippets as executable programs would fabricate runnable evidence",
))
add(mk(
    "GNURUST.VALID-PROGRAMS.EXTRAS.1",
    "valid-program corpus — GnuCOBOL-shipped programs + official contributions",
    "OpenCBS COBOL Defects Benchmark Suite and other shipped/contributed programs are inventoried with licence decisions (extras programs.json + custody.json + metrics.json)",
    "reports/valid-corpus/extras/*",
    "bash lab/valid-corpus/corpus_court_sweep.sh valid-extras",
    "no pristine-parity claim from adapted programs; adaptations are recorded with original/transformed hashes when any are applied",
    "a missing extras report",
    "claiming shipped programs as candidate evidence before licensing/dependency resolution would misrepresent custody",
))
add(mk(
    "GNURUST.VALID-PROGRAMS.OMP.1",
    "valid-program corpus — Open Mainframe Project course inventory",
    "the OMP COBOL Programming Course repository is fully inventoried (30 COBOL programs, 43 JCL, 226 images, 48 docs, 3 data, 5 support) with complete solutions separated from exercises and platform dependencies typed",
    "reports/valid-corpus/omp/programs.json + inventory.json",
    "bash lab/valid-corpus/corpus_court_sweep.sh valid-omp",
    "platform-service dependencies (z/OS datasets, JCL, DB2, CICS, VSAM, LE) are typed boundaries, never parser failures; starter exercises with intentionally missing code are not valid complete programs",
    "a missing inventory or program report",
    "describing platform-service failures as parser failures would misattribute the candidate's boundary",
))
add(mk(
    "GNURUST.VALID-PROGRAMS.XCOBOL.1",
    "valid-program corpus — X-COBOL immutable custody + classification + partitions",
    "the X-COBOL dataset (DOI 10.5281/zenodo.7968845) is under immutable custody with structural classification, repository-level licence quarantine, frozen development/validation/held-out partitions and large-scale robustness measurement",
    "reports/valid-corpus/xcobol/programs.json + partitions.json + robustness.json + licence-quarantine.json",
    "bash lab/valid-corpus/corpus_court_sweep.sh valid-xcobol",
    "unknown-licence source stays quarantined (REFERENCE_ONLY) and is not published; near-duplicate families are not independent evidence",
    "a missing xcobol report, or partitions not frozen",
    "claiming generalization from an unfrozen or contaminated held-out set would invalidate every downstream claim",
))
add(mk(
    "GNURUST.VALID-PROGRAMS.HELD-OUT.1",
    "valid-program corpus — held-out evaluation (pure measurement)",
    "the held-out evaluation (101 files) ran the candidate under a hard wall bound with 0 crashes and 0 timeouts, and the report states the held-out set was never used for implementation tuning",
    "reports/valid-corpus/held-out-results.json",
    "bash lab/valid-corpus/corpus_court_sweep.sh held-out",
    "no held-out claim after the set has been used for implementation tuning; parse/check/run success on held-out files is not language conformance",
    "a held-out report missing the never-tuned disclaimer, or absent",
    "claiming held-out generalization after tuning on the held-out set would be circular",
))
add(mk(
    "GNURUST.VALID-PROGRAMS.ACCURACY.1",
    "valid-program corpus — raw-byte accuracy dimensions",
    "accuracy.json records the raw-byte accuracy dimensions per family (compile status, execution status, report bytes sha256, raw stdout/stderr, generated files, return status)",
    "reports/valid-corpus/accuracy.json + per-family accuracy reports",
    "bash lab/valid-corpus/corpus_court_sweep.sh accuracy",
    "output normalization is never reported as raw-byte parity; warning-text parity is not semantic correctness",
    "a missing accuracy report",
    "reporting normalized output as byte parity would misstate the comparison",
))
add(mk(
    "GNURUST.PERFORMANCE.FRONTEND.1",
    "performance corpus — front-end-only measurement (View B)",
    "candidate per-phase timings (preprocess/lex/parse/resolution/layout/check/prepare) are measured separately from oracle compile (phase-metrics.json + views.json View B)",
    "reports/valid-corpus/performance/phase-metrics.json + views.json (View B)",
    "bash lab/valid-corpus/corpus_court_sweep.sh performance",
    "no native-code performance claim without a native candidate path; View A is labelled 'unlike workflows' (compiled vs interpreted) and is never described as equivalent runtime work",
    "a missing phase-metrics report, or a merged view",
    "conflating compile+run with parse/check+interpret would misrepresent what is being timed",
))
add(mk(
    "GNURUST.PERFORMANCE.PREPARED.1",
    "performance corpus — prepared-program execution (View C)",
    "already-compiled native binaries are compared with already-prepared programs run repeatedly WITHOUT reparsing (views.json View C + raw/view_c.json); prepared execution never touches the source again",
    "reports/valid-corpus/performance/views.json (View C) + raw samples",
    "bash lab/valid-corpus/corpus_court_sweep.sh performance",
    "no runtime-performance claim before correctness is established; no candidate lane is benchmarked before its output is byte-exact",
    "a missing View C report or raw samples",
    "reporting timing for a wrong-output lane would benchmark an incorrect program",
))
add(mk(
    "GNURUST.PERFORMANCE.BUSINESS.1",
    "performance corpus — scalable business workloads (Phase 8)",
    "ten purpose-built workload families (payroll, invoice, seqfile, relative, tables, strings, modules, float, report, mixed) x four deterministic scales are correctness-gated byte-exact against the host oracle before any timing (benchmarks.json)",
    "reports/valid-corpus/performance/benchmarks.json",
    "bash lab/valid-corpus/corpus_court_sweep.sh performance",
    "workloads are project-owned; inputs come from deterministic Rust generators and expected outputs are independently computed, never by the candidate",
    "a missing benchmarks report",
    "benchmarking a workload whose correctness was not established would invalidate every timing",
))
add(mk(
    "GNURUST.PERFORMANCE.CORPUS.1",
    "performance corpus — corpus throughput (View E)",
    "one full pass over 10 workloads x 4 scales per lane (oracle compile+run vs candidate prepare+run) with peak memory and raw samples retained; unfavorable results are never discarded",
    "reports/valid-corpus/performance/views.json (View E) + raw/view_e.json",
    "bash lab/valid-corpus/corpus_court_sweep.sh performance",
    "no equivalence between compiled-native and interpreted runtime work; no equivalence claim for the unlike View-A lanes",
    "a missing View E report or raw samples",
    "discarding slow candidate results would hide genuine performance characteristics",
))

with open(LADDER, "w") as f:
    json.dump(ladder, f, indent=1)
    f.write("\n")
print("claim-ladder courts now:", len(ladder["courts"]))
