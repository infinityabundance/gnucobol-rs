# GNURUST.COBGETOPT.CLOSE.1 — cobgetopt.c file closure receipt

**File:** `libcob/cobgetopt.c` (GnuCOBOL's vendored GNU `getopt_long`) · **Oracle:** GnuCOBOL 3.2.0 (FSF)
**Machine receipt:** [`cobgetopt-c-closure-receipt.json`](cobgetopt-c-closure-receipt.json)

## What this seals

`cobgetopt.c` — the GNU C Library / gnulib `getopt_long` vendored into GnuCOBOL — ported
function-for-function into `gnucobol-rs`. All **4/4** functions have a named Rust counterpart, confirmed
by **doxygen's preprocessed C parse** (`DOXYGEN-PARITY.md` → `cobgetopt.c` 4/4) and the awk inventory
(`LIBCOB-PARITY.md` 4/4). First file of the smallest-first (< 70 functions) pass.

Not clean-room: an **openly-licensed, provenance-documented, oracle-verified Rust port** of the admitted
GnuCOBOL 3.2 command-line option scanner.

## Coverage (4/4)

`crates/gnucobol-rs/src/cobgetopt.rs` — the C file-static globals
(`optind`/`optarg`/`opterr`/`optopt`/`nextchar`/`ordering`/`first_nonopt`/`last_nonopt`) become explicit
[`CobGetopt`] fields (no global mutable state, `#![forbid(unsafe_code)]`); the `char *nextchar` scan
pointer becomes a `(elem, off)` index into the owned, permutable `argv`; the `struct option`'s `int *flag`
becomes an opaque slot id recorded in `flag_writes`.

- `cob_getopt_long_long` — the scanner.
- `process_long_option` — exact / unique-abbreviation / ambiguous long-option matching + argument consume.
- `_getopt_initialize` — first-call ordering selection (`+`/`-`/`POSIXLY_CORRECT`/PERMUTE).
- `exchange` — the `argv` permutation primitive.

## Differential verification (vs the admitted oracle, FAIL=0)

| sweep | result |
|---|---|
| `cobgetopt.c getopt_long_long` (`getopt_sweep`) | 35/0 |

Identical scenarios feed the real libcob `cob_getopt_long_long` (`getopt_harness.c`, linked against the
built libcob) and the Rust port; the compared stream is the per-call `(return, optarg, optind, optopt)`
tuple (`opterr` forced to 0 on both sides). Scenarios cover short options (required/optional/missing args,
clustering, unknown), the leading `+`/`-`/`:` ordering modes, PERMUTE reordering, the `--` terminator,
long options (exact/abbrev/ambiguous/`=arg`/separate-arg/no-arg-with-`=`), `getopt_long_only`, and the
`-W foo` convenience form. 6 in-crate unit tests (`cargo test`) cover the same semantics without the oracle.

## Non-claims

- stderr diagnostic **text** is not part of the court (`opterr` forced to 0); the port emits the same
  messages but only the parse-result tuple is differentially verified.
- The `struct option`'s `int *flag` write-through is modelled as a recorded `(slot, val)` pair, not a raw
  pointer store — GnuCOBOL's own `long_options` tables never use a non-NULL `flag`, so that path is
  exercised by a unit test, not the oracle sweep.

## LICENSE-BOUNDARY / PROVENANCE

Faithful **derivative port** of GnuCOBOL 3.2 `cobgetopt.c` (itself LGPL-2.1-or-later GNU C Library /
gnulib code), **not clean-room**, licensed **LGPL-3.0-or-later**; the Apache-2.0 KOBOLD layer never mixes
(enforced by [`gpl-license-guard-receipt.json`](../license/gpl-license-guard-receipt.json)).
