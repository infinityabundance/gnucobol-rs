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

Control: host_cpu=AMD Ryzen 7 9800X3D 8-Core Processor · cobc_version=cobc (GnuCOBOL) 3.2.0 · compiler_flags=`cobc -x -O2 -o <artifact> <sources...>` · iters=3 · warmup=1 · outlier_policy: min = best-case, median = typical, no samples discarded · candidate_compat=prepared-v1 · generated_at_utc=2026-08-12T14:48:19Z

### View A — end-to-end one-shot (compile+run vs prepare+run)

**unlike workflows**: the native lane is a compiled binary; the candidate lane
is interpreted — these are NOT equivalent runtime work.

| workload | scale | oracle total ms (median) | candidate total ms (median) | ratio |
|---|---|---|---|---|
| payroll | small | 73.70 | 10.15 | 7.3x |
| invoice | small | 63.18 | 8.25 | 7.7x |
| seqfile | small | 61.94 | 4.73 | 13.1x |
| tables | small | 81.51 | 2718.29 | 0.0x |
| strings | small | 57.96 | 6.61 | 8.8x |
| float | small | 59.98 | 6.87 | 8.7x |
| report | small | 62.20 | 5.29 | 11.8x |
| relative | small | 70.40 | 17.75 | 4.0x |
| modules | small | 83.52 | 9.70 | 8.6x |
| mixed | small | 94.21 | 16.39 | 5.7x |

### View B — front-end only (oracle compile vs candidate per-phase prepare)

| workload | scale | oracle compile ms (median) | preprocess | lex | parse | resolution | layout | check | prepare | bytes/sec | lines/sec |
|---|---|---|---|---|---|---|---|---|---|---|---|
| payroll | small | 71.97 | 0.007 | 0.060 | 0.063 | 0.006 | 0.028 | 0.026 | 0.196 | 20206049 | 521512 |
| invoice | small | 61.54 | 0.006 | 0.046 | 0.049 | 0.005 | 0.023 | 0.021 | 0.155 | 19814489 | 483437 |
| seqfile | small | 63.43 | 0.006 | 0.047 | 0.051 | 0.005 | 0.021 | 0.025 | 0.161 | 18753499 | 491385 |
| tables | small | 78.68 | 0.007 | 0.056 | 0.071 | 0.005 | 0.229 | 0.550 | 1.012 | 4233921 | 110691 |
| strings | small | 57.58 | 0.005 | 0.038 | 0.047 | 0.005 | 0.016 | 0.020 | 0.135 | 17958314 | 436384 |
| float | small | 58.19 | 0.005 | 0.032 | 0.041 | 0.005 | 0.018 | 0.019 | 0.125 | 15478169 | 415521 |
| report | small | 60.42 | 0.006 | 0.038 | 0.049 | 0.005 | 0.021 | 0.024 | 0.149 | 19274338 | 490766 |
| relative | small | 67.35 | 0.009 | 0.070 | 0.065 | 0.005 | 0.024 | 0.029 | 0.209 | 24036205 | 627365 |
| modules | small | 75.99 | 0.005 | 0.038 | 0.046 | 0.005 | 0.015 | 0.024 | 0.140 | 17120344 | 458453 |
| mixed | small | 91.22 | 0.008 | 0.065 | 0.077 | 0.005 | 0.023 | 0.030 | 0.213 | 20746673 | 536430 |

### View C — repeated execution (compiled binary vs prepared program, no reparse)

