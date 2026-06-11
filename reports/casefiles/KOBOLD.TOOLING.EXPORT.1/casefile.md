<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.TOOLING.EXPORT.1 (court-casefile)

**Verdict: PASS** · tests/tooling.rs (3: maps provenance/courts/witness/non-claims, redacted field never leaks cleartext, deterministic) · crate `kobold-data-shim` kobold 0.6.6

- **Oracle:** the sealed-court decode + provenance (re-export equality)
- **Byte domain(s):** existing decode + provenance -> kobold-tooling-export-v1 field map
- **Replay:** `the sealed-court decode + provenance (re-export equality)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (3)
- emits an IDE/tooling-friendly evidence map from the EXISTING sealed-court decode + provenance: per field the qualified name, copybook path + line, PIC, USAGE, offset, length, decoded value OR redaction status (no cleartext for a redacted field), raw_sha256, the sealed court ids that produced it, findings, the witness dialect_profile_id, and per-field non-claims
- deterministic
- introduces_new_evidence:false

## Negative claims (7) — negative capability is the trust surface
- an LSP
- an IDE
- a full COBOL parser
- a source of truth
- new evidence
- bypassing redaction
- lie prevented: 'KOBOLD is the IDE/LSP/parser' / 'the export is the source of truth' -- TOOLING.EXPORT.1 is a downstream evidence MAP an IDE could consume; it is not the tool and creates no new truth

## Damage if overclaimed
a downstream tool trusting the export as authoritative (or leaking redacted data) bypasses the courts and the redaction policy

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
