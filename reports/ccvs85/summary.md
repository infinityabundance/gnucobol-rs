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
| candidate accepted | 78 |
| candidate unsupported | 273 |
| candidate parse/layout/boundary reject | 40 |
| candidate runtime fail | 0 |
| candidate timeout | 0 |
| raw output match | 28 |
| canonical output match | 0 |
| output mismatch | 41 |
| exit-status mismatch | 0 |
| generated-file mismatch | 9 |
| harness-blocked | 1 |
| dependency-blocked | 0 |
| infrastructure error | 0 |
| nondeterministic (explicitly classified) | 1 |

## By primary classification

| classification | count |
|---|---|
| `GENERATED_FILE_MISMATCH` | 9 |
| `HARNESS_BLOCKED` | 1 |
| `NON_EXECUTABLE_DATA` | 2 |
| `NON_EXECUTABLE_LIBRARY` | 119 |
| `ORACLE_COMPILE_ERROR` | 3 |
| `ORACLE_COMPILE_REJECT` | 18 |
| `ORACLE_RUN_FAIL` | 64 |
| `ORACLE_TIMEOUT` | 1 |
| `OUTPUT_MISMATCH` | 41 |
| `RAW_OUTPUT_MATCH` | 28 |
| `RUST_REJECT_PARSE` | 36 |
| `RUST_REJECT_RUNTIME_BOUNDARY` | 1 |
| `RUST_REJECT_UNSUPPORTED` | 189 |

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
| `SUBPROGRAM_BOUND_TO_MAIN` | 68 |
| `ORACLE_RUN_NONZERO_EXIT` | 64 |
| `LIBRARY_TEXT_UNIT` | 51 |
| `OUTPUT_BYTES_DIFFER` | 41 |
| `COBRUN_UNSUPPORTED_NOT_NUMERIC_LITERAL_FILE_RECORD_INFO` | 35 |
| `COBRUN_UNSUPPORTED_WITHOUT_TARGET_PARAGRAPH` | 33 |
| `RAW_OUTPUT_IDENTICAL` | 28 |
| `COBRUN_UNSUPPORTED_CONDITION_UNRECOGNIZED_RELATIONAL_OPERATOR_EXPECTED_GREATER` | 11 |
| `COBRUN_UNDEFINED_DATA_NAME_IDX` | 9 |
| `GENERATED_FILES_DIFFER` | 9 |
| `COBRUN_UNDEFINED_DATA_NAME_XRECORD_NUMBER` | 7 |
| `COBRUN_UNSUPPORTED_NOT_NUMERIC_LITERAL_FUNCTION` | 6 |
| `COBRUN_UNSUPPORTED_NOT_NUMERIC_LITERAL` | 5 |
| `COBRUN_UNSUPPORTED_UNRECOGNIZED_USAGE_COMPUTATIONAL` | 5 |
| `COBRUN_UNSUPPORTED_UNSUPPORTED_LEVEL_NUMBER` | 5 |
| `COBRUN_UNSUPPORTED_NOT_NUMERIC_LITERAL_LINE_COUNTER` | 4 |
| `COBRUN_UNSUPPORTED_NOT_NUMERIC_LITERAL_ZERO` | 4 |
| `COBRUN_UNSUPPORTED_VERB_COPY` | 4 |
| `COBC_CRASHED` | 3 |
| `COBRUN_UNSUPPORTED_NOT_NUMERIC_LITERAL_ALL` | 3 |
| `COBRUN_UNSUPPORTED_PIC_ABABX0A_UNSUPPORTEDSYMBOL` | 3 |
| `COBRUN_UNSUPPORTED_UNSUPPORTED_LEVEL_NUMBER_COPY` | 3 |
| `COBRUN_UNSUPPORTED_VERB_SECT_001` | 3 |
| `COBRUN_UNSUPPORTED_VERB_SORT_PARA` | 3 |
| `COBRUN_UNDEFINED_DATA_NAME_DN1` | 2 |
| `COBRUN_UNDEFINED_DATA_NAME_DN2` | 2 |
| `COBRUN_UNDEFINED_DATA_NAME_ENTRY` | 2 |
| `COBRUN_UNDEFINED_DATA_NAME_TABLE2_NUM_INDEX2` | 2 |
| `COBRUN_UNSUPPORTED_CALL_ID1_NOT_CONTAINED_PROGRAM_EXTERNAL` | 2 |
| `COBRUN_UNSUPPORTED_FUNCTION_RANDOM_COBC_DOES_NOT_IMPLEMENT` | 2 |
| `COBRUN_UNSUPPORTED_OPEN_PRINT_FILE_NOT_DECLARED_FILE` | 2 |
| `COBRUN_UNSUPPORTED_PIC_XXBXXBXX_UNSUPPORTEDSYMBOL` | 2 |
| `COBRUN_UNSUPPORTED_SET_TABLE2_REC_INDEX2_NOT_INTEGER` | 2 |
| `COBRUN_UNSUPPORTED_SORT_MERGE_KEY_KEY_NOT_FIELD` | 2 |
| `COBRUN_UNSUPPORTED_SORT_MERGE_KEY_NOT_FIELD_THE` | 2 |
| `COBRUN_UNSUPPORTED_SUBSCRIPT_INDEX1_NOT_INTEGER` | 2 |
| `COBRUN_UNSUPPORTED_TRAILING_TOKENS_CONDITION` | 2 |
| `COBRUN_UNSUPPORTED_VERB_NUMBER1` | 2 |
| `DATA_UNIT` | 2 |
| `COBRUN_RUNTIME_ERROR_PERFORM_VARYING_EXCEEDED_1E6_ITERATIONS` | 1 |

