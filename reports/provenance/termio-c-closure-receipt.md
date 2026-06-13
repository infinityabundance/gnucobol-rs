# GNURUST.TERMIO.CLOSE.1 — termio.c file closure receipt

**File:** `libcob/termio.c` (`DISPLAY`/`ACCEPT` terminal I/O + the `-fdump` debug feature) · **Oracle:**
GnuCOBOL 3.2.0 (FSF) · **Machine receipt:** [`termio-c-closure-receipt.json`](termio-c-closure-receipt.json)

## What this seals

`termio.c` ported function-for-function into `gnucobol-rs`. All **18/18** functions have a named, **active**
Rust counterpart — confirmed by the typed `gnucobol-rs-port-index` (18/18 active, 0 doc-only, 0 missing)
and the authoritative doxygen parse (`DOXYGEN-PARITY.md` → `termio.c` 18/18).

The C writes to `FILE *` streams; this port writes to a `&mut Vec<u8>` sink (so the emitted bytes are
testable) and carries the runtime state the C keeps in `cobglobptr`/`cobsetptr`/`COB_MODULE_PTR` /
module-static dump variables in explicit structs (`DisplaySettings`, `DumpState`).

## Coverage (18/18)

`crates/gnucobol-rs/src/termio.rs`:

- **DISPLAY-bytes core** — `cob_display_common` and its helpers `display_numeric`,
  `pretty_display_numeric`, `display_alnum`, `clean_double`, `is_field_display`, plus the `cob_display`
  statement wrapper. Emits the exact `DISPLAY` bytes for any field type: pretty (edited) vs sign-separate
  numeric, `COMP-5`/real-binary, `COMP-1`/`COMP-2`/`long double` (`%G` + `clean_double`), `FLOAT-DECIMAL`
  (IEEE-decimal print), pointer (`0x` big-endian hex), and alphanumeric verbatim.
- **ACCEPT** — `cob_accept` (read a line, `MOVE` it into the receiver as alphanumeric).
- **DUMP feature** — `cob_dump_field`, `cob_dump_field_ext`, `dump_field_internal`, `display_alnum_dump`,
  `dump_pending_output`, `cob_dump_output`, `cob_dump_file`, `setup_varname_with_indices`,
  `setup_lvlwrk_and_dump_null_adrs` (with the OCCURS-collapse state in `DumpState`).
- **Init** — `cob_init_termio`.

## Differential verification (vs the admitted oracle, FAIL=0)

| sweep | result |
|---|---|
| `termio.c cob_display_common` (`termio_display_sweep`) | 16/0 |

The DISPLAY-bytes core is byte-verified against a **cobc program oracle**: one source of truth
(`termio_display_rows`) emits the cobc program (typed `WORKING-STORAGE` fields DISPLAYed with labels) and
builds the same field storage in Rust (via the sealed `cob_move` / float encoders), then both streams are
diffed. Covered: pretty signed/unsigned/scaled numeric `DISPLAY`, `COMP-3`, `COMP` binary, `COMP-1`/
`COMP-2` float (`%G` f-form *and* e-form), and alphanumeric. The `cob_display_common` family functions are
internal/hidden in `libcob.so`, so the program oracle (not a library harness) is the right witness. 5
in-crate unit tests additionally cover the float `%G`/`clean_double` edges, the pretty path, the
`cob_display` wrapper, and the dump formatters (`display_alnum_dump` shorthands,
`setup_varname_with_indices`, `setup_lvlwrk_and_dump_null_adrs`).

## Non-claims

- The **`-fdump` debug feature** (`dump_field_internal` and friends) is ported faithfully as real
  functions, with the pure formatters unit-tested; its full byte output — the OCCURS-collapse
  `same as (n)` state machine driven by the generated code's whole-record field-tree traversal — is not
  differentially swept here (it requires the runtime `cob_module` field-tree model, a separate court).
- `cob_display` **device routing** (printer/punch files, screen redirect, `popen` pipes) is the
  surrounding I/O; this port produces the SYSOUT/SYSERR emitted bytes.
- `long double` (`COMP-2` `long double` / x87 80-bit) display is host-specific; the `%.32LG` path reads via
  the f64 approximation and is not swept.
- `cob_accept` models the line-read + `MOVE`; terminal/screen input, `crt_status`, and `SIGINT` on `^C`
  are runtime I/O, not byte-court.

## LICENSE-BOUNDARY / PROVENANCE

Faithful **derivative port** of LGPL GnuCOBOL 3.2 `termio.c`, **not clean-room**, licensed
**LGPL-3.0-or-later**; the Apache-2.0 KOBOLD layer never mixes (enforced by
[`gpl-license-guard-receipt.json`](../license/gpl-license-guard-receipt.json)).
