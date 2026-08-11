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

Control: host_cpu=AMD Ryzen 7 9800X3D 8-Core Processor · cobc_version=cobc (GnuCOBOL) 3.2.0 · compiler_flags=`cobc -x -O2 -o <artifact> <sources...>` · iters=5 · warmup=1 · outlier_policy: min = best-case, median = typical, no samples discarded · candidate_compat=prepared-v1 · generated_at_utc=2026-08-11T16:05:41Z

### View A — end-to-end one-shot (compile+run vs prepare+run)

**unlike workflows**: the native lane is a compiled binary; the candidate lane
is interpreted — these are NOT equivalent runtime work.

| workload | scale | oracle total ms (median) | candidate total ms (median) | ratio |
|---|---|---|---|---|
| payroll | small | 72.19 | 71.47 | 1.0x |
| invoice | small | 61.10 | 59.64 | 1.0x |
| seqfile | small | 59.82 | 32.98 | 1.8x |
| tables | small | 83.65 | 2154.03 | 0.0x |
| strings | small | 56.40 | 48.33 | 1.2x |
| float | small | 57.12 | 41.38 | 1.4x |
| report | small | 60.05 | 37.00 | 1.6x |
| relative | small | 66.67 | 88.51 | 0.8x |
| modules | small | 75.40 | 54.36 | 1.4x |
| mixed | small | 93.43 | 98.26 | 1.0x |

### View B — front-end only (oracle compile vs candidate per-phase prepare)

| workload | scale | oracle compile ms (median) | preprocess | lex | parse | resolution | layout | check | prepare | bytes/sec | lines/sec |
|---|---|---|---|---|---|---|---|---|---|---|---|
| payroll | small | 69.43 | 0.039 | 0.325 | 0.201 | 0.012 | 0.091 | 0.105 | 0.792 | 4987449 | 128725 |
| invoice | small | 58.00 | 0.031 | 0.247 | 0.165 | 0.011 | 0.082 | 0.073 | 0.628 | 4897072 | 119480 |
| seqfile | small | 58.75 | 0.032 | 0.248 | 0.168 | 0.011 | 0.076 | 0.079 | 0.634 | 4754066 | 124568 |
| tables | small | 78.18 | 0.043 | 0.308 | 0.263 | 0.013 | 0.363 | 0.587 | 1.757 | 2438316 | 63747 |
| strings | small | 54.38 | 0.025 | 0.207 | 0.171 | 0.010 | 0.050 | 0.071 | 0.555 | 4375831 | 106332 |
| float | small | 55.37 | 0.024 | 0.167 | 0.140 | 0.010 | 0.060 | 0.053 | 0.474 | 4090260 | 109806 |
| report | small | 57.51 | 0.030 | 0.201 | 0.187 | 0.011 | 0.070 | 0.077 | 0.596 | 4813213 | 122555 |
| relative | small | 65.33 | 0.052 | 0.395 | 0.243 | 0.011 | 0.074 | 0.114 | 0.910 | 5514179 | 143925 |
| modules | small | 73.34 | 0.026 | 0.202 | 0.167 | 0.011 | 0.056 | 0.073 | 0.556 | 4301369 | 115183 |
| mixed | small | 90.16 | 0.043 | 0.388 | 0.258 | 0.012 | 0.091 | 0.117 | 0.930 | 4742104 | 122613 |

### View C — repeated execution (compiled binary vs prepared program, no reparse)

