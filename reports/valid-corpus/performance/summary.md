# Performance corpus (Phase 8)

Correctness-gated: every workload at every scale is byte-exact against the host
GnuCOBOL 3.2.0 oracle BEFORE any timing is reported (spec 8.3). Inputs are
deterministic (seeded generators, integer-exact); expected outputs are computed
independently in Rust -- never by the candidate.

| workload | scale | records | compile ms | run ms | byte-exact |
|---|---|---|---|---|---|
| float | small | 500 | 60 | 1 | true |
| float | medium | 5000 | 60 | 5 | true |
| float | large | 50000 | 61 | 38 | true |
| float | stress | 500000 | 61 | 365 | true |
| invoice | small | 500 | 65 | 2 | true |
| invoice | medium | 5000 | 65 | 5 | true |
| invoice | large | 50000 | 64 | 37 | true |
| invoice | stress | 500000 | 66 | 354 | true |
| mixed | small | 500 | 99 | 2 | true |
| mixed | medium | 5000 | 95 | 5 | true |
| mixed | large | 50000 | 103 | 39 | true |
| mixed | stress | 500000 | 100 | 372 | true |
| modules | small | 500 | 82 | 1 | true |
| modules | medium | 5000 | 78 | 3 | true |
| modules | large | 50000 | 81 | 17 | true |
| modules | stress | 500000 | 81 | 162 | true |
| payroll | small | 500 | 79 | 2 | true |
| payroll | medium | 5000 | 76 | 5 | true |
| payroll | large | 50000 | 78 | 41 | true |
| payroll | stress | 500000 | 77 | 398 | true |
| relative | small | 500 | 71 | 2 | true |
| relative | medium | 5000 | 69 | 7 | true |
| relative | large | 50000 | 70 | 56 | true |
| relative | stress | 500000 | 72 | 541 | true |
| report | small | 500 | 63 | 1 | true |
| report | medium | 5000 | 62 | 2 | true |
| report | large | 50000 | 63 | 13 | true |
| report | stress | 500000 | 65 | 124 | true |
| seqfile | small | 500 | 66 | 1 | true |
| seqfile | medium | 5000 | 63 | 2 | true |
| seqfile | large | 50000 | 66 | 14 | true |
| seqfile | stress | 500000 | 67 | 133 | true |
| strings | small | 500 | 60 | 1 | true |
| strings | medium | 5000 | 59 | 3 | true |
| strings | large | 50000 | 61 | 20 | true |
| strings | stress | 500000 | 61 | 181 | true |
| tables | small | 500 | 84 | 2 | true |
| tables | medium | 5000 | 83 | 4 | true |
| tables | large | 50000 | 85 | 32 | true |
| tables | stress | 500000 | 84 | 335 | true |
## Phase 9 — performance views

Control: host_cpu=AMD Ryzen 7 9800X3D 8-Core Processor · cobc_version=cobc (GnuCOBOL) 3.2.0 · compiler_flags=`cobc -x -O2 -o <artifact> <sources...>` · iters=3 · warmup=1 · outlier_policy: min = best-case, median = typical, no samples discarded · candidate_compat=prepared-v1 · generated_at_utc=2026-08-12T00:51:57Z

### View A — end-to-end one-shot (compile+run vs prepare+run)

**unlike workflows**: the native lane is a compiled binary; the candidate lane
is interpreted — these are NOT equivalent runtime work.

| workload | scale | oracle total ms (median) | candidate total ms (median) | ratio |
|---|---|---|---|---|
| payroll | small | 77.16 | 10.24 | 7.5x |
| invoice | small | 64.81 | 8.42 | 7.7x |
| seqfile | small | 64.62 | 4.89 | 13.2x |
| tables | small | 84.86 | 2089.04 | 0.0x |
| strings | small | 61.11 | 6.69 | 9.1x |
| float | small | 62.37 | 6.88 | 9.1x |
| report | small | 64.01 | 5.26 | 12.2x |
| relative | small | 72.26 | 17.81 | 4.1x |
| modules | small | 80.46 | 9.45 | 8.5x |
| mixed | small | 98.27 | 16.09 | 6.1x |

### View B — front-end only (oracle compile vs candidate per-phase prepare)

| workload | scale | oracle compile ms (median) | preprocess | lex | parse | resolution | layout | check | prepare | bytes/sec | lines/sec |
|---|---|---|---|---|---|---|---|---|---|---|---|
| payroll | small | 75.79 | 0.007 | 0.060 | 0.068 | 0.005 | 0.027 | 0.028 | 0.200 | 19753580 | 509834 |
| invoice | small | 64.09 | 0.006 | 0.046 | 0.053 | 0.005 | 0.022 | 0.023 | 0.159 | 19278892 | 470370 |
| seqfile | small | 64.07 | 0.006 | 0.048 | 0.051 | 0.005 | 0.022 | 0.023 | 0.161 | 18741725 | 491077 |
| tables | small | 79.88 | 0.007 | 0.056 | 0.070 | 0.005 | 0.271 | 0.548 | 1.263 | 3393020 | 88706 |
| strings | small | 59.23 | 0.005 | 0.039 | 0.047 | 0.004 | 0.016 | 0.021 | 0.139 | 17520692 | 425750 |
| float | small | 61.03 | 0.005 | 0.033 | 0.045 | 0.005 | 0.017 | 0.019 | 0.129 | 15050388 | 404037 |
| report | small | 62.12 | 0.006 | 0.039 | 0.051 | 0.005 | 0.020 | 0.024 | 0.151 | 19007903 | 483982 |
| relative | small | 70.05 | 0.008 | 0.069 | 0.069 | 0.005 | 0.024 | 0.036 | 0.217 | 23093275 | 602753 |
| modules | small | 77.87 | 0.006 | 0.038 | 0.053 | 0.005 | 0.015 | 0.024 | 0.146 | 16370648 | 438377 |
| mixed | small | 95.66 | 0.008 | 0.065 | 0.075 | 0.005 | 0.022 | 0.032 | 0.213 | 20726972 | 535921 |

