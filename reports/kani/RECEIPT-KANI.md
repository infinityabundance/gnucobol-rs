# Kani sub-receipt — sharp reduced-surface proofs

Doctrine: prove the **sharpest** index/arithmetic invariants — the exact bounds that would be the
actual out-of-bounds access if a guard or the scale arithmetic regressed — not broad "the parser is
correct" vibes. Each harness is allocation-free and converges quickly.

Run: `cargo kani` in `crates/cobol-decimal-rs/` (cargo-kani 0.67).

| Harness | Invariant | Verdict |
|---------|-----------|---------|
| `store_window_is_in_bounds` | The scale-alignment copy window of `store_common_region` (extracted as the pure `region_window`) lies entirely within **both** buffers for *all* integer inputs: when `Some((d,s,n))`, `n>0 ∧ 0≤d ∧ d+n≤fsize ∧ 0≤s ∧ s+n≤size`. This is the one place a regression in `gcf=min(hf1,hf2)`/`lcf=max(lf1,lf2)` would silently index past a field. Proved unbounded in the inputs (full i64 reasoning over the bounded field model), so it is the strongest form of the bound, not a sample. | **SUCCESSFUL** |
| `packed_unpack_buffer_sufficient` | For any packed field with `1 ≤ digits ≤ COB_MAX_DIGITS`, decoding any nibble content via `cob_move` PACKED→DISPLAY never panics or indexes past the fixed `[u8; COB_MAX_DIGITS+1]` unpack buffer — the buffer is exactly sized and the push-guard never silently truncates a valid field. | **SUCCESSFUL** |

`Complete - 2 successfully verified harnesses, 0 failures, 2 total.`

Both proofs reference the production code paths directly (`move_ops::region_window`, `cob_move`),
so they are single-source-of-truth: the proved arithmetic is the same arithmetic the sweep
exercises against the oracle.