| workload | scale | native median (min/p95) | candidate median (min/p95) | candidate prepare ms | outputs agree |
|---|---|---|---|---|---|
| payroll | small | 1.52 (min 1.45, IQR 0.06, p95 1.59) | 70.09 (min 69.85, IQR 0.12, p95 73.27) | 1 | true |
| invoice | small | 1.43 (min 1.43, IQR 0.02, p95 1.48) | 58.39 (min 58.33, IQR 0.04, p95 61.11) | 1 | true |
| seqfile | small | 1.20 (min 1.18, IQR 0.06, p95 1.34) | 31.59 (min 31.54, IQR 0.38, p95 33.67) | 1 | true |
| tables | small | 1.76 (min 1.65, IQR 0.07, p95 1.78) | 2150.48 (min 2149.54, IQR 0.95, p95 2155.22) | 3 | true |
| strings | small | 1.26 (min 1.24, IQR 0.02, p95 1.31) | 47.03 (min 46.97, IQR 0.11, p95 47.14) | 0 | true |
| float | small | 1.45 (min 1.44, IQR 0.05, p95 1.52) | 39.95 (min 39.90, IQR 0.10, p95 40.16) | 0 | true |
| report | small | 1.21 (min 1.17, IQR 0.05, p95 1.24) | 35.64 (min 35.60, IQR 0.03, p95 35.65) | 0 | true |
| relative | small | 1.65 (min 1.64, IQR 0.05, p95 1.80) | 86.45 (min 86.34, IQR 0.06, p95 86.59) | 1 | true |
| modules | small | 1.23 (min 1.22, IQR 0.05, p95 1.31) | 53.16 (min 52.95, IQR 0.26, p95 53.50) | 0 | true |
| mixed | small | 1.47 (min 1.42, IQR 0.05, p95 1.50) | 96.05 (min 95.93, IQR 0.12, p95 96.21) | 1 | true |

### View D — runtime-operation microbenchmarks (50_000 iterations, correctness-gated)

| op | native median (min/p95) ms | candidate median (min/p95) ms | candidate prepare ms | byte-exact |
|---|---|---|---|---|
| move (decimal MOVE (display -> display)) | 3.27 (min 3.24, IQR 0.03, p95 3.29) | 610.84 (min 609.70, IQR 1.70, p95 612.73) | 0 | true |
| packed_add (packed-decimal ADD (COMP-3 accumulator)) | 3.47 (min 3.36, IQR 0.04, p95 3.56) | 645.62 (min 644.74, IQR 0.30, p95 647.97) | 0 | true |
| binary_add (binary ADD (COMP accumulator)) | 3.90 (min 3.88, IQR 0.04, p95 4.39) | 631.94 (min 631.78, IQR 0.53, p95 633.07) | 0 | true |
| float_add (float ADD (COMP-1 f32 + COMP-2 f64)) | 25.06 (min 24.22, IQR 0.35, p95 25.35) | 6041.27 (min 6031.58, IQR 9.39, p95 6054.72) | 0 | true |
| compare (alphanumeric comparison (IF A = B)) | 3.62 (min 3.57, IQR 0.09, p95 3.83) | 1249.83 (min 1244.80, IQR 4.65, p95 1250.24) | 0 | true |
| intrinsic (FUNCTION intrinsic dispatch (NUMVAL + INTEGER)) | 12.49 (min 12.37, IQR 0.12, p95 12.64) | 1723.90 (min 1716.46, IQR 3.13, p95 1727.62) | 0 | true |
| call (module CALL dispatch (contained subprogram)) | 6.95 (min 6.94, IQR 0.12, p95 7.11) | 1461.06 (min 1449.86, IQR 4.70, p95 1463.48) | 0 | true |
| seqfile (sequential-file read/write (50_000 fixed records)) | 20.73 (min 20.52, IQR 0.49, p95 21.24) | 1906.00 (min 1902.04, IQR 3.74, p95 1910.24) | 0 | true |

### View E — corpus throughput (10 workloads x 4 scales, one pass)

- oracle (compile+run) total: 6175.8 ms
- candidate (prepare+run) total: 1396725.1 ms
- peak memory: 175428 kB (VmHWM of the bench process (candidate runs in-process; oracle cobc/run are child processes, not included))

| workload | oracle total ms | candidate total ms |
|---|---|---|
| payroll | 734.9 | 77179.6 |
| invoice | 639.6 | 64141.8 |
| seqfile | 403.9 | 34689.3 |
| tables | 698.7 | 813335.0 |
| strings | 438.2 | 51731.0 |
| float | 647.4 | 44068.9 |
| report | 395.8 | 39588.2 |
| relative | 907.0 | 99935.8 |
| modules | 501.6 | 60931.8 |
| mixed | 808.7 | 111123.8 |

