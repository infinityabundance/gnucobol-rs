# GNURUST.CCVS85.4 — NIST CCVS85 differential execution report

**GENERATED** by `cargo run -p gnucobol-rs-ccvs85 -- classify` — do not edit by hand.

`GNURUST.CCVS85.4` is a differential execution report over the admitted NIST CCVS85 Version 4.0
corpus. It reports which indexed units the pinned GnuCOBOL 3.2 oracle compiles and runs, which
units the current `gnucobol-rs` front-end accepts and executes, and where their observable
results agree or differ. It is **not** a NIST certification, does **not** establish complete
COBOL-85 conformance, and does **not** turn unsupported or unexecuted units into passes.

## Totals

| measure | count |
|---|---|
| units indexed (must reconcile) | **512** |
| units by kind `CLBRY` | 51 |
| units by kind `COBOL` | 459 |
| units by kind `DATA*` | 2 |
| executable candidates | 391 |
| oracle compile pass | 370 |
| oracle compile reject | 18 |
| oracle compile error | 3 |
| oracle run pass | 304 |
| oracle run fail | 64 |
| oracle timeout | 1 |
| candidate accepted | 15 |
| candidate unsupported | 374 |
| candidate parse/layout/boundary reject | 2 |
| candidate runtime fail | 0 |
| candidate timeout | 1 |
| raw output match | 11 |
| canonical output match | 0 |
| output mismatch | 2 |
| exit-status mismatch | 0 |
| generated-file mismatch | 0 |
| harness-blocked | 1 |
| dependency-blocked | 0 |
| infrastructure error | 0 |
| nondeterministic (explicitly classified) | 0 |

## By primary classification

| classification | count |
|---|---|
| `HARNESS_BLOCKED` | 1 |
| `NON_EXECUTABLE_DATA` | 2 |
| `NON_EXECUTABLE_LIBRARY` | 119 |
| `ORACLE_COMPILE_ERROR` | 3 |
| `ORACLE_COMPILE_REJECT` | 18 |
| `ORACLE_RUN_FAIL` | 64 |
| `ORACLE_TIMEOUT` | 1 |
| `OUTPUT_MISMATCH` | 2 |
| `RAW_OUTPUT_MATCH` | 11 |
| `RUST_REJECT_PARSE` | 1 |
| `RUST_REJECT_UNSUPPORTED` | 290 |

## By CCVS85 section (name prefix)

| section | count |
|---|---|
| `AL` | 2 |
| `CM` | 9 |
| `DB` | 15 |
| `EX` | 1 |
| `IC` | 47 |
| `IF` | 45 |
| `IX` | 42 |
| `K` | 36 |
| `KK` | 1 |
| `KP` | 10 |
| `KS` | 2 |
| `NC` | 97 |
| `OB` | 9 |
| `RL` | 35 |
| `RW` | 6 |
| `SG` | 13 |
| `SM` | 17 |
| `SQ` | 85 |
| `ST` | 40 |

## By reason code (top buckets)

| reason | count |
|---|---|
| `COBRUN_UNSUPPORTED_WRITE_DUMMY_RECORD_NOT_RECORD` | 191 |
| `SUBPROGRAM_BOUND_TO_MAIN` | 68 |
| `ORACLE_RUN_NONZERO_EXIT` | 64 |
| `LIBRARY_TEXT_UNIT` | 51 |
| `COBRUN_UNSUPPORTED_PROCEDURE_DIVISION` | 34 |
| `RAW_OUTPUT_IDENTICAL` | 11 |
| `COBRUN_UNSUPPORTED_UNSUPPORTED_LEVEL_NUMBER` | 10 |
| `COBRUN_UNSUPPORTED_UNSUPPORTED_LEVEL_NUMBER_THE` | 7 |
| `COBRUN_UNSUPPORTED_NOT_NUMERIC_LITERAL_ALL` | 5 |
| `COBRUN_UNSUPPORTED_UNRECOGNIZED_USAGE_COMPUTATIONAL` | 5 |
| `COBC_CRASHED` | 3 |
| `COBRUN_UNSUPPORTED_PIC_ABABX0A_UNSUPPORTEDSYMBOL` | 3 |
| `COBRUN_UNSUPPORTED_UNSUPPORTED_LEVEL_NUMBER_COPY` | 3 |
| `COBRUN_UNSUPPORTED_VERB_SORT_PARA` | 3 |
| `COBRUN_UNSUPPORTED_PIC_XXBXXBXX_UNSUPPORTEDSYMBOL` | 2 |
| `COBRUN_UNSUPPORTED_UNSUPPORTED_LEVEL_NUMBER_THESE` | 2 |
| `COBRUN_UNSUPPORTED_UNSUPPORTED_LEVEL_NUMBER_THIS` | 2 |
| `COBRUN_UNSUPPORTED_VERB_COPY` | 2 |
| `COBRUN_UNSUPPORTED_VERB_NUMBER1` | 2 |
| `DATA_UNIT` | 2 |
| `OUTPUT_BYTES_DIFFER` | 2 |
| `COBRUN_UNDEFINED_DATA_NAME_TEST_RESULTS` | 1 |
| `COBRUN_UNSUPPORTED_DISABLE_GNUCOBOL_DOES_NOT_IMPLEMENT_THE` | 1 |
| `COBRUN_UNSUPPORTED_FUNCTION_RANDOM_COBC_DOES_NOT_IMPLEMENT` | 1 |
| `COBRUN_UNSUPPORTED_NOT_NUMERIC_LITERAL_12345678` | 1 |
| `COBRUN_UNSUPPORTED_OPEN_PRINT_FILE_NOT_DECLARED_FILE` | 1 |
| `COBRUN_UNSUPPORTED_PIC_ABA_UNSUPPORTEDSYMBOL` | 1 |
| `COBRUN_UNSUPPORTED_PIC_BADREPEAT` | 1 |
| `COBRUN_UNSUPPORTED_PIC_UNSUPPORTEDSYMBOL` | 1 |
| `COBRUN_UNSUPPORTED_PIC_XBX0XBX0X_UNSUPPORTEDSYMBOL` | 1 |
| `COBRUN_UNSUPPORTED_PIC_XBXBXBX_UNSUPPORTEDSYMBOL` | 1 |
| `COBRUN_UNSUPPORTED_PIC_XXBXX_UNSUPPORTEDSYMBOL` | 1 |
| `COBRUN_UNSUPPORTED_SORT_MERGE_KEY_SORTKEY_NOT_FIELD` | 1 |
| `COBRUN_UNSUPPORTED_UNSUPPORTED_LEVEL_NUMBER_14003` | 1 |
| `COBRUN_UNSUPPORTED_UNSUPPORTED_LEVEL_NUMBER_COMMUNICATION` | 1 |
| `COBRUN_UNSUPPORTED_UNSUPPORTED_LEVEL_NUMBER_FEATURE` | 1 |
| `COBRUN_UNSUPPORTED_UNSUPPORTED_LEVEL_NUMBER_REPLACE` | 1 |
| `COBRUN_UNSUPPORTED_UNSUPPORTED_LEVEL_NUMBER_ST102A` | 1 |
| `COBRUN_UNSUPPORTED_UNSUPPORTED_LEVEL_NUMBER_ST120A` | 1 |
| `COBRUN_UNSUPPORTED_VERB_BEANO` | 1 |

