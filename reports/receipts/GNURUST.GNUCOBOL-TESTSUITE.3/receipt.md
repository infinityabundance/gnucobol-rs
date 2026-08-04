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
| generated_at | 2026-08-04T15:00:05Z |
| git_commit | `8d8c499e8ed9e5b9307d007b2a4168ee88106c3a` |
| receipt_status | current |

**Conformance claim:** NONE — differential observation over the admitted GnuCOBOL 3.2 native Autotest suite in this environment; no GnuCOBOL test-suite parity claim, no COBOL conformance certification, no compiler-replacement claim.

## Results

```json
{
  "comparison_results_sha256": "32ff9d14f96afaa0919b3bb35133c7474d1138259f31e4a9cb1651a94a161772",
  "determinism": {
    "note": "stable summary counts + per-test classifications must be identical across two fresh full runs (timestamps deliberately excluded)",
    "pass_a": {
      "path": "$GNURUST_GNUCOBOL_TEST_DOCKER_ROOT/outputs/20260804T145746Z-8d8c499e/pass-a/summary.json",
      "summary_sha256": "df75eb50ab88a801b563cfc9bedbbb6bb8175b608ae64406105a8285616b754a"
    },
    "pass_b": {
      "path": "$GNURUST_GNUCOBOL_TEST_DOCKER_ROOT/outputs/20260804T145746Z-8d8c499e/pass-b/summary.json",
      "summary_sha256": "90abba39ca8dc4bc4e1b08128d8717cb19151850ded4e23a7709d8cd7a8292e0"
    },
    "path_notation": "paths are symbolic: $GNURUST_GNUCOBOL_TEST_DOCKER_ROOT is the configured docker root at run time; the raw unsanitized record is preserved outside git under $GNURUST_GNUCOBOL_TEST_DOCKER_ROOT/run-evidence/",
    "per_test_classifications_identical": true,
    "schema": "gnurust-gnucobol-testsuite-determinism-v1",
    "stable_summary_identical": true
  },
  "exit_status_mismatch": 0,
  "first_failure": {
    "CANDIDATE_CHECK_REJECT": 652,
    "CANDIDATE_MODULE_MODEL_UNSUPPORTED": 5,
    "CANDIDATE_PARSE_REJECT": 31,
    "CANDIDATE_RUNTIME_FAIL": 136,
    "CANDIDATE_UNSUPPORTED": 21,
    "OBSERVABLE_MATCH": 192,
    "ORACLE_SKIP": 9,
    "ORACLE_XFAIL": 31,
    "WRAPPER_INVOCATION_MALFORMED": 25,
    "WRAPPER_OPTION_UNSUPPORTED": 180
  },
  "generated_file_mismatch": 0,
  "observable_match": 192,
  "stderr_mismatch": 0,
  "stdout_mismatch": 0,
  "summary_json_sha256": "df75eb50ab88a801b563cfc9bedbbb6bb8175b608ae64406105a8285616b754a",
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
