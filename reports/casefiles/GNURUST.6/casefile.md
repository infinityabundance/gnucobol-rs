<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.6 (court-casefile)

**Verdict: PASS** · within COPY sweep + 4M fuzz · crate `gnucobol-rs` 0.2.3

- **Oracle:** cobc -P
- **Byte domain(s):** expanded source text-word stream
- **Replay:** `cobc -P`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (1)
- whole-text-word pseudo-text replacement matching cobc -P (composes across nesting)

## Negative claims (4) — negative capability is the trust surface
- REPLACE directive
- LEADING/TRAILING
- identifier operands
- lie prevented: 'substring replacement is close enough' — REPLACING is whole-text-word

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
