# Contributing to gnucobol-rs

This project has an unusually strict contribution contract because its entire value is
**provable fidelity to GnuCOBOL**. A patch that loosens any of the following is not welcome,
however convenient.

## The derivation rule (read first)

`gnucobol-rs` is a **faithful derivative port** (see [`docs/derivation-and-license.md`](docs/derivation-and-license.md)).

- Code ported from **`libcob`** must live in an **LGPL-3.0-or-later** crate and cite the
  upstream lines it mirrors (`// move.c:477`).
- Code derived from or tightly coupled to **`cobc`** must live in a **GPL-3.0-or-later** crate.
- **No untraced copied logic.** If you read upstream `.c` to write a function, cite it and
  carry its license. Do not relabel a derivative as "clean-room".

## The method (non-negotiable)

Every semantic claim is oracle-first and receipt-backed (see [`docs/porting-method.md`](docs/porting-method.md)):

1. Build the GnuCOBOL 3.2 oracle from pinned source into `lab/oracle/prefix`.
2. Port faithfully — **faithful, not improved**. A "nicer" divergence from the oracle is a bug.
3. Prove byte/verdict parity over fixtures + a differential sweep; print `PASS=n FAIL=0`.
4. Pin or classify every confounder; never blur the claim.
5. Add a Kani reduced-surface proof and fuzz the hostile surface; fix what it finds.

## Engineering constraints

- `#![forbid(unsafe_code)]` in every crate.
- `overflow-checks = true` in release; checked/saturating arithmetic at byte/parse choke points.
- Typed errors, **never panic on hostile input**.
- Zero runtime dependencies where practical.
- Gate locally before proposing: `cargo fmt --check`, `cargo clippy --all-targets -D warnings`,
  `cargo test`, and the oracle sweep.

## What not to add

No "ergonomic" wrappers, no feature/dialect supersets, no policy engines, no broad
compiler-replacement claims. Breadth arrives only as new, separately sealed campaigns.
