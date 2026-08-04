#!/usr/bin/env python3
"""Register the GnuCOBOL-testsuite + cobc-rs + methodology + runtime-math courts in the
claim-ladder and the receipt manifest. The testsuite courts' numbers are injected from the
committed summary; run AFTER the docker court completes."""
import json, sys, os

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
LADDER = os.path.join(ROOT, "reports/claim-ladder.json")
MANIFEST = os.path.join(ROOT, "lab/receipt/manifest.json")
SUMMARY = os.path.join(ROOT, "reports/gnucobol-testsuite/summary.json")
MATH = os.path.join(ROOT, "reports/gnucobol-runtime-tests/math-correctness.json")

def load(p):
    with open(p) as f:
        return json.load(f)

def save(p, d):
    with open(p, "w") as f:
        json.dump(d, f, indent=1)
        f.write("\n")

ladder = load(LADDER)
existing = {c["id"] for c in ladder["courts"]}

summ = load(SUMMARY)["summary"] if os.path.exists(SUMMARY) else {}
mathd = load(MATH) if os.path.exists(MATH) else {}
m = summ.get("comparison", {})
oracle = summ.get("oracle", {})
cand = summ.get("candidate", {})
first = summ.get("first_failure", {})

def add(court):
    if court["id"] in existing:
        print("already present:", court["id"])
        return
    ladder["courts"].append(court)
    print("added:", court["id"])

add({
    "id": "GNURUST.GNUCOBOL-TESTSUITE.1",
    "name": "GnuCOBOL 3.2 native Autotest suite custody + real-compiler baseline + invocation census",
    "proven": "the admitted gnucobol-3.2 source (sha256 8ecc77d0...) is built fresh in-tree per pass with a stock configuration (no -fpermissive, no compat -Wno-*), and the generated Autotest suite is run with the REAL admitted cobc via `make check TESTSUITEFLAGS=--jobs=12`, producing an oracle baseline of %(pass)d pass / %(skip)d skip / %(xfail)d xfail / %(fail)d fail in this environment, with a full invocation census (argv boundaries preserved: %(census)d cobc/cobcrun invocations) and the raw testsuite.log + per-group logs preserved" % {
        "pass": oracle.get("pass", 0), "skip": oracle.get("skip", 0),
        "xfail": oracle.get("xfail", 0), "fail": oracle.get("fail", 0),
        "census": 0,
    },
    "byte_domain": "admitted source identity + fresh in-tree build + the generated testsuite.log + per-group logs + the invocation census (argv preserved)",
    "oracle": "the ADMITTED GnuCOBOL 3.2 in-tree build (never a distribution package), configured identically in every tree",
    "fixtures": "gnucobol-3.2.tar.lz (admitted, sha256-pinned) + lab/gnucobol-testsuite/run-docker.sh (one-command replay) + lab/docker/gnucobol-testsuite",
    "sealed_version": "0.8.54",
    "not_proven": "no claim about gnucobol-rs (that is TESTSUITE.2/.3); the baseline measures THIS build and environment, not upstream; oracle-side skips/xfails are the suite's own declared conditions",
    "breaks_claim": "source hash drift; a different GnuCOBOL build; non-deterministic build/run environment; a missing raw log",
    "readiness": 1,
    "lie_prevented": "an oracle-side pass/fail is evidence about THIS admitted build, never a claim about upstream quality",
    "damage_if_overclaimed": "presenting the baseline as a certification would certify a compiler build the suite was not designed to certify"
})

