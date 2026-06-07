# Compatibility taxonomy — the stack of separately admitted courts

> **`gnucobol-rs` treats COBOL compatibility as a stack of separately admitted courts: bytes,
> moves, initialization, comparison, formatting, source expansion, runtime lifecycle, files,
> reports, and diagnostics. No lower layer is allowed to imply a higher layer.**

This is the spine of the project. Each layer is proven against the oracle independently, ships
its own receipt, and carries its own non-claims. A green test in a lower court is never quoted
as evidence for a higher one.

| # | Court | Question it answers | Status |
|---|-------|---------------------|--------|
| 1 | **Storage parity** | does a field hold the same bytes? | **sealing now** (`gnucobol-rs`) |
| 2 | **Move parity** | do bytes after `MOVE src→dst` match? | **sealing now** (`gnucobol-rs`) |
| 3 | Initialization parity | `VALUE` / figurative constants → initial bytes | future |
| 4 | Comparison parity | `IF a < b`, `SEARCH ALL`, collation | future |
| 5 | Display/output parity | `DISPLAY`, edited pictures, Report Writer | future |
| 6 | Source/preprocess parity | source format, `COPY`/`REPLACING`, directives | future |
| 7 | Runtime-lifecycle parity | `CALL`/`CANCEL`, storage lifetimes, global state | future |
| 8 | File-behavior parity | sequential / line-seq / relative / indexed, status codes | future |
| 9 | Diagnostic parity | message class + phase + source span | future |

## The one rule that prevents overclaiming

A pass in court *N* says nothing about court *N+1*. Concretely, today:

- storage/move parity (1–2) does **not** imply comparison (4), display/edited (5), or file (8)
  parity;
- a `DISPLAY` stdout match would **not** imply storage parity — see `docs/runtime-doctrine.md`
  ("Display vs storage").

The full enumerated future courts and their explicit non-claims live in
[`future-risk-register.md`](future-risk-register.md); the machine-readable list of every
current non-claim lives in [`../reports/negative-claims.md`](../reports/negative-claims.md).
