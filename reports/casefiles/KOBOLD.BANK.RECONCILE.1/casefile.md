<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.BANK.RECONCILE.1 (court-casefile)

**Verdict: PASS** · tests/bank_reconcile.rs (2: matched view faithful + refuses truth; mismatch renders finding in view+SARIF) · crate `kobold-data-shim` kobold 0.6.3

- **Oracle:** the court result structs themselves (no recomputation that can drift)
- **Byte domain(s):** existing court evidence -> generated operator report (json + md + sarif); introduces no new evidence
- **Replay:** `the court result structs themselves (no recomputation that can drift)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (6)
- an operator view assembled ONLY from existing court structs AND provably DERIVED from named, hash-pinned source casefiles (source_evidence: BANK.1/BANK.2
- POSTING.1
- DB2HOST.1 + any extra EXTRACT.PROFILE.1/PRIVACY.REDACTION.1/DIFF.1, each by sha256
- derived_view:true, creates_new_truth:false): declared-vs-observed count/debit/credit + matched/mismatch verdict from banking.balanced, custody seq/gaps/dups + last_chain_hash, db2 counts, dirty/unsupported counts, redaction counts, refused truth layers
- aggregates the EXISTING findings into one SARIF
- a changed source casefile changes the report hash

## Negative claims (7) — negative capability is the trust surface
- ledger acceptance
- settlement finality
- account-balance truth
- business approval
- that a match is correctness
- that the view is new evidence
- lie prevented: 'the reconciliation view says matched, so the batch is posted/accepted/correct' -- BANK.RECONCILE.1 summarizes declared-vs-observed evidence and refuses every truth layer above record truth

## Damage if overclaimed
an operator acting on a matched VIEW as ledger acceptance posts a batch on a summary; a STALE view over changed source casefiles misreports the evidence it claims to derive from

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
