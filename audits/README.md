# Audit board

> **A folder existing is not a result.** Only a current generated receipt
> (`reports/receipts/<CAMPAIGN>/receipt.json`) or an explicit row below is a claim. "We have a fuzz
> directory" is **not** "we fuzzed it."

Honest states: `green` · `partial` · `inconclusive` · `pending` · `not-run` · `oracle-unavailable`.
Not everything is green by design — `not-run` is a valid, honest state.

| Audit | State | How to reproduce |
|-------|-------|------------------|
| oracle sweeps (17 courts) | **green** | `bash lab/verify-sealed-courts.sh` (needs built oracle) |
| receipt replay (TRUST.2) | **green** | `python3 lab/receipt/run.py check` |
| doc-staleness gate | **green** | `bash lab/check-docs.sh` |
| self-contained court tests | **green** | `cargo test` |
| fuzz (cob_move/pic/edited/…) | **partial** | `cargo +nightly fuzz run <target>` — run on demand, not in CI; seeds committed for past crashes |
| Kani proofs (2 sharp invariants) | **green (on demand)** | `cargo kani` — converges; not in the fast gate |
| `cargo audit` (advisories) | **not-run** | `cargo audit` — no CI integration yet |
| unsafe audit | **green (by construction)** | `#![forbid(unsafe_code)]` in every shipped crate |
| `cargo vet` / supply-chain | **not-run** | future |
| miri | **not-run** | not applicable to the pure kernel beyond existing tests |
| license-boundary scan | **green** | LGPL/GPL/Apache split documented in `docs/license-boundaries.md`; hygiene grep pre-publish |
| schema validation (atlas/receipts JSON) | **green** | doc-gate validates all JSON parses |
| bench parity | **green (gated)** | `kobold-bench` re-checks parity after every run |
| lambda build | **partial (compile-verified)** | `cargo lambda build --features lambda`; **not** live-deployed |

The current authority over all of the above is [`/STATUS.md`](../STATUS.md).
