# GnuCOBOL testsuite differential — pass a

All 1282 generated test groups reconciled (each has exactly one final classification).

## Oracle (real admitted GnuCOBOL 3.2, in-tree build)

- pass: 1242
- fail: 0
- skip: 9
- xfail: 31
- xpass: 0
- not reached: 0

## Candidate (cobc-rs + cobrun)

- parse/check reject: 683
- unsupported: 21
- module-model unsupported: 5
- runtime fail: 136
- timeout: 0
- not reached: 0
- skipped: 0

## Comparison

- observable match: 192
- stdout mismatch: 0
- stderr mismatch: 0
- exit-status mismatch: 0
- generated-file mismatch: 0

## Claims and non-claims

- OBSERVABLE_MATCH means the test's own AT_CHECK assertions held on both sides in this environment.
- No full GnuCOBOL test-suite parity claim; no native-code generation; no COBOL conformance certification.
- Candidate execution cannot delegate to cobc/cobcrun/libcob (mechanical no-delegation proof in no-delegation.json).
- Baseline failures are observations about this admitted build, not upstream defects.
