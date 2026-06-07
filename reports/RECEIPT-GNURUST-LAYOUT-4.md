# RECEIPT-GNURUST-LAYOUT-4 — sealed: DATA DIVISION record layout

**Campaign GNURUST.4.** Goal: assign each DATA DIVISION item its byte **offset** and **size** within
an `01` record, and prove it matches the GnuCOBOL **compiler's own** record-layout decision.

## Claim (exact)

For the sealed subset — level-numbered groups + elementary items whose `PIC` is in the sealed
[`GNURUST.3`] subset, fixed `OCCURS n TIMES`, `REDEFINES <name>` where the redefining item is **no
larger** than its target, and `FILLER` — `gnucobol_rs::layout::lay_out` assigns every item the
**same byte offset and size** that `cobc` assigns. Verified for every statically-addressable item
the compiler emits, plus the record total; `OCCURS` multipliers are verified transitively via the
offset of the field that follows each table.

## Non-claims (fail closed)

- `OCCURS DEPENDING ON` (variable-length) — not parsed.
- `SYNCHRONIZED`/alignment — not modeled.
- `REDEFINES` larger than its target (record-growth case) — `LayoutError::RedefinesLarger`.
- Nested `OCCURS` element static offsets are validated transitively, not per-element.

## Oracle

`lab/oracle/layout_harness.sh` generates a program for one `01` record, runs `cobc -C`, and parses
the emitted `cob_field f_M = {size, b_REC + OFFSET, &attr};  /* name */` for each statically-
addressable item — the compiler's own offset/size decision (`GNURUST.GENC.0`), cross-checked by
runtime `LENGTH OF`. Items that are OCCURS tables or nested under an OCCURS ancestor have
runtime-computed (non-static) offsets and are skipped at the oracle; their effect is checked via the
following field's offset and the record total.

## Evidence

| Check | Result |
|-------|--------|
| Differential sweep vs `cobc` offsets/sizes | **records=6, PASS=32 FAIL=0** (`lab/oracle/layout_sweep.sh`): flat records, nested groups (to level 10), FILLER, `OCCURS` on elementary & group, `REDEFINES` of elementary & group, all sealed PIC/USAGE forms |
| Self-contained `cargo test` | layout: 2/2 (oracle record REC=29; `REDEFINES`-larger fails closed) |
| Fuzz (`layout` target, arbitrary items) | **5,000,000 runs, 0 crashes** — hostile level nesting / OCCURS counts / REDEFINES targets all yield typed `LayoutError`; the level-tree recursion and OCCURS `checked_mul` hold |
| `fmt` / `clippy -D warnings` / doc-gate | clean |

## Determinism

Same pinned oracle and env as the decimal/PIC courts (`reports/admission/RECEIPT-ADMISSION.md`).
`layout::lay_out` is a pure function of its `Item` list (`GNURUST.PUREDEC.0`).
