# RECEIPT-COBOL-DECIMAL-RS-1 — sealed: COMP-3 / zoned / display MOVE byte semantics

**Campaign GNURUST.2.** Goal: port the observable byte semantics of GnuCOBOL's packed-decimal
(COMP-3), zoned, and display numeric fields and the `MOVE` conversions between them into Rust, and
prove them **byte-identical** against the built GnuCOBOL 3.2 `libcob` oracle.

## Claim (exact)

For the three sealed elementary `cob_move` type pairs — **DISPLAY→DISPLAY**, **DISPLAY→PACKED**
(COMP-3 encode), **PACKED→DISPLAY** (COMP-3 decode) — on a little-endian ASCII host under
`LC_ALL=C.UTF-8`, the Rust `cobol-decimal-rs::cob_move` produces **byte-identical** destination
field bytes to `libcob`'s `cob_move`, for admitted `(type, digits, scale, flags)` attributes.
Byte domain: *field-storage* and *move-result* bytes (`GNURUST.BYTEDOMAIN.0`).

## Non-claims (as loud as the claim)

No decimal **arithmetic** (deferred — GMP-backed); no edited pictures; no `DISPLAY`-statement
stdout; no comparison/collation; no binary/`COMP`/float; no `COMP-6`/`COMP-X` move parity beyond
what a future receipt records; no EBCDIC-host sign mode; no files; no compiler. Every other
`cob_move` pair **fails closed** with `UnsupportedConversion`. Full list:
[`negative-claims.md`](negative-claims.md). The open future-court register:
[`../docs/future-risk-register.md`](../docs/future-risk-register.md).

## Semantics diagnosed from source (not guessed)

Ported statement-by-statement with upstream line citations (LGPL-3.0-or-later derivative):
- `cob_move` dispatch — `move.c:1446`; `store_common_region` — `move.c:147`
  (extracted as the pure `region_window` for proof);
- `cob_move_display_to_display` — `move.c:372`; `cob_move_display_to_packed` — `move.c:477`
  (the `p`/`q`/`i` packing pointer-dance incl. the one-byte-past read cleaned by `&= 0xf0`,
  reproduced faithfully and bounds-guarded);
- `cob_move_packed_to_display` — `move.c:582` (leading-zero skip, COMP-3 vs COMP-6);
- sign: `cob_packed_get_sign` — `numeric.c:967`; packed sign nibble `0x0C`/`0x0D`/`0x0F`;
  ASCII zoned overpunch (`|= 0x40`) — `cob_real_get/put_sign` `common.c:3712/3763`,
  `cob_get_sign_ascii` `common.c:1450`, `locate_sign` `common.c:3693`.

Diagnosed gotchas: `COB_GET_SIGN_ADJUST` does **not** mutate the source on an ASCII host (digit
recovered via the low-nibble `COB_D2I`), whereas `COB_GET_SIGN` (display→display) strips the
overpunch before copying; unsigned packed sign nibble is `0x0F`, signed positive `0x0C`, negative
`0x0D`; negative zero is a representable, preserved fact.

## Evidence (gate state)

| Check | Result |
|-------|--------|
| Differential sweep vs `libcob` oracle | **PASS=13152 FAIL=0** per seed, across seeds 0,1,2,7,42,12345,999999 (~92k cases) |
| Canonical roundtrip | `decode→encode` byte-identical (unit + sweep) |
| Structured edge families (`GNURUST.DECEDGE.0`) | digits 1/2/3/4/5/7/17/18 × scale {0,1,n-1,n} × {zero,one,all-9s,leading-zeros} × signed/unsigned × ±, all directions |
| Self-contained `cargo test` (no `lab/`) | 6/6 pass (golden + fail-closed + negative-zero + hostile-attr) |
| Kani (sharp invariants) | **2/2 SUCCESSFUL** — `store_window_is_in_bounds` (scale-alignment window ⊆ both buffers, all inputs), `packed_unpack_buffer_sufficient` |
| Fuzz (`cob_move`, hostile bytes/attrs) | **20,000,000 runs, 0 crashes** after fixing 1 real OOB (leading-separate sign with empty source → guarded slice; seeded as a regression) |
| `cargo fmt --check` / `clippy --all-targets -D warnings` | clean |
| Documentation refresh gate (`lab/check-docs.sh`) | **PASS** (incl. oracle freshness: live sweep FAIL=0 + selfcheck constants match) |
| ABI/constants vs built oracle (`GNURUST.CABI-FIELD.0`/`NUMCONST.0`) | match: DISPLAY=0x10, PACKED=0x12, flags 1/2/4/0x100, `COB_MAX_DIGITS=38` |

## Determinism / oracle identity

`LC_ALL=C.UTF-8`, little-endian ASCII host (`COB_EBCDIC_MACHINE` off, classified). Oracle =
built GnuCOBOL 3.2.0 (`--with-db`, BDB 5.3) — full identity, config identity, loader/archive
identity, and binary-witness policy in [`admission/RECEIPT-ADMISSION.md`](admission/RECEIPT-ADMISSION.md).
The harness binds the **admitted** `libcob` (`GNURUST.LOADER.0`).

## Upstream make-check (`GNURUST.UPSTREAMCHECK.0`)

Status: **not_run (full suite)** — honestly recorded. `make check` under `libcob/` has no target
(`Nothing to be done for 'check'`); the full GnuCOBOL `tests/` testsuite is large and was not run
in this session. The oracle is admitted via its build + the field-level `libcob` runtime harness,
not via the upstream testsuite. The decimal slice's authority is the differential byte sweep above.

## Sub-receipts

[`oracle/RECEIPT-ORACLE.md`](oracle/RECEIPT-ORACLE.md) · [`kani/RECEIPT-KANI.md`](kani/RECEIPT-KANI.md)
· [`fuzz/RECEIPT-FUZZ.md`](fuzz/RECEIPT-FUZZ.md) · [`oracle-delta-ledger.md`](oracle-delta-ledger.md)
(empty: no deltas).
