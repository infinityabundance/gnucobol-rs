<!-- GENERATED from receipt.json by gnucobol-rs-ccvs85 — DO NOT EDIT BY HAND.
     Regenerate: bash lab/ccvs85/run-docker.sh -->
# GNURUST.CCVS85.3 — gnucobol-rs execution baseline over the materialized CCVS85 units

**Verdict: PASS** · replay `bash lab/ccvs85/run-docker.sh`

| field | value |
|-------|-------|
| campaign | `GNURUST.CCVS85.3` |
| court | gnucobol-rs execution baseline over the materialized CCVS85 units |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | per-unit cobrun (native-Rust front-end + ported runtime) prepare/run/timeout outcomes with raw stdout/stderr preserved |
| replay command | `bash lab/ccvs85/run-docker.sh` |
| generated_at | 2026-08-05T22:11:34Z |
| git_commit | `c35d6c2b93577013e5257c4bf60e23975d34640e` |
| receipt_status | current |

**Conformance claim:** NONE — differential observation over the admitted NIST CCVS85 corpus; no NIST certification, no full COBOL-85 conformance, no compiler-replacement claim.

## Results

```json
{
  "candidate_accepted": 78,
  "candidate_parse_fail": 39,
  "candidate_results_sha256": "db278d2552dce4f5bc8238cb813b404ec7697a9e606d979621cb4df124360357",
  "candidate_runtime_fail": 0,
  "candidate_timeout": 0,
  "candidate_unsupported": 274,
  "no_delegation": {
    "candidate_binary_path": "/work/target/release/examples/cobrun",
    "candidate_binary_sha256": "2cab6054ac4c65c7436afa2fef6a81cba953d8a4f7f0ac2f6255db2f98f92bf9",
    "candidate_phase_isolated": true,
    "candidate_phase_note": "candidate phase isolated from the oracle (no cobc, no libcob visible)",
    "cobc_unavailable_during_candidate_phase": true,
    "cobrun_ldd_libcob_hits": 0,
    "cobrun_links_no_libcob": true,
    "cobrun_readelf_libcob_hits": 0,
    "cobrun_version": "cobrun (gnucobol-rs, reproducing GnuCOBOL) 3.2.0",
    "schema": "gnurust-ccvs85-no-delegation-v1"
  }
}
```

## Non-claims

- no suite-pass claim: this gate records candidate outcomes, it does not certify them
- candidate rejection is fail-closed (unsupported constructs are never silently run)
- no claim that candidate acceptance implies COBOL-85 conformance
- candidate execution never invokes cobc and never links libcob (mechanically enforced and recorded)

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON.
