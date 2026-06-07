# Compatibility taxonomy — the stack of separately admitted courts

> **`gnucobol-rs` treats COBOL compatibility as a stack of separately admitted courts: bytes,
> moves, initialization, comparison, formatting, source expansion, runtime lifecycle, files,
> reports, and diagnostics. No lower layer is allowed to imply a higher layer.**

This is the spine of the project. Each layer is proven against the oracle independently, ships
its own receipt, and carries its own non-claims. A green test in a lower court is never quoted
as evidence for a higher one.

| # | Court | Question it answers | Status |
|---|-------|---------------------|--------|
| 0a | **Field model** | does `PIC`+`USAGE` → `{type,digits,scale,flags,size}` match `cobc`? (`P`, COMP/COMP-5/COMP-X) | **sealed** — `pic` (`GNURUST.3`, `GNURUST.9`, `GNURUST.14`) |
| 0c | **Code page** | do raw EBCDIC DISPLAY bytes decode to the oracle's text under a named table? | **sealed** — `ebcdic` cp500 (`GNURUST.15`); cp037/numeric-zoned/DBCS fail closed |
| 0d | **Edited decode** | do edited DISPLAY field bytes decode to the oracle's value + text? | **sealed** — `edited` 16a (`GNURUST.16`); `$ * CR DB B 0 /` + numeric→edited fail closed |
| 0b | **Record layout** | do item offsets / group sizes / `OCCURS` / `REDEFINES` match `cobc`? (ODO physical-max) | **sealed** — `layout` (`GNURUST.4`, `GNURUST.10`) |
| 1 | **Storage parity** | does a field hold the same bytes? | **sealed** (`GNURUST.2`) |
| 2 | **Move parity** | do bytes after `MOVE src→dst` match? | **sealed** (`GNURUST.2`) |
| 2c | **Arithmetic parity** | do `ADD`/`SUBTRACT`/`MULTIPLY` result bytes match `cob_add`/`cob_mul`? | **sealed** — `arith` (`GNURUST.7` + packed add-sub `GNURUST.13`); DIVIDE / other modes future |
| 3 | **Initialization parity** | `VALUE` → initial record bytes | **sealed** — `init` (`GNURUST.8`); ODO/REDEFINES-VALUE, figuratives beyond ZERO/SPACE future |
| 4 | Comparison parity | LEVEL-88 predicate (`GNURUST.11`) + `SET TO TRUE` (`GNURUST.12`) **sealed**; `IF a < b`, `SEARCH ALL`, collation future | partial |
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
