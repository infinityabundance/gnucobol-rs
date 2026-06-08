<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — KOBOLD.EXTRACT.PROFILE.1 (court-casefile)

**Verdict: PASS** · tests/extract.rs (2: records provenance + refuses extraction truth; optional fields null) · crate `kobold-data-shim` kobold 0.6.3

- **Oracle:** declared operator provenance (not detected)
- **Byte domain(s):** declared extraction metadata -> provenance manifest bound to data/copybook hashes
- **Replay:** `declared operator provenance (not detected)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (2)
- records the declared extraction provenance (file organization, extract method, record-length source, copybook source, pre-KOBOLD code-set conversion, source-system cutoff, operator assumptions) bound to the data + copybook sha256
- copybook freshness and extraction truth are explicit non-claims

## Negative claims (6) — negative capability is the trust surface
- extraction truth (how/when/whence the bytes were obtained)
- VSAM/indexed-backend/file-status semantics
- CODE-SET file-I/O conversion parity
- the copybook being production truth
- currentness
- lie prevented: 'the bytes decoded, so the extract is complete/correct and the copybook is the production one' -- EXTRACT.PROFILE.1 records DECLARED provenance + holds copybook freshness as a permanent uncertainty

## Damage if overclaimed
acting on an incomplete/wrong extract or a stale copybook that decodes plausibly wrong corrupts a whole migration silently

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