add({
    "id": "GNURUST.GNUCOBOL-TESTSUITE.2",
    "name": "GnuCOBOL-testsuite candidate execution (COBC=cobc-rs through the native harness, no delegation)",
    "proven": "the SAME generated suite runs with COBC=cobc-rs (the option-policy-driven compatibility driver) and COBCRUN=cobc-rs through the suite's own `make localcheck` + atlocal bootstrap, producing truthful launcher+manifest artifacts (never native COBOL executables), with the mechanical no-delegation proof (candidate PATH stripped of the oracle, oracle prefix absent in the container, cobrun/cobc-rs link no libcob -- ldd+readelf, plus an execve trace of candidate artifacts), the candidate invocation ledger, and every test accounted for; NO suite-pass claim",
    "byte_domain": "per-test candidate outcomes (parse/check/prepare/run), raw stdout+stderr, candidate census, execve trace, launcher manifests",
    "oracle": "none used during the candidate phase (isolation is mechanically enforced); the baseline is TESTSUITE.1",
    "fixtures": "crates/cobc-rs (the driver) + crates/gnucobol-rs-testsuite (the harness) + lab/gnucobol-testsuite",
    "sealed_version": "0.8.54",
    "not_proven": "no parity claim (that is TESTSUITE.3); no claim that the launcher is a native COBOL executable; no claim that rejected options preserve semantics",
    "breaks_claim": "any candidate execution that invokes cobc/cobcrun or links libcob; a launcher misrepresented as native codegen; an unaccounted test",
    "readiness": 1,
    "lie_prevented": "'cobc-rs is a drop-in cobc' is the lie this prevents -- the artifacts are interpreter launch manifests and the boundaries are explicit",
    "damage_if_overclaimed": "treating the launcher as a native executable would misrepresent an interpreter as codegen"
})

add({
    "id": "GNURUST.GNUCOBOL-TESTSUITE.4",
    "name": "GnuCOBOL-testsuite rerun after the three-boundary reduction (module lifecycle + parser/check waves + wrapper policy) -- re-measured ledger",
    "proven": "the FULL suite reruns with the new cobc-rs/cobcrun-rs after each major boundary reduction; all %(total)d test groups still reconcile exactly; the before/after ledger (boundary-reduction) attributes every v0.8.54 classification to its measured after-state; no-delegation remains mechanically green; the math subset is regenerated from the same ledger" % {"total": summ.get("total_tests", 0)},
    "byte_domain": "the re-measured ledgers + raw rerun evidence + the boundary-reduction transitions",
    "oracle": "the TESTSUITE.1 baseline (re-run, unchanged expectations)",
    "fixtures": "reports/gnucobol-testsuite/* + lab/gnucobol-testsuite/run-docker.sh",
    "sealed_version": "0.8.54",
    "not_proven": "no claim that a boundary reduction equals a pass; no full suite-parity claim; no conformance certification",
    "breaks_claim": "a rerun that reconciles fewer than all test groups; a transition reported without raw evidence",
    "readiness": 1,
    "lie_prevented": "'the three boundaries were reduced' is the claim this court measures -- without the rerun it is projection",
    "damage_if_overclaimed": "reporting projected unlocks as measured results"
})

add({
    "id": "GNURUST.GNUCOBOL-TESTSUITE.3",
    "name": "GnuCOBOL-testsuite differential classification (baseline vs candidate, every test accounted)",
    "proven": "every generated test group receives exactly one final classification (reconciled: total == sum of all classes), with the oracle x candidate outcome pairs, first-failure attribution, raw evidence preserved, deterministic two-pass comparison, and honest totals in this environment: %(match)d OBSERVABLE_MATCH, %(check)d candidate check/parse rejects, %(module)d module-model unsupported, %(wrap)d wrapper-option unsupported, %(mal)d wrapper-malformed, %(unsup)d candidate unsupported, %(runfail)d runtime fails, %(timeout)d timeouts, %(notreach)d not-reached; the runtime/math subset is reported separately (GNURUST.GNUCOBOL-RUNTIME-MATH.1)" % {
        "match": m.get("observable_match", 0), "check": cand.get("check_reject", 0),
        "module": cand.get("module_model_unsupported", 0), "wrap": first.get("WRAPPER_OPTION_UNSUPPORTED", 0),
        "mal": first.get("WRAPPER_INVOCATION_MALFORMED", 0), "unsup": cand.get("unsupported", 0),
        "runfail": cand.get("runtime_fail", 0), "timeout": cand.get("timeout", 0),
        "notreach": cand.get("not_reached", 0),
    },
    "byte_domain": "per-test classification + reason codes + the raw baseline/candidate outputs they rest on",
    "oracle": "the TESTSUITE.1 baseline (real admitted cobc)",
    "fixtures": "reports/gnucobol-testsuite/* (ledgers, summaries, raw logs, no-delegation, determinism) + reports/receipts/GNURUST.GNUCOBOL-TESTSUITE.{1,2,3}/",
    "sealed_version": "0.8.54",
    "not_proven": "OBSERVABLE_MATCH is the test's own AT_CHECK assertion outcome in this environment, not equivalence outside it; no GnuCOBOL test-suite parity claim; no COBOL conformance certification; no performance claim (see the runtime-math performance view)",
    "breaks_claim": "a test with no final classification; a classification that hides an unsupported feature as a generic failure; a nondeterministic classification",
    "readiness": 1,
    "lie_prevented": "'the GnuCOBOL test-suite passes with gnucobol-rs' is the lie this prevents -- the honest surface is a classification, not a pass count",
    "damage_if_overclaimed": "presenting the classification as full suite parity would certify coverage the candidate does not have"
})

