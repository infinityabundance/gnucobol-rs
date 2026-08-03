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
| generated_at | 2026-08-03T14:19:48Z |
| git_commit | `40236823970679e8f24b9a4a29e0f3b510a219d8` |
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
  "comparison_results_sha256": "b9ec02c1a48104bc87d1c5d005aeb93876c60fb722ecab879bb07fcc42336b56",
  "determinism": {
    "note": "summary counts + classifications + reason buckets must be identical across two fresh full runs (timestamps deliberately excluded)",
    "pass_a": {
      "path": "/run/media/one/1tb_kingston1/docker/gnucobol-rs/outputs/20260803T140006Z-183e21a4/pass-a/summary.json",
      "summary_sha256": "62c737c3341dbc94ce4464ef82172975858c861cc1de87b5730f4fabc7e38f82"
    },
    "pass_b": {
      "path": "/run/media/one/1tb_kingston1/docker/gnucobol-rs/outputs/20260803T140006Z-183e21a4/pass-b/summary.json",
      "summary_sha256": "e3ba03ebb9ef0245f7b5dfd9cab8e660f064a531a60913909c807c86068aad9c"
    },
    "schema": "gnurust-ccvs85-determinism-v1",
    "stable_summary_identical": true
  },
  "exit_status_mismatch": 0,
  "generated_file_mismatch": 0,
  "nondeterministic": 0,
  "output_mismatch": 2,
  "raw_output_match": 11,
  "summary_json_sha256": "62c737c3341dbc94ce4464ef82172975858c861cc1de87b5730f4fabc7e38f82",
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