## Oracle × candidate outcome pairs

| pair | count |
|---|---|
| `oracle: / candidate:` | 53 |
| `oracle:bound-to-main / candidate:bound-to-main` | 68 |
| `oracle:compile-pass / candidate:reject-unsupported` | 1 |
| `oracle:error / candidate:reject-unsupported` | 3 |
| `oracle:fail / candidate:reject-runtime-boundary` | 1 |
| `oracle:fail / candidate:reject-unsupported` | 61 |
| `oracle:fail / candidate:run-pass` | 1 |
| `oracle:fail / candidate:timeout` | 1 |
| `oracle:pass / candidate:reject-parse` | 1 |
| `oracle:pass / candidate:reject-unsupported` | 290 |
| `oracle:pass / candidate:run-pass` | 13 |
| `oracle:reject / candidate:reject-unsupported` | 18 |
| `oracle:timeout / candidate:reject-unsupported` | 1 |

## Boundary

- **no NIST certification** — CCVS85 is a historical validation corpus; this report is not a
certification result and carries no NIST or GSA authority.
- **no full COBOL-85 conformance claim** — a unit the oracle compiles+runs and the candidate
matches does not imply full language conformance.
- **no full `cobc` replacement claim** — `cobrun` is a sealed-subset interpreter over the ported
runtime.
- **no native-code-generation comparison** — `cobc` emits C + native code; `cobrun` interprets;
observable stdout/report bytes are compared, not codegen.
- **no claim that an oracle rejection proves the source invalid** under every COBOL
implementation — rejection is specific to the pinned GnuCOBOL 3.2 oracle and its dialect.
- **no claim that matching output proves equivalence** outside the tested environment.
- **no claim that library/data units are executable tests** — CLBRY and DATA* units are
classified as non-executable support units.
- **no conversion of blocked units into passes** — HARNESS_BLOCKED / DEPENDENCY_BLOCKED /
INFRASTRUCTURE_ERROR units are never counted as passes.

## Environment

```json
{
  "crate_version": "0.8.50",
  "environment": {
    "LANG": "C.UTF-8",
    "LC_ALL": "C.UTF-8",
    "SOURCE_DATE_EPOCH": "725846400",
    "TZ": "UTC0",
    "libc": "ldd (Ubuntu GLIBC 2.39-0ubuntu8.7) 2.39",
    "uname": "Linux 7.1.4-1-cachyos x86_64"
  },
  "generated_at": "2026-08-03T14:01:30Z",
  "git_commit": "183e21a48b2917b128e2bbb86022ced7f2e1735f",
  "oracle": {
    "built_prefix": "/work/oracle/prefix",
    "cobc_bin_sha256": "98dd2b1081a22cb6c70d2ac30e3c9a6138c8b63fbd25e17a5ed5274a99027a4a",
    "cobc_version": "cobc (GnuCOBOL) 3.2.0",
    "cobcrun_version": "cobcrun (GnuCOBOL) 3.2.0",
    "libcob_sha256": "482b2a5da87a815dec1a7898b8a153e48b93d169010659493fe18cc8673e65d0",
    "source_sha256": "8ecc77d0a4c9401618b8b99adf2050adef14767916767c54bb42341f0ab504fb"
  }
}
```