add({
    "id": "GNURUST.GNUCOBOL-RUNTIME-MATH.1",
    "name": "GnuCOBOL runtime/mathematics correctness classification (math subset of the suite)",
    "proven": "the math/runtime subset (data_binary, data_display, data_packed, data_pointer, run_fundamental, run_functions, syn_multiply, syn_value, syn_literals) is classified from the SAME differential results as the whole suite (no favorable selection): %(n)d math tests with per-test oracle/candidate outcome pairs and first-failure attribution; performance is reported SEPARATELY and only for tests passing on both sides" % {"n": mathd.get("math_tests_total", 0)},
    "byte_domain": "per-math-test classification + the underlying raw outputs",
    "oracle": "the TESTSUITE.1 baseline",
    "fixtures": "reports/gnucobol-runtime-tests/math-correctness.{json,md}",
    "sealed_version": "0.8.54",
    "not_proven": "no performance claim here; correctness is the suite's AT_CHECK outcome in this environment",
    "breaks_claim": "a math test classified without a preserved raw output; a favorable-selection report",
    "readiness": 1,
    "lie_prevented": "'the math tests pass' is the lie this prevents -- the classification shows exactly which math tests match, which fail closed, and which are module-model blocked",
    "damage_if_overclaimed": "claiming math parity from a favorable subset would be unscientific"
})

# ---------------------------------------------------------------------------------------------
# Phase-2/3/4 boundary-reduction courts (module lifecycle, parser/check waves, wrapper options)
# ---------------------------------------------------------------------------------------------

add({
    "id": "GNURUST.MODULE.REGISTRY.1",
    "name": "interpreted module lifecycle -- cobcrun-rs runner + build-local module search (-M/cwd/COB_LIBRARY_PATH) + truthful -m artifacts",
    "proven": "cobc-rs -m writes a silent launcher+manifest+expanded-source artifact (never a native .so); cobcrun-rs resolves modules through the -M directory (trailing slash appended, GnuCOBOL semantics), the working directory and COB_LIBRARY_PATH, passes program arguments to ACCEPT FROM COMMAND-LINE, and emits cobcrun-shaped diagnostics (missing PROGRAM name / invalid module argument / cannot find module); exercised end-to-end by crates/cobc-rs/tests/module_courts.rs (GNURUST.MODULE.* courts) and by the suite rerun",
    "byte_domain": "module artifacts (launcher, manifest, expanded source), cobcrun stdout/stderr/exit, module search order",
    "oracle": "GnuCOBOL 3.2 cobcrun semantics as observed in the admitted suite's own module tests",
    "fixtures": "crates/cobc-rs/tests/module_courts.rs + reports/gnucobol-testsuite/module-lifecycle-census.{json,md}",
    "sealed_version": "0.8.54",
    "not_proven": "no native shared-object semantics; no ABI compatibility with real cobcrun; module state is interpreted, not a loaded DSO",
    "breaks_claim": "describing the module artifact as a native .so; delegating module execution to real cobcrun",
    "readiness": 1,
    "lie_prevented": "'-m produces a GnuCOBOL module' is the lie this prevents",
    "damage_if_overclaimed": "presenting an interpreted-module manifest as a shared object would break the no-native-artifact boundary"
})

