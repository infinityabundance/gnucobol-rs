<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.POSTING.1 (court-casefile)

**Verdict: PASS** · tests/posting.rs (6: clean, dup-sequence, gap-only-when-contiguous, dup-txn-id, order-mutation-changes-chain, id-required) · crate `kobold-data-shim` kobold 0.6.3

- **Oracle:** deterministic custody over the declared records (binds FILE.1/BANK.1/BANK.2/DB2HOST.1)
- **Byte domain(s):** declared posting unit -> custody manifest (identity + order hash-chain + sequence/duplicate evidence)
- **Replay:** `deterministic custody over the declared records (binds FILE.1/BANK.1/BANK.2/DB2HOST.1)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (2)
- a declared posting unit records batch identity, business date, extract metadata, a sha256 hash chain over record ORDER, sequence min/max/duplicates/(gaps when declared-contiguous), and duplicate transaction ids
- reordering records changes the chain hash

## Negative claims (8) — negative capability is the trust surface
- ledger acceptance
- settlement finality
- account balance correctness
- business truth
- gap detection without a declared-contiguous sequence
- a technical duplicate being a business duplicate
- currentness
- lie prevented: 'a reconciled, sequenced, de-duplicated batch is a posted, accepted, settled, business-true unit' -- POSTING.1 records custody evidence only

## Damage if overclaimed
treating custody as ledger acceptance posts or finalizes a batch that was only inventoried, not accepted

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
