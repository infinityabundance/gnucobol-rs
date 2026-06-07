# Changelog — cobc-oracle-rs

## [0.0.2]
- Doc freshness: corrected the stale "not published" doctrine (the crate **is** published as GPL
  tooling). Documented where this crate sits in the oracle ecosystem — the **program-shape** oracle
  (compile + run + receipt), distinct from the **runtime-library shape** (`decimal_harness`) that
  drives the byte-level sealed-court sweeps, and from the **generated replay receipts** (TRUST.2,
  `lab/receipt/`) that are the campaign evidence of record. No behavioural change.

## [0.0.1]
- Initial: drive `cobc -x`/`-C`/`-m`, capture a deterministic canonical-JSON receipt (oracle identity,
  stdout/stderr/exit, sha256 of source/generated-C/binary). Typed oracle-availability verdict; named
  compilation mode; generated-C is a witness, not authority. Self-contained SHA-256, zero runtime deps.
