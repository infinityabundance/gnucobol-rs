# reports/

The evidence ledger. Nothing in `gnucobol-rs` is "done" without a receipt here.

- `admission/` — pinned hashes (admitted tarball, built `cobc`/`libcob`), GnuCOBOL/`gcc`/`gmp`
  versions, the exact build command, and the pinned env. The shared root of trust.
- `oracle/` — parity sweep numbers (`PASS=n FAIL=0` + classified rows) and how to reproduce them.
- `kani/` — the reduced-surface proof(s) run and their verdicts.
- `fuzz/` — fuzz findings (each fixed + seeded) and run counts.
- `RECEIPT-*.md` — the per-campaign receipt: goal, the semantics diagnosed (not guessed), the
  gate state, and the exact non-claims.

These directories are **not** shipped in the published crates (`Cargo.toml` `exclude`); they
are the project's audit trail, kept in the repository.
