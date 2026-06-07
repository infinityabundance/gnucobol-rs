# RECEIPT-GNURUST-COND-11 — sealed: LEVEL-88 condition-name predicate

**Campaign GNURUST.11.** Goal: evaluate whether a LEVEL-88 condition name is **true** given the
current bytes of its parent field, per GnuCOBOL `VALUE` literals/ranges, proven against `cobc`.

## Doctrine (the one sentence)

> GNURUST.11 admits LEVEL-88 only as a parent-field byte predicate: it proves condition-name truth
> against current storage bytes, not Procedure Division control flow, `SET` semantics, or business
> validity beyond the admitted `VALUE` clauses.

## Claim (exact)

`gnucobol_rs::eval_88(attr, bytes, condition)` returns whether the condition is true for a parent
field of attributes `attr` holding `bytes`, matching GnuCOBOL, for:
- **alphanumeric** parents: a value matches iff the parent bytes equal the literal **space-padded to
  the parent length**; a `THRU` range matches iff `padded(start) <= parent <= padded(end)` byte-wise
  (native ASCII order);
- **numeric DISPLAY / COMP-3** parents: the parent's decoded **numeric value** is compared, scale-
  and sign-aware; a `THRU` range is **inclusive**;
- single values, multiple values (`VALUE 1 2 3` / `"A" "B" "C"`), and multiple ranges.

Diagnosed from the oracle: `VALUE "A"` on a `PIC X(3)` parent compares against `"A  "` (space-padded);
ranges are inclusive at both ends; numeric comparison is by value, not bytes (`88 VALUE 1` is false
for parent `2`).

## Non-claims (fail closed)

`SET condition-name TO TRUE`/`FALSE`, the `FALSE` clause, complex condition expressions, Procedure
Division branch execution, business validity, and **collating-sequence-sensitive** alphanumeric
ranges are **not** modelled. A literal whose category mismatches the parent, an unsupported parent
category, and magnitudes beyond the i128 comparison range fail closed (`ConditionError`).

## Oracle

`lab/oracle/cond_sweep.sh` builds one program per case that `MOVE`s the value into the parent and
prints whether the `88` is true (`IF C DISPLAY "T" ELSE DISPLAY "F"`). The Rust mirror encodes the
**same** parent bytes via the sealed `value_image` (`GNURUST.8`) and runs `eval_88` on them — so the
predicate is tested against the exact bytes `cobc` evaluated.

## Evidence

| Check | Result |
|-------|--------|
| Differential sweep vs `cobc` `IF`-truth | **total=103 PASS=103 FAIL=0** (`lab/oracle/cond_sweep.sh`): alphanumeric single/multi/range (padded), numeric DISPLAY single/multi/range, signed, `V`-scaled, two-range, boundary (THRU inclusive), signed range crossing zero |
| Self-contained `cargo test` | cond: 3/3 (alpha padded equality+range; numeric value+range incl. signed scaled; category-mismatch fail closed) |
| Fuzz (`cond`, arbitrary parent bytes + value tables) | **6,000,000 runs, 0 crashes** |
| `fmt` / `clippy -D warnings` / doc-gate (now runs the LEVEL-88 sweep) | clean |

## Determinism

Pure function of `(attr, bytes, condition)`; no env/locale/fs; same pinned oracle/env. Composes the
sealed decode (`GNURUST.2`) and field model (`GNURUST.3`). This is the first **business-state
predicate** layer above raw layout (`bytes → field → predicate`).
