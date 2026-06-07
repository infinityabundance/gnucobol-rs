# Compatibility taxonomy — the stack of separately admitted courts

> **`gnucobol-rs` treats COBOL compatibility as a stack of separately admitted courts: bytes,
> moves, initialization, comparison, formatting, source expansion, runtime lifecycle, files,
> reports, and diagnostics. No lower layer is allowed to imply a higher layer.**

This is the spine of the project. Each layer is proven against the oracle independently, ships
its own receipt, and carries its own non-claims. A green test in a lower court is never quoted
as evidence for a higher one.

| # | Court | Question it answers | Status |
|---|-------|---------------------|--------|
| 0a | **Field model** | does `PIC`+`USAGE` → `{type,digits,scale,flags,size}` match `cobc`? | **sealed** — `pic` (`GNURUST.3`) |
| 0b | **Record layout** | do item offsets / group sizes / `OCCURS` / `REDEFINES` match `cobc`? | **sealed** — `layout` (`GNURUST.4`) |
| 1 | **Storage parity** | does a field hold the same bytes? | **sealed** (`GNURUST.2`) |
| 2 | **Move parity** | do bytes after `MOVE src→dst` match? | **sealed** (`GNURUST.2`) |
| 2c | **Arithmetic parity** | do `ADD`/`SUBTRACT`/`MULTIPLY` result bytes match `cob_add`/`cob_mul`? | **sealed** — `arith` (`GNURUST.7`); DIVIDE / packed add-sub / other modes future |
| 3 | **Initialization parity** | `VALUE` → initial record bytes | **sealed** — `init` (`GNURUST.8`); ODO/REDEFINES-VALUE, figuratives beyond ZERO/SPACE future |
| 4 | Comparison parity | `IF a < b`, `SEARCH ALL`, collation | future |
| 5 | Display/output parity | `DISPLAY`, edited pictures, Report Writer | future |
| 6 | Source/preprocess parity | `COPY` splice (`GNURUST.5`) + `REPLACING` (`GNURUST.6`) **sealed**; source-format/`REPLACE`-directive future | partial |
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
