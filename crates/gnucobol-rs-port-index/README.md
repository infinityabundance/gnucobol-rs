# gnucobol-rs-port-index

`gnucobol-rs-port-index` — the port-governance index for the `gnucobol-rs` libcob port.

It replaces grep name-matching with **typed C↔Rust symbol parity**: the admitted libcob C source is
parsed into a symbol index carrying **preprocessor status** (compiled / `#if 0` / config-gated), the
Rust port is parsed into a **real-`fn`** index (comment/string-aware, so a doc-comment mention never
counts as a port), and the two are joined into the generated parity maps:

- `LIBCOB-PARITY.md` — typed C↔Rust symbol map (the porting-process scoreboard).
- `DOXYGEN-PARITY.md` — independent preprocessed-C function inventory ("did we miss a function?").
- `CLANG-AST-PARITY.md` — clang AST def/call-edge view (the C callgraph).
- `FUNCTION-EVIDENCE.md` — per-ported-`fn` classified evidence (direct / transitive / lifecycle).

This crate was originally an internal `publish = false` governance tool; it is published (0.1.0) so
the porting-process index is inspectable outside the repository. Byte/behaviour parity is **not**
this crate's claim — that is the per-court oracle sweeps in the repository's
`lab/verify-sealed-courts.sh`.

## Usage

```sh
cargo run -p gnucobol-rs-port-index -- parity          # join C + Rust symbol indexes
cargo run -p gnucobol-rs-port-index -- parity check    # anti-staleness gate (regenerate + diff)
cargo run -p gnucobol-rs-port-index -- evidence check  # FUNCTION-EVIDENCE freshness
cargo run -p gnucobol-rs-port-index -- ccvs85 check    # CCVS85 corpus-custody receipt freshness
cargo run -p gnucobol-rs-port-index -- clang-index generate
```

## Repository context

See `PORTING-LADDER.md`, `LIBCOB-PARITY.md`, `DOXYGEN-PARITY.md`, `CLANG-AST-PARITY.md`, and
`FUNCTION-EVIDENCE.md` in the [`gnucobol-rs`](https://github.com/infinityabundance/gnucobol-rs)
repository.

## License

LGPL-3.0-or-later, matching the project license.
