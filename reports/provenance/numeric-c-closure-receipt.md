# GNURUST.NUMERIC.CLOSE.1 — numeric.c file closure receipt

**File:** `libcob/numeric.c` (1 of 13 libcob source files) · **Oracle:** GnuCOBOL 3.2.0 (FSF)
**Machine receipt:** [`numeric-c-closure-receipt.json`](numeric-c-closure-receipt.json)

## What this seals

`numeric.c` is the **first** GnuCOBOL libcob source file ported function-for-function into
`gnucobol-rs`. Every function in the file has a named Rust counterpart — **104/104**.

The project **does not claim a clean-room implementation** for this file. It claims an **openly-licensed,
provenance-documented, oracle-verified Rust port/reconstruction** of the admitted GnuCOBOL 3.2 numeric
behaviour. This wording is deliberate: it avoids both pretending the work is clean-room (it is a
derivative port, by our own source vendoring) and underselling it as "copying C" (it is a verified,
from-the-algorithm reconstruction on a pure-Rust `mpz`/`mpf` substrate, `#![forbid(unsafe_code)]`).

## Coverage

- **104/104 functions** ported as named Rust functions across `cob_decimal.rs`, `packed.rs`, `gmp.rs`
  (pure-Rust `mpz`), `mpf.rs` (pure-Rust 2048-bit `mpf`), `int_pow.rs`, `logical.rs`, `float.rs`.
- **Zero declared bounds.** The 2048-bit `mpf` is a real binary float (not an f64 proxy); the lifecycle,
  host-int, print, sign, pool, and ieee functions are all ported as named functions.
- **Five `#if 0`-disabled functions** (`cob_add_packed`, `cob_complement_packed`, `display_add_int`,
  `display_sub_int`, `cob_display_add_int`) are ported **verbatim for literal completeness** but **not
  wired** into any active path — they are not compiled into the oracle, so their behaviour is, by
  definition, not oracle-verifiable. `cob_display_add_int`'s pre-assignment `sign` read (UB — the bug
  that disabled it) is reproduced as `0` with an explicit note.

## Differential verification (all vs the admitted oracle, FAIL=0)

| sweep | result | sweep | result |
|---|---|---|---|
| cob_decimal (arith) | 5400/0 | round (8 modes) | 6720/0 |
| arith | 5400/0 | bignum (>i128) | 16128/0 |
| packed_arith (cob_add_bcd) | 1800/0 | numcmp | 1024/0 |
| double_move (set_double via mpf) | 392/0 | comp6 | 98/0 |
| float (BID/double encode) | 1476/0 | divide / remainder | 736/0 / 768/0 |
| pow | 588/0 | logical | 2400/0 |

`get_double` (decimal→mpf→`mpf_get_d`) is bit-identical to the FLOAT.1-sealed `decimal_to_f64_trunc`.
Full sealed-courts guard: **73 green / 0 red** (incl. the doc-staleness gate).

## Non-claims

- Full-precision `mpf` use **beyond** numeric.c's byte path (intr.c's intrinsics exercise the 2048-bit
  `mpf` at >double precision — that is intr.c's port, not this file's).
- Business/accounting correctness of any computed value (bytes are witnessed, not the meaning).
- `ON SIZE ERROR` control flow, `COMPUTE` expression evaluation, Procedure-Division execution.
- The `#if 0`-disabled functions' behaviour.

## LICENSE-BOUNDARY.NUMERIC.1 / PROVENANCE.NUMERIC.1

This is a **faithful derivative port** of GPL GnuCOBOL 3.2 `numeric.c`, **not clean-room**. The GnuCOBOL
source is vendored (`research/gnucobol-3.2.tar.lz`, sha256 `8ecc77d0…`) and admitted as the oracle; the
Rust port mirrors its function names and algorithms and is therefore a derivative work, licensed
**LGPL-3.0-or-later** to honour the copyleft. The Apache-2.0 KOBOLD layer **never** mixes with this LGPL
numeric port — the GPL partition is enforced by
[`reports/license/gpl-license-guard-receipt.json`](../license/gpl-license-guard-receipt.json).
