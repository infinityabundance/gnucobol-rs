<!-- GENERATED from receipt.json by gnucobol-rs-testsuite — DO NOT EDIT BY HAND.
     Regenerate: bash lab/gnucobol-testsuite/run-docker.sh -->
# GNURUST.GNUCOBOL-TESTSUITE.1 — GnuCOBOL 3.2 native Autotest suite custody + real-compiler baseline + invocation census

**Verdict: PASS** · replay `bash lab/gnucobol-testsuite/run-docker.sh`

| field | value |
|-------|-------|
| campaign | `GNURUST.GNUCOBOL-TESTSUITE.1` |
| court | GnuCOBOL 3.2 native Autotest suite custody + real-compiler baseline + invocation census |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | admitted gnucobol-3.2 source (hash-verified), fresh in-tree build per pass, the generated Autotest testsuite run with the REAL admitted cobc, full invocation census (argv boundaries preserved), raw testsuite.log + per-group logs preserved |
| replay command | `bash lab/gnucobol-testsuite/run-docker.sh` |
| generated_at | 2026-08-12T22:06:52Z |
| git_commit | `03d8aaab3707ecbf88a41ba6ed0264d2fd0b87a0` |
| receipt_status | current |

**Conformance claim:** NONE — differential observation over the admitted GnuCOBOL 3.2 native Autotest suite in this environment; no GnuCOBOL test-suite parity claim, no COBOL conformance certification, no compiler-replacement claim.

## Results

```json
{
  "invocation_census_sha256": "ba995566cb4db2eb3b7502429899dbd134c174e94c4744cb519034f27b12ad9d",
  "invocation_census_total": 2111,
  "oracle_fail": 0,
  "oracle_not_reached": 0,
  "oracle_pass": 1242,
  "oracle_results_sha256": "f070608a43fd7e61e01f871ef7ba85c6ce75b896841d9076f3a817cec2bad23c",
  "oracle_skip": 9,
  "oracle_xfail": 31,
  "oracle_xpass": 0,
  "suite_total_tests": 1282
}
```

## Non-claims

- no claim about gnucobol-rs is made by this gate
- oracle results are specific to this admitted build + environment (stock configuration, no -fpermissive)
- oracle-side failures are observations about this build, not upstream defects
- the census records invocations, not compiler internals
- no NIST certification and no COBOL conformance claim

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON.
