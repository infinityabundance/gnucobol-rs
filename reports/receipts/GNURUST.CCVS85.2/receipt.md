<!-- GENERATED from receipt.json by gnucobol-rs-ccvs85 — DO NOT EDIT BY HAND.
     Regenerate: bash lab/ccvs85/run-docker.sh -->
# GNURUST.CCVS85.2 — CCVS85 materialization + real-GnuCOBOL oracle baseline

**Verdict: PASS** · replay `bash lab/ccvs85/run-docker.sh`

| field | value |
|-------|-------|
| campaign | `GNURUST.CCVS85.2` |
| court | CCVS85 materialization + real-GnuCOBOL oracle baseline |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | CCVS85 corpus units materialized to stable files (hashes recorded) + per-unit cobc compile/run outcomes (real GnuCOBOL 3.2, pinned source) |
| replay command | `bash lab/ccvs85/run-docker.sh` |
| generated_at | 2026-08-04T16:44:33Z |
| git_commit | `6c961627d8bfff765172661f8d232a56f36305a8` |
| receipt_status | current |

**Conformance claim:** NONE — differential observation over the admitted NIST CCVS85 corpus; no NIST certification, no full COBOL-85 conformance, no compiler-replacement claim.

## Results

```json
{
  "dependency_blocked": 0,
  "executable_candidates": 391,
  "harness_blocked": 1,
  "materialized_manifest_sha256": "984a60dd424ad31e68a512e987d16027c70de5a654c3e119c943e1bb4b0f8cbd",
  "oracle_compile_error": 3,
  "oracle_compile_pass": 370,
  "oracle_compile_reject": 18,
  "oracle_results_sha256": "b09bf7c05f5a548e336611bf31e53e7dd95ff0dedce744a8e98eb8d772809e16",
  "oracle_run_fail": 64,
  "oracle_run_pass": 304,
  "oracle_timeout": 1,
  "units_by_kind": {
    "CLBRY": 51,
    "COBOL": 459,
    "DATA*": 2
  },
  "units_indexed": 512
}
```

## Non-claims

- no claim about gnucobol-rs is made by this gate
- oracle acceptance/rejection is specific to the pinned GnuCOBOL 3.2 build and its dialect
- no NIST certification and no COBOL-85 conformance claim
- CLBRY/DATA* units are support units, not executable tests

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON.