### View C — repeated execution (compiled binary vs prepared program, no reparse)

| workload | scale | native median (min/p95) | candidate median (min/p95) | candidate prepare ms | outputs agree |
|---|---|---|---|---|---|
| payroll | small | 1.34 (min 1.34, IQR 0.12, p95 1.47) | 9.80 (min 9.70, IQR 0.13, p95 9.83) | 0 | true |
| invoice | small | 1.45 (min 1.37, IQR 0.19, p95 1.56) | 7.89 (min 7.89, IQR 0.02, p95 7.90) | 0 | true |
| seqfile | small | 1.11 (min 1.11, IQR 0.03, p95 1.13) | 4.42 (min 4.40, IQR 0.16, p95 4.56) | 0 | true |
| tables | small | 1.85 (min 1.67, IQR 0.26, p95 1.92) | 2103.53 (min 2097.59, IQR 15.12, p95 2112.72) | 2 | true |
| strings | small | 1.16 (min 1.13, IQR 0.10, p95 1.23) | 6.42 (min 6.40, IQR 0.06, p95 6.45) | 0 | true |
| float | small | 1.31 (min 1.26, IQR 0.09, p95 1.36) | 6.53 (min 6.50, IQR 0.08, p95 6.58) | 0 | true |
| report | small | 1.09 (min 1.04, IQR 0.06, p95 1.10) | 4.90 (min 4.90, IQR 0.13, p95 5.03) | 0 | true |
| relative | small | 1.62 (min 1.55, IQR 0.11, p95 1.66) | 17.41 (min 17.38, IQR 0.04, p95 17.42) | 0 | true |
| modules | small | 1.15 (min 1.09, IQR 0.11, p95 1.20) | 9.23 (min 9.19, IQR 0.04, p95 9.23) | 0 | true |
| mixed | small | 1.41 (min 1.37, IQR 0.07, p95 1.44) | 15.74 (min 15.62, IQR 0.15, p95 15.76) | 0 | true |

### View D — runtime-operation microbenchmarks (50_000 iterations, correctness-gated)

| op | native median (min/p95) ms | candidate median (min/p95) ms | candidate prepare ms | byte-exact |
|---|---|---|---|---|
| move (decimal MOVE (display -> display)) | 3.19 (min 3.12, IQR 0.13, p95 3.25) | 78.09 (min 78.04, IQR 0.07, p95 78.11) | 0 | true |
| packed_add (packed-decimal ADD (COMP-3 accumulator)) | 3.33 (min 3.27, IQR 0.83, p95 4.10) | 95.32 (min 95.19, IQR 0.43, p95 95.62) | 0 | true |
| binary_add (binary ADD (COMP accumulator)) | 3.51 (min 3.44, IQR 0.08, p95 3.52) | 87.18 (min 87.14, IQR 0.13, p95 87.27) | 0 | true |
| float_add (float ADD (COMP-1 f32 + COMP-2 f64)) | 21.91 (min 21.48, IQR 0.50, p95 21.98) | 1708.26 (min 1694.20, IQR 16.95, p95 1711.14) | 0 | true |
| compare (alphanumeric comparison (IF A = B)) | 3.67 (min 3.56, IQR 0.13, p95 3.69) | 171.88 (min 170.37, IQR 1.53, p95 171.90) | 0 | true |
| intrinsic (FUNCTION intrinsic dispatch (NUMVAL + INTEGER)) | 12.71 (min 12.66, IQR 0.22, p95 12.88) | 295.62 (min 295.17, IQR 0.50, p95 295.67) | 0 | true |
| call (module CALL dispatch (contained subprogram)) | 7.05 (min 6.96, IQR 0.16, p95 7.12) | 195.29 (min 194.54, IQR 1.46, p95 195.99) | 0 | true |
| seqfile (sequential-file read/write (50_000 fixed records)) | 20.24 (min 19.94, IQR 0.39, p95 20.33) | 266.08 (min 265.86, IQR 0.62, p95 266.48) | 0 | true |

### View E — corpus throughput (10 workloads x 4 scales, one pass)

- oracle (compile+run) total: 6497.0 ms
- candidate (prepare+run) total: 779984.4 ms
- peak memory: 195488 kB (VmHWM of the bench process (candidate runs in-process; oracle cobc/run are child processes, not included))

| workload | oracle total ms | candidate total ms |
|---|---|---|
| payroll | 769.7 | 10821.4 |
| invoice | 663.5 | 8787.7 |
| seqfile | 415.7 | 4802.5 |
| tables | 709.7 | 687856.6 |
| strings | 485.0 | 7189.9 |
| float | 657.5 | 7248.8 |
| report | 416.9 | 5493.3 |
| relative | 988.2 | 19928.3 |
| modules | 548.2 | 10537.5 |
| mixed | 842.7 | 17318.4 |

