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
