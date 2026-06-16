<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.BIGNUM.1 (court-casefile)

**Verdict: PASS** · bignum_sweep 16128/0 (15-20 digit operands x signs x scales x ROUNDED modes) + K=40 + mul_u256 carry unit tests · crate `gnucobol-rs` 0.7.85

- **Oracle:** libcob cob_mul (GMP product) + cob_decimal_get_field truncating store
- **Byte domain(s):** two numeric operands whose i128 product overflows -> receiver bytes (low-digit truncation + ROUNDED)
- **Replay:** `bash lab/oracle/bignum_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- a MULTIPLY whose exact product exceeds i128 no longer fails closed: the port carries the full 256-bit product (two operands of <=38 digits give a <=76-digit product < 2^256), converts it to exact decimal, rounds to the receiver scale and truncates to the receiver's low digits -- byte-identical to libcob (which keeps the product in GMP and stores the low-order digits). round-trips through the UNCHANGED store()
- covers the full binary-multiply domain including K = receiver_digits + dropped_digits > 38 (which an early mod-10^K reduction would lose)

## Negative claims (5) — negative capability is the trust surface
- ADD/SUBTRACT operand upscale overflow (a separate path)
- DIVIDE overflow
- chained-COMPUTE single-expression GMP precision (gnucobol-rs evaluates op-by-op)
- a single operand exceeding 38 digits (not valid COBOL)
- lie prevented: '38-digit COBOL arithmetic always fits a 128-bit integer' -- NO, a single MULTIPLY of two large operands yields up to 76 digits; failing closed there silently drops results libcob computes

## Damage if overclaimed
treating an overflowing product as unsupported (or wrapping it) silently corrupts large-value multiplications that libcob handles exactly

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
