# Negative-capability registry

A serious reviewer trusts a project that says **"no"** precisely. This registry makes every
unsupported COBOL surface visible and queryable: what is rejected, *where* it is rejected, the error
type, why, the future campaign that may seal it, and the risk if it were instead silently guessed.

This is the human companion to the machine-readable non-claims in
[`reports/negative-claims.md`](../reports/negative-claims.md) and the per-court non-claims in
[`reports/claim-ladder.json`](../reports/claim-ladder.json). The doctrine: **fail closed; unsupported
surfaces remain explicit evidence, never inferred behaviour.**

| Unsupported surface | Where rejected | Error type | Why rejected | Future campaign | Risk if guessed |
|---------------------|----------------|------------|--------------|-----------------|-----------------|
| Edited PICTURE (`Z $ , . + - CR DB * B 0 /`) | `pic::build_field` | `PicError::UnsupportedSymbol` | rendering rules not yet oracle-proven | `GNURUST.EDITED.0` | wrong displayed value / wrong width |
| `V` combined with `P` | `pic::build_field` | `PicError::ScalingPDeferred` | scale interaction not proven | `GNURUST.PIC-VP.0` | wrong scale → wrong decimal value |
| `VALUE`/`MOVE` on a P-scaled field | `init::value_image` | `InitError::Pic` | `attr.digits != size`; would mis-place bytes | `GNURUST.VALUE-P.0` | buffer mis-fill / panic avoided by failing closed |
| `OCCURS DEPENDING ON` **logical** length | layout court (non-claim) | n/a (only physical-max admitted) | runtime DEPENDING value not modelled | `GNURUST.ODO-LOGICAL.0` | treating physical-max as active length |
| Multiple / nested ODO, ODO-not-last, REDEFINES+ODO | `layout::lay_out` | `LayoutError::OdoUnsupported` | complex-ODO semantics unproven | later | wrong offsets for following fields |
| `REDEFINES` larger than target | `layout::lay_out` | `LayoutError::RedefinesLarger` | record-growth case unproven | `GNURUST.REDEFINES.GROW.0` | wrong record size |
| `SET condition-name TO FALSE` / `FALSE` clause | `cond` (not implemented) | `ConditionSetError` | needs the FALSE clause + its own oracle | `GNURUST.12b` | wrong "reset" bytes |
| Condition expressions / Procedure-Division execution | n/a (out of model) | non-claim | not a runtime / not a compiler | none (refused) | inventing control-flow semantics |
| `DIVIDE`, `ON SIZE ERROR`, rounding modes ≠ nearest-away | `arith::cob_arith` | `ArithError` (deferred) | each needs its own oracle pass | `GNURUST.14`, `GNURUST.15` | wrong financial result |
| `> 38`-digit / bignum intermediates | `arith::cob_arith` | `ArithError::OutOfRange` | i128 range exceeded | `GNURUST.ARITH-BIGNUM.0` | silent overflow / wrong value |
| `COMP`/`COMP-5`/`COMP-X`/`COMP-6`, float (`COMP-1/2`) | `pic`/decode | `PicError`/unsupported | byte/endian/size not proven | `GNURUST.BINARY.0` | wrong endianness / size |
| EBCDIC / code-page input | (none — out of host scope) | n/a (host = ASCII) | EBCDIC is a declared parameter, never inferred | `GNURUST.EBCDIC.0` | wrong characters & signs |
| EBCDIC-machine zoned sign mode | decode (non-claim) | n/a | `COB_EBCDIC_MACHINE` path not admitted | `GNURUST.EBCDIC.0` | wrong sign |
| `DECIMAL-POINT IS COMMA` | (none) | non-claim | locale-dependent | later | wrong literal parsing |
| Files / indexed I/O / SORT / Report Writer / Screen | (none — out of model) | non-claim | not data-court surfaces | far future | scope creep into runtime |
| Malformed packed beyond sealed MOVE rules | decode | bounds-guarded, typed | tolerance is a declared policy, not a default | `GNURUST.INVALID.0` | accepting corrupt data silently |
| Record-length mismatch (recon) | `recon::reconcile` | trailing partial → unsupported | cannot decode a short record | n/a | mis-aligned fields |

## How to add a row

A surface only leaves this table by becoming a **sealed court** (its own oracle sweep + receipt) or
by an explicit, receipted policy. Until then it must fail closed with a typed error — a reviewer
should be able to feed an unsupported field and watch the system refuse it, not guess (see the
counterexample-first tests referenced in each court's receipt).
