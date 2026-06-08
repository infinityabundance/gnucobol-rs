<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.LAYOUT.REDEFINES.2 (court-casefile)

**Verdict: PASS** · tests/redefines.rs (4: overlapping views + active_view false by default, deterministic+shared raw hash, declared discriminator admits active view, unknown discriminator keeps it false) · crate `kobold-data-shim` kobold 0.6.4

- **Oracle:** the shim's own REDEFINES layout (overlapping offsets) + decode
- **Byte domain(s):** shared storage region -> multiple decoded views + declared-or-refused active view
- **Replay:** `the shim's own REDEFINES layout (overlapping offsets) + decode`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (2)
- records explicit overlapping byte VIEWS created by REDEFINES: which fields share storage (offset/length/raw_sha256), each view decoded INDEPENDENTLY over the SAME bytes, and whether an active view was declared by a discriminator
- active_view stays claimed:false by default and is admitted only by a declared discriminator (unknown -> false)

## Negative claims (5) — negative capability is the trust surface
- which view is semantically active (without a declared discriminator)
- that a layout-valid view is business-meaningful
- that multiple views are simultaneously true
- write-back
- lie prevented: 'this REDEFINES decodes cleanly, therefore it is the record' -- LAYOUT.REDEFINES.2 proves overlapping byte VIEWS, not which view is active; active_view is admitted only by a declared discriminator

## Damage if overclaimed
promoting a layout-valid overlay (a loan view over an account record) into truth misreads the entire record

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
