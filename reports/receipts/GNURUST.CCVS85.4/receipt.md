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
| generated_at | 2026-08-05T22:11:34Z |
| git_commit | `c35d6c2b93577013e5257c4bf60e23975d34640e` |
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
  "comparison_results_sha256": "8953673ac2d0c68a715731ff81b0e7116dd48c4ea5f50c93e40ff58d220a678c",
  "determinism": {
    "note": "summary counts + classifications + reason buckets must be identical across two fresh full runs (timestamps deliberately excluded); per-unit oracle REPORT hashes are compared separately and any drift is recorded + explicitly classified",
    "pass_a": {
      "path": "$GNURUST_CCVS85_DOCKER_ROOT/outputs/20260805T220903Z-c35d6c2b/pass-a/summary.json",
      "summary_sha256": "3e38979f8640f7a4a285c28b5a2860b4046a9e356bccf3b3b99329193d9c0a90"
    },
    "pass_b": {
      "path": "$GNURUST_CCVS85_DOCKER_ROOT/outputs/20260805T220903Z-c35d6c2b/pass-b/summary.json",
      "summary_sha256": "0902c5e3e976fb1bb8c7bb0d70f5b1dd9e3d9577abbfa5d9e3cb4133a48a3656"
    },
    "path_notation": "paths are symbolic: $GNURUST_CCVS85_DOCKER_ROOT is the configured docker root at run time; the raw unsanitized record is preserved outside git under $GNURUST_CCVS85_DOCKER_ROOT/run-evidence/",
    "report_byte_nondeterminism": [
      {
        "name": "NC214M",
        "note": "oracle REPORT bytes differ between two fresh runs (e.g. a TIME test printing real fractional seconds); the unit is explicitly classified nondeterministic",
        "pass_a_report_sha256": "bc9c0a5b1b54758ad52357636fec8dc746c94daab737d55ba1f059724356eefb",
        "pass_b_report_sha256": "9f603196c1373c580ebd83902f125fa1856483dbe1f4234a421f15844b215e0c",
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
  "summary_json_sha256": "a6c96bc37ae729b45dcc06c9f1ac44c17305e2620051534900054c45797c9b00",
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
