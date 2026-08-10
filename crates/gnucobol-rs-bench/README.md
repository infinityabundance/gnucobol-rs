# gnucobol-rs-bench

Purpose-designed scalable COBOL performance corpus (Phase 8): ten workload families, each with
small / medium / large / stress scales, deterministic Rust data generators, independently
computed expected outputs, and a correctness gate that requires a **byte-exact** match against
the host GnuCOBOL 3.2.0 oracle BEFORE any timing is reported.

## Workloads

| workload | feature family |
|---|---|
| payroll | packed-decimal (COMP-3) rates and totals, tax, rounding, report |
| invoice | decimal multiplication, discounts, taxes, balances |
| seqfile | sequential-file batch: validation, aggregation, file-status |
| tables | OCCURS, subscripts, indexes, SORT, SEARCH ALL, aggregation |
| strings | STRING, UNSTRING, INSPECT, reference modification |
| float | COMP-1 / COMP-2, SIZE ERROR |
| report | grouping, subtotals, grand total |
| relative | relative-file insert / update / delete / traversal |
| modules | dynamic CALL, EXTERNAL data, CANCEL, reload |
| mixed | multi-module business workflow |

## Rules

- Inputs are deterministic (seeded xorshift64; integer-exact cents/hundredths, fixed
  contiguous columns -- no embedded decimal points).
- Expected outputs are computed independently in Rust; the candidate never generates its own
  expected output.
- A benchmark enters the corpus only after the correctness gate passes at every scale.
- Timing is recorded after correctness, per scale; raw samples are retained (Phase 9).

## Usage

```sh
gnucobol-rs-bench validate payroll small     # one workload, one scale
gnucobol-rs-bench validate all               # the full correctness gate (all scales)
gnucobol-rs-bench report                     # re-run the gate + write reports/valid-corpus/performance/
gnucobol-rs-bench list
```
