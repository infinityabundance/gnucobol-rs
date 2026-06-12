# GNURUST.CCONV.CLOSE.1 — cconv.c file closure receipt

**File:** `libcob/cconv.c` (character-conversion helpers) · **Oracle:** GnuCOBOL 3.2.0 (FSF)
**Machine receipt:** [`cconv-c-closure-receipt.json`](cconv-c-closure-receipt.json)

## What this seals

`cconv.c` ported function-for-function into `gnucobol-rs`. All **9/9** functions have a named, **active**
Rust counterpart — confirmed by the typed `gnucobol-rs-port-index` (9/9 active, 0 doc-only, 0 missing) and
the authoritative doxygen parse (`DOXYGEN-PARITY.md` → `cconv.c` 9/9).

Not clean-room: an openly-licensed, provenance-documented, oracle-verified Rust port of the admitted
GnuCOBOL 3.2 character-conversion helpers.

## Coverage (9/9)

`crates/gnucobol-rs/src/cconv.rs`:

- `cob_toupper` / `cob_tolower` — the 7-bit ASCII case-fold (only `A-Z`/`a-z` change; bytes ≥ 128 and all
  non-letters pass through — **not** locale folding), backed by compile-time `LOWER_TAB`/`UPPER_TAB`
  constants (the designated-init equivalent), plus `init_upper_lower` (the non-designated path) which
  reproduces those tables exactly, and the `cob_init_cconv` module init (a no-op here).
- `cob_convert_hex_digit` / `cob_convert_hex_byte` / `cob_skip_blanks` — the hex + whitespace scanners.
- `cob_field_to_string` — rtrim a field's content (trailing spaces/NULs) into a buffer with optional case
  folding; the `-1`/`-2`/`-3`/`-4`/`0` error codes are reproduced.
- `cob_load_collation` — load a `.ttbl` translation table (the one function that does filesystem +
  `$COB_CONFIG_DIR` I/O, faithful to libcob): hex-byte parse, the 256-byte (+ computed inverse) vs
  512-byte rules, and the `-1`-on-error behaviour.

## Differential verification (vs the admitted oracle, FAIL=0)

| sweep | result |
|---|---|
| `cconv.c case/hex/collation` (`cconv_sweep`) | 27/0 |

`cob_toupper`/`cob_tolower`/`cob_field_to_string`/`cob_load_collation` are **internal (hidden)** in
`libcob.so`, so the harness links the **exact oracle object** `cconv.o` extracted from the static
`libcob.a` (its only non-libc dependency, `cob_runtime_error`, is stubbed). The sweep compares: the full
256-byte upper/lower fold, a `cob_field_to_string` grid (NONE/LOWER/UPPER/LOCALE folds, rtrim, all-blank,
trailing-NUL, error codes), and `cob_load_collation` over all five shipped `.ttbl` tables (both the
`ebcdic_to_ascii` and the `ascii_to_ebcdic` halves). The static helpers `cob_convert_hex_digit`/
`cob_convert_hex_byte`/`cob_skip_blanks` are exercised transitively by the collation parse. 4 in-crate
unit tests cover the same surface without the oracle.

## Non-claims

- The `CCM_LOWER_LOCALE`/`CCM_UPPER_LOCALE` cases use the C-library locale fold; this port matches them
  under the pinned `LC_ALL=C.UTF-8` (ASCII-only fold, bytes ≥ 128 unchanged) and does not model other
  locales.
- `cob_load_collation` does real filesystem + `$COB_CONFIG_DIR` I/O (faithful to libcob); the `cconv`
  module is therefore not part of the pure-decimal kernel (`GNURUST.PUREDEC.0`).
- `cob_runtime_error` diagnostic text is not modelled (errors surface as the negative return codes).

## LICENSE-BOUNDARY / PROVENANCE

Faithful **derivative port** of GPL GnuCOBOL 3.2 `cconv.c`, **not clean-room**, licensed
**LGPL-3.0-or-later**; the Apache-2.0 KOBOLD layer never mixes (enforced by
[`gpl-license-guard-receipt.json`](../license/gpl-license-guard-receipt.json)).