add({
    "id": "GNURUST.MODULE.CALL.1",
    "name": "CALL semantics across separately compiled modules (sibling resolution + EXTERNAL sharing)",
    "proven": "caller CALLs a separately compiled callee module (the suite's caller.cob/callee.cob pattern): the callee source is resolved at compile time and EXTERNAL items are shared across the call (run-unit store), producing the oracle's exact stdout (Hello/World); exercised by GNURUST.MODULE.MULTI.1 and the suite rerun",
    "byte_domain": "caller/callee stdout, EXTERNAL storage flow, exit status",
    "oracle": "the admitted suite's caller/callee module tests (baseline stdout)",
    "fixtures": "crates/cobc-rs/tests/module_courts.rs (separately_compiled_callee_is_called_through_the_module)",
    "sealed_version": "0.8.54",
    "not_proven": "no dynamic loading of arbitrary DSOs; modules are source-resolved, not dlopen'ed",
    "breaks_claim": "claiming dlopen/ELF module loading",
    "readiness": 1,
    "lie_prevented": "'dynamic CALL loads native modules' is the lie this prevents",
    "damage_if_overclaimed": "overclaiming the module model as native dynamic loading"
})

add({
    "id": "GNURUST.MODULE.CANCEL.1",
    "name": "CANCEL semantics (persisted WORKING-STORAGE reset; active-program fatal)",
    "proven": "a called module's WORKING-STORAGE persists across calls and is reset by CANCEL (oracle-shaped C=1/C=2/C=1), and CANCELing the active non-INITIAL program raises the libcob-shaped fatal 'attempt to CANCEL active program' (exit 1, source line); exercised by GNURUST.MODULE.CANCEL.1 tests",
    "byte_domain": "module state across CALL/CANCEL, fatal-error stderr + exit code",
    "oracle": "the admitted suite's CANCEL tests (run_fundamental.at:2277-2341)",
    "fixtures": "crates/cobc-rs/tests/module_courts.rs (cancel_resets_persisted_working_storage, cancel_of_active_program_is_fatal)",
    "sealed_version": "0.8.54",
    "not_proven": "no claim about INITIAL-program edge cases beyond the tested forms; physical CANCEL semantics are not native-unload",
    "breaks_claim": "claiming CANCEL unloads a native module",
    "readiness": 1,
    "lie_prevented": "'CANCEL is a no-op' is the lie this prevents",
    "damage_if_overclaimed": "overclaiming physical module unload"
})

add({
    "id": "GNURUST.MODULE.SEARCH.1",
    "name": "cobcrun module search paths (-M dir, cwd, COB_LIBRARY_PATH) + error messages",
    "proven": "cobcrun-rs resolves modules via -M <dir> (with and without the trailing slash), cwd and COB_LIBRARY_PATH, and emits the cobcrun diagnostics for missing program name / invalid module argument / cannot find module with cobcrun's exit codes",
    "byte_domain": "search-path resolution + diagnostic stdout/stderr/exit",
    "oracle": "the admitted suite's used_binaries.at module tests (0010, 0014, 0015, 0018)",
    "fixtures": "crates/cobc-rs/tests/module_courts.rs (cobcrun_m_searches_the_module_directory, cobcrun_module_search_uses_cwd_and_library_path, cobcrun_error_messages_match_cobcrun)",
    "sealed_version": "0.8.54",
    "not_proven": "no claim that module-name case folding matches cobcrun beyond the tested forms",
    "breaks_claim": "a module resolved from a path the oracle would not search",
    "readiness": 1,
    "lie_prevented": "'cobcrun can't find anything' is the lie this prevents",
    "damage_if_overclaimed": "overclaiming search-path parity beyond the tested surface"
})

