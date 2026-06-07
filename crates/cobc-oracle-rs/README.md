# cobc-oracle-rs

**Drives the GnuCOBOL `cobc` compiler as an oracle for the [`gnucobol-rs`](https://github.com/infinityabundance/gnucobol-rs)
compatibility court.** It builds and runs a COBOL fixture and records a deterministic, canonical-JSON
**receipt** of the oracle's identity and observed behaviour (stdout/stderr/exit + SHA-256 of source,
generated C, and the executable).

This crate copies **no** GPL compiler logic — it only spawns `cobc`/the produced binary and reads
their outputs — but, being tooling tightly coupled to the GPL compiler, it is licensed
**GPL-3.0-or-later** (see `COPYING`). It is an early `0.0.1`: the API may change.

Doctrine it encodes:
- **Generated C is a witness, not authority** (`GNURUST.GENC.0`): the receipt records
  `generated_c_hash`, but semantic authority is the *runtime* (stdout/stderr/exit + field bytes).
- **Compilation mode is always named** (`GNURUST.ORACLEMODE.0`).
- **Oracle availability is a typed verdict** (`GNURUST.ORACLEAVAIL.0`), never a silent skip.
- **Canonical JSON** (`GNURUST.JSONCANON.0`): stable key order, lowercase-hex bytes, explicit nulls
  — via a tiny hand-written serializer (zero runtime dependencies, self-contained SHA-256).

## Usage

```text
cobc-oracle oracle-smoke
cobc-oracle run-fixture program.cob
cobc-oracle write-receipt --fixture program.cob --out receipt.json
```

Run with a built GnuCOBOL `cobc` on `PATH` and `LC_ALL=C.UTF-8`.
