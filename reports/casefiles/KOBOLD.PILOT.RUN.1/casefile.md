<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.PILOT.RUN.1 (court-casefile)

**Verdict: PASS** · kobold-data-shim src/bin/pilot.rs (kobold-pilot) + reports/pilot-run/ committed packet + tests/pilot_workflow.rs · crate `kobold-data-shim` kobold 0.7.1 (bin; ships next batch)

- **Oracle:** the composed sealed courts + the no-cleartext-leak abort (re-run equality)
- **Byte domain(s):** a declared extract -> a redacted, hash-bound pilot evidence packet on disk
- **Replay:** `the composed sealed courts + the no-cleartext-leak abort (re-run equality)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (5)
- the kobold-pilot runner executes the full pilot chain (EXTRACT.PROFILE.1 -> PRIVACY.REDACTION.1 -> BANK.1/2
- BANK.RECONCILE.1 -> DIFF.1 -> TOOLING.EXPORT.1 -> PILOT-PACKET.1) over a DECLARED extract and writes a redacted, hash-bound evidence packet to disk
- sensitive fields are tokenized BEFORE any artifact is written and the runner ABORTS (exit 4) if a redacted value leaks into any output, so the packet is safe to share
- the committed example run (kobold-data-shim reports/pilot-run/) is complete, derived_view, creates_new_truth:false with verified zero cleartext
- an operator points the runner at a real private extract for a real pilot

## Negative claims (7) — negative capability is the trust surface
- customer acceptance
- business correctness
- regulatory compliance
- production readiness
- ledger truth
- that the synthetic default is a real extract
- lie prevented: 'we ran a pilot, so we can take real customer data to production / the totals are business-correct / it is compliant' -- PILOT.RUN.1 produces redacted custody evidence over a DECLARED extract and refuses every higher claim, and fails closed on a cleartext leak

## Damage if overclaimed
presenting a single declared pilot run as customer-accepted/compliant/production-ready, or shipping a packet that leaked real account ids, would expose customers and manufacture readiness the courts do not provide

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
