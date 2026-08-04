<!-- GENERATED from receipt.json by gnucobol-rs-testsuite — DO NOT EDIT BY HAND.
     Regenerate: bash lab/gnucobol-testsuite/run-docker.sh -->
# GNURUST.GNUCOBOL-TESTSUITE.3 — GnuCOBOL testsuite differential comparison + per-test classification

**Verdict: PASS** · replay `bash lab/gnucobol-testsuite/run-docker.sh`

| field | value |
|-------|-------|
| campaign | `GNURUST.GNUCOBOL-TESTSUITE.3` |
| court | GnuCOBOL testsuite differential comparison + per-test classification |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | per-test oracle-vs-candidate observable comparison, first-failure attribution, all-tests-accounted reconciliation, failure buckets |
| replay command | `bash lab/gnucobol-testsuite/run-docker.sh` |
| generated_at | 2026-08-04T10:31:57Z |
| git_commit | `9da27b98024ad436f14fab0eaceae058210974f0` |
| receipt_status | current |

**Conformance claim:** NONE — differential observation over the admitted GnuCOBOL 3.2 native Autotest suite in this environment; no GnuCOBOL test-suite parity claim, no COBOL conformance certification, no compiler-replacement claim.

## Results

```json
{
  "comparison_results_sha256": "5aaa56e4f40d483dd72731599ec942dd9c519333cd03cdfe91353bd177190682",
  "determinism": {
    "note": "stable summary counts + per-test classifications must be identical across two fresh full runs (timestamps deliberately excluded)",
    "pass_a": {
      "path": "$GNURUST_GNUCOBOL_TEST_DOCKER_ROOT/outputs/20260804T103103Z-9da27b98/pass-a/summary.json",
      "summary_sha256": "0ef66c79bbd591a15f6a3ee94f400c3c80cf88c6746c920624e8fbfbe2ce7613"
    },
    "pass_b": {
      "path": "$GNURUST_GNUCOBOL_TEST_DOCKER_ROOT/outputs/20260804T103103Z-9da27b98/pass-b/summary.json",
      "summary_sha256": "0ef66c79bbd591a15f6a3ee94f400c3c80cf88c6746c920624e8fbfbe2ce7613"
    },
    "path_notation": "paths are symbolic: $GNURUST_GNUCOBOL_TEST_DOCKER_ROOT is the configured docker root at run time; the raw unsanitized record is preserved outside git under $GNURUST_GNUCOBOL_TEST_DOCKER_ROOT/run-evidence/",
    "per_test_classifications_identical": true,
    "schema": "gnurust-gnucobol-testsuite-determinism-v1",
    "stable_summary_identical": true
  },
  "exit_status_mismatch": 0,
  "first_failure": {
    "CANDIDATE_CHECK_REJECT": 439,
    "CANDIDATE_MODULE_MODEL_UNSUPPORTED": 407,
    "CANDIDATE_RUNTIME_FAIL": 2,
    "CANDIDATE_UNSUPPORTED": 22,
    "OBSERVABLE_MATCH": 173,
    "ORACLE_SKIP": 9,
    "ORACLE_XFAIL": 31,
    "WRAPPER_INVOCATION_MALFORMED": 26,
    "WRAPPER_OPTION_UNSUPPORTED": 173
  },
  "generated_file_mismatch": 0,
  "observable_match": 173,
  "stderr_mismatch": 0,
  "stdout_mismatch": 0,
  "summary_json_sha256": "0ef66c79bbd591a15f6a3ee94f400c3c80cf88c6746c920624e8fbfbe2ce7613",
  "tests_accounted": 1282
}
```

## Non-claims

- no full GnuCOBOL test-suite parity claim
- no native-code-generation comparison (cobrun interprets; cobc emits C/native)
- OBSERVABLE_MATCH is scoped to this environment and the test's own assertions
- no claim that matching output proves equivalence outside the tested environment
- no claim that accepted no-op flags preserve all semantics outside the admitted tests
- no claim that a launcher is a GnuCOBOL-compatible native executable
- no claim that GnuCOBOL baseline failures prove upstream defects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON.
