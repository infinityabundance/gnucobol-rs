# GNURUST.STRINGS.CLOSE.1 — strings.c file closure receipt

**File:** `libcob/strings.c` (3 of 13 libcob source files) · **Oracle:** GnuCOBOL 3.2.0 (FSF)
**Machine receipt:** [`strings-c-closure-receipt.json`](strings-c-closure-receipt.json)

## What this seals

`strings.c` is the **third** GnuCOBOL libcob source file ported function-for-function into `gnucobol-rs` —
all **34/34** functions have a named Rust counterpart. The C-side inventory was cross-checked directly
(exactly 34 top-level function definitions in `strings.c`), and the `libcob-parity` gate reports 34/34
with an empty missing-list.

Not clean-room: an **openly-licensed, provenance-documented, oracle-verified Rust port/reconstruction** of
the admitted GnuCOBOL 3.2 `STRING` / `UNSTRING` / `INSPECT` runtime API.

## Coverage (34/34)

All three statement runtimes are modelled as explicit structs (`CobString`, `CobUnstring`, `CobInspect`)
that carry the state the C keeps in module globals — no global mutable state, `#![forbid(unsafe_code)]`.

- **STRING** (`CobString`): `cob_string_init` → `cob_string_delimited`/`cob_string_append` → `cob_string_finish`.
- **UNSTRING** (`CobUnstring`): `cob_unstring_init` → `cob_unstring_delimited`/`cob_unstring_into`/`cob_unstring_tallying` → `cob_unstring_finish`.
- **INSPECT** (`CobInspect`): `cob_inspect_init[_converting]` → `cob_inspect_start` + `before`/`after` →
  `characters`/`all`/`leading`/`first`/`trailing`/`converting` → `cob_inspect_finish`, with every static
  helper given its exact C name: `inspect_common` (dispatcher) → `inspect_common_no_replace` /
  `inspect_common_replacing`, `inspect_find_data`, `is_marked`, `set_inspect_mark`, `do_mark`,
  `setup_repdata`, `alloc_figurative`, `cob_inspect_init_common`.
- Module lifecycle + helper: `cob_init_strings`, `cob_exit_strings`, `cob_str_memcpy`.

## Differential verification (vs the admitted oracle, FAIL=0)

| sweep | result |
|---|---|
| `GNURUST.INSPECT.1` bytes (`inspect_sweep`) | 12/0 |
| `GNURUST.STRING.UNSTRING.1` (`string_unstring_sweep`) | 7/0 |

**Transitive verification.** `CobInspect` — the 1:1 stateful port of the `strings.c` INSPECT functions —
is cross-checked **in-crate** against the oracle-sealed `inspect.rs` byte court over a **200+ case grid**
(ALL/LEADING/FIRST/CHARACTERS tally, REPLACING ALL/LEADING/FIRST, CONVERTING, BEFORE/AFTER region
restriction). Because `inspect.rs` is itself sealed against GnuCOBOL 3.2, identical results transitively
oracle-verify the stateful port.

**Fidelity note.** `set_inspect_mark` reproduces the C `size_t` underflow on the LEADING `last_marker == 0`
path (`pos_end = (pos+length).wrapping_sub(1)`; the length-0 `memset` writes nothing), so the marker
bookkeeping matches the oracle byte-for-byte.

## Non-claims

- The byte courts witness narrow, single-clause statements; un-admitted multi-clause ordering, locale/
  case-folding, and National/DBCS/UTF-8 multibyte behaviour are not claimed.
- The REPLACING size-mismatch path expands a figurative operand (`alloc_figurative`); the C
  `RANGE_INSPECT_SIZE` exception on a non-figurative mismatch is not modelled at the byte-court level.
- Procedure-Division execution, screen/report I/O, and business correctness — other files/courts.

## LICENSE-BOUNDARY / PROVENANCE

Faithful **derivative port** of GPL GnuCOBOL 3.2 `strings.c`, **not clean-room**, licensed
**LGPL-3.0-or-later**; the Apache-2.0 KOBOLD layer never mixes (enforced by
[`gpl-license-guard-receipt.json`](../license/gpl-license-guard-receipt.json)).