## Oracle × candidate outcome pairs

| pair | count |
|---|---|
| `oracle: / candidate:` | 53 |
| `oracle:bound-to-main / candidate:bound-to-main` | 68 |
| `oracle:compile-pass / candidate:reject-unsupported` | 1 |
| `oracle:error / candidate:reject-unsupported` | 3 |
| `oracle:fail / candidate:reject-parse` | 1 |
| `oracle:fail / candidate:reject-runtime-boundary` | 2 |
| `oracle:fail / candidate:reject-unsupported` | 61 |
| `oracle:pass / candidate:reject-parse` | 36 |
| `oracle:pass / candidate:reject-runtime-boundary` | 1 |
| `oracle:pass / candidate:reject-unsupported` | 189 |
| `oracle:pass / candidate:run-pass` | 78 |
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
  "meta": {
    "crate_version": "0.8.56",
    "environment": {
      "LANG": "C.UTF-8",
      "LC_ALL": "C.UTF-8",
      "SOURCE_DATE_EPOCH": "725846400",
      "TZ": "UTC0",
      "libc": "ldd (Ubuntu GLIBC 2.39-0ubuntu8.7) 2.39",
      "uname": "Linux 7.1.6-1-cachyos x86_64"
    },
    "generated_at": "2026-08-12T01:21:39Z",
    "git_commit": "8980273cf6e75d7efaefaf32627586ce6d16fe78",
    "oracle": {
      "built_prefix": "/work/oracle/prefix",
      "cobc_bin_sha256": "b7b7f4915e3e9598c52bccf81002efd2adf31d1ded6ffe8a771e3f1738ac521f",
      "cobc_version": "cobc (GnuCOBOL) 3.2.0",
      "cobcrun_version": "cobcrun (GnuCOBOL) 3.2.0",
      "libcob_sha256": "dbd54e7be7e3f3e0e1475f911800dabbfeb74fbe57946586980eeabac606b985",
      "source_sha256": "8ecc77d0a4c9401618b8b99adf2050adef14767916767c54bb42341f0ab504fb"
    }
  },
  "schema": "gnurust-ccvs85-summary-v1",
  "summary": {
    "by_final_classification": {
      "GENERATED_FILE_MISMATCH": 9,
      "HARNESS_BLOCKED": 1,
      "NON_EXECUTABLE_DATA": 2,
      "NON_EXECUTABLE_LIBRARY": 119,
      "ORACLE_COMPILE_ERROR": 3,
      "ORACLE_COMPILE_REJECT": 18,
      "ORACLE_RUN_FAIL": 64,
      "ORACLE_TIMEOUT": 1,
      "OUTPUT_MISMATCH": 41,
      "RAW_OUTPUT_MATCH": 28,
      "RUST_REJECT_PARSE": 36,
      "RUST_REJECT_RUNTIME_BOUNDARY": 1,
      "RUST_REJECT_UNSUPPORTED": 189
    },
    "by_reason_code": {
      "COBC_CRASHED": 3,
      "COBRUN_RUNTIME_ERROR_PERFORM_VARYING_EXCEEDED_1E6_ITERATIONS": 1,
      "COBRUN_UNDEFINED_DATA_NAME_DN1": 2,
      "COBRUN_UNDEFINED_DATA_NAME_DN2": 2,
      "COBRUN_UNDEFINED_DATA_NAME_ENTRY": 2,
      "COBRUN_UNDEFINED_DATA_NAME_FIELD": 1,
      "COBRUN_UNDEFINED_DATA_NAME_FILE_RECORD_INFO_120": 1,
      "COBRUN_UNDEFINED_DATA_NAME_GLO_DATA": 1,
      "COBRUN_UNDEFINED_DATA_NAME_GRP_FOR_ADD_CORR": 1,
      "COBRUN_UNDEFINED_DATA_NAME_IDX": 9,
      "COBRUN_UNDEFINED_DATA_NAME_IN3": 1,
      "COBRUN_UNDEFINED_DATA_NAME_INXEX2": 1,
      "COBRUN_UNDEFINED_DATA_NAME_OVERFLOW": 1,
      "COBRUN_UNDEFINED_DATA_NAME_RECORD2_INDEX2": 1,
      "COBRUN_UNDEFINED_DATA_NAME_REM_25ANS": 1,
      "COBRUN_UNDEFINED_DATA_NAME_SUBSCRIPTED_DATA": 1,
      "COBRUN_UNDEFINED_DATA_NAME_TABLE2_NUM_INDEX2": 2,
      "COBRUN_UNDEFINED_DATA_NAME_TEST1": 1,
      "COBRUN_UNDEFINED_DATA_NAME_XRECORD_LENGTH": 1,
      "COBRUN_UNDEFINED_DATA_NAME_XRECORD_NUMBER": 7,
      "COBRUN_UNSUPPORTED_ACCEPT_FROM_TERMINAL_CONSOLE_INTERACTIVE_INPUT": 1,
      "COBRUN_UNSUPPORTED_CALL_ID1_NOT_CONTAINED_PROGRAM_EXTERNAL": 2,
      "COBRUN_UNSUPPORTED_CLOSE_REEL_NOT_DECLARED_FILE": 1,
      "COBRUN_UNSUPPORTED_CONDITION_UNRECOGNIZED_RELATIONAL_OPERATOR_EXPECTED_GREATER": 11,
      "COBRUN_UNSUPPORTED_DISABLE_GNUCOBOL_DOES_NOT_IMPLEMENT_THE": 1,
      "COBRUN_UNSUPPORTED_DIVIDE_REMAINDER_RECEIVER_0009_MUST_NUMERIC": 1,
      "COBRUN_UNSUPPORTED_FUNCTION_RANDOM_COBC_DOES_NOT_IMPLEMENT": 2,
      "COBRUN_UNSUPPORTED_INSPECT_REGION_CLAUSE_NEAR_WORD_WRK": 1,
      "COBRUN_UNSUPPORTED_MOVE_WITHOUT": 1,
      "COBRUN_UNSUPPORTED_MULTI_DIMENSION_LEAF_NEEDS_SUBSCRIPT_GOT": 1,
      "COBRUN_UNSUPPORTED_NOT_NUMERIC_LITERAL": 5,
      "COBRUN_UNSUPPORTED_NOT_NUMERIC_LITERAL_12345678": 1,
      "COBRUN_UNSUPPORTED_NOT_NUMERIC_LITERAL_ALL": 3,
      "COBRUN_UNSUPPORTED_NOT_NUMERIC_LITERAL_BORTED": 1,
      "COBRUN_UNSUPPORTED_NOT_NUMERIC_LITERAL_FILE_RECORD_INFO": 35,
      "COBRUN_UNSUPPORTED_NOT_NUMERIC_LITERAL_FUNCTION": 6,
      "COBRUN_UNSUPPORTED_NOT_NUMERIC_LITERAL_LINE_COUNTER": 4,
      "COBRUN_UNSUPPORTED_NOT_NUMERIC_LITERAL_S22": 1,
      "COBRUN_UNSUPPORTED_NOT_NUMERIC_LITERAL_WRK_2V1": 1,
      "COBRUN_UNSUPPORTED_NOT_NUMERIC_LITERAL_XTAB": 1,
      "COBRUN_UNSUPPORTED_NOT_NUMERIC_LITERAL_YEAR": 1,
      "COBRUN_UNSUPPORTED_NOT_NUMERIC_LITERAL_ZERO": 4,
      "COBRUN_UNSUPPORTED_OPEN_FS1_NOT_DECLARED_FILE": 1,
      "COBRUN_UNSUPPORTED_OPEN_PRINT_FILE_NOT_DECLARED_FILE": 2,
      "COBRUN_UNSUPPORTED_OPEN_TEST_FILE_NOT_DECLARED_FILE": 1,
      "COBRUN_UNSUPPORTED_PIC_000000000000000020_BADREPEAT": 1,
      "COBRUN_UNSUPPORTED_PIC_ABABX0A_UNSUPPORTEDSYMBOL": 3,
      "COBRUN_UNSUPPORTED_PIC_ABA_UNSUPPORTEDSYMBOL": 1,
      "COBRUN_UNSUPPORTED_PIC_BADREPEAT": 1,
      "COBRUN_UNSUPPORTED_PIC_UNSUPPORTEDSYMBOL": 1,
      "COBRUN_UNSUPPORTED_PIC_XBX0XBX0X_UNSUPPORTEDSYMBOL": 1,
      "COBRUN_UNSUPPORTED_PIC_XBXBXBX_UNSUPPORTEDSYMBOL": 1,
      "COBRUN_UNSUPPORTED_PIC_XXBXXBXX_UNSUPPORTEDSYMBOL": 2,
      "COBRUN_UNSUPPORTED_PIC_XXBXX_UNSUPPORTEDSYMBOL": 1,
      "COBRUN_UNSUPPORTED_PROCEDURE_DIVISION": 1,
      "COBRUN_UNSUPPORTED_SET_TABLE2_REC_INDEX2_NOT_INTEGER": 2,
      "COBRUN_UNSUPPORTED_SORT_MERGE_KEY_KEY1_NOT_FIELD": 1,
      "COBRUN_UNSUPPORTED_SORT_MERGE_KEY_KEY_NOT_FIELD": 2,
      "COBRUN_UNSUPPORTED_SORT_MERGE_KEY_NOT_FIELD_THE": 2,
      "COBRUN_UNSUPPORTED_SORT_MERGE_KEY_SORTKEY_NOT_FIELD": 1,
      "COBRUN_UNSUPPORTED_SUBSCRIPT_IN1_NOT_INTEGER": 1,
      "COBRUN_UNSUPPORTED_SUBSCRIPT_INDEX1_NOT_INTEGER": 2,
      "COBRUN_UNSUPPORTED_SUBSCRIPT_INDEX2_NOT_INTEGER": 1,
      "COBRUN_UNSUPPORTED_SUBSCRIPT_NOT_INTEGER": 1,
      "COBRUN_UNSUPPORTED_SUBSCRIPT_SUB_NOT_INTEGER": 1,
      "COBRUN_UNSUPPORTED_TRAILING_TOKENS_CONDITION": 2,
      "COBRUN_UNSUPPORTED_TRAILING_TOKENS_CONDITION_NEXT": 1,
      "COBRUN_UNSUPPORTED_UNRECOGNIZED_USAGE_COMPUTATIONAL": 5,
      "COBRUN_UNSUPPORTED_UNSUPPORTED_LEVEL_NUMBER": 5,
      "COBRUN_UNSUPPORTED_UNSUPPORTED_LEVEL_NUMBER_14003": 1,
      "COBRUN_UNSUPPORTED_UNSUPPORTED_LEVEL_NUMBER_COMMUNICATION": 1,
      "COBRUN_UNSUPPORTED_UNSUPPORTED_LEVEL_NUMBER_COPY": 3,
      "COBRUN_UNSUPPORTED_UNSUPPORTED_LEVEL_NUMBER_FEATURE": 1,
      "COBRUN_UNSUPPORTED_UNSUPPORTED_LEVEL_NUMBER_REPLACE": 1,
      "COBRUN_UNSUPPORTED_UNSUPPORTED_LEVEL_NUMBER_THE": 1,
      "COBRUN_UNSUPPORTED_UNSUPPORTED_LEVEL_NUMBER_THIS": 1,
      "COBRUN_UNSUPPORTED_VERB": 1,
      "COBRUN_UNSUPPORTED_VERB_BEANO": 1,
      "COBRUN_UNSUPPORTED_VERB_COPY": 4,
      "COBRUN_UNSUPPORTED_VERB_HOUSEKEEPING": 1,
      "COBRUN_UNSUPPORTED_VERB_NUMBER1": 2,
      "COBRUN_UNSUPPORTED_VERB_SEC20": 1,
      "COBRUN_UNSUPPORTED_VERB_SEC40": 1,
      "COBRUN_UNSUPPORTED_VERB_SECT_001": 3,
      "COBRUN_UNSUPPORTED_VERB_SECT_IC219_0001": 1,
      "COBRUN_UNSUPPORTED_VERB_SORT_PARA": 3,
      "COBRUN_UNSUPPORTED_WITHOUT_TARGET_PARAGRAPH": 33,
      "DATA_UNIT": 2,
      "EXEC85_DRIVER_REQUIRES_MODULE_LIBRARY": 1,
      "GENERATED_FILES_DIFFER": 9,
      "LIBRARY_TEXT_UNIT": 51,
      "ORACLE_RUN_NONZERO_EXIT": 64,
      "ORACLE_RUN_TIMEOUT": 1,
      "OUTPUT_BYTES_DIFFER": 41,
      "RAW_OUTPUT_IDENTICAL": 28,
      "SUBPROGRAM_BOUND_TO_MAIN": 68,
      "WORK_RUN_MATERIALIZED_ADAPTED_CM101M_COB_231_ERROR": 1,
      "WORK_RUN_MATERIALIZED_ADAPTED_CM102M_COB_279_ERROR": 1,
      "WORK_RUN_MATERIALIZED_ADAPTED_CM103M_COB_182_ERROR": 1,
      "WORK_RUN_MATERIALIZED_ADAPTED_CM105M_COB_176_ERROR": 1,
      "WORK_RUN_MATERIALIZED_ADAPTED_CM401M_COB_ERROR_NOT": 1,
      "WORK_RUN_MATERIALIZED_ADAPTED_IF119A_COB_599_ERROR": 1,
      "WORK_RUN_MATERIALIZED_ADAPTED_IF120A_COB_490_ERROR": 1,
      "WORK_RUN_MATERIALIZED_ADAPTED_IF121A_COB_487_ERROR": 1,
      "WORK_RUN_MATERIALIZED_ADAPTED_IF122A_COB_509_ERROR": 1,
      "WORK_RUN_MATERIALIZED_ADAPTED_IF123A_COB_602_ERROR": 1,
      "WORK_RUN_MATERIALIZED_ADAPTED_IF128A_COB_536_ERROR": 1,
      "WORK_RUN_MATERIALIZED_ADAPTED_IF129A_COB_539_ERROR": 1,
      "WORK_RUN_MATERIALIZED_ADAPTED_IF132A_COB_485_ERROR": 1,
      "WORK_RUN_MATERIALIZED_ADAPTED_IF137A_COB_518_ERROR": 1,
      "WORK_RUN_MATERIALIZED_ADAPTED_IF138A_COB_485_ERROR": 1,
      "WORK_RUN_MATERIALIZED_ADAPTED_IF141A_COB_512_ERROR": 1,
      "WORK_RUN_MATERIALIZED_ADAPTED_IF402M_COB_ERROR_SYNTAX": 1,
      "WORK_RUN_MATERIALIZED_ADAPTED_IX110A_COB_121_ERROR": 1
    },
    "by_section": {
      "AL": 2,
      "CM": 9,
      "DB": 15,
      "EX": 1,
      "IC": 47,
      "IF": 45,
      "IX": 42,
      "K": 36,
      "KK": 1,
      "KP": 10,
      "KS": 2,
      "NC": 97,
      "OB": 9,
      "RL": 35,
      "RW": 6,
      "SG": 13,
      "SM": 17,
      "SQ": 85,
      "ST": 40
    },
    "candidate_accepted": 78,
    "candidate_parse_fail": 40,
    "candidate_runtime_fail": 0,
    "candidate_timeout": 0,
    "candidate_unsupported": 273,
    "canonical_output_match": 0,
    "dependency_blocked": 0,
    "executable_candidates": 391,
    "exit_status_mismatch": 0,
    "generated_file_mismatch": 9,
    "harness_blocked": 1,
    "infrastructure_error": 0,
    "non_executable_data": 2,
    "non_executable_library": 119,
    "nondeterministic": 0,
    "oracle_candidate_pair": {
      "oracle: / candidate:": 53,
      "oracle:bound-to-main / candidate:bound-to-main": 68,
      "oracle:compile-pass / candidate:reject-unsupported": 1,
      "oracle:error / candidate:reject-unsupported": 3,
      "oracle:fail / candidate:reject-parse": 1,
      "oracle:fail / candidate:reject-runtime-boundary": 2,
      "oracle:fail / candidate:reject-unsupported": 61,
      "oracle:pass / candidate:reject-parse": 36,
      "oracle:pass / candidate:reject-runtime-boundary": 1,
      "oracle:pass / candidate:reject-unsupported": 189,
      "oracle:pass / candidate:run-pass": 78,
      "oracle:reject / candidate:reject-unsupported": 18,
      "oracle:timeout / candidate:reject-unsupported": 1
    },
    "oracle_compile_error": 3,
    "oracle_compile_pass": 370,
    "oracle_compile_reject": 18,
    "oracle_run_fail": 64,
    "oracle_run_pass": 304,
    "oracle_timeout": 1,
    "output_mismatch": 41,
    "raw_output_match": 28,
    "units_by_kind": {
      "CLBRY": 51,
      "COBOL": 459,
      "DATA*": 2
    },
    "units_total": 512
  }
}
```