| workload | scale | native median (min/p95) | candidate median (min/p95) | candidate prepare ms | outputs agree |
|---|---|---|---|---|---|
| payroll | small | 1.34 (min 1.31, IQR 0.08, p95 1.39) | 10.14 (min 10.00, IQR 0.33, p95 10.32) | 0 | true |
| invoice | small | 1.36 (min 1.27, IQR 0.10, p95 1.37) | 8.02 (min 8.01, IQR 0.09, p95 8.10) | 0 | true |
| seqfile | small | 1.08 (min 1.05, IQR 0.07, p95 1.12) | 4.53 (min 4.48, IQR 0.14, p95 4.62) | 0 | true |
| tables | small | 1.82 (min 1.65, IQR 0.57, p95 2.22) | 2145.13 (min 2143.90, IQR 14.23, p95 2158.13) | 2 | true |
| strings | small | 1.15 (min 1.14, IQR 0.04, p95 1.17) | 6.24 (min 6.20, IQR 0.04, p95 6.24) | 0 | true |
| float | small | 1.28 (min 1.27, IQR 0.08, p95 1.35) | 6.46 (min 6.43, IQR 0.05, p95 6.47) | 0 | true |
| report | small | 1.08 (min 1.07, IQR 0.15, p95 1.22) | 4.90 (min 4.90, IQR 0.02, p95 4.92) | 0 | true |
| relative | small | 1.58 (min 1.54, IQR 0.04, p95 1.58) | 17.36 (min 17.22, IQR 0.51, p95 17.73) | 0 | true |
| modules | small | 1.12 (min 1.10, IQR 0.08, p95 1.18) | 9.28 (min 9.19, IQR 0.27, p95 9.45) | 0 | true |
| mixed | small | 1.35 (min 1.31, IQR 0.03, p95 1.35) | 15.76 (min 15.71, IQR 0.04, p95 15.76) | 0 | true |

### View D — runtime-operation microbenchmarks (50_000 iterations, correctness-gated)

| op | native median (min/p95) ms | candidate median (min/p95) ms | candidate prepare ms | byte-exact |
|---|---|---|---|---|
| move (decimal MOVE (display -> display)) | 3.21 (min 3.12, IQR 0.11, p95 3.23) | 78.28 (min 78.12, IQR 0.24, p95 78.36) | 0 | true |
| packed_add (packed-decimal ADD (COMP-3 accumulator)) | 3.30 (min 3.22, IQR 0.09, p95 3.31) | 94.80 (min 94.12, IQR 1.20, p95 95.33) | 0 | true |
| binary_add (binary ADD (COMP accumulator)) | 3.52 (min 3.51, IQR 0.13, p95 3.64) | 87.10 (min 87.07, IQR 0.12, p95 87.19) | 0 | true |
| float_add (float ADD (COMP-1 f32 + COMP-2 f64)) | 21.49 (min 21.39, IQR 0.42, p95 21.81) | 1707.63 (min 1706.87, IQR 3.38, p95 1710.26) | 0 | true |
| compare (alphanumeric comparison (IF A = B)) | 3.54 (min 3.51, IQR 0.05, p95 3.56) | 168.13 (min 168.07, IQR 0.13, p95 168.19) | 0 | true |
| intrinsic (FUNCTION intrinsic dispatch (NUMVAL + INTEGER)) | 12.48 (min 12.44, IQR 0.23, p95 12.66) | 297.30 (min 295.47, IQR 11.27, p95 306.74) | 0 | true |
| call (module CALL dispatch (contained subprogram)) | 7.00 (min 6.98, IQR 0.16, p95 7.14) | 198.13 (min 197.71, IQR 0.77, p95 198.49) | 0 | true |
| seqfile (sequential-file read/write (50_000 fixed records)) | 20.13 (min 20.04, IQR 0.49, p95 20.53) | 266.02 (min 265.32, IQR 0.71, p95 266.03) | 0 | true |

### View E — corpus throughput (10 workloads x 4 scales, one pass)

- oracle (compile+run) total: 6308.3 ms
- candidate (prepare+run) total: 753471.5 ms
- peak memory: 194472 kB (VmHWM of the bench process (candidate runs in-process; oracle cobc/run are child processes, not included))

| workload | oracle total ms | candidate total ms |
|---|---|---|
| payroll | 759.9 | 10782.0 |
| invoice | 663.3 | 8739.2 |
| seqfile | 407.4 | 4785.6 |
| tables | 699.1 | 662768.1 |
| strings | 453.3 | 7020.6 |
| float | 636.9 | 7122.9 |
| report | 399.1 | 5397.1 |
| relative | 970.6 | 19557.9 |
| modules | 523.4 | 10386.7 |
| mixed | 795.2 | 16911.5 |

