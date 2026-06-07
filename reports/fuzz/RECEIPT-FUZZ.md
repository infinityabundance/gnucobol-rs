# Fuzz sub-receipt — hostile bytes/attrs, fix-and-seed

Detached fuzz crate (`crates/cobol-decimal-rs/fuzz/`, empty `[workspace]` → not built by the parent
gate; verified: `cargo clippy --all-targets` from the workspace root does not compile it). Entry
behind the `fuzzing` feature (`__fuzz_cob_move`). cargo-fuzz 0.13 on nightly.

Target: arbitrary bytes + arbitrary `(type, digits, scale, flags)` attributes driven through
`cob_move` and the value decoders. The only assertion is **panic-freedom** (`GNURUST.PANICPOLICY.0`):
a corrupt/oversized field must yield a typed result or guarded bytes — never a panic, OOB index, or
arithmetic overflow.

## Findings (the fuzzer earned its keep)

| # | Finding | Fix | Regression seed |
|---|---------|-----|-----------------|
| 1 | Harness-internal `split - 9` underflow in `__fuzz_cob_move` (not library code). | Robust `body.split_at(at % (len+1))`. | `corpus/cob_move/regression-harness-split-underflow` |
| 2 | **Real library OOB**: `&src_full[s_off..]` panicked when `s_off=1` (leading-separate sign) but the source field was empty (`move_ops.rs` display→packed/display→display). The upstream C reads OOB here (UB); the port must fail closed. | Bounds-guarded slice `src.get(s_off..).unwrap_or(&[])` in both move functions. | `corpus/cob_move/regression-leadsep-empty-src` |

Each crash was fixed, committed as a regression corpus seed, and verified to replay cleanly.

## Run

`cargo +nightly fuzz run cob_move -- -runs=20000000 -max_len=80` →
**`#20000000 DONE`, 0 crashes**, after the fixes. Parity (the differential sweep) re-checked
**PASS=13152 FAIL=0** afterward — the guards only affect degenerate inputs the sweep does not feed.

Bounded clean run; not claimed as "saturation".
