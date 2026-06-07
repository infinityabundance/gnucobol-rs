---
receipt_schema: gnurust-campaign-receipt-v1
campaign: GNURUST.13
byte_domain: field-storage bytes of the receiving (f1) field
oracle: GnuCOBOL 3.2.0 cob_add / cob_sub (cob_add_bcd path); see reports/admission/
sweep: lab/oracle/arith_sweep.sh  total=5400  PASS=5400  FAIL=0
fuzz: arith target, 8_000_000 runs, 0 crashes
sealed_version: gnucobol-rs 0.3.3
---

# RECEIPT-GNURUST-ADDBCD-13 — sealed: packed ADD/SUBTRACT via cob_add_bcd

**Campaign GNURUST.13.** Goal: seal `ADD`/`SUBTRACT` into a **PACKED (COMP-3) receiving field** — the
path libcob routes through `cob_add_bcd` (`numeric.c:3021`, via `cob_addsub_optimized`), which
GNURUST.7 deferred.

## Doctrine (the one sentence)

> GNURUST.13 admits packed ADD/SUBTRACT only as receiving-field byte semantics for the admitted
> DISPLAY and COMP-3 operands; it does not claim DIVIDE, SIZE ERROR control flow, COMPUTE expression
> semantics, all rounding modes, bignum behavior, or malformed packed-decimal tolerance beyond the
> sealed MOVE courts.

## Claim (exact) — **byte domain: receiving-field storage bytes**

`gnucobol_rs::cob_arith(Op::Add|Subtract, …)` produces the **receiving (`f1`) field bytes**
byte-identical to libcob `cob_add`/`cob_sub` for a **PACKED** (or DISPLAY) receiver and DISPLAY/COMP-3
operands: signed/unsigned, scale differences (left>right, right>left, equal), truncation and
`ROUNDED` (nearest-away), odd/even COMP-3 digit counts, carry into the final digit, and overflow into
a smaller receiver. Packed parity is **byte** parity (a numeric-value check would hide sign-nibble
differences) — the sweep compares raw `f1` bytes.

The same integer-decimal `compute`/`store` used for the `cob_decimal` path produces these bytes,
because `cob_add_bcd` is the same arithmetic (exact sum → align to receiver scale → round/truncate →
overflow-truncate → store), **with one cob_add_bcd-specific rule** (below).

## Two oracle facts diagnosed (not guessed)

1. **The `opt` artifact.** GNURUST.7's deferral was a test artifact: I passed `opt = COB_STORE_ROUND`
   (bit 0 only); `cob_add_bcd` rounds only when the **mode** bit is also set, so bit-0-alone
   truncates. The real `ADD ... ROUNDED` opt is `COB_STORE_ROUND | COB_STORE_NEAR_AWAY_FROM_ZERO`
   (`0x21`). With the correct opt, `cob_add_bcd` rounds nearest-away, matching this port.
2. **Negative zero on truncation.** When a **negative** result truncates to **zero magnitude**
   (`7 + -7.6 = -0.6` → scale-0 → `-0`), `cob_add_bcd` keeps the **negative** sign nibble (`-0`,
   `0x0D`); the `cob_decimal`/DISPLAY path yields `+0` (`0x0C`). This port carries a per-path
   `sign_on_zero` flag (true only for packed ADD/SUBTRACT) to reproduce it. (This is distinct from the
   already-sealed negative-zero-**on-overflow** of GNURUST.7.)

## Non-claims (fail closed / out of scope)

`DIVIDE`, `ON SIZE ERROR` / `NOT ON SIZE ERROR` control flow, rounding modes other than nearest-away,
`COMPUTE` expression trees, `>38`-digit (bignum) intermediates (`ArithError::OutOfRange`), float /
`COMP-1` / `COMP-2`, and malformed packed input beyond the sealed `MOVE` decode rules.

## Evidence

| Check | Result |
|-------|--------|
| Differential sweep vs `cob_add`/`cob_sub`/`cob_mul` (receiving-field bytes) | **total=5400 PASS=5400 FAIL=0** (`lab/oracle/arith_sweep.sh`): receiver × operand ∈ {DISPLAY, COMP-3}², ADD/SUB/MUL, signed/unsigned, scale L>R/R>L/=, truncate/ROUNDED(`0x21`), odd/even digits, carry, overflow, negative-zero-on-truncation |
| Self-contained `cargo test` | arith: 6/6 (incl. `packed_add_sub_via_bcd`) |
| Fuzz (`arith`) | **8,000,000 runs, 0 crashes** |
| GNURUST.7 regression | subsumed — the prior 1800 rows are within this 5400 (no DISPLAY/MUL change) |
| `fmt` / `clippy -D warnings` / doc-gate | clean |

## Determinism

Pure function of its inputs; zero deps; same pinned oracle/env. The earlier `PackedAddSubDeferred`
error variant is retained for API stability but is no longer produced.
