<!-- GENERATED from receipt.json by gnucobol-rs-testsuite — DO NOT EDIT BY HAND.
     Regenerate: bash lab/gnucobol-testsuite/run-docker.sh -->
# GNURUST.GNUCOBOL-TESTSUITE.2 — candidate execution: the native suite run with COBC=cobc-rs (make localcheck), no-delegation proof, all tests accounted

**Verdict: PASS** · replay `bash lab/gnucobol-testsuite/run-docker.sh`

| field | value |
|-------|-------|
| campaign | `GNURUST.GNUCOBOL-TESTSUITE.2` |
| court | candidate execution: the native suite run with COBC=cobc-rs (make localcheck), no-delegation proof, all tests accounted |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | cobc-rs adapter + cobrun interpreter outcomes per test, raw candidate testsuite.log + group logs, mechanical no-delegation proof (linkage scans + PATH isolation) |
| replay command | `bash lab/gnucobol-testsuite/run-docker.sh` |
| generated_at | 2026-08-04T10:31:57Z |
| git_commit | `9da27b98024ad436f14fab0eaceae058210974f0` |
| receipt_status | current |

**Conformance claim:** NONE — differential observation over the admitted GnuCOBOL 3.2 native Autotest suite in this environment; no GnuCOBOL test-suite parity claim, no COBOL conformance certification, no compiler-replacement claim.

## Results

```json
{
  "candidate_module_model_unsupported": 407,
  "candidate_not_reached": 0,
  "candidate_parse_check_reject": 439,
  "candidate_results_sha256": "a33e39a1ec83b98f742a8286d44d5d09a8b84c4b8a148753c01a1bf3c0a88f8e",
  "candidate_runtime_fail": 2,
  "candidate_skipped": 0,
  "candidate_timeout": 0,
  "candidate_unsupported": 22,
  "no_delegation": {
    "candidate_binary_sha256": "474ecd9046e4f41c8c2d23ae4f51ee87687ae8501f50a73dfe0dbbe91136d371",
    "candidate_phase_isolated": true,
    "candidate_phase_note": "candidate phase isolated from the oracle (only /work/run/candidate-bin/cobc + /work/run/candidate-bin/cobcrun on PATH)",
    "cobc_resolves_to_candidate_during_candidate_phase": true,
    "cobc_rs_binary_sha256": "3e27898f4284843cec26dbce8720fc9621338a2be5f668923c93d6d8f280c13f",
    "cobc_rs_ldd_libcob_hits": 0,
    "cobc_rs_links_no_libcob": true,
    "cobc_rs_readelf_libcob_hits": 0,
    "cobcrun_resolves_to_candidate_during_candidate_phase": true,
    "cobrun_ldd_libcob_hits": 0,
    "cobrun_links_no_libcob": true,
    "cobrun_readelf_libcob_hits": 0,
    "cobrun_version": "cobrun (gnucobol-rs, reproducing GnuCOBOL) 3.2.0",
    "oracle_prefix_absent_during_candidate_phase": true,
    "schema": "gnurust-gnucobol-testsuite-no-delegation-v1"
  }
}
```

## Non-claims

- no suite-pass or parity claim: this gate records candidate outcomes, it does not certify them
- candidate rejection is fail-closed (unsupported constructs are never silently run)
- generated launch artifacts are interpreter manifests, NOT native COBOL executables
- candidate execution never invokes cobc and never links libcob (mechanically enforced + recorded)

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON.
