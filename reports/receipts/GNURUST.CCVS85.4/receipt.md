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
| generated_at | 2026-08-03T14:35:16Z |
| git_commit | `ee2300e13307ce8de3c5cc86236a0796fb1e702f` |
| receipt_status | current |

**Conformance claim:** NONE — differential observation over the admitted NIST CCVS85 corpus; no NIST certification, no full COBOL-85 conformance, no compiler-replacement claim.

## Results

```json
{
  "by_final_classification": {
    "HARNESS_BLOCKED": 1,
    "NON_EXECUTABLE_DATA": 2,
    "NON_EXECUTABLE_LIBRARY": 119,
    "ORACLE_COMPILE_ERROR": 3,
    "ORACLE_COMPILE_REJECT": 18,
    "ORACLE_RUN_FAIL": 64,
    "ORACLE_TIMEOUT": 1,
    "OUTPUT_MISMATCH": 2,
    "RAW_OUTPUT_MATCH": 11,
    "RUST_REJECT_PARSE": 1,
    "RUST_REJECT_UNSUPPORTED": 290
  },
  "canonical_output_match": 0,
  "comparison_results_sha256": "820e3ded79dae505b84ecd3e822aa27936754618268971ef3db3f7fdfb3c04fd",
  "determinism": {
    "note": "summary counts + classifications + reason buckets must be identical across two fresh full runs (timestamps deliberately excluded); per-unit oracle REPORT hashes are compared separately and any drift is recorded + explicitly classified",
    "pass_a": {
      "path": "/run/media/one/1tb_kingston1/docker/gnucobol-rs/outputs/20260803T143229Z-ee2300e1/pass-a/summary.json",
      "summary_sha256": "3a114d774a5a9b53939b73836babe4c2a1ae0f9351943d7216099d6460fd1de8"
    },
    "pass_b": {
      "path": "/run/media/one/1tb_kingston1/docker/gnucobol-rs/outputs/20260803T143229Z-ee2300e1/pass-b/summary.json",
      "summary_sha256": "0a81766dd3dbb00a4a1e974f7f8e515f7a4eddd257b5d3a7220ed623c8470f87"
    },
    "report_byte_nondeterminism": [
      {
        "name": "NC214M",
        "note": "oracle REPORT bytes differ between two fresh runs (e.g. a TIME test printing real fractional seconds); the unit is explicitly classified nondeterministic",
        "pass_a_report_sha256": "9200edc52b2cc703ea1e16b6ca3e4e5f66401cfa10170755ed539c06de3334e3",
        "pass_b_report_sha256": "0bc0394556c8cb16b4c229b9d74bb8f85ab546f8da9b93c3f120530630a5e8ce",
        "unit_index": 269
      }
    ],
    "schema": "gnurust-ccvs85-determinism-v1",
    "stable_summary_identical": true
  },
  "exit_status_mismatch": 0,
  "generated_file_mismatch": 0,
  "nondeterministic": 1,
  "output_mismatch": 2,
  "raw_output_match": 11,
  "summary_json_sha256": "52b8d3d263ad9aed44ead5b103ed23ddd9d33932a3f2bcabbcb7d0be9fe5193e",
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
