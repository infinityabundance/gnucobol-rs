# RECEIPT-GNURUST-SET88-12 — sealed: SET condition-name TO TRUE

**Campaign GNURUST.12** (TRUE-only; `SET ... TO FALSE` / the `FALSE` clause are explicitly deferred
to a `GNURUST.12b` sub-court). Goal: the inverse of `GNURUST.11` — `condition → canonical parent
bytes`.

## Doctrine (the one sentence)

> GNURUST.12 admits `SET condition-name TO TRUE` only as an oracle-proven parent-byte construction:
> it computes the storage bytes GnuCOBOL writes to make the admitted LEVEL-88 condition true,
> without claiming Procedure Division execution, FALSE-clause behavior, condition expressions, or
> business validity beyond the selected `VALUE` clause.

## Claim (exact) — the rule, proven against `cobc`

`gnucobol_rs::set_88_true(attr, size, condition)` (and the in-place `apply_set_88_true`) produces the
**canonical parent bytes** GnuCOBOL writes for `SET condition-name TO TRUE`, for an admitted
DISPLAY/COMP-3/alphanumeric parent. The chosen value is the **first** `VALUE` entry:
- a single/first literal is used as-is;
- a `THRU` **range** uses its **lower bound**;

then encoded into the parent field (alphanumeric: space-padded to `size`; numeric: zoned / packed via
the sealed `cob_move` `GNURUST.2`). Diagnosed, not assumed — the oracle shows `VALUE "A" "B" "C"` →
`"A  "`, `VALUE 5 7 9` → `5`, `VALUE 1 THRU 3` → `1`, `VALUE 1.5 THRU 2.5` → `1.5`, and `S9(3) COMP-3
VALUE 1 THRU 5` → packed `00 1c`.

**Round-trip self-check (built into the sweep and a unit test):** the bytes produced by
`set_88_true` always satisfy `eval_88` (`GNURUST.11`) — `predicate → bytes → predicate` is consistent.

## Non-claims (fail closed)

`SET ... TO FALSE`, the `FALSE` clause, condition expressions, Procedure Division execution, business
validity, collating-sequence-sensitive alphanumeric ranges, and a chosen literal whose category
mismatches the parent → `ConditionSetError`. P-scaled / edited parents and ODO-logical ambiguity
remain fail-closed (inherited from `value_image`).

## Oracle

`lab/oracle/set_sweep.sh` builds one program per case that `SET`s the condition TRUE and dumps the
parent's raw bytes via a `REDEFINES X(size)` (`-free` to avoid the 72-column margin). The Rust mirror
runs `set_88_true` and **also self-checks `eval_88`** on its own output before the byte comparison.

## Evidence

| Check | Result |
|-------|--------|
| `SET ... TO TRUE` differential sweep vs `cobc` final parent bytes | **total=52 PASS=52 FAIL=0** (`lab/oracle/set_sweep.sh`): alphanumeric single/multi(first)/range(lower); numeric DISPLAY single/multi/range, signed, `V`-scaled; COMP-3 single/range; every Rust output also passes the `eval_88` self-check |
| Self-contained `cargo test` | cond: 4/4 (incl. `set_true_picks_first_and_round_trips`) |
| Value-court regression after the shared `encode_numeric` refactor | value sweep `PASS=392 FAIL=0` (unchanged) |
| Fuzz (`cond`, now also `set_88_true` + round-trip) | **6,000,000 runs, 0 crashes** |
| `fmt` / `clippy -D warnings` / doc-gate (now runs the SET sweep) | clean |

## Determinism

Pure function of `(attr, size, condition)`; reuses the sealed numeric encoder (shared with
`value_image`, `GNURUST.8`) and `cob_move` (`GNURUST.2`). With `GNURUST.11` this completes the
minimal LEVEL-88 court: `eval_88` (bytes → predicate) and `set_88_true` (predicate → bytes).
