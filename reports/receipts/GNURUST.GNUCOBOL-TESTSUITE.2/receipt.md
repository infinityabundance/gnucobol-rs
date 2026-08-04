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
| generated_at | 2026-08-04T15:07:09Z |
| git_commit | `d95e32354538de5feef0e5f93b1a488fec145516` |
| receipt_status | current |

**Conformance claim:** NONE — differential observation over the admitted GnuCOBOL 3.2 native Autotest suite in this environment; no GnuCOBOL test-suite parity claim, no COBOL conformance certification, no compiler-replacement claim.

## Results

```json
{
  "candidate_module_model_unsupported": 4,
  "candidate_not_reached": 0,
  "candidate_parse_check_reject": 683,
  "candidate_results_sha256": "f37a1563e9ab9691a758ea691d470c19171d3c109bb98cd09bcc105479eca173",
  "candidate_runtime_fail": 136,
  "candidate_skipped": 0,
  "candidate_timeout": 0,
  "candidate_unsupported": 21,
  "no_delegation": {
    "candidate_bin": {
      "cobc": {
        "resolves_to": "/work/target/release/cobc-rs",
        "symlink_target": "cobc-rs"
      },
      "cobc_rs_sha256": "eb029f12c15b9fd2b444403cf1cb4b2d30b1b570fd36700be1a0a94bb4cb2858",
      "cobcrun": {
        "resolves_to": "/work/target/release/cobc-rs",
        "symlink_target": "cobcrun-rs"
      },
      "cobrun_sha256": "ece2ea6de61dff76b4443f6846925dd1267bc3f0bb64c81a54479fa033ee063f"
    },
    "candidate_binary_sha256": "ece2ea6de61dff76b4443f6846925dd1267bc3f0bb64c81a54479fa033ee063f",
    "candidate_phase_isolated": true,
    "candidate_phase_note": "candidate phase isolated from the oracle (only /work/run/candidate-bin/cobc + /work/run/candidate-bin/cobcrun on PATH)",
    "cobc_resolves_to_candidate_during_candidate_phase": true,
    "cobc_rs_binary_sha256": "eb029f12c15b9fd2b444403cf1cb4b2d30b1b570fd36700be1a0a94bb4cb2858",
    "cobc_rs_ldd_libcob_hits": 0,
    "cobc_rs_links_no_libcob": true,
    "cobc_rs_readelf_libcob_hits": 0,
    "cobcrun_resolves_to_candidate_during_candidate_phase": true,
    "cobrun_ldd_libcob_hits": 0,
    "cobrun_links_no_libcob": true,
    "cobrun_readelf_libcob_hits": 0,
    "cobrun_version": "cobrun (gnucobol-rs, reproducing GnuCOBOL) 3.2.0",
    "oracle_prefix_absent_during_candidate_phase": true,
    "schema": "gnurust-gnucobol-testsuite-no-delegation-v2"
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
