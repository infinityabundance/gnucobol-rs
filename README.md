# gnucobol-rs

**A Rust-native compatibility court for GnuCOBOL — it begins with byte-exact COBOL
data semantics proven against upstream GnuCOBOL 3.2, not a compiler.**

`gnucobol-rs` is **not** a GnuCOBOL replacement, **not** a COBOL compiler, and **not** a
`libcob` replacement. It does not define COBOL truth. It is an *oracle-first*, receipt-bearing
porting surface: it reproduces **observable GnuCOBOL byte/runtime semantics** in memory-safe
Rust, one narrow slice at a time, and proves each slice against a locally built GnuCOBOL
oracle under pinned settings — stating, as loudly as each positive claim, exactly what it
does *not* claim.

## Why this exists

COBOL's bedrock is not its syntax. It is **byte layout, decimal representation, and field
movement** — packed decimal (COMP-3), zoned decimal, display numerics, and the `MOVE`
semantics between them. Those are exactly the things a migration team must trust to the
byte. `gnucobol-rs` owns them first, with proof, before reaching for anything larger.

## Compatibility axes (claimed independently, never bundled)

| Axis | Meaning | Status |
|------|---------|--------|
| **byte-layout** | a field's bytes match GnuCOBOL's exactly | **sealed** — COMP-3/zoned/display (`GNURUST.2`) |
| **runtime** | a runtime operation (`MOVE`, …) matches `libcob` | **sealed** — decimal `MOVE` (`GNURUST.2`) |
| **field model** | `PIC`+`USAGE` → `{type, digits, scale, flags, size}` matches `cobc` | **sealed** — `pic` (`GNURUST.3`) |
| **record layout** | item byte offsets / group sizes / `OCCURS` / `REDEFINES` match `cobc` | **sealed** — `layout` (`GNURUST.4`) |
| **copybook expansion** | `COPY` splice matches the `cobc` preprocessor | **sealed** — `copybook` (`GNURUST.5`) |
| source | source-form / `COPY REPLACING` / directives | future campaign |
| behavior | program stdout/stderr/exit matches `cobc -x` output | oracle harness only |
| diagnostic | compiler messages match `cobc` | not claimed |
| compiler-replacement | emit native code | **not claimed — requires future receipts** |

## The admitted oracle

Upstream **GnuCOBOL 3.2** (`cobc` + `libcob`) is the source of truth. Because it is not
installed system-wide, it is **built from pinned source** (`research/gnucobol-3.2.tar.lz`,
sha256 recorded in `reports/admission/`) into a gitignored `lab/oracle/prefix`. "Correct"
here always means *matches the built oracle*, never *matches our reading of a spec*.

## Crates

| Crate | Derives from | License | Scope |
|-------|--------------|---------|-------|
| [`gnucobol-rs`](crates/gnucobol-rs) | `libcob/move.c`, `libcob/numeric.c`, `libcob/common.c` | **LGPL-3.0-or-later** | COMP-3 / zoned / display byte semantics + `MOVE` between them |
| [`cobc-oracle-rs`](crates/cobc-oracle-rs) | drives `cobc` (no GPL code copied) | **GPL-3.0-or-later** | build/run `cobc` fixtures, capture deterministic JSON receipts |

## License & derivation boundary

This is a **faithful derivative port**, not a clean-room reimplementation: functions are
ported statement-by-statement with upstream line citations (e.g. `// move.c:477`). The port
therefore **inherits upstream copyleft**:

- crates derived from **`libcob`** (LGPL-3.0-or-later) are **LGPL-3.0-or-later**;
- crates derived from **`cobc`** (GPL-3.0-or-later) are **GPL-3.0-or-later**.

The FSF copyright notice is retained. See [`docs/derivation-and-license.md`](docs/derivation-and-license.md),
[`COPYING.LESSER`](COPYING.LESSER) (LGPL-3.0), and [`COPYING`](COPYING) (GPL-3.0).

## Compatibility is a stack of courts

`gnucobol-rs` treats COBOL compatibility as a stack of **separately admitted courts** — bytes,
moves, field model, record layout, initialization, comparison, formatting, source expansion,
runtime lifecycle, files, reports, diagnostics — and **no lower layer is allowed to imply a higher
layer**. Sealed today: storage bytes + `MOVE` bytes (`GNURUST.2`), `PIC`→field-model (`GNURUST.3`),
DATA DIVISION record layout (`GNURUST.4`), and `COPY` copybook expansion (`GNURUST.5`). The full
taxonomy is in
[`docs/compatibility-taxonomy.md`](docs/compatibility-taxonomy.md); every named future court and
its non-claim is in [`docs/future-risk-register.md`](docs/future-risk-register.md); the
machine-readable list of every non-claim is in [`reports/negative-claims.md`](reports/negative-claims.md).

## Project status, features, and MSRV

- **Independent project.** This is an independent Rust compatibility/porting effort and is **not**
  the upstream GnuCOBOL project, nor endorsed by it. GnuCOBOL is the admitted oracle.
- **Feature flags never change admitted semantics.** Cargo features gate only surface (e.g.
  `serde`, `cli`, `fuzzing`, `kani`), never dialect/behavior. A dialect or "accept invalid data"
  mode, if ever added, is explicit runtime config or a typed policy with its own receipt — never a
  hidden feature toggle.
- **MSRV 1.74** applies to the library crates and their **self-contained** tests (which pass
  without a local GnuCOBOL). The oracle sweep needs host tools (a built `cobc`/`libcob`) outside
  the MSRV guarantee.

## Method

Admit pinned source → read it → build the real upstream as an executable oracle → port
faithfully with citations → prove byte parity over a fixture matrix + differential sweep →
pin or classify every confounder → Kani the sharp invariants → fuzz the hostile surface and
fix what it finds → gate → seal with receipts and exact non-claims. See
[`docs/porting-method.md`](docs/porting-method.md) and [`docs/claim-boundary.md`](docs/claim-boundary.md).

A **documentation refresh gate** ([`docs/doc-gate.md`](docs/doc-gate.md), `lab/check-docs.sh`)
runs alongside fmt/clippy/test/sweep and fails if any doc drifts from the code, the receipts, or
the oracle — so nothing goes stale as the compatibility register grows.
