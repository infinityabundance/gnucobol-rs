<!-- GENERATED from receipt.json by gnucobol-rs-testsuite — DO NOT EDIT BY HAND.
     Regenerate: bash lab/gnucobol-testsuite/run-diagnostic-unblocked-docker.sh -->
# GNURUST.GNUCOBOL-TESTSUITE.DIAGNOSTIC-UNBLOCKED.1 — diagnostic-unblocked testsuite lane: mechanically restricted derivative exposing later semantic checks hidden behind exact compiler-diagnostic wording

**Verdict: PASS** · replay `bash lab/gnucobol-testsuite/run-diagnostic-unblocked-docker.sh`

| field | value |
|-------|-------|
| campaign | `GNURUST.GNUCOBOL-TESTSUITE.DIAGNOSTIC-UNBLOCKED.1` |
| court | diagnostic-unblocked testsuite lane: mechanically restricted derivative exposing later semantic checks hidden behind exact compiler-diagnostic wording |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | diagnostic-ignore.patch + transformations.json + tree-manifest.json + semantic-reachability.{json,md} + pristine-vs-diagnostic-unblocked.{json,md} + corpus-cross-check.{json,md} + both passes' raw testsuite evidence under reports/gnucobol-testsuite/diagnostic-unblocked/raw/ |
| replay command | `bash lab/gnucobol-testsuite/run-diagnostic-unblocked-docker.sh` |
| generated_at | 2026-08-12T19:18:42Z |
| git_commit | `e34b30bf30dcc9761b2fe2d39025012dc0137331` |
| receipt_status | current |

**Conformance claim:** NONE — semantic-reachability observation over a mechanically restricted derivative of the admitted GnuCOBOL 3.2 Autotest suite; no test-suite parity claim, no diagnostic-compatibility claim, no COBOL conformance certification.

## Results

```json
{
  "at_check_pristine": 3422,
  "at_check_transformed": 3422,
  "at_setup_pristine": 1344,
  "at_setup_transformed": 1344,
  "cross_check_agreed": 33,
  "cross_check_candidate_failures_on_valid_steps": 9,
  "cross_check_contract_contradictions": 0,
  "cross_check_matched": 33,
  "diagnostic_expectations_ignored": 621,
  "gate_failures": [],
  "gate_lifted_no_progress": 1,
  "generated_testsuite_bytes": 7946652,
  "generated_testsuite_sha256": "9ef79a4baa6ca98386858a42b486e8e8b67d8d05bb0a73b0b17a0148b0e77049",
  "group_index_identical": true,
  "groups_affected": 404,
  "groups_execution_reached": 27,
  "groups_later_compile_reached": 111,
  "groups_no_additional_step": 27,
  "groups_progressed_further": 377,
  "newly_exposed_compile_failures": 17,
  "newly_exposed_runtime_failures": 15,
  "newly_matched_runtime_checks": 12,
  "newly_reached_checks": 140,
  "newly_reached_runtime_checks": 27,
  "oracle_pristine_xpass": [],
  "oracle_unblocked_xpass": [
    116,
    323,
    336,
    350
  ],
  "patch_reproducible": true,
  "patch_sha256": "712a0b172021c7ec650c6d97e348b465efd355699e309e956d95f99a2dc69230",
  "pristine_candidate_xpass": 0,
  "pristine_group_passes": 196,
  "stderr_ignored": 620,
  "stdout_ignored": 1,
  "suite_groups": 1282,
  "transformations_reproducible": true,
  "transformer_version": "gnurust-diag-unblocked-transform-v1",
  "unblocked_candidate_xpass": 2,
  "unblocked_group_passes": 326
}
```

## Non-claims

- diagnostic-unblocked results are NOT pristine upstream testsuite passes
- ignored compiler diagnostic text is NOT diagnostic compatibility
- expected exit statuses, semantic runtime output and generated-file expectations are still enforced exactly
- the pristine upstream testsuite remains the compatibility authority and is untouched
- no new language/runtime compatibility claim from diagnostic-only steps
- the transformer decides solely from upstream test structure, never from candidate behaviour
- no candidate parser-success claim for steps validated only by the real cobc oracle

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON.
