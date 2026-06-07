# RECEIPT-GNURUST-PIC-SCALING-9 — sealed: PIC `P` scaling

**Campaign GNURUST.9.** Goal: close the `P` scaling-position hole in the PIC court (`GNURUST.3`),
making `build_field` produce the same `(type, digits, scale, size)` as `cobc` for `P` pictures.

## Claim (exact) — the asymmetric rule, proven against `cobc`

For a numeric PICTURE with a single contiguous run of `P` at one end of the `9`s (let `n` = number
of `9`s, `p` = number of `P`s), `gnucobol_rs::build_field` returns:

| form | `digits` | `scale` | storage `size` |
|------|---------|---------|----------------|
| **trailing P** (`999PPP`, `9PPP`, `99P`, `S999PP`) | `n + p` | `-p` | `n` (DISPLAY) / `n/2+1` (COMP-3) |
| **leading P** (`PPP999`, `PPP9`, `P99`, `PP99`) | `n` | `n + p` | `n` (DISPLAY) / `n/2+1` (COMP-3) |

The key subtleties, **diagnosed from the `cobc` attr witness + `LENGTH OF`, not guessed**:
- only the `9`s are **stored** (`size = n`); the `P` positions are assumed, not stored — yet for
  **trailing** P the attr `digits` *includes* the P positions (`n+p`) while for **leading** P it does
  not (`n`);
- COMP-3 `size` uses the **stored** digit count `n/2+1` even though `attr.digits` carries the P
  (`999PPP` COMP-3 → `digits 6, size 2`);
- the sign flag is unaffected (`S999PP` → `HAVE_SIGN`, `digits 5, scale -2`).

## Non-claims (fail closed)

`V` combined with `P` (`9PV9`), `P` at **both** ends (`P9P`), `P`-only with no `9` (`PPP`), and
edited pictures → typed `PicError::ScalingPDeferred` / `NoDigits` / `UnsupportedSymbol`. **VALUE and
MOVE on a P-scaled field are a separate court** (`GNURUST.VALUE-P.0`): because a P field's
`attr.digits != size`, `value_image` **fails closed** on it (`InitError`) rather than mis-place a
digits-wide rendering into the smaller stored field.

## Oracle

`lab/oracle/pic_harness.sh` (generated-C `cob_field_attr` witness + `LENGTH OF`) — the compiler's own
PICTURE→attribute decision. The Rust `build_field` mirror is compared field-for-field.

## Evidence

| Check | Result |
|-------|--------|
| PIC differential sweep vs `cobc` | **PASS=288 FAIL=0** (`lab/oracle/pic_sweep.sh`): the prior 192 + 96 P cases — trailing/leading × DISPLAY/COMP-3 × signed/unsigned × `n∈{1,2,3,5}` `p∈{1,2,3}` |
| Self-contained `cargo test` | pic: 5/5 (P matches-oracle for both ends + COMP-3 + signed; V+P / both-ends / P-only fail closed) |
| Fuzz (`pic` 6M, `init` 3M incl. P shapes) | **0 crashes** — P fields fail closed in the VALUE court, never panic |
| `fmt` / `clippy -D warnings` / doc-gate | clean |

## Determinism

Pure function of `(pic, usage, sign flags)`; same pinned oracle/env. Extends the sealed `GNURUST.3`
field model; `size` composes into the layout (`GNURUST.4`) unchanged.
