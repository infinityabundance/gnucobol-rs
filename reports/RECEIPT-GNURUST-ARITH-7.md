# RECEIPT-GNURUST-ARITH-7 — sealed: decimal arithmetic (ADD/SUBTRACT/MULTIPLY)

**Campaign GNURUST.7.** Goal: implement `ADD` / `SUBTRACT` / `MULTIPLY` on numeric fields with
truncation and `ROUNDED`, in **pure-Rust integer decimal** (zero deps), proven byte-identical
against libcob's `cob_add` / `cob_sub` / `cob_mul`.

## Claim (exact)

`gnucobol_rs::cob_arith(op, a, a_attr, b, b_attr, round)` computes `a := a (op) b` and returns
`a`'s field bytes, **byte-identical to libcob**, for:
- `op ∈ {ADD, SUBTRACT}` with a **DISPLAY** receiving field, and `op = MULTIPLY` with a DISPLAY or
  PACKED receiving field (all routed through libcob's `cob_decimal` path);
- DISPLAY and COMP-3 operands, signed/unsigned, cross-scale;
- `round ∈ {Truncate, NearAwayFromZero}` — truncation toward zero (`shift_decimal`) and the default
  `ROUNDED` mode (`COB_STORE_NEAR_AWAY_FROM_ZERO`), ported from `cob_decimal_get_field` /
  `cob_decimal_do_round` (`numeric.c:2055` / `:1937`);
- overflow truncates to the low digits (default `TRUNC_ON_OVERFLOW`), **preserving the value's sign**
  — an overflowed negative result stores negative zero (e.g. `-40` into 1 digit → `-0` = `0x70`).

Pure-Rust `i128` integer-decimal magnitude + scale reproduces libcob's GMP integer-decimal result;
no floating point, no runtime dependency. The store path renders to a DISPLAY temp and `cob_move`s
into the target type — reusing the sealed `GNURUST.2` encoder.

## Non-claims (fail closed)

- **`ADD`/`SUBTRACT` into a PACKED field** — libcob routes these through a *separate* BCD path
  (`cob_add_bcd`, via `cob_addsub_optimized`, `numeric.c:2299`) whose rounding/overflow differs from
  the `cob_decimal` path; ported here would be a different algorithm, so it **fails closed**
  (`ArithError::PackedAddSubDeferred`) — deferred to `GNURUST.ARITH-BCD.0`. (Diagnosed, not guessed:
  the oracle truncated `-29.7`→`-29` for packed `ADD` even with `ROUNDED`.)
- **`DIVIDE`** (rounding + remainder), the other **six rounding modes**, `ON SIZE ERROR` exception
  semantics, float receiving fields — deferred.
- Operands/intermediates beyond the **i128** range (`>38` significant digits / product overflow) —
  `ArithError::OutOfRange`; the GMP-grade bignum is `GNURUST.ARITH-BIGNUM.0`.

## Oracle

`lab/oracle/arith_harness.c` links the built libcob, constructs `cob_field`s, calls the real
`cob_add`/`cob_sub`/`cob_mul(f1, f2, opt)` (opt `0`=truncate, `COB_STORE_ROUND`=nearest-away), and
dumps `f1`. The Rust mirror (`examples/arith_rows`) consumes identical rows.

## Evidence

| Check | Result |
|-------|--------|
| Differential sweep vs `cob_add`/`cob_sub`/`cob_mul` | **PASS=1800 FAIL=0** (`lab/oracle/arith_sweep.sh`), with **900** packed-add/sub rows classified out to the deferred BCD sub-court. ADD/SUB/MUL × DISPLAY/COMP-3 × signed/unsigned (4 sign combos) × digits/scales × truncate/ROUNDED × rounding-tie & overflow values |
| Self-contained `cargo test` | arith: 6/6 (add/multiply/rounded; packed add/sub fails closed; packed multiply ok) |
| Fuzz (`arith` target, arbitrary operands/attrs/op) | **8,000,000 runs, 0 crashes** — i128 `checked_*` + `pow10` bounds hold |
| `fmt` / `clippy -D warnings` / doc-gate | clean |

## Determinism

Pure function of its inputs (`GNURUST.PUREDEC.0`); no env/locale/fs; zero runtime deps; no float.
Same pinned oracle/env as the other courts.
