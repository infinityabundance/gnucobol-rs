# RECEIPT-GNURUST-PIC-3 — sealed: PICTURE → field model

**Campaign GNURUST.3.** Goal: parse a COBOL `PIC` clause + `USAGE` into the same
`(type, digits, scale, flags)` field model the decimal court uses, plus the storage `size`, and
prove it matches the GnuCOBOL **compiler's own** field-attribute computation.

## Claim (exact)

For the sealed picture subset — symbols `9 X A S V`, fixed repeats `(n)`, the
`SIGN [LEADING|TRAILING] [SEPARATE]` clause, and `USAGE DISPLAY` / `COMP-3`
(`PACKED-DECIMAL` / `COMPUTATIONAL-3`) — `gnucobol_rs::pic::build_field` produces the **identical**
`{type, digits, scale, flags, size}` that `cobc` emits as the field's `cob_field_attr` and storage
size. The size is independently consistent with runtime `LENGTH OF`.

## Non-claims (fail closed)

- **`P` scaling symbol** — deferred (`GNURUST.PIC-SCALING-P.0`): GnuCOBOL's leading-`P` vs
  trailing-`P` digit/scale rules are asymmetric (e.g. `9(3)PPP` → digits 6, scale −3; `PPP9(3)` →
  digits 3, scale 6) and not yet sealed. `build_field` returns `PicError::ScalingPDeferred`.
- **Edited pictures** (`Z * $ , . + - CR DB B 0 /`) — `PicError::UnsupportedSymbol` (future court).
- **Other `USAGE`** (`COMP`/`COMP-5`/`COMP-1/2`/`COMP-X`/`COMP-6`) — not parsed here.
- Mixed alphanumeric+numeric, empty, zero/oversized repeats — typed `PicError`.

## Oracle (the compiler's authoritative PIC→attr decision)

`lab/oracle/pic_harness.sh` generates a program declaring each PIC as `01 F PIC … [USAGE] [SIGN].`,
runs `cobc -C`, and parses the emitted `static const cob_field_attr a_N = {type,digits,scale,flags}`
and `cob_field f_M = {size, …, &a_N}`. This is **generated C used only as the witness of the
compiler's field-attribute computation** (`GNURUST.GENC.0`) — not a runtime semantic claim — and is
cross-checked by runtime `LENGTH OF` (e.g. `S9(5)V99 COMP-3` → 4, `9(8)V9(4) COMP-3` → 7, `99P`→2).

## Evidence

| Check | Result |
|-------|--------|
| Differential sweep vs `cobc` field attrs | **PASS=192 FAIL=0** (`lab/oracle/pic_sweep.sh`): int parts {0,1,2,3,5,8,9,12} × frac {0,1,2,4} × {unsigned, signed-overpunch, signed leading/trailing separate} × {DISPLAY, COMP-3} + alphanumeric `X(n)` widths |
| Self-contained `cargo test` | 10/10 (oracle-captured samples; P & edited & garbage fail closed; DOS regression) |
| Fuzz (`pic` target, arbitrary strings) | **5,000,000 runs, 0 crashes** after fixing 1 real bug the fuzzer found |
| `fmt` / `clippy -D warnings` / doc-gate | clean |

## Fuzz finding (fixed)

The fuzzer hit an **OOM (libFuzzer exit 71)**: a giant repeat like `9(999999999)` made the parser
materialize a billion-element `Vec`. Fixed by **streaming `(symbol, count)` terms** and accumulating
counts in O(1) memory, with a `MAX_POSITIONS` (1,000,000) resource bound that rejects absurd
declared sizes as `PicError::BadRepeat` (a resource guard, not a semantic claim) — `GNURUST.DOS.0`.
Seeded as a regression test.

## Determinism

Same pinned oracle and env as the decimal court (`reports/admission/RECEIPT-ADMISSION.md`),
`LC_ALL=C.UTF-8`, little-endian ASCII host. The `pic` module is a pure function of its inputs
(`GNURUST.PUREDEC.0`): no env/locale/fs reads.
