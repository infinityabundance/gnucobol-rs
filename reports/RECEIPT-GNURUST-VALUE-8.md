# RECEIPT-GNURUST-VALUE-8 — sealed: initial record image from VALUE clauses

**Campaign GNURUST.8.** Goal: compute the WORKING-STORAGE bytes a GnuCOBOL `01` record holds at
program start (its `VALUE`-clause initialization), proven against `cobc`-initialized storage.

## Claim (exact)

`gnucobol_rs::value_image(items)` returns the initial bytes of a flat `01` record of elementary
`9 X A S V` / `COMP-3` items, **byte-identical to `cobc`'s** WORKING-STORAGE initialization, for:
- **alphanumeric** (`X`) `VALUE "lit"` → the literal **left-justified, space-padded**; unvalued → spaces;
- **numeric DISPLAY** `VALUE n` → zoned digits at the field scale with a trailing **overpunch** sign
  (`-7` → `"00w"`); unvalued or `VALUE ZERO` → `'0'` fill;
- **COMP-3** `VALUE n` / unvalued / `VALUE ZERO` → a **canonical packed value** via the sealed
  `cob_move` encode — and crucially an **unvalued COMP-3 is a packed zero with the proper sign
  nibble** (`0x0C` signed / `0x0F` unsigned), *not* raw `0x00` (diagnosed from the runtime oracle,
  correcting an initial misread of the generated C).

The numeric literal is parsed, aligned to the field scale (append/drop fraction digits), width-checked,
rendered zoned, and — for COMP-3 — `cob_move`d to packed, reusing the sealed `GNURUST.2` encoder.

## Non-claims (fail closed)

`OCCURS` / `REDEFINES` interaction with `VALUE`, edited/`P`/unsupported PICs, `VALUE` literals that do
not fit the PIC, alphanumeric `VALUE` on numeric fields (and vice-versa), and a record with **no**
`VALUE` anywhere (compiler may skip init) are out of the sealed subset → typed [`InitError`] or
classified out of the sweep. Figurative constants beyond `ZERO`/`SPACE` are deferred.

## Oracle

`lab/oracle/value_sweep.sh` builds one COBOL program per case from a `gen_value` spec, compiles with
the built `cobc`, runs it, and captures **`DISPLAY REC WITH NO ADVANCING`** — a group `DISPLAY` emits
the record's raw initial bytes. The Rust mirror (`examples/value_rows`) consumes identical specs.

## Evidence

| Check | Result |
|-------|--------|
| Differential sweep vs `cobc`-initialized storage | **PASS=392 FAIL=0** (`lab/oracle/value_sweep.sh`): DISPLAY/COMP-3, signed/unsigned, integer & `V`-scaled, valued / unvalued / `ZERO`, mixed alnum lead+trail per record |
| Self-contained `cargo test` | init: 2/2 (the probed `val.cob` record; signed overpunch + unvalued-packed canonical zero) |
| Fuzz (`init` target, arbitrary record specs) | 3,000,000 runs, 0 crashes (literal parser + zoned alignment + `cob_move` surface) |
| `fmt` / `clippy -D warnings` / doc-gate (now runs the VALUE sweep) | clean |

## Determinism

Pure function of `items` (`GNURUST.PUREDEC.0`); no env/locale/fs; same pinned oracle/env as the other
courts. Composes `GNURUST.3` (PIC), `GNURUST.4` (layout), `GNURUST.2` (`cob_move`).
