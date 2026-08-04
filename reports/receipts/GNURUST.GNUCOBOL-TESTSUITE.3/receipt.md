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
| generated_at | 2026-08-04T14:46:50Z |
| git_commit | `93ec073d2fd28565c0241c5868a0352c39b14360` |
| receipt_status | current |

**Conformance claim:** NONE — differential observation over the admitted GnuCOBOL 3.2 native Autotest suite in this environment; no GnuCOBOL test-suite parity claim, no COBOL conformance certification, no compiler-replacement claim.

## Results

```json
{
  "comparison_results_sha256": "d7205a9aac83c16892102349099b6bc4dd70d20c3f347640e9ca83e6c74284fd",
  "determinism": {
    "note": "stable summary counts + per-test classifications must be identical across two fresh full runs (timestamps deliberately excluded)",
    "pass_a": {
      "path": "$GNURUST_GNUCOBOL_TEST_DOCKER_ROOT/outputs/20260804T144533Z-93ec073d/pass-a/summary.json",
      "summary_sha256": "eb28968bda6580321f9e45fe57691fa526a788a0bd424ae1bc93ec5ac44791d1"
    },
    "pass_b": {
      "path": "$GNURUST_GNUCOBOL_TEST_DOCKER_ROOT/outputs/20260804T144533Z-93ec073d/pass-b/summary.json",
      "summary_sha256": "eb28968bda6580321f9e45fe57691fa526a788a0bd424ae1bc93ec5ac44791d1"
    },
    "path_notation": "paths are symbolic: $GNURUST_GNUCOBOL_TEST_DOCKER_ROOT is the configured docker root at run time; the raw unsanitized record is preserved outside git under $GNURUST_GNUCOBOL_TEST_DOCKER_ROOT/run-evidence/",
    "per_test_classifications_identical": true,
    "schema": "gnurust-gnucobol-testsuite-determinism-v1",
    "stable_summary_identical": true
  },
  "exit_status_mismatch": 0,
  "first_failure": {
    "CANDIDATE_CHECK_REJECT": 671,
    "CANDIDATE_MODULE_MODEL_UNSUPPORTED": 6,
    "CANDIDATE_PARSE_REJECT": 29,
    "CANDIDATE_RUNTIME_FAIL": 131,
    "CANDIDATE_UNSUPPORTED": 22,
    "OBSERVABLE_MATCH": 178,
    "ORACLE_SKIP": 9,
    "ORACLE_XFAIL": 31,
    "WRAPPER_INVOCATION_MALFORMED": 25,
    "WRAPPER_OPTION_UNSUPPORTED": 180
  },
  "generated_file_mismatch": 0,
  "observable_match": 178,
  "stderr_mismatch": 0,
  "stdout_mismatch": 0,
  "summary_json_sha256": "eb28968bda6580321f9e45fe57691fa526a788a0bd424ae1bc93ec5ac44791d1",
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
