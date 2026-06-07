# Operator checklist — how *not* to overtrust a receipt

A project earns trust by teaching its users where its claims stop. Before quoting any
`gnucobol-rs` parity result as evidence, confirm:

- [ ] **Oracle availability** was a *typed verdict*, not a silent skip (`GNURUST.ORACLEAVAIL.0`):
      `available` / `unavailable-expected` / `unavailable-unexpected` / `version-mismatch` /
      `config-mismatch`. A green run with `unavailable-unexpected` is not green.
- [ ] **The admitted `libcob` was loaded** — the harness resolved `<lab>/oracle/prefix/lib/libcob`,
      not a system copy (`GNURUST.LOADER.0`).
- [ ] **Runtime config was pinned** — config-dir/`runtime.cfg` hashes match the admission receipt;
      `COB_RUNTIME_CONFIG` unset or recorded (config is semantics; last-wins is silent).
- [ ] **Backend pinned** if any file surface is involved (it is **not**, today) — see
      `backend-matrix.md`.
- [ ] **Env whitelist pinned** — `LC_ALL=C.UTF-8`; no stray `COB_*` overrides.
- [ ] **Process isolation** — oracle/state tests ran in fresh child processes, not in-process,
      not relying on Rust test order (`docs/runtime-doctrine.md`).
- [ ] **Binary hashes are marked host-specific witnesses** — never cross-platform evidence
      (`GNURUST.BINWITNESS.0`).
- [ ] **The claim is in the sealed court** — storage/move bytes only; not display, comparison,
      files, arithmetic, or compilation. The full non-claims list is `reports/negative-claims.md`.
- [ ] **Any divergence is classified**, not waved off (`reports/oracle-delta-ledger.md`).

If any box is unchecked, the result is a *witness*, not a *proof*.
