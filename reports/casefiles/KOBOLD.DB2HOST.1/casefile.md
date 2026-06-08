<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.DB2HOST.1 (court-casefile)

**Verdict: PASS** · tests/db2host.rs (5: null/present/truncation, missing-fails-closed, wrong-usage-fails-closed) · crate `kobold-data-shim` kobold 0.6.3

- **Oracle:** composed gnucobol-rs courts (COMP-3/COMP-5) + declared kobold-db2host-indicator-manifest-v1
- **Byte domain(s):** declared value/indicator pairing -> null/truncation state (bytes preserved)
- **Replay:** `composed gnucobol-rs courts (COMP-3/COMP-5) + declared kobold-db2host-indicator-manifest-v1`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (3)
- a decoded field is marked semantic_null / truncation_evidence ONLY via a declared S9(4) COMP-5 indicator pairing (negative=null, zero=present, positive=truncation)
- decoded bytes are always preserved
- missing or wrong-usage indicator fails closed

## Negative claims (7) — negative capability is the trust surface
- SQL execution
- Db2 precompiler transformation
- SQLCA/SQLCODE interpretation
- DBRM/package identity
- the host value being the database value without its indicator
- database truth
- lie prevented: 'a cleanly decoded value is the database value' -- the DB2 null/truncation indicator can override it

## Damage if overclaimed
acting on a decoded value the database marked NULL or truncated posts wrong or missing data

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