add({
    "id": "GNURUST.MODULE.PARALLEL.1",
    "name": "module lifecycle under parallel execution (same basenames in isolated directories)",
    "proven": "100 concurrent cobc-rs/cobcrun-rs invocations with colliding source basenames in separate directories each see their OWN module (atomic manifest writes, no cross-test leakage, no shared mutable state)",
    "byte_domain": "per-directory stdout correctness under concurrency",
    "oracle": "deterministic per-directory expectation (each dir's own output)",
    "fixtures": "crates/cobc-rs/tests/module_courts.rs (one_hundred_parallel_modules_with_colliding_basenames_stay_isolated)",
    "sealed_version": "0.8.54",
    "not_proven": "no claim about concurrency inside a single program's execution",
    "breaks_claim": "any cross-directory module leakage under concurrency",
    "readiness": 1,
    "lie_prevented": "'parallel tests corrupt each other's modules' is the lie this prevents",
    "damage_if_overclaimed": "claiming concurrency safety without the stress court"
})

add({
    "id": "GNURUST.COBC-RS.NATIVE-MODE-BOUNDARY.1",
    "name": "native-code modes (-C/-S/-c) remain an honest typed boundary; adapter-compatible cases map truthfully",
    "proven": "the option-policy registry rejects native-code modes rather than faking artifacts; adapter-compatible workflows (later executable semantics only) are mapped onto candidate manifests with the translation recorded; native-artifact tests (generated C/assembly/object structure, symbols, relocations, linker behavior) remain a typed boundary with no support claim",
    "byte_domain": "wrapper option-policy registry + per-test classifications + the invocation census",
    "oracle": "the admitted suite's native-mode tests (used_binaries.at -C/-S/-c)",
    "fixtures": "docs/generated/cobc-rs-option-compatibility.md + reports/gnucobol-testsuite/unsupported-option-census.{json,md}",
    "sealed_version": "0.8.54",
    "not_proven": "no native code generation; no C/assembly/object emission; no linker behavior",
    "breaks_claim": "emitting fake C/assembly/object files to satisfy a path-existence check",
    "readiness": 1,
    "lie_prevented": "'-c produces an object file' is the lie this prevents",
    "damage_if_overclaimed": "counterfeiting native artifacts would misrepresent the interpreter as a code generator"
})

add({
    "id": "GNURUST.COBC-RS.POLICY-COMPLETE.1",
    "name": "wrapper option-policy registry completeness (every observed option has an explicit policy)",
    "proven": "every option in the real invocation census maps to an explicit policy (translated / accepted-proven-no-op / rejected-unsupported / rejected-ambiguous); the machine invariant 'observed options == explicit policy + intentional unknown-option tests + program args after the delimiter' holds; no unknown semantic option is silently discarded",
    "byte_domain": "policy registry export + invocation census + the generated compatibility document",
    "oracle": "the real invocation census (argv boundaries preserved)",
    "fixtures": "docs/generated/cobc-rs-option-compatibility.md (freshness-gated)",
    "sealed_version": "0.8.54",
    "not_proven": "no claim that accepted no-op flags preserve semantics outside the admitted tests",
    "breaks_claim": "an observed option without an explicit policy; a silently ignored semantic option",
    "readiness": 1,
    "lie_prevented": "'cobc-rs ignores unknown flags safely' is the lie this prevents",
    "damage_if_overclaimed": "claiming policy completeness without the census reconciliation"
})

