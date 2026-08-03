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
| generated_at | 2026-08-03T19:56:02Z |
| git_commit | `a902ca8e8e4990827f5b4f86b30275919408348b` |
| receipt_status | current |

**Conformance claim:** NONE — differential observation over the admitted NIST CCVS85 corpus; no NIST certification, no full COBOL-85 conformance, no compiler-replacement claim.

## Results

```json
{
  "by_final_classification": {
    "GENERATED_FILE_MISMATCH": 16,
    "HARNESS_BLOCKED": 1,
    "NON_EXECUTABLE_DATA": 2,
    "NON_EXECUTABLE_LIBRARY": 119,
    "ORACLE_COMPILE_ERROR": 3,
    "ORACLE_COMPILE_REJECT": 18,
    "ORACLE_RUN_FAIL": 64,
    "ORACLE_TIMEOUT": 1,
    "OUTPUT_MISMATCH": 42,
    "RAW_OUTPUT_MATCH": 27,
    "RUST_REJECT_PARSE": 55,
    "RUST_REJECT_RUNTIME_BOUNDARY": 3,
    "RUST_REJECT_UNSUPPORTED": 161
  },
  "canonical_output_match": 0,
  "comparison_results_sha256": "6b8ac37c1e20cef5081b9ddc1a2550672196fa9f62139f14b47c359d1944ad72",
  "determinism": {
    "note": "summary counts + classifications + reason buckets must be identical across two fresh full runs (timestamps deliberately excluded); per-unit oracle REPORT hashes are compared separately and any drift is recorded + explicitly classified",
    "pass_a": {
      "path": "$GNURUST_CCVS85_DOCKER_ROOT/outputs/20260803T195332Z-a902ca8e/pass-a/summary.json",
      "summary_sha256": "abff7443295199d92424dcf8c5a186c00e7eb09c584c329a8bc47d5a67669f00"
    },
    "pass_b": {
      "path": "$GNURUST_CCVS85_DOCKER_ROOT/outputs/20260803T195332Z-a902ca8e/pass-b/summary.json",
      "summary_sha256": "3e00359293473c2a0dfd0b1ee32edb7632256207b27f9131881ed2a38fa74039"
    },
    "path_notation": "paths are symbolic: $GNURUST_CCVS85_DOCKER_ROOT is the configured docker root at run time; the raw unsanitized record is preserved outside git under $GNURUST_CCVS85_DOCKER_ROOT/run-evidence/",
    "report_byte_nondeterminism": [
      {
        "name": "NC214M",
        "note": "oracle REPORT bytes differ between two fresh runs (e.g. a TIME test printing real fractional seconds); the unit is explicitly classified nondeterministic",
        "pass_a_report_sha256": "c344dbb25a48ee23260bca3d41019c36de10bd50d010c90cd9fd238722e140cc",
        "pass_b_report_sha256": "49e084051f8af882ac539d85e1bc0c35835d0b3a63430ae4137fdcef3683318e",
        "unit_index": 269
      }
    ],
    "schema": "gnurust-ccvs85-determinism-v1",
    "stable_summary_identical": true
  },
  "exit_status_mismatch": 0,
  "generated_file_mismatch": 16,
  "nondeterministic": 1,
  "output_mismatch": 42,
  "raw_output_match": 27,
  "summary_json_sha256": "4db99e613bab60d11f98582797e4e491636579a683bdc2741c59116e019c7d41",
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
