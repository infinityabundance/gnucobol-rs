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
| generated_at | 2026-08-12T22:23:10Z |
| git_commit | `4f633c3ed297ca23935a081453d61c86e3de91db` |
| receipt_status | current |

**Conformance claim:** NONE — differential observation over the admitted NIST CCVS85 corpus; no NIST certification, no full COBOL-85 conformance, no compiler-replacement claim.

## Results

```json
{
  "by_final_classification": {
    "GENERATED_FILE_MISMATCH": 9,
    "HARNESS_BLOCKED": 1,
    "NON_EXECUTABLE_DATA": 2,
    "NON_EXECUTABLE_LIBRARY": 119,
    "ORACLE_COMPILE_ERROR": 3,
    "ORACLE_COMPILE_REJECT": 18,
    "ORACLE_RUN_FAIL": 64,
    "ORACLE_TIMEOUT": 1,
    "OUTPUT_MISMATCH": 41,
    "RAW_OUTPUT_MATCH": 28,
    "RUST_REJECT_PARSE": 36,
    "RUST_REJECT_RUNTIME_BOUNDARY": 1,
    "RUST_REJECT_UNSUPPORTED": 189
  },
  "canonical_output_match": 0,
  "comparison_results_sha256": "908e427a498b1a226bdd25504913ac8c299b33d4df1b83ae77afe37a116027a4",
  "determinism": {
    "note": "summary counts + classifications + reason buckets must be identical across two fresh full runs (timestamps deliberately excluded); per-unit oracle REPORT hashes are compared separately and any drift is recorded + explicitly classified",
    "pass_a": {
      "path": "$GNURUST_CCVS85_DOCKER_ROOT/outputs/20260812T222057Z-4f633c3e/pass-a/summary.json",
      "summary_sha256": "de8fb5fabae55f3c51a33a96ffaea1ff820efc460b48a4b153deaacb270fff95"
    },
    "pass_b": {
      "path": "$GNURUST_CCVS85_DOCKER_ROOT/outputs/20260812T222057Z-4f633c3e/pass-b/summary.json",
      "summary_sha256": "c02551f7cd19b57c862cea4c499cf9f30c4ded89e8aba6ce5ac6b06911c20e8a"
    },
    "path_notation": "paths are symbolic: $GNURUST_CCVS85_DOCKER_ROOT is the configured docker root at run time; the raw unsanitized record is preserved outside git under $GNURUST_CCVS85_DOCKER_ROOT/run-evidence/",
    "report_byte_nondeterminism": [
      {
        "name": "NC214M",
        "note": "oracle REPORT bytes differ between two fresh runs (e.g. a TIME test printing real fractional seconds); the unit is explicitly classified nondeterministic",
        "pass_a_report_sha256": "1480afd00adcb22ec92e3894ea7e070b989ebd07a66de04798fcb0c2f9ccff82",
        "pass_b_report_sha256": "c344dbb25a48ee23260bca3d41019c36de10bd50d010c90cd9fd238722e140cc",
        "unit_index": 269
      }
    ],
    "schema": "gnurust-ccvs85-determinism-v1",
    "stable_summary_identical": true
  },
  "exit_status_mismatch": 0,
  "generated_file_mismatch": 9,
  "nondeterministic": 1,
  "output_mismatch": 41,
  "raw_output_match": 28,
  "summary_json_sha256": "d370f6628341cb51b993e658afaa818f1231b7329a813e4c85ceaa9e74d10935",
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