add({
    "id": "GNURUST.GNUCOBOL-TESTSUITE.BOUNDARY-REDUCTION.1",
    "name": "three-boundary reduction ledger (module 407 / parser-check 439 / wrapper-option 173) -- before/after per test",
    "proven": "the boundary-reduction baseline (commit 25fb3410b) is bound to the suite/oracle/candidate identity + ledger + raw-evidence hashes; every v0.8.54 classification has a measured after-state from the rerun with its transition (MODULE_BOUNDARY_TO_MATCH / MODULE_BOUNDARY_TO_PARSER_REJECT / ...); no test is unaccounted; transitions are measured, never projected",
    "byte_domain": "boundary-reduction.{json,md} + classification-transitions.{json,md} + the raw rerun evidence",
    "oracle": "the v0.8.54 baseline record (reports/gnucobol-testsuite/boundary-reduction-baseline.json)",
    "fixtures": "reports/gnucobol-testsuite/boundary-reduction-baseline.json + boundary-reduction.{json,md} + classification-transitions.{json,md}",
    "sealed_version": "0.8.54",
    "not_proven": "no claim that a boundary reduction equals a pass; transitions are re-measured classifications",
    "breaks_claim": "a reclassification without raw evidence; a projected (unmeasured) unlock",
    "readiness": 1,
    "lie_prevented": "'407 module tests are fixed' is the lie this prevents",
    "damage_if_overclaimed": "presenting reclassification as implementation would hide the real boundaries"
})

add({
    "id": "GNURUST.GNUCOBOL-RUNTIME-MATH.2",
    "name": "runtime/mathematics campaign after the boundary-reduction work (math subset re-measured from the same full ledger)",
    "proven": "the math subset is recomputed from the SAME full-suite ledger as every other test after the rerun; the machine invariant sum(math classifications) == math test count == 323 (unique ids, ids subset of the suite) holds; the generator fails on any violation; performance is re-reported only for correctness-matched programs",
    "byte_domain": "math-correctness.{json,md} + math-performance.{json,csv,md} + raw-samples/",
    "oracle": "the TESTSUITE.1 baseline",
    "fixtures": "reports/gnucobol-runtime-tests/*",
    "sealed_version": "0.8.54",
    "not_proven": "no performance claim from end-to-end interpreter-vs-native timing; no equivalence claim outside the tested environment",
    "breaks_claim": "a math summary that does not reconcile to 323; prose counts that drift from the ledger",
    "readiness": 1,
    "lie_prevented": "'the 22/21 wrapper-option discrepancy was cosmetic' is the lie this prevents -- the reconciliation is machine-enforced",
    "damage_if_overclaimed": "claiming math parity from a non-reconciling ledger"
})

save(LADDER, ladder)
print("claim-ladder now has", len(ladder["courts"]), "courts")
add({
    "id": "GNURUST.METHODOLOGY.LIBCOB.1",
    "name": "libcob runtime port methodology (faithful derivative, NOT clean-room) -- documented + machine-recorded",
    "proven": "docs/methodology/libcob-rust-port.md + reports/methodology/libcob-port-provenance.json record the admitted source identity (gnucobol-3.2, sha256 8ecc77d0...), the 13 in-scope libcob files, the statement-by-statement translation method with upstream line citations, the 100%% symbol parity (LIBCOB-PARITY.md), the LGPL-3.0-or-later inheritance, and the explicit non-claims; the tooling history is recorded as UNKNOWN where the committed record does not show it",
    "byte_domain": "the provenance records + parity reports",
    "oracle": "n/a (documentation + machine records, cross-checked against the parity tooling)",
    "fixtures": "docs/methodology/libcob-rust-port.md + reports/methodology/libcob-port-provenance.json + LIBCOB-PARITY.md",
    "sealed_version": "0.8.54",
    "not_proven": "not every libcob function is behaviorally proven byte-equal; only the sealed corpus is",
    "breaks_claim": "calling the runtime clean-room; claiming full behavioral equality beyond the sealed corpus",
    "readiness": 1,
    "lie_prevented": "'the runtime is a clean-room reimplementation' is the lie this prevents",
    "damage_if_overclaimed": "mislabeling a derivative as clean-room would be a licensing/provenance error"
})

