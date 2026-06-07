# fixtures/

Small, committed inputs and oracle-generated expected outputs, so `cargo test` proves parity
**without** the admitted bundle or built oracle present.

- `hello/` — minimal COBOL programs for the `cobc-oracle-rs` program-oracle smoke.
- `decimal/` — `(PIC, value)` rows and the libcob-harness byte dumps they must match, used by
  `gnucobol-rs`' self-contained golden tests.

Every `expected` value here was produced by the **real** built GnuCOBOL 3.2 oracle, not by the
Rust port. Regenerate with the sweep scripts under `lab/oracle/` when the pinned oracle changes.
