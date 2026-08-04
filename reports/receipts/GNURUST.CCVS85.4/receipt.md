<!-- GENERATED from receipt.json by gnucobol-rs-ccvs85 — DO NOT EDIT BY HAND.
     Regenerate: bash lab/ccvs85/run-docker.sh -->
# GNURUST.CCVS85.4 — CCVS85 differential comparison + per-unit classification

**Verdict: PASS** · replay `bash lab/ccvs85/run-docker.sh`

| field | value |
|-------|-------|
| campaign | `GNURUST.CCVS85.4` |
| court | CCVS85 differential comparison + per-unit classification |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | per-unit oracle-vs-candidate observable comparison: raw output, canonical output, generated files, exit status, CCVS85 verdict counts |
| replay command | `bash lab/ccvs85/run-docker.sh` |
| generated_at | 2026-08-04T16:44:33Z |
| git_commit | `6c961627d8bfff765172661f8d232a56f36305a8` |
| receipt_status | current |

**Conformance claim:** NONE — differential observation over the admitted NIST CCVS85 corpus; no NIST certification, no full COBOL-85 conformance, no compiler-replacement claim.

## Results

```json
{
  "by_final_classification": {
    "GENERATED_FILE_MISMATCH": 8,
    "HARNESS_BLOCKED": 1,
    "NON_EXECUTABLE_DATA": 2,
    "NON_EXECUTABLE_LIBRARY": 119,
    "ORACLE_COMPILE_ERROR": 3,
    "ORACLE_COMPILE_REJECT": 18,
    "ORACLE_RUN_FAIL": 64,
    "ORACLE_TIMEOUT": 1,
    "OUTPUT_MISMATCH": 42,
    "RAW_OUTPUT_MATCH": 27,
    "RUST_REJECT_PARSE": 36,
    "RUST_REJECT_RUNTIME_BOUNDARY": 1,
    "RUST_REJECT_UNSUPPORTED": 190
  },
  "canonical_output_match": 0,
  "comparison_results_sha256": "cee01525ba85d6d801069a61bd8bdb8302fec1da7547fd775c8f9489e0753a16",
  "determinism": {
    "note": "summary counts + classifications + reason buckets must be identical across two fresh full runs (timestamps deliberately excluded); per-unit oracle REPORT hashes are compared separately and any drift is recorded + explicitly classified",
    "pass_a": {
      "path": "$GNURUST_CCVS85_DOCKER_ROOT/outputs/20260804T164214Z-6c961627/pass-a/summary.json",
      "summary_sha256": "21fb5b26790a781cae01121c5cf513ef16353c7882ffd86266820f73ca269ccb"
    },
    "pass_b": {
      "path": "$GNURUST_CCVS85_DOCKER_ROOT/outputs/20260804T164214Z-6c961627/pass-b/summary.json",
      "summary_sha256": "e4c0b07c36ea8c321d5938e72958f1186166b7e6597e2fb2828813c7fa0b4692"
    },
    "path_notation": "paths are symbolic: $GNURUST_CCVS85_DOCKER_ROOT is the configured docker root at run time; the raw unsanitized record is preserved outside git under $GNURUST_CCVS85_DOCKER_ROOT/run-evidence/",
    "report_byte_nondeterminism": [
      {
        "name": "NC214M",
        "note": "oracle REPORT bytes differ between two fresh runs (e.g. a TIME test printing real fractional seconds); the unit is explicitly classified nondeterministic",
        "pass_a_report_sha256": "bf6988dc0cde779fcf3a74de49056cfffb439361539da910e48e681a27c0827a",
        "pass_b_report_sha256": "cd939db4536a7852d14d7f7acd41c0b75668778f12104bd9eccc432f5cdd675d",
        "unit_index": 269
      }
    ],
    "schema": "gnurust-ccvs85-determinism-v1",
    "stable_summary_identical": true
  },
  "exit_status_mismatch": 0,
  "generated_file_mismatch": 8,
  "nondeterministic": 1,
  "output_mismatch": 42,
  "raw_output_match": 27,
  "summary_json_sha256": "f2052993818aee0ea9557bb1b41040ba901a4246891f657837bbd204845b61fd",
  "units_accounted": 512
}
```

## Non-claims

- no NIST certification
- no full COBOL-85 conformance claim
- no full cobc replacement claim
- no native-code-generation comparison (cobrun interprets; cobc emits C/native)
- no claim that an oracle rejection proves the source invalid under every COBOL implementation
- no claim that matching output proves equivalence outside the tested environment
- no claim that library/data units are executable tests
- no conversion of blocked units into passes

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON.