add({
    "id": "GNURUST.METHODOLOGY.PARSER.1",
    "name": "parser/front-end provenance audit (independently written per the author's committed claim; strict clean-room not independently verifiable)",
    "proven": "docs/methodology/parser-front-end-provenance.md + reports/methodology/parser-provenance.json reconstruct the parser history from the committed record (origin commit 9357a7cac 'from-scratch (NOT cobc-derived)', 150+ follow-on commits, oracle-differential development), and state explicitly that the tooling and consulted-materials history is UNKNOWN -- so strict clean-room process separation cannot be independently verified, and the documentation qualifies the term accordingly",
    "byte_domain": "the provenance records + the commit history they cite",
    "oracle": "n/a (historical/process documentation)",
    "fixtures": "docs/methodology/parser-front-end-provenance.md + reports/methodology/parser-provenance.json",
    "sealed_version": "0.8.54",
    "not_proven": "strict clean-room separation is NOT claimed; no claim that cobc source was never consulted (no evidence either way)",
    "breaks_claim": "asserting strict clean-room provenance without the tooling/materials record",
    "readiness": 1,
    "lie_prevented": "'the parser is strict clean-room' is the lie this prevents -- the honest position is independently written per the author's claim",
    "damage_if_overclaimed": "a strict clean-room claim would be legally load-bearing and unverifiable from the record"
})

save(LADDER, ladder)
print("claim-ladder now has", len(ladder["courts"]), "courts")

# ---- receipt manifest campaigns --------------------------------------------------------------
man = load(MANIFEST)
for cid in [
    "GNURUST.GNUCOBOL-TESTSUITE.1",
    "GNURUST.GNUCOBOL-TESTSUITE.2",
    "GNURUST.GNUCOBOL-TESTSUITE.3",
    "GNURUST.GNUCOBOL-TESTSUITE.4",
    "GNURUST.MODULE.REGISTRY.1",
    "GNURUST.MODULE.CALL.1",
    "GNURUST.MODULE.CANCEL.1",
    "GNURUST.MODULE.SEARCH.1",
    "GNURUST.MODULE.PARALLEL.1",
    "GNURUST.COBC-RS.NATIVE-MODE-BOUNDARY.1",
    "GNURUST.COBC-RS.POLICY-COMPLETE.1",
    "GNURUST.GNUCOBOL-TESTSUITE.BOUNDARY-REDUCTION.1",
    "GNURUST.GNUCOBOL-RUNTIME-MATH.2",
]:
    if cid in man["campaigns"]:
        print("manifest already has", cid)
        continue
    man["campaigns"][cid] = {
        "court": [c["name"] for c in ladder["courts"] if c["id"] == cid][0],
        "sweep": "gnucobol-testsuite/run-docker.sh",
        "byte_domain": "see the claim-ladder entry; the receipts are generated by the gnucobol-rs-testsuite harness receipts-finalize (Docker court), not by the xtask sweep runner",
        "non_claims": [
            "no GnuCOBOL test-suite parity claim; no COBOL conformance certification",
            "OBSERVABLE_MATCH is scoped to this environment and the test's own assertions",
            "the launcher artifact is an interpreter launch manifest, never a native COBOL executable",
            "candidate execution never invokes cobc/cobcrun and never links libcob (mechanically enforced)",
            "no performance claim from end-to-end interpreter-vs-native timing",
        ],
    }
# the testsuite courts must NOT be re-run by the xtask sweep runner
if "GNURUST.GNUCOBOL-TESTSUITE.1" not in man.get("non_xtask", []):
    man["non_xtask"] = man.get("non_xtask", []) + [
        "GNURUST.GNUCOBOL-TESTSUITE.1", "GNURUST.GNUCOBOL-TESTSUITE.2", "GNURUST.GNUCOBOL-TESTSUITE.3",
        "GNURUST.GNUCOBOL-TESTSUITE.4", "GNURUST.GNUCOBOL-TESTSUITE.BOUNDARY-REDUCTION.1"]
save(MANIFEST, man)
print("manifest campaigns:", len(man["campaigns"]))
