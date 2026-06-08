<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.BANK.RECONCILE.1 (court-casefile)

**Verdict: PASS** · tests/bank_reconcile.rs (2: matched view faithful + refuses truth; mismatch renders finding in view+SARIF) · crate `kobold-data-shim` kobold 0.6.3

- **Oracle:** the court result structs themselves (no recomputation that can drift)
- **Byte domain(s):** existing court evidence -> generated operator report (json + md + sarif); introduces no new evidence
- **Replay:** `the court result structs themselves (no recomputation that can drift)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (2)
- an operator view assembled ONLY from existing court structs (BANK.1/2 summary, POSTING.1 custody, DB2HOST.1 indicators, PRIVACY redaction counts): declared-vs-observed count/debit/credit + matched/mismatch verdict, custody seq min/max/gaps/dups + last_chain_hash, db2 null/truncation counts, dirty/unsupported/unknown counts, redaction counts, and refused truth layers
- aggregates the EXISTING findings into one SARIF view

## Negative claims (7) — negative capability is the trust surface
- ledger acceptance
- settlement finality
- account-balance truth
- business approval
- that a match is correctness
- that the view is new evidence
- lie prevented: 'the reconciliation view says matched, so the batch is posted/accepted/correct' -- BANK.RECONCILE.1 summarizes declared-vs-observed evidence and refuses every truth layer above record truth

## Damage if overclaimed
an operator acting on a matched VIEW as if it were ledger acceptance posts or finalizes a batch on a summary, not a decision

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
