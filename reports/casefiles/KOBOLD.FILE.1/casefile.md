<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.FILE.1 (operator-casefile)

**Verdict: PASS** · tests/file.rs 10 green; ingest==reconcile offsets · crate `kobold-data-shim` kobold 0.6.3

- **Oracle:** deterministic ingest invariants (not GnuCOBOL file I/O)
- **Byte domain(s):** raw byte stream -> record spans (offset,len)
- **Replay:** `deterministic ingest invariants (not GnuCOBOL file I/O)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (1)
- raw stream splits into fixed-length records with true offsets, named trailing/partial policies, byte-stable file audit, stable exit codes

## Negative claims (7) — negative capability is the trust surface
- GnuCOBOL file organization
- indexed/relative I/O
- line-sequential runtime parity
- auto-resynchronization
- encoding auto-detection
- silent repair of dirty bytes
- lie prevented: 'a fixed-record file just chunks cleanly' -- partial/trailing-newline shapes are policy decisions, not silent behavior

## Damage if overclaimed
silently absorbing a partial/extra record drops or duplicates a banking record

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
